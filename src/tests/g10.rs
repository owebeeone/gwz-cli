//! S3.2 acceptance coverage for human `gwz log` rendering and docs.

use std::io::{self, Write};
use std::path::Path;

use clap::CommandFactory;

use super::*;
use crate::tests::g01::{TempDir, request_meta, strings};

fn log_invocation(args: Vec<String>, cwd: &Path) -> Box<LogInvocation> {
    let invocation =
        parse_args_with_request_id(args, "req_log_render", cwd).expect("log arguments parse");
    match invocation.request {
        CliRequest::Log(log) => log,
        other => panic!("expected log request, got {other:?}"),
    }
}

fn member(id: &str, path: &str, hash_byte: char) -> gwz_core::LogEntryMember {
    gwz_core::LogEntryMember {
        member_id: id.to_owned(),
        member_path: path.to_owned(),
        source_kind: Some(gwz_core::SourceKind::Git),
        commit: hash_byte.to_string().repeat(40),
        parents: vec!["f".repeat(40)],
    }
}

fn entry(members: Vec<gwz_core::LogEntryMember>, subject: &str) -> gwz_core::LogEntry {
    gwz_core::LogEntry {
        members,
        provenance: gwz_core::LogMergeProvenance {
            kind: gwz_core::LogMergeKind::None,
            gwz_commit_id: None,
        },
        author: gwz_core::GitObjectIdentity {
            name: "Ada Lovelace".to_owned(),
            email: "ada@example.test".to_owned(),
            time_ms: Some(0),
            timezone_offset_minutes: Some(-60),
        },
        committer: gwz_core::GitObjectIdentity {
            name: "Grace Hopper".to_owned(),
            email: "grace@example.test".to_owned(),
            time_ms: Some(0),
            timezone_offset_minutes: Some(330),
        },
        subject: subject.to_owned(),
        body: None,
        ordering_timestamp_ms: Some(0),
        author_timestamp_seconds: 0,
        committer_timestamp_seconds: 0,
        ordering_timestamp_seconds: 0,
        lossy: Some(false),
    }
}

#[test]
fn full_flag_and_command_help_describe_the_human_modes_and_no_pager() {
    assert!(!log_invocation(strings(["log"]), Path::new("/cwd")).full);
    assert!(log_invocation(strings(["log", "--full"]), Path::new("/cwd")).full);

    let mut command = Cli::command();
    let help = command
        .find_subcommand_mut("log")
        .unwrap()
        .render_long_help()
        .to_string();
    for phrase in [
        "--full",
        "git-style blocks",
        "workspace-relative member paths",
        "does not use a pager",
    ] {
        assert!(help.contains(phrase), "missing `{phrase}` in:\n{help}");
    }
    assert!(cli_reference_markdown().contains("Command page: [log](commands/log.md)."));
    assert!(
        !include_str!("../log_exec.rs").contains("pager::"),
        "the standing Q-11 ruling keeps log on the direct-output lifecycle"
    );
}

#[test]
fn public_docs_cover_log_discovery_limits_lock_ranges_and_machine_provenance() {
    let quick_start = include_str!("../../docs/QuickStart.md");
    let concepts = include_str!("../../docs/Concepts.md");
    let workflows = include_str!("../../docs/Workflows.md");
    let command = include_str!("../../docs/commands/log.md");
    let machine = include_str!("../../docs/MachineOutput.md");

    assert!(quick_start.contains("gwz log"));
    assert!(concepts.contains("Workspace History"));
    assert!(workflows.contains("Inspect Workspace History"));
    for needle in ["50", "+lock..HEAD", "bare `+lock`", "--json", "--jsonl"] {
        assert!(
            command.contains(needle),
            "log command page is missing `{needle}`"
        );
    }
    for needle in [
        "gwz.log/v0",
        "marker:<uuid-v7>",
        "marker-invalid",
        "heuristic",
        "revision_unresolved",
    ] {
        assert!(
            machine.contains(needle),
            "machine-output guide is missing `{needle}`"
        );
    }
}

#[test]
fn compact_rendering_uses_committer_offset_complete_subject_and_member_sets() {
    let long_subject = format!("subject-{}", "x".repeat(240));
    let singleton = entry(vec![member("mem_api", "members/api", 'a')], &long_subject);
    assert_eq!(
        render_log_entry(&singleton, false, false).unwrap(),
        format!("1970-01-01 05:30:00 +0530 members/api aaaaaaaaaaaa {long_subject}")
    );
    for (seconds, expected_date) in [
        (i64::MIN, "-292277022657-01-27 13:59:52 +0530"),
        (i64::MAX, "292277026596-12-04 21:00:07 +0530"),
    ] {
        let mut extreme = singleton.clone();
        extreme.committer_timestamp_seconds = seconds;
        assert!(
            render_log_entry(&extreme, false, false)
                .unwrap()
                .starts_with(expected_date),
            "{seconds}"
        );
    }

    let small = entry(
        vec![
            member("@root", ".", 'a'),
            member("mem_api", "members/api", 'b'),
            member("mem_web", "members/web", 'c'),
        ],
        "small set",
    );
    assert!(
        render_log_entry(&small, false, false)
            .unwrap()
            .contains("[., members/api, members/web]")
    );

    let root_large = entry(
        vec![
            member("@root", ".", 'a'),
            member("mem_a", "a", 'b'),
            member("mem_b", "b", 'c'),
            member("mem_c", "c", 'd'),
        ],
        "large root set",
    );
    assert!(
        render_log_entry(&root_large, false, false)
            .unwrap()
            .contains("[root+3]")
    );

    let member_large = entry(
        vec![
            member("mem_a", "a", 'a'),
            member("mem_b", "b", 'b'),
            member("mem_c", "c", 'c'),
            member("mem_d", "d", 'd'),
        ],
        "large member set",
    );
    assert!(
        render_log_entry(&member_large, false, false)
            .unwrap()
            .contains("[4 members]")
    );
}

#[test]
fn human_rendering_rejects_an_entry_without_members() {
    let empty = entry(Vec::new(), "impossible entry");
    let error = render_log_entry(&empty, false, false)
        .expect_err("a commit-log entry must contain at least one member");
    assert_eq!(error, "commit-log entry has no members");
}

#[test]
fn full_rendering_has_complete_member_table_git_identity_date_and_body() {
    let mut record = entry(
        vec![
            member("@root", ".", 'a'),
            member("mem_api", "members/api", 'b'),
        ],
        "full subject",
    );
    record.body = Some("\nbody line\nsecond line".to_owned());
    let rendered = render_log_entry(&record, true, false).unwrap();

    assert!(rendered.starts_with(&format!("commit {}\n", "a".repeat(40))));
    assert!(rendered.contains("Members:\n    ID"), "{rendered}");
    for needle in [
        "@root".to_owned(),
        "mem_api".to_owned(),
        "members/api".to_owned(),
        "a".repeat(40),
        "b".repeat(40),
        "Author: Ada Lovelace <ada@example.test>".to_owned(),
        "Date:   1969-12-31 23:00:00 -0100".to_owned(),
        "    full subject\n    \n    body line\n    second line".to_owned(),
    ] {
        assert!(
            rendered.contains(&needle),
            "missing `{needle}` in:\n{rendered}"
        );
    }
}

#[test]
fn human_fields_are_lossy_and_c0_sanitized_without_width_truncation() {
    let path = format!("members/\u{fffd}\u{1b}-{}", "p".repeat(200));
    let subject = format!("fix\tthing\u{1b}[31m-{}", "s".repeat(300));
    let mut record = entry(vec![member("mem_bad", &path, 'd')], &subject);
    record.author.name = "Ad\u{0}a\u{fffd}".to_owned();
    record.body = Some("body\tcell\nnext\u{7}line".to_owned());

    let compact = render_log_entry(&record, false, false).unwrap();
    assert!(compact.contains(&format!("members/��-{}", "p".repeat(200))));
    assert!(compact.contains(&format!("fix thing�[31m-{}", "s".repeat(300))));
    assert!(!compact.contains('\u{1b}'));
    assert!(!compact.contains('\t'));

    let full = render_log_entry(&record, true, false).unwrap();
    assert!(full.contains("Author: Ad�a� <ada@example.test>"));
    assert!(full.contains("    body cell\n    next�line"));
    assert!(
        full.chars()
            .all(|character| character == '\n' || !character.is_control()),
        "{full:?}"
    );
}

#[test]
fn color_policy_uses_only_the_flag_and_stdout_tty_state() {
    assert!(log_color_enabled(LogColor::Always, false));
    assert!(log_color_enabled(LogColor::Always, true));
    assert!(!log_color_enabled(LogColor::Never, false));
    assert!(!log_color_enabled(LogColor::Never, true));
    assert!(!log_color_enabled(LogColor::Auto, false));
    assert!(log_color_enabled(LogColor::Auto, true));

    let record = entry(vec![member("@root", ".", 'a')], "color subject");
    assert!(
        !render_log_entry(&record, false, false)
            .unwrap()
            .contains('\u{1b}')
    );
    assert!(
        render_log_entry(&record, false, true)
            .unwrap()
            .contains("\u{1b}[")
    );
}

#[test]
fn degradation_summary_is_stderr_safe_and_names_member_reason_and_operand() {
    let record = gwz_core::LogDegradation {
        member_id: "mem_api".to_owned(),
        member_path: "members/api\u{1b}".to_owned(),
        source_kind: Some(gwz_core::SourceKind::Git),
        reason: gwz_core::LogDegradationReason::RevisionUnresolved,
        operand: Some("topic\tname".to_owned()),
        message: Some("missing\u{7} ref".to_owned()),
    };
    assert_eq!(
        render_log_degradation(&record, false),
        "gwz log: degraded members/api�: revision unresolved for 'topic name' — missing� ref"
    );
}

#[test]
fn real_runner_renders_compact_and_full_records_and_releases_each_spool() {
    let workspace = initialized_workspace("log-human-compact");
    let hash = commit_root_at(
        workspace.path(),
        0,
        330,
        "fixed\tcompact\u{1b}[31m subject\n\nbody\tline",
    );
    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
            "--color=never".into(),
        ],
        workspace.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = run_log_with_registry_io(
        &log,
        OutputMode::Human,
        workspace.path(),
        "op_log_human".into(),
        &registry,
        LogIo {
            stdout: &mut stdout,
            stderr: &mut stderr,
            stdout_is_tty: true,
        },
    )
    .unwrap();

    assert_eq!(exit.code, 0);
    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        format!(
            "1970-01-01 05:30:00 +0530 . {} fixed compact�[31m subject\n",
            &hash[..12]
        )
    );
    assert!(stderr.is_empty());
    assert!(
        registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err()
    );

    let full = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
            "--full".into(),
            "--body".into(),
            "--color=always".into(),
        ],
        workspace.path(),
    );
    let full_registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut full_stdout = Vec::new();
    let mut full_stderr = Vec::new();
    let exit = run_log_with_registry_io(
        &full,
        OutputMode::Human,
        workspace.path(),
        "op_log_full".into(),
        &full_registry,
        LogIo {
            stdout: &mut full_stdout,
            stderr: &mut full_stderr,
            stdout_is_tty: false,
        },
    )
    .unwrap();
    let rendered = String::from_utf8(full_stdout).unwrap();
    assert_eq!(exit.code, 0);
    assert!(
        rendered.contains(&format!("\u{1b}[33mcommit {hash}\u{1b}[0m")),
        "{rendered}"
    );
    assert!(
        rendered.contains("\u{1b}[36mMembers:\u{1b}[0m"),
        "{rendered}"
    );
    assert!(rendered.contains("    body line\n"), "{rendered}");
    assert!(full_stderr.is_empty());
    assert!(
        full_registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err()
    );
}

#[test]
fn zero_entry_run_has_empty_stdout_success_and_benign_degradation_on_stderr() {
    let workspace = initialized_workspace("log-human-empty");
    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
        ],
        workspace.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = run_log_with_registry_io(
        &log,
        OutputMode::Human,
        workspace.path(),
        "op_log_empty".into(),
        &registry,
        LogIo {
            stdout: &mut stdout,
            stderr: &mut stderr,
            stdout_is_tty: false,
        },
    )
    .unwrap();

    assert_eq!(exit.code, 0);
    assert!(stdout.is_empty());
    let summary = String::from_utf8(stderr).unwrap();
    assert!(
        summary.contains("gwz log: degraded .: unborn history"),
        "{summary}"
    );
}

struct ReleaseOnBrokenPipe<'a> {
    registry: &'a gwz_core::operation::CommitLogOutputRegistry,
    writes: usize,
}

impl Write for ReleaseOnBrokenPipe<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.writes == 1 {
            self.registry.release("commitlog_000000000001");
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "consumer closed"))
        } else {
            Ok(bytes.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn multi_page_broken_pipe_stops_before_later_output_or_registry_read_and_releases() {
    let workspace = initialized_workspace("log-human-epipe-sentinel");
    commit_root_series(workspace.path(), 129);
    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
            "--no-limit".into(),
            "--color=never".into(),
        ],
        workspace.path(),
    );
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = ReleaseOnBrokenPipe {
        registry: &registry,
        writes: 0,
    };
    let mut stderr = Vec::new();

    let exit = run_log_with_registry_io(
        &log,
        OutputMode::Human,
        workspace.path(),
        "op_log_epipe_sentinel".into(),
        &registry,
        LogIo {
            stdout: &mut stdout,
            stderr: &mut stderr,
            stdout_is_tty: false,
        },
    )
    .expect("BrokenPipe must terminate before the released registry is read again");

    assert_eq!(exit.code, 0);
    assert_eq!(stdout.writes, 1, "no output attempt may follow BrokenPipe");
    assert!(stderr.is_empty(), "BrokenPipe must not spray stderr");
    assert!(
        registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err(),
        "the caller-owned spool must be released"
    );
}

#[test]
fn real_runner_preserves_human_vs_machine_ownership_and_releases_every_mode() {
    let workspace = initialized_workspace("log-human-mode-boundary");
    let hash = commit_root_at(workspace.path(), 0, 0, "mode sentinel");
    let log = log_invocation(
        vec![
            "--root".into(),
            workspace.path().to_string_lossy().into_owned(),
            "log".into(),
            "--color=never".into(),
        ],
        workspace.path(),
    );
    let expected_record = serde_json::json!({
        "author": {
            "email": "log@example.test",
            "name": "Log Test",
            "time": { "offset_min": 0, "time": 0 },
        },
        "committer": {
            "email": "log@example.test",
            "name": "Log Test",
            "time": { "offset_min": 0, "time": 0 },
        },
        "members": [{
            "hash": hash.clone(),
            "member_id": "@root",
            "member_path": ".",
            "parents": [],
        }],
        "provenance": "none",
        "record": "entry",
        "subject": "mode sentinel",
    });
    let expected_json = format!(
        "{}\n",
        serde_json::json!({
            "records": [expected_record.clone()],
            "schema": "gwz.log/v0",
        })
    );
    let expected_jsonl =
        format!("{{\"record\":\"header\",\"schema\":\"gwz.log/v0\"}}\n{expected_record}\n");

    for output in [OutputMode::Human, OutputMode::Json, OutputMode::Jsonl] {
        let registry = gwz_core::operation::CommitLogOutputRegistry::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run_log_with_registry_io(
            &log,
            output,
            workspace.path(),
            "op_log_mode_boundary".into(),
            &registry,
            LogIo {
                stdout: &mut stdout,
                stderr: &mut stderr,
                stdout_is_tty: false,
            },
        )
        .unwrap();

        assert_eq!(exit.code, 0, "{output:?}");
        assert!(stderr.is_empty(), "{output:?}");
        let rendered = String::from_utf8(stdout).unwrap();
        match output {
            OutputMode::Human => assert_eq!(
                rendered,
                format!(
                    "1970-01-01 00:00:00 +0000 . {} mode sentinel\n",
                    &hash[..12]
                )
            ),
            OutputMode::Json => assert_eq!(rendered, expected_json),
            OutputMode::Jsonl => assert_eq!(rendered, expected_jsonl),
            OutputMode::Porcelain => unreachable!(),
        }
        assert!(
            registry
                .read(
                    "commitlog_000000000001",
                    &gwz_core::operation::CommitLogReadRequest::default(),
                )
                .is_err(),
            "{output:?} must release its spool"
        );
    }
}

fn initialized_workspace(prefix: &str) -> TempDir {
    let temp = TempDir::new(prefix);
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_log_render".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    temp
}

fn commit_root_at(path: &Path, seconds: i64, offset: i32, message: &str) -> String {
    let repository = git2::Repository::open(path).unwrap();
    std::fs::write(path.join("history.txt"), "history\n").unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("history.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let time = git2::Time::new(seconds, offset);
    let identity = git2::Signature::new("Log Test", "log@example.test", &time).unwrap();
    repository
        .commit(Some("HEAD"), &identity, &identity, message, &tree, &[])
        .unwrap()
        .to_string()
}

fn commit_root_series(path: &Path, count: usize) {
    let repository = git2::Repository::open(path).unwrap();
    std::fs::write(path.join("history.txt"), "history\n").unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("history.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let mut parent = None;
    for index in 0..count {
        let time = git2::Time::new(i64::try_from(index).unwrap(), 0);
        let identity = git2::Signature::new("Log Test", "log@example.test", &time).unwrap();
        let parent_commit = parent.map(|oid| repository.find_commit(oid).unwrap());
        let parents = parent_commit.iter().collect::<Vec<_>>();
        parent = Some(
            repository
                .commit(
                    Some("HEAD"),
                    &identity,
                    &identity,
                    &format!("entry {index}"),
                    &tree,
                    &parents,
                )
                .unwrap(),
        );
    }
}
