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

#[test]
fn stage_gate_allows_a_conflicted_member_and_rejects_the_root() {
    let temp = TempDir::new("cli-merge-stage-gate");
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

    let merge_dir = temp.path().join(".gwz/merge");
    fs::create_dir_all(&merge_dir).unwrap();
    let digest = |path| format!("{:x}", Sha256::digest(fs::read(path).unwrap()));
    let lock_sha256 = digest(temp.path().join("gwz.conf/gwz.lock.yml"));
    let manifest_sha256 = digest(temp.path().join("gwz.conf/gwz.yml"));
    fs::write(
        merge_dir.join("merge_cli.yaml"),
        format!(
            r#"schema: gwz.merge-operation/v0
record_schema_version: 0
writer_version: test
workspace_id: ws_cli
merge_id: merge_cli
operation_id: op_merge
state: awaiting_resolution
source_ref: feature/source
created_at: now
baseline: {{lock_sha256: {lock_sha256}, manifest_sha256: {manifest_sha256}}}
selected_targets: [mem_app]
participants:
  mem_app:
    path: repos/app
    target_kind: member
    target_branch: main
    before_commit: before
    source_commit: source
    commit_message: merge
    state: conflicted
    expected_merge_head: source
    conflict_paths: [resolution.txt]
"#
        ),
    )
    .unwrap();

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
}
