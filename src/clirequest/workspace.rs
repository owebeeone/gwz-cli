use clap::Args;

use crate::*;

use super::*;

#[derive(Clone, Debug, Args)]
pub(crate) struct CloneArgs {
    #[arg(value_name = "url", help = "Git URL of the workspace root repository")]
    pub(crate) url: String,

    #[arg(
        value_name = "directory",
        help = "Target directory for the cloned workspace",
        long_help = "Target directory for the cloned workspace. Defaults to a directory named after the workspace repository."
    )]
    pub(crate) dir: Option<String>,
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
        let target = match &self.dir {
            Some(dir) => dir.clone(),
            None => repo_name_from_url(&self.url)?,
        };
        Ok(CliRequest::CloneWorkspace {
            meta,
            url: self.url.clone(),
            target,
        })
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
