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
        },
        new_operation_id(),
        &events,
    )?;
    Ok(FamilyRead {
        root_path: response.root_path,
        members: response.members,
    })
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
