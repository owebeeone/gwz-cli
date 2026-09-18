//! The in-process gwz-core calls the hooks make: read the family index
//! lock-free (as `local list` does), create a lane, dispose one.
//!
//! Every workspace semantic stays in core. This module builds the same
//! requests the `gwz local` verbs build and presents the answers in the
//! shapes the hook's own decisions need.

use std::path::{Path, PathBuf};

use super::integration::{OwnerOp, WaitOwner, dispose_wait, owner};
use crate::*;

/// The family as one lock-free read saw it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FamilyRead {
    /// The family root as core observed it, absent when no family exists.
    pub(crate) root_path: Option<String>,
    pub(crate) members: Vec<gwz_core::LocalFamilyMemberEntry>,
}

impl FamilyRead {
    pub(crate) fn row(&self, name: &str) -> Option<&gwz_core::LocalFamilyMemberEntry> {
        self.members.iter().find(|entry| entry.name == name)
    }

    pub(crate) fn ready_rows(&self) -> u32 {
        self.members
            .iter()
            .filter(|entry| entry.recorded_state == gwz_core::LocalMemberState::Ready)
            .count() as u32
    }

    /// The absolute path the index recorded for a row, joined lexically with
    /// the observed root exactly as the listing renders it.
    pub(crate) fn absolute_path(&self, entry: &gwz_core::LocalFamilyMemberEntry) -> PathBuf {
        PathBuf::from(member_path(self.root_path.as_deref(), &entry.path))
    }

    /// The owner recorded on a row (R20), read through the one integration
    /// point.
    pub(crate) fn owner_of(&self, entry: &gwz_core::LocalFamilyMemberEntry) -> Option<String> {
        owner(OwnerOp::Read(entry))
    }
}

fn meta(root: &Path) -> gwz_core::RequestMeta {
    gwz_core::RequestMeta {
        request_id: new_request_id(),
        schema_version: "gwz.protocol/v0".to_owned(),
        workspace: Some(gwz_core::WorkspaceRef {
            root: Some(root.to_string_lossy().into_owned()),
            workspace_id: None,
        }),
        invocation: Some(gwz_core::InvocationContext {
            caller_cwd: root.to_string_lossy().into_owned(),
        }),
        ..Default::default()
    }
}

/// `gwz local list`, in process and without the family lock.
pub(crate) fn read_family(root: &Path) -> Result<FamilyRead, gwz_core::model::ModelError> {
    let backend = gwz_core::git::Git2Backend::new();
    let events = gwz_core::operation::NullSink;
    let response = gwz_core::workspace_ops::handle_local_family(
        &backend,
        root,
        gwz_core::LocalFamilyRequest {
            meta: meta(root),
            op: gwz_core::LocalFamilyOp::List,
            name: None,
            keep: None,
            force_hazards: Vec::new(),
            // `list` takes no lock, so there is nothing to wait for (R21).
            wait_seconds: None,
        },
        new_operation_id(),
        &events,
    )?;
    Ok(FamilyRead {
        root_path: response.root_path,
        members: response.members,
    })
}

/// D6's completeness check, on the reuse path (F2).
///
/// A reuse must stay a reuse, so this is deliberately cheap: gwz-core's own
/// observation of the lane is already in the family row (`decide` requires
/// `observed_state` to be `ready`, which is the pointer and the allocation
/// marker), and what that does not cover is the lane's *contents* -- the
/// repositories a session is about to work in. Those are checked against
/// the source workspace's own member listing, by existence alone: never a
/// tree walk, and never a fetch or a status.
///
/// Returns what is wrong with the first repository that is, or `None` when
/// the lane is fit to hand back to a session.
pub(crate) fn lane_incompleteness(source: &Path, lane: &Path) -> Option<String> {
    if !lane.is_dir() {
        return Some("its directory is gone".to_owned());
    }
    // The root is a repository of the workspace like any other, and `ls`
    // does not list it.
    if source.join(".git").exists() && !lane.join(".git").exists() {
        return Some("its root is not a repository".to_owned());
    }
    let response = gwz_core::workspace_ops::handle_ls(
        source,
        gwz_core::LsRequest {
            meta: meta(source),
            include_unmaterialized: None,
        },
        new_operation_id(),
    );
    let members = match response {
        Ok(response) => response.members.unwrap_or_default(),
        Err(error) => {
            return Some(format!(
                "the workspace's members cannot be listed: {}",
                error.message
            ));
        }
    };
    for member in members {
        if !member.materialized {
            continue;
        }
        let Ok(relative) = Path::new(&member.abspath).strip_prefix(source) else {
            continue;
        };
        let in_lane = lane.join(relative);
        if !in_lane.is_dir() {
            return Some(format!("`{}` is missing", member.id));
        }
        if !in_lane.join(".git").exists() {
            return Some(format!("`{}` is not a repository", member.id));
        }
    }
    None
}

/// `gwz local clone <name>` at the family's default destination, in process,
/// with this session recorded as the owner (R20) and progress on stderr.
pub(crate) fn clone_lane(
    root: &Path,
    name: &str,
    session_id: &str,
) -> Result<(), gwz_core::model::ModelError> {
    let backend = gwz_core::git::Git2Backend::new();
    let progress = StderrProgressSink::new("cloning");
    let mut request = gwz_core::CloneLocalWorkspaceRequest {
        meta: meta(root),
        name: name.to_owned(),
        // Absent means core's `../<root-dirname>-<name>` default (D2).
        dest: None,
        mode: gwz_core::LocalCloneMode::Verbatim,
        branch: None,
        copy_source: None,
        owner: None,
        // The create hook owns its attempt loop and never waits inside the
        // clone (D6): a busy lock comes straight back and the loop decides.
        wait_seconds: None,
    };
    owner(OwnerOp::Attach(&mut request, session_id));
    gwz_core::workspace_ops::handle_clone_local_workspace(
        &backend,
        root,
        request,
        new_operation_id(),
        &progress,
    )?;
    Ok(())
}

/// `gwz local dispose <name>`, in process, from the family root and never
/// from inside the lane. Never `--keep`, never `--force` (D3).
///
/// Returns who owned the wait, so the caller knows whether to poll itself.
pub(crate) fn dispose_lane(
    family_root: &Path,
    name: &str,
    wait: std::time::Duration,
) -> (
    WaitOwner,
    Result<gwz_core::LocalFamilyResponse, gwz_core::model::ModelError>,
) {
    let backend = gwz_core::git::Git2Backend::new();
    let events = gwz_core::operation::NullSink;
    let mut request = gwz_core::LocalFamilyRequest {
        meta: meta(family_root),
        op: gwz_core::LocalFamilyOp::Dispose,
        name: Some(name.to_owned()),
        keep: None,
        force_hazards: Vec::new(),
        wait_seconds: None,
    };
    let wait_owner = dispose_wait(&mut request, wait);
    let result = gwz_core::workspace_ops::handle_local_family(
        &backend,
        family_root,
        request,
        new_operation_id(),
        &events,
    );
    (wait_owner, result)
}

/// A refusal that is the family lock held by another operation, rather than
/// a hazard or a shape error. gwz-core maps the store's `Busy` onto
/// `OpenOperation` and names the lock in the message (section 1).
pub(crate) fn is_family_busy(error: &gwz_core::model::ModelError) -> bool {
    error.code == gwz_core::model::ErrorCode::OpenOperation && error.message.contains("family lock")
}

/// A refusal that is core saying the family or the member is not there.
pub(crate) fn is_absent(error: &gwz_core::model::ModelError) -> bool {
    error.code == gwz_core::model::ErrorCode::MemberNotFound
}
