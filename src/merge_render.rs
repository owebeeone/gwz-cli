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
    if let Some(record) = &response.record {
        lines.push(format!(
            "record: {} ({})",
            debug_kebab(record.source_version),
            if record.archived { "archived" } else { "open" }
        ));
        if let Some(outcome) = record.terminal_outcome {
            lines.push(format!("terminal outcome: {}", debug_kebab(outcome)));
        }
        if let Some(acceptance) = &record.acceptance {
            lines.push(format!("acceptance: {}", debug_kebab(acceptance.kind)));
            if !acceptance.missing_gaps.is_empty() {
                lines.push(format!(
                    "acceptance gaps: {}",
                    acceptance
                        .missing_gaps
                        .iter()
                        .map(|gap| debug_kebab(*gap))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        if let Some(recovery) = &record.recovery {
            lines.push(format!(
                "record recovery: {} from {}; resume {}",
                debug_kebab(recovery.base_phase),
                debug_kebab(recovery.origin_state),
                debug_kebab(recovery.resume_action)
            ));
        }
    }
    lines.push(render_participant_counts(&response.participant_counts));
    if let Some(step) = response.publication_step {
        lines.push(format!("publication: {}", debug_kebab(step)));
    }
    if response.open {
        lines.push("recovery commands:".to_owned());
        lines.push("  inspect:  gwz merge --status".to_owned());
        lines.push("  continue: gwz merge --continue".to_owned());
        lines.push("  abort:    gwz merge --abort".to_owned());
        lines.push("  preserve: gwz merge --abort --preserve".to_owned());
    }

    if let Some(entries) = &response.preservation {
        lines.push("remaining preservation artifacts:".to_owned());
        for entry in entries {
            lines.push(format!("  {} ({})", entry.path, entry.target_id));
            if let (Some(name), Some(commit)) =
                (entry.backup_ref.as_deref(), entry.backup_commit.as_deref())
            {
                lines.push(format!("    backup ref: {name} @ {commit}"));
            }
            if let (Some(id), Some(object)) =
                (entry.stash_id.as_deref(), entry.stash_object_id.as_deref())
            {
                lines.push(format!("    stash: {id} @ {object}"));
            }
        }
    }

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
    let mut value = serde_json::json!({
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
        "record": response.record.as_ref().map(merge_record_projection_json),
    });
    // DR-1: the machine truth about crash recovery, so Json/Porcelain and every
    // other consumer never depends on the stderr warning. Ops that made no
    // decision (abort, status, gc) omit the key entirely rather than render a
    // null one, which keeps every pre-DR-1 payload byte-identical.
    if let Some(crash_recovery) = &response.crash_recovery {
        value["crash_recovery"] = merge_crash_recovery_json(crash_recovery);
    }
    value
}

fn merge_crash_recovery_json(value: &gwz_core::MergeCrashRecovery) -> serde_json::Value {
    // M5d (`GwzM5-8M5d-Charter.md` §3): `handles_ok` rides the same object,
    // rendered exactly as its two optional siblings are -- present as a key,
    // `null` when core left it absent, which it does above the bar. Below the
    // bar `false` says the record was written raw and that a selected-root or
    // `--preserve` abort may refuse here, so a machine consumer never has to
    // parse the stderr sentence to learn it.
    serde_json::json!({
        "supported": value.supported,
        "filesystem": value.filesystem,
        "gap": value.gap.map(|gap| format!("{gap:?}")),
        "handles_ok": value.handles_ok,
    })
}

fn merge_record_projection_json(record: &gwz_core::MergeRecordProjection) -> serde_json::Value {
    serde_json::json!({
        "source_version": format!("{:?}", record.source_version),
        "archived": record.archived,
        "terminal_outcome": record.terminal_outcome.map(|value| format!("{value:?}")),
        "acceptance": record.acceptance.as_ref().map(merge_acceptance_json),
        "recovery": record.recovery.as_ref().map(|value| serde_json::json!({
            "origin_state": format!("{:?}", value.origin_state),
            "base_phase": format!("{:?}", value.base_phase),
            "next_action": format!("{:?}", value.next_action),
            "resume_action": format!("{:?}", value.resume_action),
        })),
    })
}

fn merge_acceptance_json(value: &gwz_core::MergeAcceptanceProjection) -> serde_json::Value {
    serde_json::json!({
        "kind": format!("{:?}", value.kind),
        "supported_persisted": value.supported_persisted.as_ref().map(|installed| serde_json::json!({
            "kind": format!("{:?}", installed.kind),
            "v1": installed.v1.as_ref().map(accepted_workspace_json),
        })),
        "legacy_complete": value.legacy_complete.as_ref().map(|workspace| serde_json::json!({
            "baseline_lock_sha256": workspace.baseline_lock_sha256,
            "lock_yaml": workspace.lock_yaml,
            "lock_sha256": workspace.lock_sha256,
            "members": workspace.members.iter().map(accepted_member_json).collect::<Vec<_>>(),
            "root": accepted_root_json(&workspace.root),
        })),
        "legacy_source": value.legacy_source.map(|source| format!("{source:?}")),
        "legacy_evidence": value.legacy_evidence.as_ref().map(|evidence| serde_json::json!({
            "lock_yaml": evidence.lock_yaml,
            "lock_sha256": evidence.lock_sha256,
            "members": evidence.members.iter().map(|member| serde_json::json!({
                "member_id": member.member_id,
                "selected": member.selected,
                "state": member.state.map(|state| format!("{state:?}")),
                "integration": member.integration.as_ref().map(accepted_integration_json),
                "lock_member": member.lock_member.as_ref().map(accepted_lock_member_json),
            })).collect::<Vec<_>>(),
            "root": evidence.root.as_ref().map(accepted_root_json),
            "composition_commit": evidence.composition_commit,
            "composition_tree": evidence.composition_tree,
            "candidate_hashes": evidence.candidate_hashes.iter().map(|hash| serde_json::json!({
                "path": hash.path,
                "sha256": hash.sha256,
            })).collect::<Vec<_>>(),
        })),
        "missing_gaps": value.missing_gaps.iter().map(|gap| format!("{gap:?}")).collect::<Vec<_>>(),
    })
}

fn accepted_workspace_json(
    value: &gwz_core::MergeAcceptedWorkspaceV1Projection,
) -> serde_json::Value {
    serde_json::json!({
        "operation_baseline_lock_sha256": value.operation_baseline_lock_sha256,
        "metadata_base": {
            "source": format!("{:?}", value.metadata_base.source),
            "source_commit": value.metadata_base.source_commit,
            "manifest_yaml": value.metadata_base.manifest_yaml,
            "manifest_sha256": value.metadata_base.manifest_sha256,
            "lock_yaml": value.metadata_base.lock_yaml,
            "lock_sha256": value.metadata_base.lock_sha256,
        },
        "lock_yaml": value.lock_yaml,
        "lock_sha256": value.lock_sha256,
        "members": value.members.iter().map(accepted_member_json).collect::<Vec<_>>(),
        "root": accepted_root_json(&value.root),
    })
}

fn accepted_member_json(value: &gwz_core::MergeAcceptedMemberV1Projection) -> serde_json::Value {
    serde_json::json!({
        "member_id": value.member_id,
        "kind": format!("{:?}", value.kind),
        "integration": value.integration.as_ref().map(accepted_integration_json),
        "final_checkout": value.final_checkout.as_ref().map(|checkout| serde_json::json!({
            "branch": checkout.branch,
            "commit": checkout.commit,
        })),
        "lock_member": value.lock_member.as_ref().map(accepted_lock_member_json),
    })
}

fn accepted_integration_json(
    value: &gwz_core::MergeAcceptedIntegrationProjection,
) -> serde_json::Value {
    serde_json::json!({
        "branch": value.branch,
        "before_commit": value.before_commit,
        "resulting_commit": value.resulting_commit,
    })
}

fn accepted_lock_member_json(
    value: &gwz_core::MergeAcceptedLockMemberProjection,
) -> serde_json::Value {
    serde_json::json!({
        "path": value.path,
        "source_id": value.source_id,
        "source_kind": format!("{:?}", value.source_kind),
        "commit": value.commit,
        "branch": value.branch,
        "detached": value.detached,
        "upstream": value.upstream,
        "dirty": value.dirty,
        "materialized": value.materialized,
    })
}

fn accepted_root_json(value: &gwz_core::MergeAcceptedRootProjection) -> serde_json::Value {
    serde_json::json!({
        "kind": format!("{:?}", value.kind),
        "commit": value.commit,
        "symbolic_branch": value.symbolic_branch,
        "publication_branch": value.publication_branch,
        "lock_worktree_sha256": value.lock_worktree_sha256,
        "manifest_worktree_sha256": value.manifest_worktree_sha256,
        "lock_commit_sha256": value.lock_commit_sha256,
        "manifest_commit_sha256": value.manifest_commit_sha256,
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
