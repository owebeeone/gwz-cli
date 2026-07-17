use super::*;

#[test]
pub(crate) fn error_path_renders_structured_json_envelope() {
    // F9: a top-level error carrying a gwz-core code renders envelope-consistent JSON.
    let error = CliError {
        message: "member has uncommitted changes".to_owned(),
        code: Some(gwz_core::model::ErrorCode::DirtyMember),
    };
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
    assert_eq!(json["kind"], "response");
    assert!(json["members"].as_array().unwrap().is_empty());
    assert_eq!(
        json["errors"][0]["message"],
        "member has uncommitted changes"
    );
    assert!(
        json["errors"][0]["code"]
            .as_str()
            .unwrap()
            .contains("Dirty")
    );
    assert_eq!(
        error.human_message(),
        "DirtyMember: member has uncommitted changes"
    );

    // A CLI validation error (no gwz-core code) still renders structured, code null.
    let plain = CliError::new("--json and --jsonl are mutually exclusive");
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&plain)).unwrap();
    assert!(json["errors"][0]["code"].is_null());
    assert_eq!(
        plain.human_message(),
        "--json and --jsonl are mutually exclusive"
    );
}

#[test]
pub(crate) fn json_renderer_outputs_structured_response() {
    let response = CliResponse::envelope(sample_response(
        gwz_core::AggregateStatus::Ok,
        gwz_core::MemberStatus::Ok,
    ));

    let json: serde_json::Value =
        serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();

    assert_eq!(json["kind"], "response");
    assert_eq!(json["meta"]["aggregate_status"], "Ok");
    assert_eq!(json["members"][0]["member_id"], "mem_app");
    assert_eq!(json["members"][0]["status"], "Ok");
}

#[test]
pub(crate) fn jsonl_renderer_emits_response_event_and_result_in_order() {
    let response = sample_response(
        gwz_core::AggregateStatus::Accepted,
        gwz_core::MemberStatus::Planned,
    );
    let event = sample_event();
    let result = sample_result();

    let lines = render_jsonl_stream(&CliResponse::envelope(response), &[event], Some(&result))
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["kind"], "response");
    assert_eq!(lines[1]["kind"], "event");
    assert_eq!(lines[2]["kind"], "result");
}

#[test]
pub(crate) fn merge_renderers_report_action_plan_results_and_m0_guidance() {
    let mut response = gwz_core::MergeResponse {
        response: sample_response(
            gwz_core::AggregateStatus::Conflicted,
            gwz_core::MemberStatus::Conflicted,
        ),
        ..Default::default()
    };
    response.response.meta.action = gwz_core::ActionKind::Merge;
    response.repos = vec![
        merge_repo("lib", gwz_core::MergeParticipantState::Merged),
        merge_repo("docs", gwz_core::MergeParticipantState::Conflicted),
    ];
    response.repos[1].conflict_paths = vec!["guide.md".to_owned()];
    let cli = CliResponse::merge(response);

    let human = render_response(&cli, OutputMode::Human);
    assert!(human.contains("action: merge"));
    assert!(human.contains("feature/x -> main  merged"));
    assert!(human.contains("ordinary Git commands in docs/"));
    assert!(!human.contains("gwz merge --continue"));

    let json = response_json(&cli);
    assert_eq!(json["meta"]["action"], "Merge");
    assert_eq!(json["merge"]["repos"][1]["state"], "Conflicted");
    let jsonl = render_response(&cli, OutputMode::Jsonl);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&jsonl).unwrap()["meta"]["action"],
        "Merge"
    );
}

fn merge_repo(path: &str, state: gwz_core::MergeParticipantState) -> gwz_core::MergeRepoSummary {
    gwz_core::MergeRepoSummary {
        target_id: format!("mem_{path}"),
        target_kind: gwz_core::TargetKind::Member,
        path: path.to_owned(),
        source_ref: "feature/x".to_owned(),
        source_commit: "source123".to_owned(),
        target_branch: "main".to_owned(),
        before_commit: "before123".to_owned(),
        state,
        ..Default::default()
    }
}

#[test]
pub(crate) fn branch_listing_groups_by_branch_and_current_marker() {
    let response = CliResponse::branch(gwz_core::BranchResponse {
        response: branch_response_envelope(),
        repos: Some(vec![
            branch_repo("mem_cli", "gwz-cli", "main", "main"),
            branch_repo("mem_cli", "gwz-cli", "release", "main"),
            branch_repo("mem_core", "gwz-core", "main", "main"),
            branch_repo("mem_py", "gwz-py", "main", "release"),
            branch_repo("mem_py", "gwz-py", "release", "release"),
        ]),
    });

    assert_eq!(
        render_response(&response, OutputMode::Human),
        "*main: gwz-cli gwz-core\nmain: gwz-py\n*release: gwz-py\nrelease: gwz-cli"
    );
}

#[test]
pub(crate) fn branch_listing_uses_member_path_when_short_name_is_ambiguous() {
    let response = CliResponse::branch(gwz_core::BranchResponse {
        response: branch_response_envelope(),
        repos: Some(vec![
            branch_repo("mem_app", "apps/app", "main", "main"),
            branch_repo("mem_vendor_app", "vendor/app", "main", "main"),
        ]),
    });

    assert_eq!(
        render_response(&response, OutputMode::Human),
        "*main: apps/app vendor/app"
    );
}

#[test]
pub(crate) fn human_renderer_smoke_covers_success_rejection_and_member_failure() {
    let success = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Ok,
            gwz_core::MemberStatus::Ok,
        )),
        OutputMode::Human,
    );
    assert!(success.contains("status: Ok"));

    let rejected = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Rejected,
            gwz_core::MemberStatus::Rejected,
        )),
        OutputMode::Human,
    );
    assert!(rejected.contains("status: Rejected"));

    let failed = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Failed,
            gwz_core::MemberStatus::Failed,
        )),
        OutputMode::Human,
    );
    assert!(failed.contains("RemoteRejected"));
}

#[test]
pub(crate) fn human_renderer_surfaces_response_messages() {
    let mut response = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);
    response.meta.message =
        Some("attached mem_shared; no snapshot or marker commit evidence was available".to_owned());

    let rendered = render_response(&CliResponse::envelope(response), OutputMode::Human);

    assert!(rendered.contains("no snapshot or marker commit evidence was available"));
}

fn branch_response_envelope() -> gwz_core::ResponseEnvelope {
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            request_id: "req_branch".to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            action: gwz_core::ActionKind::Branch,
            aggregate_status: gwz_core::AggregateStatus::Ok,
            operation_id: Some("op_branch".to_owned()),
            message: None,
            attribution: None,
        },
        members: Vec::new(),
        errors: Vec::new(),
    }
}

fn branch_repo(
    member_id: &str,
    member_path: &str,
    branch: &str,
    current_branch: &str,
) -> gwz_core::BranchRepoSummary {
    gwz_core::BranchRepoSummary {
        member_id: member_id.to_owned(),
        member_path: member_path.to_owned(),
        source_kind: gwz_core::SourceKind::Git,
        result: gwz_core::BranchActionResult::Listed,
        branch: Some(branch.to_owned()),
        current_branch: Some(current_branch.to_owned()),
        detached: false,
        unborn: false,
        head: Some("abc123".to_owned()),
        upstream: None,
        ahead: Some(0),
        behind: Some(0),
        source_ref: None,
        target_branch: None,
        resulting_commit: None,
        conflict_paths: Vec::new(),
    }
}

#[test]
pub(crate) fn exit_code_mapping_distinguishes_success_rejected_and_failed() {
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Ok,
            gwz_core::MemberStatus::Ok,
        )),
        0
    );
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Rejected,
            gwz_core::MemberStatus::Rejected,
        )),
        2
    );
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Failed,
            gwz_core::MemberStatus::Failed,
        )),
        1
    );
    // F5: a dirty workspace is normal -- exit 0.
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Dirty,
            gwz_core::MemberStatus::Ok,
        )),
        0
    );
    // A conflicted workspace needs action -- exit 1.
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Conflicted,
            gwz_core::MemberStatus::Conflicted,
        )),
        1
    );
}

#[test]
pub(crate) fn no_files_still_surfaces_dirty_member_counts() {
    // F16: `--no-files` suppresses the per-file list (empty `file_changes`), but a dirty
    // member must still surface via its first-class counts — not silently vanish.
    let mut envelope = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);
    envelope.members[0].git_status = Some(dirty_git_status("mem_app", 0, 2, 1));
    let cli = CliResponse::envelope(envelope);
    let workspace = empty_workspace_git_status();

    let mut lines = Vec::new();
    append_suppressed_dirty_summary(&mut lines, &cli, &workspace);
    let out = lines.join("\n");
    assert!(out.contains("file detail omitted"), "got: {out}");
    assert!(
        out.contains("repos/app: 0 staged, 2 unstaged, 1 untracked"),
        "got: {out}"
    );

    // A clean member contributes nothing.
    let mut clean = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);
    clean.members[0].git_status = Some(dirty_git_status("mem_app", 0, 0, 0));
    let mut clean_lines = Vec::new();
    append_suppressed_dirty_summary(&mut clean_lines, &CliResponse::envelope(clean), &workspace);
    assert!(clean_lines.is_empty());
}

pub(crate) fn dirty_git_status(
    member_id: &str,
    staged: i64,
    unstaged: i64,
    untracked: i64,
) -> gwz_core::GitStatus {
    gwz_core::GitStatus {
        member_id: member_id.to_owned(),
        branch: Some("main".to_owned()),
        detached: false,
        head: None,
        upstream: None,
        ahead: None,
        behind: None,
        staged,
        unstaged,
        untracked,
        dirty: staged + unstaged + untracked > 0,
    }
}

pub(crate) fn empty_workspace_git_status() -> gwz_core::WorkspaceGitStatus {
    gwz_core::WorkspaceGitStatus {
        clean: false,
        file_changes: Vec::new(),
        branches: Vec::new(),
        branch_groups: Vec::new(),
        branch_differences: Vec::new(),
        root_status: None,
        root_file_changes: Vec::new(),
    }
}

pub(crate) fn sample_response(
    aggregate_status: gwz_core::AggregateStatus,
    member_status: gwz_core::MemberStatus,
) -> gwz_core::ResponseEnvelope {
    let error = (member_status == gwz_core::MemberStatus::Failed
        || member_status == gwz_core::MemberStatus::Rejected)
        .then(|| gwz_core::GwzError {
            code: gwz_core::GwzErrorCode::RemoteRejected,
            message: "remote rejected".to_owned(),
            member_id: Some("mem_app".to_owned()),
            member_path: Some("repos/app".to_owned()),
            target_kind: None,
            detail: None,
        });
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            request_id: "req_render".to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            action: gwz_core::ActionKind::Status,
            aggregate_status,
            operation_id: Some("op_render".to_owned()),
            message: None,
            attribution: None,
        },
        members: vec![gwz_core::MemberResponse {
            member_id: "mem_app".to_owned(),
            member_path: "repos/app".to_owned(),
            source_kind: gwz_core::SourceKind::Git,
            status: member_status,
            error,
            planned: None,
            state: None,
            git_status: None,
            lock_match: None,
            target_kind: None,
        }],
        errors: Vec::new(),
    }
}

pub(crate) fn sample_event() -> gwz_core::OperationEvent {
    gwz_core::OperationEvent {
        operation_id: "op_render".to_owned(),
        request_id: "req_render".to_owned(),
        sequence: 0,
        timestamp_ms: 1,
        kind: gwz_core::EventKind::OperationStarted,
        severity: gwz_core::Severity::Info,
        member_id: None,
        member_path: None,
        target_kind: None,
        message: Some("started".to_owned()),
        member: None,
        error: None,
        attribution: None,
        progress: None,
        merge_state: None,
    }
}

pub(crate) fn sample_result() -> gwz_core::OperationResult {
    gwz_core::OperationResult {
        operation_id: "op_render".to_owned(),
        request_id: "req_render".to_owned(),
        action: gwz_core::ActionKind::Status,
        aggregate_status: gwz_core::AggregateStatus::Ok,
        started_at_ms: 1,
        finished_at_ms: 2,
        members: Vec::new(),
        errors: Vec::new(),
        attribution: None,
    }
}
