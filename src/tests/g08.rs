//! D5 cli unit coverage for `gwz diff`: argument lowering, unsupported-option
//! rejection, the (TTY-free) pager decision, and pager command resolution.

use std::path::Path;

use super::*;
use crate::pager::{PagerContext, PagerDecision, decide, resolve_pager_command};
use crate::tests::g01::strings;

/// Parse a `gwz diff …` argv and return the lowered [`DiffInvocation`].
fn diff_invocation(args: Vec<String>) -> Box<crate::DiffInvocation> {
    let invocation =
        parse_args_with_request_id(args, "req_test", Path::new("/cwd")).expect("diff args parse");
    match invocation.request {
        CliRequest::Diff(diff) => diff,
        other => panic!("expected diff request, got {other:?}"),
    }
}

// ── comparison-flag lowering ─────────────────────────────────────────────────

#[test]
pub(crate) fn plain_diff_lowers_to_worktree_request() {
    let diff = diff_invocation(strings(["diff"]));
    assert!(diff.request.operands.is_empty());
    assert!(diff.request.explicit_pathspecs.is_empty());
    assert_eq!(diff.request.cached, None);
    assert_eq!(diff.request.merge_base, None);
    assert!(!diff.quiet);
    assert!(!diff.exit_code);
}

#[test]
pub(crate) fn cached_and_staged_both_set_first_class_cached_field() {
    // --cached and its --staged alias both become the first-class request field,
    // never an operand tunnel (D0 invariant 3).
    for flag in ["--cached", "--staged"] {
        let diff = diff_invocation(strings(["diff", flag]));
        assert_eq!(diff.request.cached, Some(true), "{flag}");
        assert!(diff.request.operands.is_empty(), "{flag}");
    }
}

#[test]
pub(crate) fn merge_base_is_a_first_class_field() {
    let diff = diff_invocation(strings(["diff", "--merge-base", "main"]));
    assert_eq!(diff.request.merge_base, Some(true));
    assert_eq!(diff.request.operands, strings(["main"]));
}

#[test]
pub(crate) fn range_operand_stays_raw_for_core() {
    // A...B is kept raw in operands; core lowers it to merge_base per repo (D0 §7).
    let diff = diff_invocation(strings(["diff", "main...topic"]));
    assert_eq!(diff.request.operands, strings(["main...topic"]));
    assert_eq!(
        diff.request.merge_base, None,
        "client must not pre-resolve ..."
    );
}

#[test]
pub(crate) fn snapshot_operand_before_dashes_is_preserved_raw() {
    let diff = diff_invocation(strings(["diff", "+start-project"]));
    assert_eq!(diff.request.operands, strings(["+start-project"]));
}

#[test]
pub(crate) fn pathspecs_after_double_dash_are_separated_from_operands() {
    let diff = diff_invocation(strings([
        "diff",
        "HEAD",
        "--",
        "gwz-core/src",
        "+notes.txt",
    ]));
    assert_eq!(diff.request.operands, strings(["HEAD"]));
    // A leading `+` after `--` is a literal path, never a snapshot operand.
    assert_eq!(
        diff.request.explicit_pathspecs,
        strings(["gwz-core/src", "+notes.txt"])
    );
}

// ── option lowering ──────────────────────────────────────────────────────────

#[test]
pub(crate) fn output_format_flags_lower_to_wire_format() {
    let cases = [
        ("--stat", gwz_core::DiffOutputFormat::Stat),
        ("--numstat", gwz_core::DiffOutputFormat::Numstat),
        ("--shortstat", gwz_core::DiffOutputFormat::Shortstat),
        ("--summary", gwz_core::DiffOutputFormat::Summary),
        ("--name-only", gwz_core::DiffOutputFormat::NameOnly),
        ("--name-status", gwz_core::DiffOutputFormat::NameStatus),
        ("--raw", gwz_core::DiffOutputFormat::Raw),
    ];
    for (flag, format) in cases {
        let diff = diff_invocation(strings(["diff", flag]));
        assert_eq!(diff.display_format, format, "{flag}");
    }
}

#[test]
pub(crate) fn mutually_exclusive_formats_are_rejected() {
    let err = parse_args_with_request_id(
        strings(["diff", "--stat", "--name-only"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        err.message.contains("mutually exclusive"),
        "{}",
        err.message
    );
    assert_eq!(err.code, Some(gwz_core::model::ErrorCode::InvalidRequest));
}

#[test]
pub(crate) fn find_renames_threshold_lowers_to_options() {
    for spec in ["-M90", "-M90%"] {
        let diff = diff_invocation(strings(["diff", spec]));
        let options = diff.request.options.unwrap();
        assert_eq!(options.find_renames, Some(true), "{spec}");
        assert_eq!(options.rename_threshold, Some(90), "{spec}");
    }
    // Bare -M enables detection with the default threshold.
    let diff = diff_invocation(strings(["diff", "-M"]));
    let options = diff.request.options.unwrap();
    assert_eq!(options.find_renames, Some(true));
    assert_eq!(options.rename_threshold, None);
}

#[test]
pub(crate) fn prefix_and_line_prefix_flags_lower_to_options() {
    let diff = diff_invocation(strings([
        "diff",
        "--src-prefix",
        "x/",
        "--dst-prefix",
        "y/",
        "--line-prefix",
        "> ",
    ]));
    let options = diff.request.options.unwrap();
    assert_eq!(options.src_prefix.as_deref(), Some("x/"));
    assert_eq!(options.dst_prefix.as_deref(), Some("y/"));
    assert_eq!(options.line_prefix.as_deref(), Some("> "));

    let no_prefix = diff_invocation(strings(["diff", "--no-prefix"]));
    assert_eq!(no_prefix.request.options.unwrap().no_prefix, Some(true));
}

#[test]
pub(crate) fn z_flag_sets_null_terminated_option() {
    let diff = diff_invocation(strings(["diff", "-z", "--name-only"]));
    assert_eq!(diff.request.options.unwrap().null_terminated, Some(true));
}

#[test]
pub(crate) fn quiet_implies_exit_code_and_no_patch_any_difference() {
    let diff = diff_invocation(strings(["diff", "--quiet"]));
    assert!(diff.quiet);
    // --quiet implies --exit-code (AD8).
    assert!(diff.exit_code);
    let options = diff.request.options.unwrap();
    // Fast path: any_difference manifest mode, no byte log.
    assert_eq!(
        options.manifest_mode,
        Some(gwz_core::DiffManifestMode::AnyDifference)
    );
    assert_eq!(
        options.output_format,
        Some(gwz_core::DiffOutputFormat::NoPatch)
    );
}

#[test]
pub(crate) fn exit_code_alone_keeps_patch_format() {
    let diff = diff_invocation(strings(["diff", "--exit-code"]));
    assert!(diff.exit_code);
    assert!(!diff.quiet);
    // Still a patch format (--exit-code prints the patch, then exits 1/0).
    assert_eq!(diff.display_format, gwz_core::DiffOutputFormat::Patch);
}

#[test]
pub(crate) fn unsupported_git_option_is_rejected_by_clap() {
    // Unsupported diff knobs (e.g. --word-diff, -C/--find-copies) are unknown to
    // the parser, so they are rejected rather than silently ignored (D0).
    for arg in ["--word-diff", "--find-copies", "-C"] {
        let err = parse_args_with_request_id(strings(["diff", arg]), "req_test", Path::new("/cwd"))
            .unwrap_err();
        assert!(!err.message.is_empty(), "{arg} should be rejected");
    }
}

// ── pager decision (TTY-free, unit-testable) ─────────────────────────────────

fn ctx(stdout_is_tty: bool) -> PagerContext {
    PagerContext {
        stdout_is_tty,
        no_pager: false,
        machine_output: false,
        quiet: false,
        null_terminated: false,
    }
}

#[test]
pub(crate) fn pager_launches_only_for_human_patch_on_a_tty() {
    assert_eq!(decide(ctx(true)), PagerDecision::Pager);
    // Not a TTY (piped) → direct, never a pager.
    assert_eq!(decide(ctx(false)), PagerDecision::Direct);
}

#[test]
pub(crate) fn pager_suppressed_by_no_pager_quiet_machine_and_z() {
    let base = ctx(true);
    assert_eq!(
        decide(PagerContext {
            no_pager: true,
            ..base
        }),
        PagerDecision::Direct
    );
    assert_eq!(
        decide(PagerContext {
            quiet: true,
            ..base
        }),
        PagerDecision::Direct
    );
    assert_eq!(
        decide(PagerContext {
            machine_output: true,
            ..base
        }),
        PagerDecision::Direct
    );
    assert_eq!(
        decide(PagerContext {
            null_terminated: true,
            ..base
        }),
        PagerDecision::Direct
    );
}

// ── pager quit → ProducerStop ────────────────────────────────────────────────

#[test]
pub(crate) fn reader_ending_its_stream_stops_the_producer() {
    // This is the pager-quit → ProducerStop contract the CLI relies on: when the
    // consumer (pager/pipe) goes away, `diff_exec` ends the reader's log stream;
    // under stop_when=last_reader that fires ProducerStop, which the producer's
    // render loop observes via `should_stop` and halts (D6). Here we drive the
    // core log directly to prove the mechanism the CLI reader depends on.
    use gwz_core::diff::DiffLogRegistry;

    use gwz_core::diff::LogReadRequest;

    let registry = DiffLogRegistry::new();
    let (log_id, log) = registry.create();
    // Seed one record and establish a reader stream (a probe read registers it).
    log.push(vec![1, 2, 3]);
    let stream = "diff-out-test";
    let _ = registry.read(
        &log_id,
        &LogReadRequest {
            stream_id: stream.to_owned(),
            // Probe (timeout 0) so the read returns immediately after registering.
            timeout_ms: Some(0),
            max_records: Some(8),
            ..Default::default()
        },
    );
    assert!(!log.should_stop(), "producer should not be stopped yet");

    // The consumer went away — the CLI reader ends its stream on BrokenPipe.
    log.end_stream(stream);

    // The last reader leaving trips ProducerStop, so the producer would halt.
    assert!(
        log.should_stop(),
        "ending the last reader's stream must signal ProducerStop"
    );
}

#[test]
pub(crate) fn pager_command_resolution_prefers_git_pager_then_pager_then_less() {
    assert_eq!(
        resolve_pager_command(Some("delta"), Some("more")),
        Some("delta".to_owned())
    );
    assert_eq!(
        resolve_pager_command(None, Some("more")),
        Some("more".to_owned())
    );
    assert_eq!(
        resolve_pager_command(None, None),
        Some("less -FRX".to_owned())
    );
    // `cat` (or empty) means effective passthrough → no pager process.
    assert_eq!(resolve_pager_command(Some("cat"), None), None);
    assert_eq!(resolve_pager_command(Some("  "), Some("cat")), None);
}
