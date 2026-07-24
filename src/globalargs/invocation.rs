#[cfg(test)]
use clap::Parser;

use crate::*;

#[cfg(test)]
pub(crate) fn parse_args_with_request_id(
    args: Vec<String>,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    let cli = Cli::try_parse_from(std::iter::once("gwz".to_owned()).chain(args))
        .map_err(|error| CliError::new(error.to_string()))?;
    invocation_from_cli(cli, request_id, current_dir)
}

pub(crate) fn invocation_from_cli(
    cli: Cli,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    cli.validate()?;
    let output = cli.output_mode();
    let meta = cli.request_meta(request_id);
    let workspace_root = cli
        .global
        .root
        .clone()
        .unwrap_or_else(|| current_dir.to_string_lossy().into_owned());
    let request = cli.command_request(meta, workspace_root, current_dir)?;
    Ok(CliInvocation {
        request,
        output,
        start_dir: current_dir.to_path_buf(),
    })
}

impl StatusArgs {
    pub(crate) fn validate(&self, global: &GlobalArgs) -> Result<(), CliError> {
        if self.porcelain && (global.json || global.jsonl) {
            return Err(CliError::new(
                "--porcelain cannot be combined with --json or --jsonl",
            ));
        }
        if self.no_files && self.no_branches {
            return Err(CliError::new(
                "--no-files and --no-branches cannot both be supplied",
            ));
        }
        if self.combined && self.no_combined {
            return Err(CliError::new(
                "--combined and --no-combined cannot both be supplied",
            ));
        }
        if self.porcelain && self.no_combined {
            return Err(CliError::new(
                "--porcelain cannot be combined with --no-combined",
            ));
        }
        if self.no_combined && (self.no_files || self.no_branches) {
            return Err(CliError::new(
                "--no-files and --no-branches can only be used with combined status",
            ));
        }
        Ok(())
    }

    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let combined = !self.no_combined;
        Ok(CliRequest::Status(gwz_core::StatusRequest {
            meta,
            mode: Some(if combined {
                gwz_core::StatusMode::Combined
            } else {
                gwz_core::StatusMode::Summary
            }),
            include_file_changes: Some(if combined { !self.no_files } else { true }),
            include_branch_summary: if combined {
                Some(!self.no_branches)
            } else {
                Some(true)
            },
            path_style: Some(gwz_core::StatusPathStyle::WorkspaceRelative),
        }))
    }
}

pub(crate) fn new_request_id() -> String {
    format!("req_{}", unique_suffix())
}
