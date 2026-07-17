use crate::*;

pub(crate) fn render_merge_response(response: &gwz_core::MergeResponse) -> String {
    let mut lines = vec![
        "action: merge".to_owned(),
        format!("status: {:?}", response.response.meta.aggregate_status),
    ];
    for repo in &response.repos {
        let state = merge_state_label(repo.state);
        let outcome = match repo.predicted {
            Some(predicted) if repo.state == gwz_core::MergeParticipantState::Planned => {
                format!("{state} ({})", merge_analysis_label(predicted))
            }
            _ => state.to_owned(),
        };
        let mut line = format!(
            "{}  {} -> {}  {}",
            repo.path, repo.source_ref, repo.target_branch, outcome
        );
        if let Some(commit) = repo.resulting_commit.as_ref().or(repo.live_commit.as_ref()) {
            line.push_str(&format!("  {commit}"));
        }
        if !repo.conflict_paths.is_empty() {
            line.push_str(&format!("  {}", repo.conflict_paths.join(", ")));
        }
        if let Some(error) = &repo.error {
            line.push_str(&format!("  {:?}: {}", error.code, error.message));
        }
        lines.push(line);
    }
    for error in &response.response.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    for repo in response
        .repos
        .iter()
        .filter(|repo| repo.state == gwz_core::MergeParticipantState::Conflicted)
    {
        lines.push(String::new());
        lines.push(format!(
            "Resolve or abort this member with ordinary Git commands in {}/.",
            repo.path.trim_end_matches('/')
        ));
    }
    if response
        .repos
        .iter()
        .any(|repo| repo.state == gwz_core::MergeParticipantState::Conflicted)
    {
        lines.push(
            "Other members may already have changed; M0 has no coordinated continue or".into(),
        );
        lines.push("rollback. The workspace lock reflects clean member outcomes.".into());
    }
    lines.join("\n")
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
        },
        "repos": response.repos.iter().map(|repo| serde_json::json!({
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
            "error": repo.error.as_ref().map(error_json),
        })).collect::<Vec<_>>(),
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
