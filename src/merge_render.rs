use crate::*;

pub(crate) fn render_merge_response(response: &gwz_core::MergeResponse) -> String {
    let mut lines = vec![
        "action: merge".to_owned(),
        format!("status: {:?}", response.response.meta.aggregate_status),
        format!("state: {}", debug_kebab(response.state)),
    ];

    if response.state == gwz_core::MergeOperationState::Idle {
        lines.push("No coordinated merge is open.".to_owned());
        return lines.join("\n");
    }

    lines.push(format!(
        "merge: {} ({})",
        response.merge_id.as_deref().unwrap_or("unknown"),
        if response.open { "open" } else { "closed" }
    ));
    lines.push(render_participant_counts(&response.participant_counts));
    if let Some(step) = response.publication_step {
        lines.push(format!("publication: {}", debug_kebab(step)));
    }
    lines.push("recovery: participant eligibility shown below".to_owned());

    if !response.operation_drift.is_empty() {
        lines.push("operation drift:".to_owned());
        for drift in &response.operation_drift {
            lines.push(format!("  {}: {}", debug_kebab(drift.kind), drift.message));
        }
    }

    if !response.repos.is_empty() {
        lines.push("participants:".to_owned());
    }
    for repo in &response.repos {
        let mut line = format!(
            "  {} ({})  {}",
            repo.path,
            repo.target_id,
            merge_state_label(repo.state)
        );
        if repo.state == gwz_core::MergeParticipantState::Planned
            && let Some(predicted) = repo.predicted
        {
            line.push_str(&format!(" ({})", merge_analysis_label(predicted)));
        }
        lines.push(line);
        lines.push(format!(
            "    source: {} @ {}",
            repo.source_ref, repo.source_commit
        ));
        lines.push(format!(
            "    recorded: branch {}; before {}; result {}",
            repo.target_branch,
            repo.before_commit,
            repo.resulting_commit.as_deref().unwrap_or("-")
        ));
        lines.push(format!(
            "    live: commit {}",
            repo.live_commit.as_deref().unwrap_or("unknown"),
        ));
        lines.push(format!(
            "    recovery: continue {}; abort {}",
            eligibility_label(repo.continue_eligible),
            eligibility_label(repo.abort_eligible)
        ));
        if let Some(pending) = &repo.pending_action {
            let detail = pending
                .message
                .as_deref()
                .map(|message| format!(": {message}"))
                .unwrap_or_default();
            lines.push(format!(
                "    pending action: {} ({}){}",
                debug_kebab(pending.kind),
                debug_kebab(pending.state),
                detail
            ));
        }
        if !repo.conflict_paths.is_empty() {
            lines.push(format!("    conflicts: {}", repo.conflict_paths.join(", ")));
        }
        for drift in &repo.drift {
            lines.push(format!(
                "    drift: {}: {}",
                debug_kebab(drift.kind),
                drift.message
            ));
        }
        if let Some(error) = &repo.error {
            lines.push(format!("    error: {:?}: {}", error.code, error.message));
        }
    }
    for error in &response.response.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

fn render_participant_counts(counts: &gwz_core::MergeParticipantCounts) -> String {
    let values = [
        ("planned", counts.planned),
        ("up-to-date", counts.up_to_date),
        ("fast-forwarded", counts.fast_forwarded),
        ("merged", counts.merged),
        ("conflicted", counts.conflicted),
        ("failed", counts.failed),
        ("unattempted", counts.unattempted),
        ("continued", counts.continued),
        ("aborted", counts.aborted),
        ("rolled-back", counts.rolled_back),
    ];
    let details = values
        .into_iter()
        .filter(|(_, count)| *count != 0)
        .map(|(label, count)| format!("{label} {count}"))
        .collect::<Vec<_>>();
    if details.is_empty() {
        format!("participants: total {}", counts.total)
    } else {
        format!(
            "participants: total {}; {}",
            counts.total,
            details.join("; ")
        )
    }
}

fn eligibility_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "eligible",
        Some(false) => "blocked",
        None => "unknown",
    }
}

fn debug_kebab(value: impl std::fmt::Debug) -> String {
    format!("{value:?}")
        .chars()
        .enumerate()
        .fold(String::new(), |mut label, (index, ch)| {
            if index > 0 && ch.is_ascii_uppercase() {
                label.push('-');
            }
            label.push(ch.to_ascii_lowercase());
            label
        })
}

pub(crate) fn merge_response_json(response: &gwz_core::MergeResponse) -> serde_json::Value {
    serde_json::json!({
        "merge_id": response.merge_id,
        "state": format!("{:?}", response.state),
        "open": response.open,
        "participant_counts": {
            "total": response.participant_counts.total,
            "planned": response.participant_counts.planned,
            "up_to_date": response.participant_counts.up_to_date,
            "fast_forwarded": response.participant_counts.fast_forwarded,
            "merged": response.participant_counts.merged,
            "conflicted": response.participant_counts.conflicted,
            "failed": response.participant_counts.failed,
            "unattempted": response.participant_counts.unattempted,
            "continued": response.participant_counts.continued,
            "aborted": response.participant_counts.aborted,
            "rolled_back": response.participant_counts.rolled_back,
        },
        "repos": response.repos.iter().map(merge_repo_summary_json).collect::<Vec<_>>(),
        "operation_drift": response.operation_drift.iter().map(|drift| serde_json::json!({
            "kind": format!("{:?}", drift.kind),
            "message": drift.message,
        })).collect::<Vec<_>>(),
        "preservation": response.preservation.as_ref().map(|entries| entries.iter().map(|entry| {
            serde_json::json!({
                "target_id": entry.target_id,
                "path": entry.path,
                "backup_ref": entry.backup_ref,
                "backup_commit": entry.backup_commit,
                "stash_id": entry.stash_id,
                "stash_object_id": entry.stash_object_id,
            })
        }).collect::<Vec<_>>()),
        "publication_step": response.publication_step.map(|step| format!("{step:?}")),
    })
}

pub(crate) fn merge_repo_summary_json(repo: &gwz_core::MergeRepoSummary) -> serde_json::Value {
    serde_json::json!({
        "target_id": repo.target_id,
        "target_kind": format!("{:?}", repo.target_kind),
        "path": repo.path,
        "source_ref": repo.source_ref,
        "source_commit": repo.source_commit,
        "target_branch": repo.target_branch,
        "before_commit": repo.before_commit,
        "resulting_commit": repo.resulting_commit,
        "live_commit": repo.live_commit,
        "state": format!("{:?}", repo.state),
        "predicted": repo.predicted.map(|value| format!("{value:?}")),
        "prediction_complete": repo.prediction_complete,
        "conflict_paths": repo.conflict_paths,
        "continue_eligible": repo.continue_eligible,
        "abort_eligible": repo.abort_eligible,
        "drift": repo.drift.iter().map(merge_participant_drift_json).collect::<Vec<_>>(),
        "error": repo.error.as_ref().map(error_json),
        "pending_action": repo.pending_action.as_ref().map(|pending| serde_json::json!({
            "kind": format!("{:?}", pending.kind),
            "state": format!("{:?}", pending.state),
            "message": pending.message,
        })),
    })
}

fn merge_participant_drift_json(drift: &gwz_core::MergeParticipantDrift) -> serde_json::Value {
    serde_json::json!({
        "kind": format!("{:?}", drift.kind),
        "message": drift.message,
        "expected_branch": drift.expected_branch,
        "live_branch": drift.live_branch,
        "expected_head": drift.expected_head,
        "live_head": drift.live_head,
        "expected_merge_head": drift.expected_merge_head,
        "live_merge_head": drift.live_merge_head,
    })
}

fn merge_state_label(state: gwz_core::MergeParticipantState) -> &'static str {
    match state {
        gwz_core::MergeParticipantState::Planned => "planned",
        gwz_core::MergeParticipantState::UpToDate => "up-to-date",
        gwz_core::MergeParticipantState::FastForwarded => "fast-forwarded",
        gwz_core::MergeParticipantState::Merged => "merged",
        gwz_core::MergeParticipantState::Conflicted => "conflicted",
        gwz_core::MergeParticipantState::Failed => "failed",
        gwz_core::MergeParticipantState::Unattempted => "unattempted",
        gwz_core::MergeParticipantState::Continued => "continued",
        gwz_core::MergeParticipantState::Aborted => "aborted",
        gwz_core::MergeParticipantState::RolledBack => "rolled-back",
    }
}

fn merge_analysis_label(kind: gwz_core::MergeAnalysisKind) -> &'static str {
    match kind {
        gwz_core::MergeAnalysisKind::UpToDate => "up-to-date",
        gwz_core::MergeAnalysisKind::FastForward => "fast-forward",
        gwz_core::MergeAnalysisKind::TrueMerge => "merge commit",
        gwz_core::MergeAnalysisKind::Unknown => "unknown",
    }
}
