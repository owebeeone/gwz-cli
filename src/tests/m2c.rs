use std::fs;

use sha2::{Digest, Sha256};

use super::g01::{TempDir, request_meta};
use super::*;

#[test]
fn top_level_root_error_retains_structured_target_context() {
    let root = CliError::from_model(
        gwz_core::model::ModelError::new(
            gwz_core::model::ErrorCode::MergeDrift,
            "post-merge work exists",
        )
        .with_member("@root", "."),
    );

    let json: serde_json::Value = serde_json::from_str(&render_error_json(&root)).unwrap();
    assert_eq!(json["errors"][0]["member_id"], "@root");
    assert_eq!(json["errors"][0]["member_path"], ".");
    assert_eq!(json["errors"][0]["target_kind"], "Root");
}

/// The charter §2 sentence for a pre-0.14 (v0) envelope, frozen verbatim.
const PRE_014_REFUSAL: &str =
    "this is a pre-0.14 merge; use gwz 0.13.0 (the last release before 0.14) to continue or abort";
/// The open-v1 remedy, deliberately SUPPRESSED for a v0 envelope: under 0.14
/// all three of those verbs refuse, so printing it would be false.
const OPEN_V1_REMEDY: &str = "use merge status, merge continue, or merge abort";

/// Build a workspace with one member and answer its root.
fn stage_gate_workspace(temp: &TempDir) {
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    gwz_core::workspace_ops::handle_create_repo(
        &gwz_core::git::Git2Backend::new(),
        temp.path(),
        gwz_core::CreateRepoRequest {
            meta: request_meta("req_repo"),
            member_path: "repos/app".to_owned(),
            initial_branch: None,
            member_id: Some("mem_app".to_owned()),
            source_id: Some("src_app".to_owned()),
        },
        "op_repo",
    )
    .unwrap();
}

/// Commit ids the v1 decoder accepts: it validates them as lowercase Git
/// object ids, so `before`/`source` placeholders are `MergeRecordUnreadable`.
/// They lead with a letter because an all-digit scalar is a YAML number.
/// (The commit message below is likewise the v1 canonical form: a non-empty
/// body, then the merge/operation trailer the validator requires.)
const BEFORE_COMMIT: &str = "aa11bb22cc33dd44ee55ff66aa77bb88cc99dd00";
const SOURCE_COMMIT: &str = "bb22cc33dd44ee55ff66aa77bb88cc99dd00aa11";

/// Plant one open record under `temp`, in the given envelope.
///
/// The v1 decoder validates the whole body, so the baseline carries the
/// workspace's REAL manifest and lock bytes beside their digests -- what a
/// merge actually writes -- rather than the two placeholder digests a v0
/// envelope was never decoded far enough to check. They are embedded as JSON
/// strings, which are valid YAML double-quoted scalars, so no block-scalar
/// re-indentation can perturb the bytes the digests must match. `root_branch`
/// with no `@root` participant is the unborn-attached root a fresh workspace
/// has.
fn write_open_record(temp: &TempDir, schema: &str, schema_version: u32) {
    let merge_dir = temp.path().join(".gwz/merge");
    fs::create_dir_all(&merge_dir).unwrap();
    let read = |name: &str| fs::read_to_string(temp.path().join("gwz.conf").join(name)).unwrap();
    let manifest_yaml = read("gwz.yml");
    let lock_yaml = read("gwz.lock.yml");
    let digest = |text: &str| format!("{:x}", Sha256::digest(text.as_bytes()));
    let manifest_sha256 = digest(&manifest_yaml);
    let lock_sha256 = digest(&lock_yaml);
    let quoted = |text: &str| serde_json::Value::String(text.to_owned()).to_string();
    let manifest_quoted = quoted(&manifest_yaml);
    let lock_quoted = quoted(&lock_yaml);
    fs::write(
        merge_dir.join("merge_cli.yaml"),
        format!(
            r#"schema: {schema}
record_schema_version: {schema_version}
writer_version: test
workspace_id: ws_cli
merge_id: merge_cli
operation_id: op_merge
state: awaiting_resolution
source_ref: feature/source
created_at: now
baseline:
  lock_sha256: {lock_sha256}
  manifest_sha256: {manifest_sha256}
  lock_yaml: {lock_quoted}
  manifest_yaml: {manifest_quoted}
  root_branch: main
selected_targets: [mem_app]
participants:
  mem_app:
    path: repos/app
    target_kind: member
    target_branch: main
    before_commit: {BEFORE_COMMIT}
    source_commit: {SOURCE_COMMIT}
    commit_message: "merge\n\nGWZ-Merge-ID: merge_cli\nGWZ-Operation-ID: op_merge"
    state: conflicted
    expected_merge_head: {SOURCE_COMMIT}
    conflict_paths: [resolution.txt]
"#
        ),
    )
    .unwrap();
}

/// M5d (`GwzM5-8M5d-Charter.md` §2): the record is **v1**.
///
/// The subject of this test is the stage gate -- that during an open merge
/// `add` reaches a conflicted member's resolution file and is refused at the
/// root -- not the record envelope. `ACTIVE_WRITER_FLOOR` is now `V1`, so v1 is
/// the shape every conflicted merge leaves on disk and the only one that
/// reaches this gate at all; a v0 envelope refuses BOTH stages with the
/// charter's sentence and would erase the distinction this test exists to
/// pin. That refusal is pinned separately by
/// `stage_gate_refuses_a_pre_014_record_without_the_open_v1_remedy`.
#[test]
fn stage_gate_allows_a_conflicted_member_and_rejects_the_root() {
    let temp = TempDir::new("cli-merge-stage-gate");
    stage_gate_workspace(&temp);
    write_open_record(&temp, "gwz.merge-operation/v1", 1);

    fs::write(temp.path().join("repos/app/resolution.txt"), "resolved\n").unwrap();
    let allowed = parse_args_with_request_id(
        vec![
            "--root".to_owned(),
            temp.path().to_string_lossy().into_owned(),
            "add".to_owned(),
            "repos/app/resolution.txt".to_owned(),
        ],
        "req_stage_member",
        temp.path(),
    )
    .unwrap();
    execute_invocation(&allowed).unwrap();

    fs::write(temp.path().join("root-new.txt"), "blocked\n").unwrap();
    let blocked = parse_args_with_request_id(
        vec![
            "--root".to_owned(),
            temp.path().to_string_lossy().into_owned(),
            "add".to_owned(),
            "root-new.txt".to_owned(),
        ],
        "req_stage_root",
        temp.path(),
    )
    .unwrap();
    let error = execute_invocation(&blocked).unwrap_err();
    assert_eq!(error.code, Some(gwz_core::model::ErrorCode::OpenOperation));
    // An open v1 merge answers with the SCOPE refusal, which names the record
    // and points at `merge status` -- proof that the gate read the record
    // rather than stopping at its envelope.
    let message = error.message.clone();
    assert_eq!(
        message,
        "merge 'merge_cli' is open; add may target only its conflicted participants; \
         use merge status to inspect the allowed repositories"
    );
    assert!(!message.contains(PRE_014_REFUSAL), "{message}");
}

/// M5d (`GwzM5-8M5d-Charter.md` §2): a v0 envelope is not a merge lifecycle.
///
/// `add` is the gate's one `Conditional` row -- it consults the record to see
/// whether the path belongs to a conflicted participant. Against a v0 envelope
/// there is no record to consult, because 0.14 never constructs a v0 body, so
/// BOTH stages refuse with the charter's one sentence and the open-v1 remedy is
/// suppressed. The record on disk is otherwise identical to the v1 fixture
/// above, so the envelope is the only variable.
#[test]
fn stage_gate_refuses_a_pre_014_record_without_the_open_v1_remedy() {
    let temp = TempDir::new("cli-merge-stage-gate-v0");
    stage_gate_workspace(&temp);
    write_open_record(&temp, "gwz.merge-operation/v0", 0);

    fs::write(temp.path().join("repos/app/resolution.txt"), "resolved\n").unwrap();
    fs::write(temp.path().join("root-new.txt"), "blocked\n").unwrap();
    for (request_id, path) in [
        ("req_stage_member_v0", "repos/app/resolution.txt"),
        ("req_stage_root_v0", "root-new.txt"),
    ] {
        let invocation = parse_args_with_request_id(
            vec![
                "--root".to_owned(),
                temp.path().to_string_lossy().into_owned(),
                "add".to_owned(),
                path.to_owned(),
            ],
            request_id,
            temp.path(),
        )
        .unwrap();
        let error = execute_invocation(&invocation).unwrap_err();
        assert_eq!(error.code, Some(gwz_core::model::ErrorCode::OpenOperation));
        let message = error.message.clone();
        assert!(
            message.contains(PRE_014_REFUSAL),
            "{path}: expected the charter sentence, got: {message}"
        );
        assert!(
            !message.contains(OPEN_V1_REMEDY),
            "{path}: the open-v1 remedy is false under 0.14: {message}"
        );
    }
}
