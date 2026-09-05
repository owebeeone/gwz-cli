use clap::Args;

use crate::*;

use super::*;

#[derive(Clone, Debug, Args)]
pub(crate) struct CloneArgs {
    // With `--local` there is no URL: the first positional is the destination
    // directory instead (design §4). Clap keeps its own required-argument error
    // for the ordinary URL clone through `required_unless_present`.
    #[arg(
        value_name = "url",
        required_unless_present = "local",
        help = "Git URL of the workspace root repository",
        long_help = "Git URL of the workspace root repository. With --local there is no URL: this positional is the destination directory of the new local clone."
    )]
    pub(crate) url: Option<String>,

    #[arg(
        value_name = "directory",
        help = "Target directory for the cloned workspace",
        long_help = "Target directory for the cloned workspace. Defaults to a directory named after the workspace repository. Not accepted with --local, which takes a single destination positional."
    )]
    pub(crate) dir: Option<String>,

    #[arg(
        long,
        help = "Clone this workspace locally as a new family member",
        long_help = "Clone the current workspace into a second working copy on this machine and register it in the local clone family. Requires --name. Mutually exclusive with a workspace URL."
    )]
    pub(crate) local: bool,

    #[arg(
        long,
        value_name = "name",
        help = "Family member name for --local",
        long_help = "Family member name for the new local clone. Required with --local. `root`, `origin` and Git's reserved ref names are not accepted, and a name already recorded in the family is refused."
    )]
    pub(crate) name: Option<String>,

    #[arg(
        long,
        help = "Copy the source tree as it sits (the --local default)",
        long_help = "Copy the source tree as it sits, including staged edits, unstaged edits, untracked files and build directories. This is the default local mode. It is refused while the source has an open coordinated merge. Mutually exclusive with --clean and --bare."
    )]
    pub(crate) verbatim: bool,

    #[arg(
        long,
        help = "Check out the frozen source state without worktree dirt",
        long_help = "Check out the frozen source state in the destination: no worktree or index dirt is inherited, and no build directories are copied. Mutually exclusive with --verbatim."
    )]
    pub(crate) clean: bool,

    #[arg(
        long,
        help = "Create bare member repositories (implies --clean)",
        long_help = "Create the destination as a share point: the same workspace layout, with every member repository bare. Implies --clean. Verbs that need a worktree refuse there; push, fetch, log, `gwz local list` and dispose work."
    )]
    pub(crate) bare: bool,

    #[arg(
        short = 'b',
        value_name = "branch",
        help = "Create this branch in every destination repository (--clean/--bare only)",
        long_help = "Create this branch in every destination repository at the frozen commit, before the clone is marked ready. Accepted only with --clean or --bare. If the branch already exists in any member, the whole create is refused."
    )]
    pub(crate) branch: Option<String>,

    #[arg(
        long,
        value_name = "name|path",
        help = "Copy from this family member or path instead of the current workspace",
        long_help = "Copy from this family member or path instead of the current workspace. Not yet supported: the protocol field is unallocated pending an operator decision, so this flag is refused rather than silently ignored."
    )]
    pub(crate) from: Option<String>,
}

impl InitArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        if self.update {
            if !self.urls.is_empty() {
                return Err(CliError::new(
                    "--update cannot be combined with source URLs",
                ));
            }
            if !self.path_prefix.trim().is_empty() {
                return Err(CliError::new("--update cannot be combined with --path"));
            }
            Ok(CliRequest::UpdateBootstrap { meta })
        } else if self.urls.is_empty() {
            Ok(CliRequest::CreateWorkspace(
                gwz_core::CreateWorkspaceRequest {
                    meta,
                    workspace_root,
                    workspace_id: None,
                },
            ))
        } else {
            Ok(CliRequest::InitFromSources(
                gwz_core::InitFromSourcesRequest {
                    meta,
                    workspace_root,
                    sources: self
                        .urls
                        .iter()
                        .cloned()
                        .map(|url| {
                            Ok(gwz_core::SourceUrl {
                                path: init_source_path(&self.path_prefix, &url)?,
                                url,
                                remote_name: None,
                                branch: None,
                            })
                        })
                        .collect::<Result<Vec<_>, CliError>>()?,
                    target: Some(gwz_core::MaterializeTarget {
                        kind: gwz_core::MaterializeTargetKind::Head,
                        name: None,
                        commit: None,
                    }),
                    workspace_id: None,
                },
            ))
        }
    }
}

impl CloneArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        if self.local {
            self.local_request(meta)
        } else {
            self.url_request(meta)
        }
    }

    fn url_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        for (flag, present) in [
            ("--name", self.name.is_some()),
            ("--verbatim", self.verbatim),
            ("--clean", self.clean),
            ("--bare", self.bare),
            ("-b <branch>", self.branch.is_some()),
            ("--from", self.from.is_some()),
        ] {
            if present {
                return Err(CliError::invalid_request(format!(
                    "{flag} is accepted only with --local"
                )));
            }
        }
        // `required_unless_present = "local"` already made Clap answer for a
        // missing URL here; this is the type-level remainder.
        let url = self
            .url
            .clone()
            .ok_or_else(|| CliError::invalid_request("clone requires a workspace URL"))?;
        let target = match &self.dir {
            Some(dir) => dir.clone(),
            None => repo_name_from_url(&url)?,
        };
        Ok(CliRequest::CloneWorkspace { meta, url, target })
    }

    /// `gwz clone --local --name <name> [dest]` (design §4). Every refusal
    /// below happens before the request is built; core owns name validity,
    /// destination defaulting and every workspace check.
    fn local_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let Some(name) = self.name.clone() else {
            return Err(CliError::invalid_request(
                "clone --local requires --name <name>",
            ));
        };
        if self.dir.is_some() {
            return Err(CliError::invalid_request(
                "clone --local accepts a single destination directory, not a workspace URL",
            ));
        }
        if self.verbatim && self.clean {
            return Err(CliError::invalid_request(
                "--verbatim and --clean are mutually exclusive",
            ));
        }
        if self.verbatim && self.bare {
            return Err(CliError::invalid_request(
                "--verbatim and --bare are mutually exclusive (--bare implies --clean)",
            ));
        }
        // Design §7 holds `CloneLocalWorkspaceRequest` tag 6 unallocated
        // pending an operator decision on its wire name, so the flag is
        // parsed and refused rather than dropped or encoded elsewhere.
        if self.from.is_some() {
            return Err(CliError::unsupported(
                "--from <name|path> is not yet supported by this gwz build",
            ));
        }
        let mode = if self.bare {
            gwz_core::LocalCloneMode::Bare
        } else if self.clean {
            gwz_core::LocalCloneMode::Clean
        } else {
            gwz_core::LocalCloneMode::Verbatim
        };
        if self.branch.is_some() && mode == gwz_core::LocalCloneMode::Verbatim {
            return Err(CliError::invalid_request(
                "-b <branch> is accepted only with --clean or --bare",
            ));
        }
        Ok(CliRequest::CloneLocalWorkspace(
            gwz_core::CloneLocalWorkspaceRequest {
                meta,
                name,
                // Absent means core's `../<root-dirname>-<name>` default.
                dest: self.url.clone(),
                mode,
                branch: self.branch.clone(),
                // Tag 6 `copy_source` (operator ruling 2026-09-05): `--from`
                // is still refused above; lane CR wires it in. Compile-required
                // literal only (LCM1.0c follow-up 2).
                copy_source: None,
            },
        ))
    }
}

pub(crate) fn init_source_path(path_prefix: &str, url: &str) -> Result<Option<String>, CliError> {
    let prefix = path_prefix
        .replace('\\', "/")
        .trim_matches(|value| value == '/')
        .to_owned();
    if prefix.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(format!("{prefix}/{}", repo_name_from_url(url)?)))
}

pub(crate) fn repo_name_from_url(url: &str) -> Result<String, CliError> {
    let trimmed = url.trim_end_matches(['/', '\\']);
    let segment = trimmed
        .rsplit(['/', '\\', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(trimmed);
    let name = segment.strip_suffix(".git").unwrap_or(segment);
    if name.is_empty() {
        Err(CliError::new(
            "source URL does not include a repository name",
        ))
    } else {
        Ok(name.to_owned())
    }
}
