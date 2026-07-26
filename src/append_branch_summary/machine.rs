use crate::*;

use super::*;

/// Streams each operation event to stdout as a JSON line, flushed immediately,
/// so `--jsonl` consumers see records live as the operation runs instead of
/// batched at the end. stdout is block-buffered when piped, hence the flush.
pub(crate) struct JsonlSink;

impl gwz_core::operation::EventSink for JsonlSink {
    fn deliver(&self, event: gwz_core::OperationEvent) {
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{}", event_json(&event));
        let _ = out.flush();
    }
}

pub(crate) fn render_jsonl_stream(
    response: &CliResponse,
    events: &[gwz_core::OperationEvent],
    result: Option<&gwz_core::OperationResult>,
) -> String {
    let mut lines = Vec::with_capacity(1 + events.len() + usize::from(result.is_some()));
    lines.push(response_json(response).to_string());
    lines.extend(events.iter().map(|event| event_json(event).to_string()));
    if let Some(result) = result {
        lines.push(result_json(result).to_string());
    }
    lines.join("\n")
}

pub(crate) fn response_json(response: &CliResponse) -> serde_json::Value {
    serde_json::json!({
        "kind": "response",
        "meta": response_meta_json(&response.envelope.meta),
        "members": response.envelope.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": response.envelope.errors.iter().map(error_json).collect::<Vec<_>>(),
        "workspace_git_status": response.workspace_git_status.as_ref().map(workspace_git_status_json),
        "branch_repos": response.branch_repos.as_ref().map(|repos| {
            repos.iter().map(branch_repo_json).collect::<Vec<_>>()
        }),
        "merge": response.merge_response.as_ref().map(merge_response_json),
        "stash_bundles": response.stash_bundles.as_ref().map(|bundles| {
            bundles.iter().map(stash_bundle_json).collect::<Vec<_>>()
        }),
    })
}

pub(crate) fn branch_repo_json(repo: &gwz_core::BranchRepoSummary) -> serde_json::Value {
    serde_json::json!({
        "member_id": repo.member_id,
        "member_path": repo.member_path,
        "source_kind": format!("{:?}", repo.source_kind),
        "result": format!("{:?}", repo.result),
        "branch": repo.branch,
        "current_branch": repo.current_branch,
        "detached": repo.detached,
        "unborn": repo.unborn,
        "head": repo.head,
        "upstream": repo.upstream,
        "ahead": repo.ahead,
        "behind": repo.behind,
        "source_ref": repo.source_ref,
        "target_branch": repo.target_branch,
        "resulting_commit": repo.resulting_commit,
        "conflict_paths": repo.conflict_paths,
    })
}

pub(crate) fn stash_bundle_json(bundle: &gwz_core::StashBundle) -> serde_json::Value {
    serde_json::json!({
        "schema": bundle.schema,
        "workspace_id": bundle.workspace_id,
        "stash_id": bundle.stash_id,
        "created_at": bundle.created_at,
        "message_suffix": bundle.message_suffix,
        "include_untracked": bundle.include_untracked,
        "include_ignored": bundle.include_ignored,
        "selected_members": bundle.selected_members,
        "members": bundle.members.iter().map(stash_bundle_member_json).collect::<Vec<_>>(),
        "warnings": bundle.warnings.iter().map(|warning| serde_json::json!({
            "code": warning.code,
            "message": warning.message,
            "member_id": warning.member_id,
        })).collect::<Vec<_>>(),
        "drift": bundle.drift.iter().map(|drift| serde_json::json!({
            "code": drift.code,
            "message": drift.message,
            "member_id": drift.member_id,
        })).collect::<Vec<_>>(),
    })
}

pub(crate) fn stash_bundle_member_json(member: &gwz_core::StashBundleMember) -> serde_json::Value {
    serde_json::json!({
        "member_id": member.member_id,
        "path": member.path,
        "participation": format!("{:?}", member.participation),
        "push_lifecycle": format!("{:?}", member.push_lifecycle),
        "restore_state": format!("{:?}", member.restore_state),
        "branch_before": member.branch_before,
        "head_before": member.head_before,
        "full_stash_message": member.full_stash_message,
        "dirty_summary": {
            "staged": member.dirty_summary.staged,
            "unstaged": member.dirty_summary.unstaged,
            "untracked": member.dirty_summary.untracked,
            "ignored": member.dirty_summary.ignored,
        },
        "native_stash_object_id": member.native_stash_object_id,
        "native_stash_display_ref": member.native_stash_display_ref,
        "error": member.error.as_ref().map(|error| serde_json::json!({
            "code": error.code,
            "message": error.message,
        })),
    })
}

/// F9: render a top-level CLI error as structured JSON, envelope-consistent with
/// `response_json` (same keys; the error sits in `errors`, no members).
pub(crate) fn render_error_json(error: &CliError) -> String {
    serde_json::json!({
        "kind": "response",
        "meta": serde_json::Value::Null,
        "members": [],
        "errors": [{
            "code": error
                .code
                .map(|code| format!("{:?}", gwz_core::GwzErrorCode::from(code))),
            "message": error.message,
            "member_id": error.member_id,
            "member_path": error.member_path,
            "target_kind": error.target_kind,
            "detail": serde_json::Value::Null,
        }],
        "workspace_git_status": serde_json::Value::Null,
    })
    .to_string()
}

pub(crate) fn result_json(result: &gwz_core::OperationResult) -> serde_json::Value {
    serde_json::json!({
        "kind": "result",
        "operation_id": result.operation_id,
        "request_id": result.request_id,
        "action": format!("{:?}", result.action),
        "aggregate_status": format!("{:?}", result.aggregate_status),
        "started_at_ms": result.started_at_ms,
        "finished_at_ms": result.finished_at_ms,
        "members": result.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": result.errors.iter().map(error_json).collect::<Vec<_>>(),
        "attribution": result.attribution.as_ref().map(attribution_json),
    })
}

pub(crate) fn event_json(event: &gwz_core::OperationEvent) -> serde_json::Value {
    serde_json::json!({
        "kind": "event",
        "operation_id": event.operation_id,
        "request_id": event.request_id,
        "sequence": event.sequence,
        "timestamp_ms": event.timestamp_ms,
        "event_kind": format!("{:?}", event.kind),
        "severity": format!("{:?}", event.severity),
        "member_id": event.member_id,
        "member_path": event.member_path,
        "message": event.message,
        "member": event.member.as_ref().map(member_json),
        "error": event.error.as_ref().map(error_json),
        "attribution": event.attribution.as_ref().map(attribution_json),
        "progress": event.progress.as_ref().map(git_transfer_progress_json),
        "target_kind": event.target_kind.map(|value| format!("{value:?}")),
        "merge_state": event.merge_state.map(|value| format!("{value:?}")),
        "merge_member": event.merge_member.as_ref().map(merge_repo_summary_json),
        "artifact_path": event.artifact_path,
    })
}

fn attribution_json(attribution: &gwz_core::OperationAttribution) -> serde_json::Value {
    serde_json::json!({
        "actor": attribution.actor.as_ref().map(|actor| serde_json::json!({
            "actor_id": actor.actor_id,
            "display_name": actor.display_name,
            "email": actor.email,
            "authority": actor.authority,
        })),
        "git_author": attribution.git_author.as_ref().map(git_identity_json),
        "git_committer": attribution.git_committer.as_ref().map(git_identity_json),
        "credential_ref": attribution.credential_ref,
    })
}

fn git_identity_json(identity: &gwz_core::GitObjectIdentity) -> serde_json::Value {
    serde_json::json!({
        "name": identity.name,
        "email": identity.email,
        "time_ms": identity.time_ms,
        "timezone_offset_minutes": identity.timezone_offset_minutes,
    })
}
