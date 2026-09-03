use super::*;

mod shared_rendering;

pub(crate) use shared_rendering::*;

#[test]
pub(crate) fn error_path_renders_structured_json_envelope() {
    // F9: a top-level error carrying a gwz-core code renders envelope-consistent JSON.
    let error = CliError {
        message: "member has uncommitted changes".to_owned(),
        code: Some(gwz_core::model::ErrorCode::DirtyMember),
        member_id: None,
        member_path: None,
        target_kind: None,
        record_context: None,
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

    let contextual = CliError::from_model(
        gwz_core::model::ModelError::new(
            gwz_core::model::ErrorCode::GitCommandFailed,
            "revspec 'feature/x' not found",
        )
        .with_member("mem_a", "a"),
    );
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&contextual)).unwrap();
    assert_eq!(json["errors"][0]["member_id"], "mem_a");
    assert_eq!(json["errors"][0]["member_path"], "a");
    assert_eq!(json["errors"][0]["target_kind"], "Member");

    let compatibility = CliError::from_model(
        gwz_core::model::ModelError::new(
            gwz_core::model::ErrorCode::UnsupportedRecordVersion,
            "merge record requires A1",
        )
        .with_record_context(gwz_core::MergeRecordCompatibilityContext {
            merge_id: "merge_1".to_owned(),
            schema: Some("gwz.merge-operation/v1".to_owned()),
            record_schema_version: Some(1),
            required_wave: Some(gwz_core::MergeRecordRequiredWave::A1),
            legacy_mode: None,
        }),
    );
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&compatibility)).unwrap();
    assert_eq!(json["errors"][0]["record_context"]["merge_id"], "merge_1");
    assert_eq!(
        json["errors"][0]["record_context"]["schema"],
        "gwz.merge-operation/v1"
    );
    assert_eq!(
        json["errors"][0]["record_context"]["record_schema_version"],
        1
    );
    assert_eq!(json["errors"][0]["record_context"]["required_wave"], "A1");
    assert!(json["errors"][0]["record_context"]["legacy_mode"].is_null());
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
pub(crate) fn response_errors_render_record_compatibility_context() {
    let error = gwz_core::GwzError {
        code: gwz_core::GwzErrorCode::UnsupportedRecordVersion,
        message: "merge record requires A2".to_owned(),
        member_id: None,
        member_path: None,
        detail: None,
        target_kind: None,
        record_context: Some(gwz_core::MergeRecordCompatibilityContext {
            merge_id: "merge_2".to_owned(),
            schema: Some("gwz.merge-operation/v2".to_owned()),
            record_schema_version: Some(2),
            required_wave: Some(gwz_core::MergeRecordRequiredWave::A2),
            legacy_mode: None,
        }),
    };

    let json = error_json(&error);

    assert_eq!(json["record_context"]["merge_id"], "merge_2");
    assert_eq!(json["record_context"]["required_wave"], "A2");
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
pub(crate) fn merge_event_json_keeps_state_outcome_and_artifact_fields() {
    let mut event = sample_event();
    event.kind = gwz_core::EventKind::MemberFinished;
    event.member_id = Some("mem_app".to_owned());
    event.member_path = Some("repos/app".to_owned());
    event.message = None;
    event.target_kind = Some(gwz_core::TargetKind::Member);
    event.merge_state = Some(gwz_core::MergeOperationState::Finalizing);
    event.artifact_path = Some(".gwz/merge/merge_1.yaml".to_owned());
    event.merge_member = Some(gwz_core::MergeRepoSummary {
        target_id: "mem_app".to_owned(),
        target_kind: gwz_core::TargetKind::Member,
        path: "repos/app".to_owned(),
        source_ref: "feature/x".to_owned(),
        source_commit: "source123".to_owned(),
        target_branch: "main".to_owned(),
        before_commit: "before123".to_owned(),
        resulting_commit: Some("merge123".to_owned()),
        live_commit: Some("merge123".to_owned()),
        state: gwz_core::MergeParticipantState::Merged,
        ..gwz_core::MergeRepoSummary::default()
    });

    let json = event_json(&event);
    assert_eq!(json["target_kind"], "Member");
    assert_eq!(json["merge_state"], "Finalizing");
    assert_eq!(json["artifact_path"], ".gwz/merge/merge_1.yaml");
    assert_eq!(json["merge_member"]["target_id"], "mem_app");
    assert_eq!(json["merge_member"]["state"], "Merged");
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../gwz-core/protocol/fixtures/cli_parity/merge_event.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(json, fixture);
}

#[test]
pub(crate) fn merge_renderers_report_open_status_and_structured_drift() {
    let response = parity_merge_response();
    let cli = CliResponse::merge(response);

    let human = render_response(&cli, OutputMode::Human);
    assert_eq!(
        human,
        include_str!("../../tests/fixtures/merge_status_human.txt").trim_end()
    );
    assert!(human.contains("gwz merge --status"));
    assert!(human.contains("gwz merge --continue"));
    assert!(human.contains("gwz merge --abort"));

    let fixture: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(canonical_merge_response_fixture()).unwrap())
            .unwrap();
    assert_eq!(response_json(&cli), fixture);
    for mode in [OutputMode::Json, OutputMode::Jsonl] {
        let rendered: serde_json::Value =
            serde_json::from_str(&render_response(&cli, mode)).unwrap();
        assert_eq!(rendered, fixture);
    }
}

#[test]
pub(crate) fn merge_renderer_reports_only_remaining_post_gc_stash_evidence() {
    let mut response = parity_merge_response();
    let evidence = response.preservation.as_mut().unwrap().first_mut().unwrap();
    evidence.backup_ref = None;
    evidence.backup_commit = None;
    let cli = CliResponse::merge(response);

    let human = render_response(&cli, OutputMode::Human);
    assert!(human.contains("remaining preservation artifacts:"));
    assert!(!human.contains("backup ref:"));
    assert!(human.contains("stash: stash-parity-1 @ stashobj123"));
    let machine = response_json(&cli);
    assert!(machine["merge"]["preservation"][0]["backup_ref"].is_null());
    assert!(machine["merge"]["preservation"][0]["backup_commit"].is_null());
}

#[test]
pub(crate) fn merge_renderer_reports_idle_without_fabricating_an_operation() {
    let response = gwz_core::MergeResponse {
        response: gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                request_id: "req-idle".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Merge,
                aggregate_status: gwz_core::AggregateStatus::Noop,
                operation_id: Some("op-idle".to_owned()),
                message: None,
                attribution: None,
            },
            members: Vec::new(),
            errors: Vec::new(),
        },
        state: gwz_core::MergeOperationState::Idle,
        ..Default::default()
    };
    let cli = CliResponse::merge(response);

    assert_eq!(
        render_response(&cli, OutputMode::Human),
        "action: merge\nstatus: Noop\nstate: idle\nNo coordinated merge is open."
    );
    let json = response_json(&cli);
    assert!(json["merge"]["merge_id"].is_null());
    assert_eq!(json["merge"]["state"], "Idle");
    assert_eq!(json["merge"]["open"], false);
    assert_eq!(json["merge"]["participant_counts"]["total"], 0);
    assert!(json["merge"]["repos"].as_array().unwrap().is_empty());
    assert!(
        json["merge"]["operation_drift"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(json["merge"]["record"].is_null());
}

#[test]
fn merge_record_json_keeps_every_archive_acceptance_variant_explicit() {
    use gwz_core::{MergeAcceptanceKind as Kind, MergeAcceptanceProjection as Acceptance};
    let variants = [
        Acceptance {
            kind: Kind::LegacyComplete,
            legacy_complete: Some(Default::default()),
            legacy_source: Some(gwz_core::MergeLegacyAcceptanceSource::Candidate),
            ..Default::default()
        },
        Acceptance {
            kind: Kind::LegacyUnavailable,
            legacy_evidence: Some(Default::default()),
            missing_gaps: vec![gwz_core::MergeLegacyAcceptanceGap::ExactLockBytes],
            ..Default::default()
        },
        Acceptance {
            kind: Kind::NotAccepted,
            ..Default::default()
        },
    ];
    for acceptance in variants {
        let response = gwz_core::MergeResponse {
            state: gwz_core::MergeOperationState::Aborted,
            record: Some(gwz_core::MergeRecordProjection {
                source_version: gwz_core::MergeRecordVersion::V0,
                archived: true,
                terminal_outcome: Some(gwz_core::MergeTerminalOutcome::Aborted),
                acceptance: Some(acceptance),
                recovery: None,
            }),
            ..Default::default()
        };
        let record = &merge_response_json(&response)["record"];
        assert!(record["recovery"].is_null());
        assert_eq!(record["archived"], true);
        assert!(record["acceptance"]["missing_gaps"].is_array());
        assert!(record["acceptance"].get("legacy_complete").is_some());
        assert!(record["acceptance"].get("legacy_evidence").is_some());
        assert!(record["acceptance"].get("legacy_source").is_some());
        assert!(record["acceptance"].get("supported_persisted").is_some());
        let human = render_merge_response(&response);
        assert!(human.contains("record: v0 (archived)"));
        assert!(human.contains("terminal outcome: aborted"));
        assert!(human.contains("acceptance: "));
    }
}

#[test]
pub(crate) fn merge_json_carries_the_crash_recovery_decision_only_when_one_was_made() {
    // DR-1 §3.4 channel 2: the response is the machine truth, so Json and
    // Porcelain consumers never have to read the stderr warning. An op that
    // decided nothing (abort, status, gc) omits the key rather than rendering
    // a null one, which keeps every pre-DR-1 payload byte-identical.
    let undecided = gwz_core::MergeResponse::default();
    assert!(
        merge_response_json(&undecided)
            .get("crash_recovery")
            .is_none(),
        "{}",
        merge_response_json(&undecided)
    );
    // The warning arrived live on stderr, so the human rendering gains nothing.
    let human_before = render_merge_response(&undecided);

    let rows = [
        (
            gwz_core::MergeCrashRecoveryGap::NoDurableIdentity,
            "NoDurableIdentity",
        ),
        (
            gwz_core::MergeCrashRecoveryGap::RemoteFilesystem,
            "RemoteFilesystem",
        ),
        (
            gwz_core::MergeCrashRecoveryGap::VolatileFilesystem,
            "VolatileFilesystem",
        ),
    ];
    for (gap, label) in rows {
        let response = gwz_core::MergeResponse {
            crash_recovery: Some(gwz_core::MergeCrashRecovery {
                supported: false,
                filesystem: Some("btrfs".to_owned()),
                gap: Some(gap),
            }),
            ..gwz_core::MergeResponse::default()
        };
        let json = merge_response_json(&response);
        assert_eq!(json["crash_recovery"]["supported"], false);
        assert_eq!(json["crash_recovery"]["filesystem"], "btrfs");
        assert_eq!(json["crash_recovery"]["gap"], label);
        assert_eq!(render_merge_response(&response), human_before);
    }

    // Above the bar: supported, with no gap and an optional filesystem name.
    let supported = gwz_core::MergeResponse {
        crash_recovery: Some(gwz_core::MergeCrashRecovery {
            supported: true,
            filesystem: None,
            gap: None,
        }),
        ..gwz_core::MergeResponse::default()
    };
    let json = merge_response_json(&supported);
    assert_eq!(json["crash_recovery"]["supported"], true);
    assert!(json["crash_recovery"]["filesystem"].is_null());
    assert!(json["crash_recovery"]["gap"].is_null());
}

fn canonical_merge_response_fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../gwz-core/protocol/fixtures/cli_parity/merge_response.json")
}

fn parity_merge_response() -> gwz_core::MergeResponse {
    let mut repos = vec![
        merge_repo("lib", gwz_core::MergeParticipantState::Planned),
        merge_repo("docs", gwz_core::MergeParticipantState::Conflicted),
        merge_repo("api", gwz_core::MergeParticipantState::Continued),
        merge_repo("tools", gwz_core::MergeParticipantState::Aborted),
        merge_repo("web", gwz_core::MergeParticipantState::RolledBack),
        merge_repo("worker", gwz_core::MergeParticipantState::Failed),
    ];
    repos[0].predicted = Some(gwz_core::MergeAnalysisKind::TrueMerge);
    repos[0].prediction_complete = Some(true);
    repos[0].conflict_paths = vec!["src/lib.rs".to_owned()];
    repos[0].pending_action = Some(gwz_core::MergePendingActionSummary {
        kind: gwz_core::MergePendingActionKind::TrueMerge,
        state: gwz_core::MergePendingActionState::NotStarted,
        message: Some("Git action is durably journaled and has not started".to_owned()),
    });
    repos[1].conflict_paths = vec!["guide.md".to_owned()];
    repos[1].prediction_complete = Some(false);
    repos[1].continue_eligible = Some(false);
    repos[1].abort_eligible = Some(true);
    repos[1].live_commit = Some("before123".to_owned());
    repos[1].drift = vec![gwz_core::MergeParticipantDrift {
        kind: gwz_core::MergeParticipantDriftKind::HeadAdvanced,
        message: "HEAD advanced while merge was open".to_owned(),
        expected_branch: Some("main".to_owned()),
        live_branch: Some("main".to_owned()),
        expected_head: Some("before123".to_owned()),
        live_head: Some("live456".to_owned()),
        expected_merge_head: Some("source123".to_owned()),
        live_merge_head: Some("source123".to_owned()),
    }];
    for (index, commit) in [(2, "continued123"), (3, "before123"), (4, "before123")] {
        repos[index].prediction_complete = Some(true);
        repos[index].continue_eligible = Some(false);
        repos[index].abort_eligible = Some(false);
        repos[index].resulting_commit = Some(commit.to_owned());
        repos[index].live_commit = Some(commit.to_owned());
    }
    let member_error = gwz_core::GwzError {
        code: gwz_core::GwzErrorCode::GitCommandFailed,
        message: "member 'mem_worker' at 'worker': revspec 'feature/x' not found".to_owned(),
        member_id: Some("mem_worker".to_owned()),
        member_path: Some("worker".to_owned()),
        detail: Some("source ref was not found in the member repository".to_owned()),
        target_kind: Some(gwz_core::TargetKind::Member),
        record_context: None,
    };
    repos[5].prediction_complete = Some(false);
    repos[5].continue_eligible = Some(false);
    repos[5].abort_eligible = Some(false);
    repos[5].error = Some(member_error.clone());

    gwz_core::MergeResponse {
        response: gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                request_id: "req-parity-1".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Merge,
                aggregate_status: gwz_core::AggregateStatus::Failed,
                operation_id: Some("op-parity-1".to_owned()),
                message: None,
                attribution: None,
            },
            members: Vec::new(),
            errors: vec![member_error],
        },
        merge_id: Some("merge-parity-1".to_owned()),
        state: gwz_core::MergeOperationState::Halted,
        open: true,
        participant_counts: gwz_core::MergeParticipantCounts {
            total: 6,
            planned: 1,
            conflicted: 1,
            failed: 1,
            continued: 1,
            aborted: 1,
            rolled_back: 1,
            ..Default::default()
        },
        repos,
        operation_drift: vec![gwz_core::MergeOperationDrift {
            kind: gwz_core::MergeOperationDriftKind::BaselineManifestChanged,
            message: "manifest changed after planning".to_owned(),
        }],
        preservation: Some(vec![gwz_core::MergePreservation {
            target_id: "mem_docs".to_owned(),
            path: "docs".to_owned(),
            backup_ref: Some("refs/gwz/preserve/merge-parity-1/mem_docs".to_owned()),
            backup_commit: Some("backup123".to_owned()),
            stash_id: Some("stash-parity-1".to_owned()),
            stash_object_id: Some("stashobj123".to_owned()),
        }]),
        publication_step: Some(gwz_core::MergePublicationStep::VerifyingPublication),
        record: Some(parity_record_projection()),
        crash_recovery: None,
    }
}

fn parity_record_projection() -> gwz_core::MergeRecordProjection {
    gwz_core::MergeRecordProjection {
        source_version: gwz_core::MergeRecordVersion::V1,
        archived: false,
        terminal_outcome: None,
        acceptance: Some(gwz_core::MergeAcceptanceProjection {
            kind: gwz_core::MergeAcceptanceKind::SupportedPersisted,
            supported_persisted: Some(gwz_core::MergeInstalledAcceptedWorkspaceProjection {
                kind: gwz_core::MergeInstalledAcceptedWorkspaceKind::V1,
                v1: Some(gwz_core::MergeAcceptedWorkspaceV1Projection {
                    operation_baseline_lock_sha256: "baseline-lock-sha".to_owned(),
                    metadata_base: gwz_core::MergeAcceptedMetadataBaseProjection {
                        source: gwz_core::MergeAcceptedMetadataSource::OperationBaseline,
                        source_commit: None,
                        manifest_yaml: "schema: gwz.workspace/v0\n".to_owned(),
                        manifest_sha256: "manifest-sha".to_owned(),
                        lock_yaml: "schema: gwz.lock/v0\n".to_owned(),
                        lock_sha256: "baseline-lock-sha".to_owned(),
                    },
                    lock_yaml: "schema: gwz.lock/v0\n".to_owned(),
                    lock_sha256: "accepted-lock-sha".to_owned(),
                    members: vec![gwz_core::MergeAcceptedMemberV1Projection {
                        member_id: "mem_docs".to_owned(),
                        kind: gwz_core::MergeAcceptedMemberKind::Selected,
                        integration: Some(gwz_core::MergeAcceptedIntegrationProjection {
                            branch: "main".to_owned(),
                            before_commit: "before123".to_owned(),
                            resulting_commit: "result123".to_owned(),
                        }),
                        final_checkout: Some(gwz_core::MergeAcceptedCheckoutProjection {
                            branch: "main".to_owned(),
                            commit: "result123".to_owned(),
                        }),
                        lock_member: Some(gwz_core::MergeAcceptedLockMemberProjection {
                            path: "docs".to_owned(),
                            source_id: "src_docs".to_owned(),
                            source_kind: gwz_core::SourceKind::Git,
                            commit: Some("result123".to_owned()),
                            branch: Some("main".to_owned()),
                            detached: Some(false),
                            upstream: None,
                            dirty: Some(false),
                            materialized: Some(true),
                        }),
                    }],
                    root: gwz_core::MergeAcceptedRootProjection {
                        kind: gwz_core::MergeAcceptedRootKind::BornAttached,
                        commit: Some("root123".to_owned()),
                        symbolic_branch: Some("main".to_owned()),
                        publication_branch: Some("main".to_owned()),
                        lock_worktree_sha256: "root-lock-sha".to_owned(),
                        manifest_worktree_sha256: "root-manifest-sha".to_owned(),
                        lock_commit_sha256: None,
                        manifest_commit_sha256: None,
                    },
                }),
            }),
            legacy_complete: None,
            legacy_source: None,
            legacy_evidence: None,
            missing_gaps: Vec::new(),
        }),
        recovery: Some(gwz_core::MergeRecoveryProjection {
            origin_state: gwz_core::MergeRecoveryOriginState::Finalizing,
            base_phase: gwz_core::MergeCompatibilityBasePhase::PublishingPrefix,
            next_action: gwz_core::MergeCompatibilityNextAction::ReportRecoveryRequired,
            resume_action: gwz_core::MergeCompatibilityNextAction::PublishCandidate,
        }),
    }
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
