use clap::Args;

use crate::*;

use super::*;

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct MaterializeArgs {
    #[arg(
        long,
        help = "Materialize the workspace lock",
        long_help = "Materialize the workspace lock. This is the default target."
    )]
    pub(crate) lock: bool,

    #[arg(long, help = "Materialize repository heads")]
    pub(crate) head: bool,

    #[arg(long, value_name = "name", help = "Materialize a workspace snapshot")]
    pub(crate) snapshot: Option<String>,

    #[arg(long, value_name = "name", help = "Materialize a workspace tag")]
    pub(crate) tag: Option<String>,

    #[arg(
        long = "switch",
        value_name = "branch",
        help = "Switch workspace members to a branch"
    )]
    pub(crate) switch: Option<String>,
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct PullArgs {
    #[arg(
        long,
        help = "Pull repository heads",
        long_help = "Pull repository heads. This is the default target."
    )]
    pub(crate) head: bool,

    #[arg(long, value_name = "name", help = "Pull a workspace snapshot")]
    pub(crate) snapshot: Option<String>,
}

impl MaterializeArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Materialize(gwz_core::MaterializeRequest {
            meta,
            target: self.target()?,
        }))
    }

    pub(crate) fn target(&self) -> Result<gwz_core::MaterializeTarget, CliError> {
        let targets = usize::from(self.lock)
            + usize::from(self.head)
            + usize::from(self.snapshot.is_some())
            + usize::from(self.tag.is_some())
            + usize::from(self.switch.is_some());
        if targets > 1 {
            return Err(CliError::new("only one target flag may be supplied"));
        }
        if self.head {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Head,
                name: None,
                commit: None,
            })
        } else if let Some(name) = &self.snapshot {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Snapshot,
                name: Some(name.clone()),
                commit: None,
            })
        } else if let Some(name) = &self.tag {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Tag,
                name: Some(name.clone()),
                commit: None,
            })
        } else if let Some(name) = &self.switch {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Branch,
                name: Some(name.clone()),
                commit: None,
            })
        } else {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Lock,
                name: None,
                commit: None,
            })
        }
    }
}

impl SnapshotArgs {
    pub(crate) fn source(&self) -> Option<gwz_core::SnapshotSource> {
        self.branch.as_ref().map(|branch| match branch {
            Some(name) => gwz_core::SnapshotSource {
                kind: gwz_core::SnapshotSourceKind::Branch,
                branch: Some(name.clone()),
            },
            None => gwz_core::SnapshotSource {
                kind: gwz_core::SnapshotSourceKind::Current,
                branch: None,
            },
        })
    }
}

impl PullArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match (self.head, self.snapshot.as_ref()) {
            (true, Some(_)) => Err(CliError::new("only one target flag may be supplied")),
            (_, Some(name)) => Ok(CliRequest::PullSnapshot(gwz_core::PullSnapshotRequest {
                meta,
                snapshot_id: name.clone(),
            })),
            _ => Ok(CliRequest::PullHead(gwz_core::PullHeadRequest { meta })),
        }
    }
}
