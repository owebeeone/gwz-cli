use clap::{Args, Subcommand};

use crate::*;

use super::*;

#[derive(Clone, Debug, Args)]
pub(crate) struct AddArgs {
    #[arg(
        value_name = "repo-path",
        help = "Path to an existing local git repository"
    )]
    pub(crate) repo_path: String,

    #[arg(
        long,
        value_name = "member-id",
        help = "Explicit member designation id"
    )]
    pub(crate) member_id: Option<String>,

    #[arg(long, value_name = "source-id", help = "Explicit logical source id")]
    pub(crate) source_id: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum RepoCommandArgs {
    #[command(
        about = "Add an existing git repository as a member",
        long_about = ADD_LONG,
        after_long_help = ADD_AFTER
    )]
    Add(AddArgs),
    #[command(
        about = "Clone and register a new repository member",
        long_about = REPO_CLONE_LONG,
        after_long_help = REPO_CLONE_AFTER
    )]
    Clone(RepoCloneArgs),
    #[command(
        about = "Create a new repository member",
        long_about = REPO_CREATE_LONG,
        after_long_help = REPO_CREATE_AFTER
    )]
    Create(RepoCreateArgs),
    #[command(
        about = "Detach a repository member without deleting its checkout",
        long_about = REPO_DETACH_LONG,
        after_long_help = REPO_DETACH_AFTER
    )]
    Detach(RepoDetachArgs),
    #[command(
        about = "Reattach an inactive repository designation",
        long_about = REPO_ATTACH_LONG,
        after_long_help = REPO_ATTACH_AFTER
    )]
    Attach(RepoAttachArgs),
    #[command(
        about = "Refresh member metadata from local git config",
        long_about = REPO_SYNC_LONG,
        after_long_help = REPO_SYNC_AFTER
    )]
    Sync(RepoSyncArgs),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoCreateArgs {
    #[arg(
        value_name = "member-path",
        help = "Workspace-relative path for the new repository member"
    )]
    pub(crate) member_path: String,

    #[arg(
        long,
        value_name = "member-id",
        help = "Explicit member designation id"
    )]
    pub(crate) member_id: Option<String>,

    #[arg(long, value_name = "source-id", help = "Explicit logical source id")]
    pub(crate) source_id: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoCloneArgs {
    #[arg(value_name = "url", help = "Git URL of the repository to clone")]
    pub(crate) url: String,

    #[arg(
        value_name = "member-path",
        help = "Workspace-relative target path; defaults from the URL"
    )]
    pub(crate) member_path: Option<String>,

    #[arg(
        long,
        value_name = "member-id",
        help = "Explicit member designation id"
    )]
    pub(crate) member_id: Option<String>,

    #[arg(long, value_name = "source-id", help = "Explicit logical source id")]
    pub(crate) source_id: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoDetachArgs {
    #[arg(
        value_name = "member",
        help = "Active member id or workspace-relative path"
    )]
    pub(crate) member: String,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoAttachArgs {
    #[arg(value_name = "member-id", help = "Inactive member designation id")]
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct RepoSyncArgs {
    #[arg(
        value_name = "member-path",
        help = "Workspace-relative member path to sync"
    )]
    pub(crate) member_path: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct CommitArgs {
    #[arg(
        short = 'm',
        long,
        value_name = "message",
        help = "Commit message applied to every committed repo"
    )]
    pub(crate) message: String,

    #[arg(
        short = 'a',
        long,
        help = "Stage tracked modifications first (git commit -a)"
    )]
    pub(crate) all: bool,

    #[arg(
        long = "commit-marker",
        conflicts_with = "no_commit_marker",
        help = "Create and persist a GWZ commit marker"
    )]
    pub(crate) commit_marker: bool,

    #[arg(
        long = "no-commit-marker",
        help = "Disable GWZ commit marker creation for this commit"
    )]
    pub(crate) no_commit_marker: bool,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct StageArgs {
    #[arg(
        value_name = "pathspec",
        help = "Paths to stage; resolved relative to the current directory like `git add`"
    )]
    pub(crate) pathspecs: Vec<String>,

    #[arg(
        short = 'A',
        long = "all",
        help = "Stage all changes across every workspace repo (git add -A)"
    )]
    pub(crate) all: bool,
}

impl AddArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::AddExistingRepo(
            gwz_core::AddExistingRepoRequest {
                meta,
                repository_path: self.repo_path.clone(),
                member_path: None,
                member_id: self.member_id.clone(),
                source_id: self.source_id.clone(),
            },
        ))
    }
}

impl RepoArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match &self.command {
            RepoCommandArgs::Add(args) => args.request(meta),
            RepoCommandArgs::Clone(args) => args.request(meta),
            RepoCommandArgs::Create(args) => {
                Ok(CliRequest::CreateRepo(gwz_core::CreateRepoRequest {
                    meta,
                    member_path: args.member_path.clone(),
                    initial_branch: None,
                    member_id: args.member_id.clone(),
                    source_id: args.source_id.clone(),
                }))
            }
            RepoCommandArgs::Detach(args) => args.request(meta),
            RepoCommandArgs::Attach(args) => args.request(meta),
            RepoCommandArgs::Sync(args) => args.request(meta),
        }
    }
}

impl RepoCloneArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::CloneRepoMember(
            gwz_core::CloneRepoMemberRequest {
                meta,
                source: gwz_core::SourceUrl {
                    url: self.url.clone(),
                    path: self.member_path.clone(),
                    remote_name: None,
                    branch: None,
                },
                member_id: self.member_id.clone(),
                source_id: self.source_id.clone(),
            },
        ))
    }
}

impl RepoDetachArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let meta = single_repo_lifecycle_selector(meta, &self.member, "repo detach")?;
        Ok(CliRequest::DetachRepoMember(
            gwz_core::DetachRepoMemberRequest { meta },
        ))
    }
}

impl RepoAttachArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        gwz_core::model::MemberId::parse_str(&self.member_id).map_err(|_| {
            CliError::new(
                "repo attach requires a member id starting with mem_ and portable characters",
            )
        })?;
        let meta = single_repo_lifecycle_selector(meta, &self.member_id, "repo attach")?;
        Ok(CliRequest::AttachRepoMember(
            gwz_core::AttachRepoMemberRequest { meta },
        ))
    }
}

fn single_repo_lifecycle_selector(
    mut meta: gwz_core::RequestMeta,
    selector: &str,
    command: &str,
) -> Result<gwz_core::RequestMeta, CliError> {
    if meta.selection.is_some() {
        return Err(CliError::new(format!(
            "{command} member cannot be combined with global selection"
        )));
    }
    meta.selection = Some(gwz_core::Selection {
        targets: vec![selector.to_owned()],
        ..Default::default()
    });
    Ok(meta)
}

impl RepoSyncArgs {
    pub(crate) fn request(&self, mut meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        if let Some(member_path) = &self.member_path {
            if meta.selection.is_some() {
                return Err(CliError::new(
                    "repo sync member path cannot be combined with global selection",
                ));
            }
            meta.selection = Some(gwz_core::Selection {
                targets: vec![member_path.clone()],
                ..Default::default()
            });
        }
        Ok(CliRequest::RepoSync(gwz_core::RepoSyncRequest { meta }))
    }
}

impl CommitArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Commit(gwz_core::CommitRequest {
            meta,
            message: self.message.clone(),
            all: self.all.then_some(true),
            commit_marker: if self.commit_marker {
                Some(true)
            } else if self.no_commit_marker {
                Some(false)
            } else {
                None
            },
        }))
    }
}

impl LsArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Ls {
            request: gwz_core::LsRequest {
                meta,
                include_unmaterialized: self.unmaterialized.then_some(true),
            },
            local: self.local,
        })
    }
}

impl StageArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        cwd: &std::path::Path,
    ) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Stage(gwz_core::StageRequest {
            meta,
            cwd: cwd.to_string_lossy().into_owned(),
            pathspecs: self.pathspecs.clone(),
            all: self.all.then_some(true),
        }))
    }
}
