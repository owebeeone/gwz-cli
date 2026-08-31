//! S3.3 machine-output coverage for the ordered commit-log record stream.

use std::collections::VecDeque;
use std::io;
use std::path::Path;

use serde_json::{Value, json};

use super::*;
use crate::tests::g01::{TempDir, request_meta};

const MARKER: &str = "01987b0c-2f75-7c4a-9a32-8fd22f7d7c91";
const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PARENT_1: &str = "1111111111111111111111111111111111111111";
const PARENT_2: &str = "2222222222222222222222222222222222222222";

fn identity(name: &str, email: &str, seconds: i64, offset_min: i64) -> gwz_core::GitObjectIdentity {
    gwz_core::GitObjectIdentity {
        name: name.to_owned(),
        email: email.to_owned(),
        time_ms: seconds.checked_mul(1_000),
        timezone_offset_minutes: Some(offset_min),
    }
}

fn member(id: &str, path: &str, hash: &str, parents: &[&str]) -> gwz_core::LogEntryMember {
    gwz_core::LogEntryMember {
        member_id: id.to_owned(),
        member_path: path.to_owned(),
        source_kind: Some(gwz_core::SourceKind::Git),
        commit: hash.to_owned(),
        parents: parents.iter().map(|parent| (*parent).to_owned()).collect(),
    }
}

fn entry_record(
    members: Vec<gwz_core::LogEntryMember>,
    provenance: gwz_core::LogMergeProvenance,
    subject: &str,
    body: Option<&str>,
    lossy: Option<bool>,
) -> gwz_core::LogOutputRecord {
    gwz_core::LogOutputRecord {
        kind: gwz_core::LogOutputRecordKind::Entry,
        entry: Some(gwz_core::LogEntry {
            members,
            provenance,
            author: identity("Author \u{fffd}", "author@example.test", -7, 630),
            committer: identity("Committer", "commit@example.test", 22, -345),
            subject: subject.to_owned(),
            body: body.map(str::to_owned),
            ordering_timestamp_ms: Some(22_000),
            author_timestamp_seconds: -7,
            committer_timestamp_seconds: 22,
            ordering_timestamp_seconds: 22,
            lossy,
        }),
        degradation: None,
    }
}

fn degradation_record(reason: gwz_core::LogDegradationReason) -> gwz_core::LogOutputRecord {
    gwz_core::LogOutputRecord {
        kind: gwz_core::LogOutputRecordKind::Degradation,
        entry: None,
        degradation: Some(gwz_core::LogDegradation {
            member_id: "mem_bad".to_owned(),
            member_path: "members/bad".to_owned(),
            source_kind: Some(gwz_core::SourceKind::Git),
            reason,
            operand: Some("missing..HEAD".to_owned()),
            message: Some("cannot\nresolve".to_owned()),
        }),
    }
}

fn pages(
    records: &[gwz_core::LogOutputRecord],
) -> VecDeque<gwz_core::operation::CommitLogReadResponse> {
    let split = records.len().min(2);
    let mut responses = VecDeque::new();
    if split != 0 {
        responses.push_back(gwz_core::operation::CommitLogReadResponse {
            records: records[..split].to_vec(),
            next_cursor: 41,
            state: gwz_core::operation::CommitLogReadState::Data,
        });
    }
    if records.len() > split {
        responses.push_back(gwz_core::operation::CommitLogReadResponse {
            records: records[split..].to_vec(),
            next_cursor: 99,
            state: gwz_core::operation::CommitLogReadState::Data,
        });
    }
    responses.push_back(gwz_core::operation::CommitLogReadResponse {
        records: Vec::new(),
        next_cursor: if records.len() > split { 99 } else { 41 },
        state: gwz_core::operation::CommitLogReadState::Eof,
    });
    responses
}

fn render(
    output: OutputMode,
    records: &[gwz_core::LogOutputRecord],
) -> (String, Vec<gwz_core::operation::CommitLogReadRequest>) {
    let mut remaining = pages(records);
    let mut requests = Vec::new();
    let mut bytes = Vec::new();
    write_log_machine_output_with(
        output,
        |request| {
            requests.push(request.clone());
            Ok(remaining
                .pop_front()
                .expect("renderer read past explicit EOF"))
        },
        &mut bytes,
    )
    .unwrap();
    assert!(remaining.is_empty());
    (String::from_utf8(bytes).unwrap(), requests)
}

fn contract_records() -> Vec<gwz_core::LogOutputRecord> {
    vec![
        entry_record(
            vec![
                member("mem_a", "members/a", HASH_A, &[PARENT_2, PARENT_1]),
                member("mem_z", "members/z", HASH_B, &[]),
            ],
            gwz_core::LogMergeProvenance {
                kind: gwz_core::LogMergeKind::Marker,
                gwz_commit_id: Some(MARKER.to_owned()),
            },
            "subject\ncontrol\u{0}",
            Some("\nbody \"quoted\"\\tail"),
            Some(true),
        ),
        degradation_record(gwz_core::LogDegradationReason::RevisionUnresolved),
        entry_record(
            vec![member("@root", ".", HASH_B, &[PARENT_1])],
            gwz_core::LogMergeProvenance {
                kind: gwz_core::LogMergeKind::None,
                gwz_commit_id: Some("marker-invalid".to_owned()),
            },
            "literal \u{fffd}",
            None,
            Some(false),
        ),
    ]
}

fn machine_log_invocation(root: &Path) -> Box<LogInvocation> {
    machine_log_invocation_with_body(root, false)
}

fn machine_log_invocation_with_body(root: &Path, include_body: bool) -> Box<LogInvocation> {
    let mut args = vec![
        "--root".to_owned(),
        root.to_string_lossy().into_owned(),
        "log".to_owned(),
    ];
    if include_body {
        args.push("--body".to_owned());
    }
    let invocation = parse_args_with_request_id(args, "req_machine_log", root).unwrap();
    match invocation.request {
        CliRequest::Log(log) => log,
        other => panic!("expected log request, got {other:?}"),
    }
}

fn mixed_machine_workspace(prefix: &str) -> TempDir {
    let workspace = TempDir::new(prefix);
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: workspace.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli_machine_log".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
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

    let repository = git2::Repository::open(workspace.path()).unwrap();
    std::fs::write(workspace.path().join("history.txt"), "history\n").unwrap();
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
            "root history\nbody line\n",
            &tree,
            &[],
        )
        .unwrap();
    workspace
}

#[test]
fn l_jsn_1_json_is_one_ordered_schema_document_with_uniform_members_and_exact_fields() {
    let records = contract_records();
    let (rendered, requests) = render(OutputMode::Json, &records);
    assert_eq!(
        rendered.lines().count(),
        1,
        "JSON strings must escape newlines"
    );
    let document: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(document["schema"], "gwz.log/v0");
    let rendered_records = document["records"].as_array().unwrap();
    assert_eq!(rendered_records.len(), 3);
    assert_eq!(
        rendered_records[0],
        json!({
            "record": "entry",
            "members": [
                {"member_id": "mem_a", "member_path": "members/a", "hash": HASH_A,
                 "parents": [PARENT_2, PARENT_1]},
                {"member_id": "mem_z", "member_path": "members/z", "hash": HASH_B,
                 "parents": []},
            ],
            "provenance": format!("marker:{MARKER}"),
            "author": {"name": "Author \u{fffd}", "email": "author@example.test",
                "time": {"time": -7, "offset_min": 630}},
            "committer": {"name": "Committer", "email": "commit@example.test",
                "time": {"time": 22, "offset_min": -345}},
            "subject": "subject\ncontrol\u{0}",
            "body": "\nbody \"quoted\"\\tail",
            "lossy": true,
        })
    );
    assert_eq!(rendered_records[1]["record"], "degradation");
    assert_eq!(rendered_records[2]["record"], "entry");
    assert_eq!(rendered_records[2]["members"].as_array().unwrap().len(), 1);
    assert_eq!(rendered_records[2]["provenance"], "marker-invalid");
    assert!(rendered_records[2].get("body").is_none());
    assert!(
        rendered_records[2].get("lossy").is_none(),
        "lossy is driven by the protocol bit, never inferred from U+FFFD"
    );
    assert_eq!(requests.len(), 3);
}

#[test]
fn l_env_13_jsonl_has_exact_header_single_lines_and_stops_at_explicit_eof() {
    let records = contract_records();
    let (first, requests) = render(OutputMode::Jsonl, &records);
    let (second, _) = render(OutputMode::Jsonl, &records);
    assert_eq!(
        first, second,
        "identical records must produce identical bytes"
    );

    let lines = first.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], r#"{"record":"header","schema":"gwz.log/v0"}"#);
    let jsonl_records = lines[1..]
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let (json, _) = render(OutputMode::Json, &records);
    assert_eq!(
        jsonl_records,
        serde_json::from_str::<Value>(&json).unwrap()["records"]
            .as_array()
            .unwrap()
            .clone(),
        "both machine modes must serialize the same ordered records"
    );
    assert_eq!(
        jsonl_records
            .iter()
            .map(|record| record["record"].clone())
            .collect::<Vec<_>>(),
        [json!("entry"), json!("degradation"), json!("entry")]
    );
    assert_eq!(
        requests,
        [
            gwz_core::operation::CommitLogReadRequest {
                cursor: None,
                max_records: Some(128)
            },
            gwz_core::operation::CommitLogReadRequest {
                cursor: Some(41),
                max_records: Some(128)
            },
            gwz_core::operation::CommitLogReadRequest {
                cursor: Some(99),
                max_records: Some(128)
            },
        ]
    );
}

#[test]
fn l_env_13_empty_outputs_and_one_record_bytes_are_canonical() {
    for (output, expected) in [
        (
            OutputMode::Json,
            "{\"records\":[],\"schema\":\"gwz.log/v0\"}\n",
        ),
        (
            OutputMode::Jsonl,
            "{\"record\":\"header\",\"schema\":\"gwz.log/v0\"}\n",
        ),
    ] {
        let mut reads = 0;
        let mut bytes = Vec::new();
        write_log_machine_output_with(
            output,
            |request| {
                reads += 1;
                assert_eq!(request.cursor, None);
                Ok(gwz_core::operation::CommitLogReadResponse {
                    records: Vec::new(),
                    next_cursor: 0,
                    state: gwz_core::operation::CommitLogReadState::Eof,
                })
            },
            &mut bytes,
        )
        .unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
        assert_eq!(reads, 1);
    }

    let record = degradation_record(gwz_core::LogDegradationReason::Unborn);
    let expected_record = concat!(
        "{\"member_id\":\"mem_bad\",\"member_path\":\"members/bad\",",
        "\"message\":\"cannot\\nresolve\",\"operand\":\"missing..HEAD\",",
        "\"reason\":\"unborn\",\"record\":\"degradation\"}"
    );
    let (json, _) = render(OutputMode::Json, std::slice::from_ref(&record));
    assert_eq!(
        json,
        format!("{{\"records\":[{expected_record}],\"schema\":\"gwz.log/v0\"}}\n")
    );
    let (jsonl, _) = render(OutputMode::Jsonl, std::slice::from_ref(&record));
    assert_eq!(
        jsonl,
        format!("{{\"record\":\"header\",\"schema\":\"gwz.log/v0\"}}\n{expected_record}\n")
    );

    let mut entry = entry_record(
        vec![member("mem_a", "members/a", HASH_A, &[PARENT_2, PARENT_1])],
        gwz_core::LogMergeProvenance::default(),
        "s",
        None,
        Some(false),
    );
    let payload = entry.entry.as_mut().unwrap();
    payload.author = identity("A", "a@x", 1, 2);
    payload.author_timestamp_seconds = 1;
    payload.committer = identity("C", "c@x", 4, -3);
    payload.committer_timestamp_seconds = 4;
    let (jsonl, _) = render(OutputMode::Jsonl, std::slice::from_ref(&entry));
    assert_eq!(
        jsonl.lines().nth(1).unwrap(),
        concat!(
            "{\"author\":{\"email\":\"a@x\",\"name\":\"A\",\"time\":{\"offset_min\":2,\"time\":1}},",
            "\"committer\":{\"email\":\"c@x\",\"name\":\"C\",\"time\":{\"offset_min\":-3,\"time\":4}},",
            "\"members\":[{\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
            "\"member_id\":\"mem_a\",\"member_path\":\"members/a\",",
            "\"parents\":[\"2222222222222222222222222222222222222222\",",
            "\"1111111111111111111111111111111111111111\"]}],",
            "\"provenance\":\"none\",\"record\":\"entry\",\"subject\":\"s\"}"
        )
    );
}

#[test]
fn l_env_12_times_use_exact_i64_seconds_and_each_commit_offset() {
    let mut record = entry_record(
        vec![member("mem_a", "members/a", HASH_A, &[])],
        gwz_core::LogMergeProvenance::default(),
        "extreme times",
        None,
        None,
    );
    let entry = record.entry.as_mut().unwrap();
    entry.author.time_ms = None;
    entry.author_timestamp_seconds = i64::MIN;
    entry.author.timezone_offset_minutes = Some(630);
    entry.committer.time_ms = Some(42_000);
    entry.committer_timestamp_seconds = i64::MAX;
    entry.committer.timezone_offset_minutes = Some(-345);

    let (rendered, _) = render(OutputMode::Json, std::slice::from_ref(&record));
    let document: Value = serde_json::from_str(&rendered).unwrap();
    let rendered = &document["records"][0];
    assert_eq!(
        rendered["author"]["time"],
        json!({"time": i64::MIN, "offset_min": 630})
    );
    assert_eq!(
        rendered["committer"]["time"],
        json!({"time": i64::MAX, "offset_min": -345})
    );
}

#[derive(Default)]
struct ImmediateBrokenPipe {
    writes: usize,
}

#[derive(Default)]
struct FlushBrokenPipe {
    flushes: usize,
}

impl io::Write for ImmediateBrokenPipe {
    fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for FlushBrokenPipe {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
    }
}

#[test]
fn l_env_9_machine_broken_pipe_stops_before_any_hidden_spool_read() {
    let mut reads = 0;
    let mut writer = ImmediateBrokenPipe::default();
    let error = write_log_machine_output_with(
        OutputMode::Jsonl,
        |_| {
            reads += 1;
            panic!("closed consumer must stop before reading the spool")
        },
        &mut writer,
    )
    .unwrap_err();
    match error {
        LogMachineOutputError::Write(error) => {
            assert_eq!(error.kind(), io::ErrorKind::BrokenPipe)
        }
        other => panic!("expected write error, got {other:?}"),
    }
    assert_eq!(writer.writes, 1);
    assert_eq!(reads, 0);

    for output in [OutputMode::Json, OutputMode::Jsonl] {
        let mut writer = FlushBrokenPipe::default();
        let error = write_log_machine_output_with(
            output,
            |_| panic!("a buffered prefix must flush before the first spool read"),
            &mut writer,
        )
        .unwrap_err();
        match error {
            LogMachineOutputError::Write(error) => {
                assert_eq!(error.kind(), io::ErrorKind::BrokenPipe)
            }
            other => panic!("expected flush error, got {other:?}"),
        }
        assert_eq!(writer.flushes, 1);
    }
}

#[test]
fn actual_machine_runner_preserves_core_mixed_order_and_releases() {
    let workspace = mixed_machine_workspace("log-machine-runner");
    let log = machine_log_invocation(workspace.path());
    for output in [OutputMode::Json, OutputMode::Jsonl] {
        let registry = gwz_core::operation::CommitLogOutputRegistry::new();
        let mut bytes = Vec::new();
        let exit = run_log_with_registry(
            &log,
            output,
            workspace.path(),
            format!("op_machine_{output:?}"),
            &registry,
            &mut bytes,
        )
        .unwrap();
        assert_eq!(exit.code, 1, "contribution plus read failure is Partial");

        let rendered = String::from_utf8(bytes).unwrap();
        let records = match output {
            OutputMode::Json => serde_json::from_str::<Value>(&rendered).unwrap()["records"]
                .as_array()
                .unwrap()
                .clone(),
            OutputMode::Jsonl => rendered
                .lines()
                .skip(1)
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .collect(),
            OutputMode::Human | OutputMode::Porcelain => unreachable!(),
        };
        assert_eq!(
            records
                .iter()
                .map(|record| record["record"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["degradation", "entry"]
        );
        assert_eq!(records[0]["member_id"], "mem_missing");
        assert_eq!(records[1]["subject"], "root history");
        assert!(
            registry
                .read(
                    "commitlog_000000000001",
                    &gwz_core::operation::CommitLogReadRequest::default(),
                )
                .is_err(),
            "runner must release the consumed spool"
        );
    }

    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut writer = ImmediateBrokenPipe::default();
    let exit = run_log_with_registry(
        &log,
        OutputMode::Jsonl,
        workspace.path(),
        "op_machine_epipe".to_owned(),
        &registry,
        &mut writer,
    )
    .unwrap();
    assert_eq!(exit.code, 0);
    assert_eq!(writer.writes, 1);
    assert!(
        registry
            .read(
                "commitlog_000000000001",
                &gwz_core::operation::CommitLogReadRequest::default(),
            )
            .is_err(),
        "runner must release the unread spool after machine EPIPE"
    );
}

#[test]
fn f1_actual_machine_runner_preserves_default_and_explicit_body() {
    let workspace = mixed_machine_workspace("log-machine-runner-body");
    for output in [OutputMode::Json, OutputMode::Jsonl] {
        for (include_body, expected_body) in [(false, None), (true, Some("body line\n"))] {
            let log = machine_log_invocation_with_body(workspace.path(), include_body);
            let registry = gwz_core::operation::CommitLogOutputRegistry::new();
            let mut bytes = Vec::new();
            let exit = run_log_with_registry(
                &log,
                output,
                workspace.path(),
                format!("op_machine_body_{output:?}_{include_body}"),
                &registry,
                &mut bytes,
            )
            .unwrap();
            assert_eq!(exit.code, 1, "contribution plus read failure is Partial");

            let rendered = String::from_utf8(bytes).unwrap();
            let records = match output {
                OutputMode::Json => serde_json::from_str::<Value>(&rendered).unwrap()["records"]
                    .as_array()
                    .unwrap()
                    .clone(),
                OutputMode::Jsonl => rendered
                    .lines()
                    .skip(1)
                    .map(|line| serde_json::from_str::<Value>(line).unwrap())
                    .collect(),
                OutputMode::Human | OutputMode::Porcelain => unreachable!(),
            };
            let entry = records
                .iter()
                .find(|record| record["record"] == "entry")
                .expect("real root history entry");
            assert_eq!(entry["subject"], "root history");
            match expected_body {
                None => assert!(entry.get("body").is_none()),
                Some(body) => assert_eq!(entry.get("body").and_then(Value::as_str), Some(body)),
            }
        }
    }
}

#[test]
fn f2_machine_runner_releases_after_typed_read_failure() {
    let workspace = mixed_machine_workspace("log-machine-runner-read-failure");
    let log = machine_log_invocation(workspace.path());
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut bytes = Vec::new();
    let mut captured_id = None;
    let error = run_log_with_registry_and_machine_for_test(
        &log,
        OutputMode::Jsonl,
        workspace.path(),
        "op_machine_read_failure".to_owned(),
        &registry,
        &mut bytes,
        |registry, log_id, output, writer| {
            captured_id = Some(log_id.to_owned());
            registry
                .read(
                    log_id,
                    &gwz_core::operation::CommitLogReadRequest::default(),
                )
                .expect("injected renderer must receive a live spool id");
            write_log_machine_output_with(
                output,
                |_| {
                    Err(gwz_core::model::ModelError::new(
                        gwz_core::model::ErrorCode::IoError,
                        "injected commit-log spool read failure",
                    ))
                },
                writer,
            )
        },
    )
    .expect_err("typed machine read failure must reach the runner");

    assert_eq!(error.code, Some(gwz_core::model::ErrorCode::IoError));
    assert_eq!(error.message, "injected commit-log spool read failure");
    let log_id = captured_id.expect("injected renderer captured the real spool id");
    let released = registry
        .read(
            &log_id,
            &gwz_core::operation::CommitLogReadRequest::default(),
        )
        .expect_err("read failure must not retain the spool");
    assert_eq!(released.code, gwz_core::model::ErrorCode::InvalidRequest);
    registry.release(&log_id);
}

#[test]
fn f2_machine_runner_releases_after_invalid_record() {
    let workspace = mixed_machine_workspace("log-machine-runner-invalid-record");
    let log = machine_log_invocation(workspace.path());
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut bytes = Vec::new();
    let mut captured_id = None;
    let error = run_log_with_registry_and_machine_for_test(
        &log,
        OutputMode::Json,
        workspace.path(),
        "op_machine_invalid_record".to_owned(),
        &registry,
        &mut bytes,
        |registry, log_id, output, writer| {
            captured_id = Some(log_id.to_owned());
            registry
                .read(
                    log_id,
                    &gwz_core::operation::CommitLogReadRequest::default(),
                )
                .expect("injected renderer must receive a live spool id");
            let mut response = Some(gwz_core::operation::CommitLogReadResponse {
                records: vec![gwz_core::LogOutputRecord {
                    kind: gwz_core::LogOutputRecordKind::Entry,
                    entry: None,
                    degradation: None,
                }],
                next_cursor: 1,
                state: gwz_core::operation::CommitLogReadState::Data,
            });
            write_log_machine_output_with(
                output,
                |_| Ok(response.take().expect("one invalid record page")),
                writer,
            )
        },
    )
    .expect_err("inconsistent record must reach the runner");

    assert_eq!(error.code, Some(gwz_core::model::ErrorCode::InternalError));
    assert_eq!(
        error.message,
        "commit-log entry record has inconsistent payload arms"
    );
    let log_id = captured_id.expect("injected renderer captured the real spool id");
    let released = registry
        .read(
            &log_id,
            &gwz_core::operation::CommitLogReadRequest::default(),
        )
        .expect_err("invalid record must not retain the spool");
    assert_eq!(released.code, gwz_core::model::ErrorCode::InvalidRequest);
    registry.release(&log_id);
}

#[test]
fn l_coa_6_all_machine_provenance_tokens_and_marker_invalid_encoding_are_exact() {
    let cases = [
        (gwz_core::LogMergeKind::None, None, "none"),
        (gwz_core::LogMergeKind::Heuristic, None, "heuristic"),
        (
            gwz_core::LogMergeKind::Marker,
            Some(MARKER),
            "marker:01987b0c-2f75-7c4a-9a32-8fd22f7d7c91",
        ),
        (
            gwz_core::LogMergeKind::None,
            Some("marker-invalid"),
            "marker-invalid",
        ),
    ];
    let records = cases
        .iter()
        .map(|(kind, marker, _)| {
            entry_record(
                vec![member("mem_a", "members/a", HASH_A, &[])],
                gwz_core::LogMergeProvenance {
                    kind: *kind,
                    gwz_commit_id: marker.map(str::to_owned),
                },
                "subject",
                None,
                None,
            )
        })
        .collect::<Vec<_>>();
    let (rendered, _) = render(OutputMode::Jsonl, &records);
    assert_eq!(
        rendered
            .lines()
            .skip(1)
            .map(
                |line| serde_json::from_str::<Value>(line).unwrap()["provenance"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            )
            .collect::<Vec<_>>(),
        cases
            .iter()
            .map(|case| case.2.to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn l_jsn_2_degradation_reasons_are_stable_and_optional_context_is_preserved() {
    let cases = [
        (
            gwz_core::LogDegradationReason::RepositoryUnreadable,
            "repository_unreadable",
        ),
        (
            gwz_core::LogDegradationReason::RepositoryMissing,
            "repository_missing",
        ),
        (gwz_core::LogDegradationReason::Unborn, "unborn"),
        (
            gwz_core::LogDegradationReason::RevisionUnresolved,
            "revision_unresolved",
        ),
        (
            gwz_core::LogDegradationReason::SnapshotEntryMissing,
            "snapshot_entry_missing",
        ),
        (
            gwz_core::LogDegradationReason::LockEntryMissing,
            "lock_entry_missing",
        ),
        (
            gwz_core::LogDegradationReason::UnsupportedSourceKind,
            "unsupported_source_kind",
        ),
    ];
    let records = cases
        .iter()
        .map(|case| degradation_record(case.0))
        .collect::<Vec<_>>();
    let (rendered, _) = render(OutputMode::Json, &records);
    let document: Value = serde_json::from_str(&rendered).unwrap();
    let rendered_records = document["records"].as_array().unwrap();
    assert_eq!(
        rendered_records
            .iter()
            .map(|record| record["reason"].as_str().unwrap())
            .collect::<Vec<_>>(),
        cases.iter().map(|case| case.1).collect::<Vec<_>>()
    );
    assert!(rendered_records.iter().all(|record| {
        record["record"] == "degradation"
            && record["member_id"] == "mem_bad"
            && record["member_path"] == "members/bad"
            && record["operand"] == "missing..HEAD"
            && record["message"] == "cannot\nresolve"
    }));
}
