//! `gwz diff` argument surface + request lowering (D5, cli half).
//!
//! A git-like operand parser: revisions/ranges/`+snapshot`s stay **raw** in
//! `operands` for the core to classify (AD9/D0 §4), pathspecs after `--` go in
//! `pathspecs`, and every supported presentation/filter flag is lowered onto the
//! structured [`DiffRequest`](gwz_core::DiffRequest) / `DiffOptions`. Comparison
//! flags (`--cached`/`--staged`, `--merge-base`) become first-class request
//! fields, never operand tunnels (D0 invariant 3).
//!
//! Unsupported git options are rejected with `invalid_request` (per D0) *before*
//! the request is built, so a user sees a clear diagnostic rather than a silent
//! downgrade.

use clap::Args;

use crate::{CliError, CliRequest};

/// `gwz diff [<options>] [<operand>…] [-- <pathspec>…]`.
///
/// `operands` holds the raw positional tokens *before* `--` (revisions, ranges,
/// `+snapshot`s — all classified by core per repo). `pathspecs` holds the
/// literal pathspecs *after* `--`. The two are kept apart by clap's `last = true`
/// on `pathspecs`, matching `gwz forall`'s `-- <cmd>` split and git's `--`.
#[derive(Clone, Debug, Default, Args)]
pub(crate) struct DiffArgs {
    // ── comparison selectors (first-class request fields) ────────────────────
    #[arg(
        long = "cached",
        alias = "staged",
        help = "Diff the index against HEAD (git diff --cached/--staged)"
    )]
    pub(crate) cached: bool,

    #[arg(
        long = "merge-base",
        help = "Use the merge base of the operand and HEAD as the old side"
    )]
    pub(crate) merge_base: bool,

    // ── rename detection ─────────────────────────────────────────────────────
    #[arg(
        short = 'M',
        long = "find-renames",
        value_name = "n",
        num_args = 0..=1,
        default_missing_value = "",
        help = "Detect renames; optional similarity threshold (e.g. -M90 or -M90%)"
    )]
    pub(crate) find_renames: Option<String>,

    // ── output format (mutually-exclusive porcelain selectors) ───────────────
    #[arg(long = "stat", help = "Show a diffstat instead of a patch")]
    pub(crate) stat: bool,

    #[arg(
        long = "numstat",
        help = "Machine-readable diffstat (added/deleted/path)"
    )]
    pub(crate) numstat: bool,

    #[arg(long = "shortstat", help = "Only the summary line of --stat")]
    pub(crate) shortstat: bool,

    #[arg(long = "summary", help = "Condensed creation/rename/mode summary")]
    pub(crate) summary: bool,

    #[arg(long = "name-only", help = "Show only names of changed files")]
    pub(crate) name_only: bool,

    #[arg(long = "name-status", help = "Show names and status of changed files")]
    pub(crate) name_status: bool,

    #[arg(long = "raw", help = "Show the diff in raw format")]
    pub(crate) raw: bool,

    // ── patch shaping ────────────────────────────────────────────────────────
    #[arg(
        short = 'z',
        help = "NUL line-terminate name/status/raw records (git diff -z)"
    )]
    pub(crate) null_terminated: bool,

    #[arg(
        short = 'U',
        long = "unified",
        value_name = "n",
        help = "Generate diffs with <n> lines of context"
    )]
    pub(crate) unified: Option<i64>,

    #[arg(
        long = "src-prefix",
        value_name = "prefix",
        help = "Show the given source prefix instead of \"a/\""
    )]
    pub(crate) src_prefix: Option<String>,

    #[arg(
        long = "dst-prefix",
        value_name = "prefix",
        help = "Show the given destination prefix instead of \"b/\""
    )]
    pub(crate) dst_prefix: Option<String>,

    #[arg(
        long = "no-prefix",
        help = "Do not show any source or destination prefix"
    )]
    pub(crate) no_prefix: bool,

    #[arg(
        long = "line-prefix",
        value_name = "prefix",
        help = "Prepend an additional prefix to every line of output"
    )]
    pub(crate) line_prefix: Option<String>,

    // ── exit-status control ──────────────────────────────────────────────────
    #[arg(
        long = "exit-code",
        help = "Exit 1 if differences exist, 0 otherwise (like git diff --exit-code)"
    )]
    pub(crate) exit_code: bool,

    #[arg(long = "quiet", help = "Suppress all output; implies --exit-code")]
    pub(crate) quiet: bool,

    // ── pager control ────────────────────────────────────────────────────────
    #[arg(
        long = "no-pager",
        help = "Do not pipe human patch output through a pager"
    )]
    pub(crate) no_pager: bool,

    // ── raw operands / pathspecs ─────────────────────────────────────────────
    #[arg(
        value_name = "operand",
        help = "Revisions, ranges (A..B, A...B), or +snapshot ids. Classified by core. Put pathspecs after `--`.",
        allow_hyphen_values = false
    )]
    pub(crate) operands: Vec<String>,

    #[arg(
        last = true,
        value_name = "pathspec",
        help = "Literal pathspecs, resolved relative to the current directory (a leading `+` here is a path, not a snapshot)."
    )]
    pub(crate) pathspecs: Vec<String>,
}

impl DiffArgs {
    /// Lower parsed args into a [`CliRequest::Diff`]. Rejects unsupported /
    /// contradictory combinations with `invalid_request` before building the
    /// request. `workspace_cwd` is the workspace-relative logical cwd (AD10),
    /// computed by the caller from the physical cwd and the resolved root.
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_cwd: String,
    ) -> Result<CliRequest, CliError> {
        let format = self.output_format()?;
        let quiet = self.quiet;
        // --quiet implies --exit-code (AD8).
        let exit_code = self.exit_code || quiet;

        let (find_renames, rename_threshold) = self.rename_options()?;

        // --quiet answers purely from the manifest via any_difference (fast path).
        let manifest_mode = if quiet {
            Some(gwz_core::DiffManifestMode::AnyDifference)
        } else {
            None
        };
        // Under --quiet the format is forced to no_patch (no byte log at all).
        let effective_format = if quiet {
            Some(gwz_core::DiffOutputFormat::NoPatch)
        } else {
            format
        };

        let options = gwz_core::DiffOptions {
            output_format: effective_format,
            context_lines: self.unified,
            find_renames,
            rename_threshold,
            null_terminated: self.null_terminated.then_some(true),
            src_prefix: self.src_prefix.clone(),
            dst_prefix: self.dst_prefix.clone(),
            no_prefix: self.no_prefix.then_some(true),
            line_prefix: self.line_prefix.clone(),
            manifest_mode,
            ..Default::default()
        };

        Ok(CliRequest::Diff(Box::new(DiffInvocation {
            request: gwz_core::DiffRequest {
                meta,
                workspace_cwd: Some(workspace_cwd),
                operands: self.operands.clone(),
                explicit_pathspecs: self.pathspecs.clone(),
                options: Some(options),
                cached: self.cached.then_some(true),
                merge_base: self.merge_base.then_some(true),
            },
            display_format: format.unwrap_or(gwz_core::DiffOutputFormat::Patch),
            quiet,
            exit_code,
            no_pager: self.no_pager,
        })))
    }

    /// Resolve the mutually-exclusive output-format selectors into a wire format.
    /// `None` means "the default patch format" (kept distinct from an explicit
    /// selector so `--quiet` can override cleanly). More than one selector is an
    /// `invalid_request`.
    fn output_format(&self) -> Result<Option<gwz_core::DiffOutputFormat>, CliError> {
        use gwz_core::DiffOutputFormat as F;
        let mut chosen: Vec<(&str, F)> = Vec::new();
        if self.stat {
            chosen.push(("--stat", F::Stat));
        }
        if self.numstat {
            chosen.push(("--numstat", F::Numstat));
        }
        if self.shortstat {
            chosen.push(("--shortstat", F::Shortstat));
        }
        if self.summary {
            chosen.push(("--summary", F::Summary));
        }
        if self.name_only {
            chosen.push(("--name-only", F::NameOnly));
        }
        if self.name_status {
            chosen.push(("--name-status", F::NameStatus));
        }
        if self.raw {
            chosen.push(("--raw", F::Raw));
        }
        match chosen.as_slice() {
            [] => Ok(None),
            [(_, format)] => Ok(Some(*format)),
            [(a, _), (b, _), ..] => Err(CliError::invalid_request(format!(
                "{a} and {b} are mutually exclusive"
            ))),
        }
    }

    /// Map `-M[<n>]` onto `find_renames` + `rename_threshold`. `-M` with no value
    /// enables detection at the default threshold; `-M90` / `-M90%` sets 90.
    fn rename_options(&self) -> Result<(Option<bool>, Option<i64>), CliError> {
        let Some(spec) = &self.find_renames else {
            return Ok((None, None));
        };
        let trimmed = spec.trim().trim_end_matches('%');
        if trimmed.is_empty() {
            return Ok((Some(true), None));
        }
        let threshold: i64 = trimmed
            .parse()
            .map_err(|_| CliError::invalid_request(format!("invalid rename threshold '{spec}'")))?;
        if !(0..=100).contains(&threshold) {
            return Err(CliError::invalid_request(format!(
                "rename threshold must be between 0 and 100, got {threshold}"
            )));
        }
        Ok((Some(true), Some(threshold)))
    }
}

/// A fully-lowered `gwz diff` invocation: the wire request plus the client-side
/// presentation decisions (display format, exit-code policy, pager policy) that
/// live outside the core protocol (AD6).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DiffInvocation {
    pub(crate) request: gwz_core::DiffRequest,
    /// The format the user asked for (patch by default) — drives whether the
    /// client reads the byte log and how it renders manifest-only modes.
    pub(crate) display_format: gwz_core::DiffOutputFormat,
    pub(crate) quiet: bool,
    pub(crate) exit_code: bool,
    pub(crate) no_pager: bool,
}
