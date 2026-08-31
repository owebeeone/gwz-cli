//! S3.1 CLI coverage for `gwz log`: complete request lowering, aggregate exit
//! status, typed rejection routing, and no-pager broken-pipe cleanup.

use std::io;
use std::path::Path;

use clap::CommandFactory;

use super::*;
use crate::tests::g01::{TempDir, request_meta, strings};

fn log_invocation(args: Vec<String>, cwd: &Path) -> Box<crate::LogInvocation> {
    let invocation =
        parse_args_with_request_id(args, "req_log", cwd).expect("log arguments should parse");
    match invocation.request {
        CliRequest::Log(log) => log,
        other => panic!("expected log request, got {other:?}"),
    }
}

#[test]
fn cap_lowering_distinguishes_omitted_n_zero_and_no_limit() {
    assert_eq!(
        log_invocation(strings(["log"]), Path::new("/cwd"))
            .request
            .options
            .unwrap()
            .max_entries,
        None
    );
    assert_eq!(
        log_invocation(strings(["log", "-n", "17"]), Path::new("/cwd"))
            .request
            .options
            .unwrap()
            .max_entries,
        Some(17)
    );
    for args in [strings(["log", "-n", "0"]), strings(["log", "--no-limit"])] {
        assert_eq!(
            log_invocation(args, Path::new("/cwd"))
                .request
                .options
                .unwrap()
                .max_entries,
            Some(0)
        );
    }
}

#[test]
fn cap_conflict_and_negative_value_are_clap_rejections() {
    for args in [
        strings(["log", "-n", "1", "--no-limit"]),
        strings(["log", "-n", "-1"]),
    ] {
        let error = parse_args_with_request_id(args, "req_log", Path::new("/cwd"))
            .expect_err("invalid cap spelling must be rejected");
        assert!(!error.message.is_empty());
    }
}

#[test]
fn all_filter_and_behavior_flags_lower_without_client_semantics() {
    let log = log_invocation(
        strings([
            "log",
            "--since",
            "2026-08-01T02:03:04+10:00",
            "--until",
            "@1788000000",
            "--author",
            "Ada <ada@example.test>",
            "--grep",
            "fix(?P<area>core)",
            "--no-merges",
            "--first-parent",
            "--strict",
            "--no-coalesce",
            "--body",
            "--tagged",
        ]),
        Path::new("/cwd"),
    );
    let options = log.request.options.unwrap();
    assert_eq!(options.since.as_deref(), Some("2026-08-01T02:03:04+10:00"));
    assert_eq!(options.until.as_deref(), Some("@1788000000"));
    assert_eq!(options.author.as_deref(), Some("Ada <ada@example.test>"));
    assert_eq!(options.grep.as_deref(), Some("fix(?P<area>core)"));
    assert_eq!(options.no_merges, Some(true));
    assert_eq!(options.first_parent, Some(true));
    assert_eq!(options.strict, Some(true));
    assert_eq!(options.coalesce, Some(false));
    assert_eq!(options.include_body, Some(true));
    assert_eq!(log.request.tagged, Some(true));
}

#[test]
fn absent_behavior_flags_remain_wire_none() {
    let log = log_invocation(strings(["log"]), Path::new("/cwd"));
    let options = log.request.options.unwrap();
    assert_eq!(options.no_merges, None);
    assert_eq!(options.first_parent, None);
    assert_eq!(options.strict, None);
    assert_eq!(options.coalesce, None);
    assert_eq!(options.include_body, None);
    assert_eq!(log.request.tagged, None);
}

#[test]
fn operands_and_post_dash_pathspecs_stay_in_distinct_wire_fields() {
    let log = log_invocation(
        strings([
            "log",
            "main..topic",
            "+release.one",
            "--",
            "+literal-path",
            "src/lib.rs",
        ]),
        Path::new("/cwd"),
    );
    assert_eq!(
        log.request.operands,
        strings(["main..topic", "+release.one"])
    );
    assert_eq!(
        log.request.explicit_pathspecs,
        strings(["+literal-path", "src/lib.rs"])
    );
}

#[test]
fn workspace_cwd_and_global_selection_policy_are_preserved() {
    let temp = TempDir::new("log-cwd");
    let nested = temp.path().join("members/app/src");
    std::fs::create_dir_all(&nested).unwrap();
    let root = temp.path().to_string_lossy().into_owned();
    let log = log_invocation(
        vec![
            "--root".into(),
            root,
            "--all".into(),
            "--target".into(),
            "mem_api".into(),
            "--member".into(),
            "mem_app".into(),
            "--member-path".into(),
            "members/lib".into(),
            "--no-target".into(),
            "mem_skip".into(),
            "--no-member".into(),
            "mem_old".into(),
            "--no-member-path".into(),
            "members/tmp".into(),
            "--jobs".into(),
            "7".into(),
            "log".into(),
        ],
        &nested,
    );
    assert_eq!(
        log.request.workspace_cwd.as_deref(),
        Some("members/app/src")
    );
    assert_eq!(log.request.meta.request_id, "req_log");
    assert_eq!(log.request.meta.schema_version, "gwz.protocol/v0");
    assert_eq!(
        log.request
            .meta
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.root.as_deref()),
        Some(temp.path().to_str().unwrap())
    );
    let selection = log.request.meta.selection.unwrap();
    assert_eq!(
        selection.targets,
        strings(["@all", "mem_api", "mem_app", "members/lib"])
    );
    assert_eq!(
        selection.exclude_targets,
        strings(["mem_skip", "mem_old", "members/tmp"])
    );
    assert_eq!(
        log.request.meta.policy.unwrap().concurrency,
        Some(7),
        "global jobs must survive log lowering"
    );
}

#[test]
fn clap_help_contains_exact_s31_surface_and_not_s32_full() {
    let mut command = Cli::command();
    let help = command
        .find_subcommand_mut("log")
        .expect("log subcommand")
        .render_long_help()
        .to_string();
    for flag in [
        "-n <n>",
        "--no-limit",
        "--since",
        "--until",
        "--author",
        "--grep",
        "--no-merges",
        "--first-parent",
        "--strict",
        "--no-coalesce",
        "--body",
        "--tagged",
        "--color",
    ] {
        assert!(help.contains(flag), "missing {flag} from:\n{help}");
    }
    assert!(help.contains("Rust regex"), "{help}");
    assert!(help.contains("case-sensitive"), "{help}");
    assert!(help.contains("date-only is local midnight"), "{help}");
    assert!(
        help.contains("Promote any selected-repository degradation to failure"),
        "{help}"
    );
    assert!(!help.contains("history is unreadable"), "{help}");
    assert!(!help.contains("--full"), "S3.2 owns --full:\n{help}");
}

#[test]
fn color_accepts_exact_vocabulary_and_defaults_to_auto() {
    assert_eq!(
        log_invocation(strings(["log"]), Path::new("/cwd")).color,
        LogColor::Auto
    );
    for (value, expected) in [
        ("always", LogColor::Always),
        ("never", LogColor::Never),
        ("auto", LogColor::Auto),
    ] {
        assert_eq!(
            log_invocation(strings(["log", "--color", value]), Path::new("/cwd")).color,
            expected
        );
    }
    assert!(
        parse_args_with_request_id(
            strings(["log", "--color", "sometimes"]),
            "req_log",
            Path::new("/cwd")
        )
        .is_err()
    );
}

#[test]
fn repeated_single_value_options_follow_clap_standard_denial() {
    for args in [
        strings(["log", "--color", "always", "--color", "never"]),
        strings(["log", "--since", "2026-08-01", "--since", "2026-08-02"]),
    ] {
        let error = parse_args_with_request_id(args, "req_log", Path::new("/cwd"))
            .expect_err("clap must own repeated single-value behavior");
        assert!(
            error.message.contains("cannot be used multiple times"),
            "{}",
            error.message
        );
    }
}

fn response_with(status: gwz_core::AggregateStatus) -> gwz_core::ResponseEnvelope {
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            aggregate_status: status,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn log_aggregate_exit_mapping_uses_the_shared_response_seam() {
    for status in [
        gwz_core::AggregateStatus::Accepted,
        gwz_core::AggregateStatus::Ok,
        gwz_core::AggregateStatus::Noop,
        gwz_core::AggregateStatus::Dirty,
    ] {
        assert_eq!(
            exit_code_for_response(&response_with(status)),
            0,
            "{status:?}"
        );
    }
    for status in [
        gwz_core::AggregateStatus::Partial,
        gwz_core::AggregateStatus::Failed,
    ] {
        assert_eq!(
            exit_code_for_response(&response_with(status)),
            1,
            "{status:?}"
        );
    }
    assert_eq!(
        exit_code_for_response(&response_with(gwz_core::AggregateStatus::Rejected)),
        2
    );
}

#[test]
fn invalid_regex_and_time_are_process_rejections_before_valid_workspace_access() {
    let temp = TempDir::new("log-invalid-filter");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_log".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    for (grep, since, needle) in [
        (Some("("), None, "regex"),
        (None, Some("three days ago"), "RFC3339"),
    ] {
        let mut log = log_invocation(strings(["log"]), Path::new("/cwd"));
        log.request.meta.workspace = Some(gwz_core::WorkspaceRef {
            root: Some(temp.path().to_string_lossy().into_owned()),
            workspace_id: None,
        });
        let options = log.request.options.as_mut().unwrap();
        options.grep = grep.map(str::to_owned);
        options.since = since.map(str::to_owned);
        let registry = gwz_core::operation::CommitLogOutputRegistry::new();
        let mut stdout = Vec::new();
        let error = run_log_with_registry(
            &log,
            OutputMode::Human,
            temp.path(),
            "op_log_rejection".into(),
            &registry,
            &mut stdout,
        )
        .expect_err("invalid input/non-workspace must refuse");
        assert_eq!(exit_code_for_log_error(&error), 2, "{error:?}");
        assert_eq!(error.code, Some(gwz_core::model::ErrorCode::InvalidRequest));
        assert!(error.message.contains(needle), "{}", error.message);
        assert!(stdout.is_empty());
    }
}

#[test]
fn non_workspace_start_is_a_process_rejection() {
    let log = log_invocation(strings(["log"]), Path::new("/cwd"));
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = Vec::new();
    let error = run_log_with_registry(
        &log,
        OutputMode::Human,
        Path::new("/definitely/missing/gwz-workspace"),
        "op_log_non_workspace".into(),
        &registry,
        &mut stdout,
    )
    .expect_err("non-workspace start must refuse");
    assert_eq!(
        error.code,
        Some(gwz_core::model::ErrorCode::WorkspaceNotFound)
    );
    assert_eq!(exit_code_for_log_error(&error), 2);
    assert!(stdout.is_empty());
}

#[test]
fn typed_handler_refusals_are_two_but_execution_failures_are_one() {
    use gwz_core::model::{ErrorCode, ModelError};

    for code in [
        ErrorCode::InvalidRequest,
        ErrorCode::WorkspaceNotFound,
        ErrorCode::ManifestNotFound,
        ErrorCode::ManifestInvalid,
        ErrorCode::SchemaUnsupported,
        ErrorCode::PathEscape,
        ErrorCode::PermissionDenied,
        ErrorCode::MemberNotFound,
        ErrorCode::MemberInactive,
        ErrorCode::SnapshotNotFound,
        ErrorCode::TagNotFound,
        ErrorCode::LockNotFound,
        ErrorCode::TagInvalid,
        ErrorCode::UnsupportedOperation,
        ErrorCode::StashNotFound,
    ] {
        let error = CliError::from_model(ModelError::new(code, "teaching refusal"));
        assert_eq!(exit_code_for_log_error(&error), 2, "{code:?}");
    }
    for code in [
        ErrorCode::GitCommandFailed,
        ErrorCode::ExternalToolMissing,
        ErrorCode::RemoteRejected,
        ErrorCode::IoError,
        ErrorCode::InternalError,
    ] {
        let error = CliError::from_model(ModelError::new(code, "execution failure"));
        assert_eq!(exit_code_for_log_error(&error), 1, "{code:?}");
    }
}

#[test]
fn real_inactive_member_selector_is_a_process_rejection() {
    let workspace = initialized_workspace("log-inactive-member");
    gwz_core::workspace_ops::handle_create_repo(
        &gwz_core::git::Git2Backend::new(),
        workspace.path(),
        gwz_core::CreateRepoRequest {
            meta: request_meta("req_member"),
            member_path: "inactive".to_owned(),
            initial_branch: None,
            member_id: Some("mem_inactive".to_owned()),
            source_id: Some("src_inactive".to_owned()),
        },
        "op_member",
    )
    .unwrap();
    let mut detach_meta = request_meta("req_detach");
    detach_meta.selection = Some(gwz_core::Selection {
        targets: vec!["mem_inactive".to_owned()],
        ..Default::default()
    });
    gwz_core::workspace_ops::handle_detach_repo_member(
        &gwz_core::git::Git2Backend::new(),
        workspace.path(),
        gwz_core::DetachRepoMemberRequest { meta: detach_meta },
        "op_detach",
    )
    .unwrap();

    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "--target".into(),
            "mem_inactive".into(),
            "log".into(),
        ],
        workspace.path(),
    );
    assert_real_log_rejection(
        &log,
        workspace.path(),
        gwz_core::model::ErrorCode::MemberInactive,
    );
}

#[test]
fn real_empty_root_missing_snapshot_and_missing_tag_are_process_rejections() {
    let empty = TempDir::new("log-empty-explicit-root");
    let empty_log = log_invocation(
        vec![
            "--root".into(),
            empty.path().to_string_lossy().into_owned(),
            "log".into(),
        ],
        empty.path(),
    );
    assert_real_log_rejection(
        &empty_log,
        empty.path(),
        gwz_core::model::ErrorCode::ManifestNotFound,
    );

    let workspace = initialized_workspace("log-missing-reference");
    for (args, expected) in [
        (
            vec![
                "--root".into(),
                workspace.path().to_string_lossy().into_owned(),
                "log".into(),
                "+missing".into(),
            ],
            gwz_core::model::ErrorCode::SnapshotNotFound,
        ),
        (
            vec![
                "--root".into(),
                workspace.path().to_string_lossy().into_owned(),
                "log".into(),
                "--tagged".into(),
                "absent".into(),
            ],
            gwz_core::model::ErrorCode::TagNotFound,
        ),
    ] {
        let log = log_invocation(args, workspace.path());
        assert_real_log_rejection(&log, workspace.path(), expected);
    }
}

fn assert_real_log_rejection(
    log: &LogInvocation,
    start: &Path,
    expected: gwz_core::model::ErrorCode,
) {
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = Vec::new();
    let error = run_log_with_registry(
        log,
        OutputMode::Human,
        start,
        "op_log_real_rejection".into(),
        &registry,
        &mut stdout,
    )
    .expect_err("request must be rejected before output");
    assert_eq!(error.code, Some(expected));
    assert_eq!(exit_code_for_log_error(&error), 2, "{expected:?}");
    assert!(stdout.is_empty(), "rejected request wrote stdout bytes");
}

fn initialized_workspace(prefix: &str) -> TempDir {
    let temp = TempDir::new(prefix);
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_log".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    temp
}

fn commit_root(path: &Path) {
    let repository = git2::Repository::open(path).unwrap();
    std::fs::write(path.join("history.txt"), "history\n").unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("history.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let identity = git2::Signature::now("Log Test", "log@example.test").unwrap();
    repository
        .commit(
            Some("HEAD"),
            &identity,
            &identity,
            "root history",
            &tree,
            &[],
        )
        .unwrap();
}

#[test]
fn actual_runner_consumes_partial_and_failed_core_aggregates() {
    let workspace = initialized_workspace("log-runner-aggregate");
    gwz_core::workspace_ops::handle_create_repo(
        &gwz_core::git::Git2Backend::new(),
        workspace.path(),
        gwz_core::CreateRepoRequest {
            meta: request_meta("req_member"),
            member_path: "missing".to_owned(),
            initial_branch: None,
            member_id: Some("mem_missing".to_owned()),
            source_id: Some("src_missing".to_owned()),
        },
        "op_member",
    )
    .unwrap();
    std::fs::remove_dir_all(workspace.path().join("missing")).unwrap();
    commit_root(workspace.path());

    for (extra, status) in [
        (Vec::<String>::new(), "Partial"),
        (vec!["--strict".to_owned()], "Failed"),
    ] {
        let mut args = vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
        ];
        args.extend(extra);
        let log = log_invocation(args, workspace.path());
        let registry = gwz_core::operation::CommitLogOutputRegistry::new();
        let mut stdout = Vec::new();
        let exit = run_log_with_registry(
            &log,
            OutputMode::Human,
            workspace.path(),
            format!("op_log_{status}"),
            &registry,
            &mut stdout,
        )
        .unwrap();
        assert_eq!(exit.code, 1, "{status}");
        assert_eq!(
            String::from_utf8(stdout).unwrap(),
            format!("status: {status}\n")
        );
    }
}

#[derive(Default)]
struct BrokenPipeWriter {
    writes: usize,
}

#[derive(Default)]
struct WriteZeroWriter {
    writes: usize,
}

impl io::Write for WriteZeroWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        Err(io::Error::new(io::ErrorKind::WriteZero, "short write"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for BrokenPipeWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "consumer closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn broken_pipe_is_immediate_success_and_releases_unread_log() {
    let temp = TempDir::new("log-epipe");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_log".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    let log = log_invocation(
        vec![
            "--root".into(),
            temp.path().to_string_lossy().into_owned(),
            "log".into(),
        ],
        temp.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = BrokenPipeWriter::default();

    let exit = run_log_with_registry(
        &log,
        OutputMode::Human,
        temp.path(),
        "op_log_epipe".into(),
        &registry,
        &mut stdout,
    )
    .expect("broken stdout consumer is a clean termination");

    assert_eq!(exit.code, 0);
    assert_eq!(stdout.writes, 1, "must stop on the first failed write");
    let released = registry
        .read(
            "commitlog_000000000001",
            &gwz_core::operation::CommitLogReadRequest::default(),
        )
        .expect_err("runner must release even when the spool was never read");
    assert_eq!(released.code, gwz_core::model::ErrorCode::InvalidRequest);
}

#[test]
fn actual_runner_output_io_failure_stays_process_failure_and_releases() {
    let workspace = initialized_workspace("log-output-io");
    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
        ],
        workspace.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = WriteZeroWriter::default();
    let error = run_log_with_registry(
        &log,
        OutputMode::Human,
        workspace.path(),
        "op_log_output_io".into(),
        &registry,
        &mut stdout,
    )
    .expect_err("non-EPIPE output I/O must fail");

    assert_eq!(error.code, Some(gwz_core::model::ErrorCode::IoError));
    assert_eq!(exit_code_for_log_error(&error), 1);
    assert_eq!(stdout.writes, 1);
    assert!(
        registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err(),
        "output failure must release the spool"
    );
}

#[test]
fn broken_pipe_overrides_nonzero_aggregate_but_other_io_errors_do_not() {
    for aggregate_code in [1, 2] {
        let exit = log_exit_after_write(
            aggregate_code,
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "consumer closed")),
        )
        .unwrap();
        assert_eq!(exit.code, 0, "aggregate exit {aggregate_code}");
    }
    let error = log_exit_after_write(
        0,
        Err(io::Error::new(io::ErrorKind::WriteZero, "short write")),
    )
    .expect_err("non-EPIPE output errors must remain failures");
    assert_eq!(error.code, Some(gwz_core::model::ErrorCode::IoError));
}

#[test]
fn machine_error_broken_pipe_is_clean_and_never_falls_back_to_raw_println() {
    let error = CliError::invalid_request("invalid log regex");
    let mut closed = BrokenPipeWriter::default();
    let exit = write_log_machine_error(&error, &mut closed).unwrap();
    assert_eq!(exit.code, 0);
    assert_eq!(closed.writes, 1);

    let mut open = Vec::new();
    let exit = write_log_machine_error(&error, &mut open).unwrap();
    assert_eq!(exit.code, 2);
    let rendered = String::from_utf8(open).unwrap();
    assert!(rendered.contains("invalid log regex"), "{rendered}");
    assert!(rendered.ends_with('\n'));
}

#[test]
fn successful_log_dispatch_emits_only_the_plumbing_status_and_releases() {
    let temp = TempDir::new("log-dispatch");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_log".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    let log = log_invocation(
        vec![
            "--root".into(),
            temp.path().to_string_lossy().into_owned(),
            "log".into(),
        ],
        temp.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = Vec::new();
    let exit = run_log_with_registry(
        &log,
        OutputMode::Human,
        temp.path(),
        "op_log_dispatch".into(),
        &registry,
        &mut stdout,
    )
    .unwrap();

    assert_eq!(exit.code, 0);
    assert_eq!(String::from_utf8(stdout).unwrap(), "status: Ok\n");
    assert!(
        registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err(),
        "successful plumbing output must also release the unread spool"
    );
}
