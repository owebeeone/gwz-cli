//! `gwz log` argument surface and lowering into the core commit-log request.
//!
//! The CLI preserves raw operands and post-`--` pathspecs. Range, time, regex,
//! and filtering semantics remain wholly owned by `gwz-core`.

use clap::{Args, ValueEnum};

use crate::{CliError, CliRequest, parse_non_negative_i64};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum LogColor {
    Always,
    Never,
    #[default]
    Auto,
}

/// `gwz log [OPTIONS] [operand]... [-- <pathspec>...]`.
#[derive(Clone, Debug, Default, Args)]
pub(crate) struct LogArgs {
    #[arg(
        short = 'n',
        value_name = "n",
        value_parser = parse_non_negative_i64,
        conflicts_with = "no_limit",
        help = "Limit the global result to N entries (0 disables the limit)"
    )]
    pub(crate) max_entries: Option<i64>,

    #[arg(long, help = "Disable the global result limit")]
    pub(crate) no_limit: bool,

    #[arg(
        long,
        value_name = "time",
        help = "Include commits at or after TIME (RFC3339/ISO-8601; date-only is local midnight, offset-less is local, or use @epoch-seconds)"
    )]
    pub(crate) since: Option<String>,

    #[arg(
        long,
        value_name = "time",
        help = "Include commits at or before TIME (RFC3339/ISO-8601; date-only is local midnight, offset-less is local, or use @epoch-seconds)"
    )]
    pub(crate) until: Option<String>,

    #[arg(
        long,
        value_name = "regex",
        help = "Match a case-sensitive Rust regex (not Git regex syntax) against `Name <email>`"
    )]
    pub(crate) author: Option<String>,

    #[arg(
        long,
        value_name = "regex",
        help = "Match a case-sensitive Rust regex (not Git regex syntax) against the full raw commit message"
    )]
    pub(crate) grep: Option<String>,

    #[arg(long, help = "Exclude merge commits before workspace coalescing")]
    pub(crate) no_merges: bool,

    #[arg(long, help = "Follow only each commit's first parent")]
    pub(crate) first_parent: bool,

    #[arg(long, help = "Promote any selected-repository degradation to failure")]
    pub(crate) strict: bool,

    #[arg(long, help = "Disable workspace-level commit coalescing")]
    pub(crate) no_coalesce: bool,

    #[arg(
        long,
        help = "Include commit message bodies in --full and machine output"
    )]
    pub(crate) body: bool,

    #[arg(long, help = "Render git-style blocks with a complete member table")]
    pub(crate) full: bool,

    #[arg(
        long,
        help = "Select only repositories containing every supplied local tag"
    )]
    pub(crate) tagged: bool,

    #[arg(
        long,
        value_enum,
        default_value = "auto",
        value_name = "when",
        help = "Colorize output: always, never, or auto"
    )]
    pub(crate) color: LogColor,

    #[arg(
        value_name = "operand",
        help = "Revisions, ranges, or +snapshot ids; classified by core. Put pathspecs after `--`."
    )]
    pub(crate) operands: Vec<String>,

    #[arg(
        last = true,
        value_name = "pathspec",
        help = "Literal pathspecs relative to the invocation directory"
    )]
    pub(crate) pathspecs: Vec<String>,
}

impl LogArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_cwd: String,
    ) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Log(Box::new(LogInvocation {
            request: gwz_core::LogRequest {
                meta,
                workspace_cwd: Some(workspace_cwd),
                operands: self.operands.clone(),
                explicit_pathspecs: self.pathspecs.clone(),
                options: Some(gwz_core::LogOptions {
                    max_entries: if self.no_limit {
                        Some(0)
                    } else {
                        self.max_entries
                    },
                    since: self.since.clone(),
                    until: self.until.clone(),
                    author: self.author.clone(),
                    grep: self.grep.clone(),
                    no_merges: self.no_merges.then_some(true),
                    first_parent: self.first_parent.then_some(true),
                    strict: self.strict.then_some(true),
                    coalesce: self.no_coalesce.then_some(false),
                    include_body: self.body.then_some(true),
                }),
                tagged: self.tagged.then_some(true),
            },
            color: self.color,
            full: self.full,
        })))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogInvocation {
    pub(crate) request: gwz_core::LogRequest,
    pub(crate) color: LogColor,
    pub(crate) full: bool,
}
