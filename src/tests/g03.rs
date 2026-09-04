use super::*;

pub(crate) fn progress_event(
    kind: gwz_core::EventKind,
    member_path: Option<&str>,
    progress: Option<gwz_core::GitTransferProgress>,
) -> gwz_core::OperationEvent {
    gwz_core::OperationEvent {
        operation_id: "op".to_owned(),
        request_id: "req".to_owned(),
        sequence: 0,
        timestamp_ms: 0,
        kind,
        severity: gwz_core::Severity::Info,
        member_id: member_path.map(|_| "m".to_owned()),
        member_path: member_path.map(str::to_owned),
        target_kind: None,
        message: None,
        member: None,
        error: None,
        attribution: None,
        progress,
        merge_state: None,
        merge_member: None,
        artifact_path: None,
    }
}

pub(crate) fn receiving(recv: i64, total: i64, bytes: i64) -> gwz_core::GitTransferProgress {
    gwz_core::GitTransferProgress {
        phase: gwz_core::GitProgressPhase::Receiving,
        received_objects: Some(recv),
        total_objects: Some(total),
        received_bytes: Some(bytes),
        indexed_deltas: None,
        total_deltas: None,
    }
}

#[test]
pub(crate) fn progress_model_folds_member_lifecycle() {
    use gwz_core::EventKind;
    let mut model = ProgressModel::new("cloning");

    assert!(model.apply(&progress_event(
        EventKind::MemberStarted,
        Some("repos/foo"),
        None
    )));
    assert!(model.apply(&progress_event(
        EventKind::MemberStarted,
        Some("repos/bar"),
        None
    )));
    assert_eq!((model.started, model.finished, model.active()), (2, 0, 2));

    assert!(model.apply(&progress_event(
        EventKind::MemberProgress,
        Some("repos/foo"),
        Some(receiving(10, 100, 2048)),
    )));
    assert_eq!(model.current_path.as_deref(), Some("repos/foo"));
    assert!(model.current_progress.is_some());

    // Finishing the current member clears the surfaced detail.
    assert!(model.apply(&progress_event(
        EventKind::MemberFinished,
        Some("repos/foo"),
        None
    )));
    assert_eq!((model.finished, model.active()), (1, 1));
    assert_eq!(model.current_path, None);
    assert!(model.current_progress.is_none());

    // Finishing a non-current member only moves the counts.
    model.current_path = Some("repos/bar".to_owned());
    assert!(model.apply(&progress_event(
        EventKind::MemberFinished,
        Some("repos/baz"),
        None
    )));
    assert_eq!((model.finished, model.active()), (2, 0));
    assert_eq!(model.current_path.as_deref(), Some("repos/bar"));
}

#[test]
pub(crate) fn progress_model_ignores_non_member_events() {
    use gwz_core::EventKind;
    let mut model = ProgressModel::new("materializing");
    assert!(!model.apply(&progress_event(EventKind::OperationStarted, None, None)));
    assert!(!model.apply(&progress_event(EventKind::ArtifactWritten, None, None)));
    assert!(!model.apply(&progress_event(EventKind::OperationFinished, None, None)));
    assert_eq!((model.started, model.finished), (0, 0));
}

pub(crate) fn diagnostic_event(
    severity: gwz_core::Severity,
    message: &str,
) -> gwz_core::OperationEvent {
    let mut event = progress_event(gwz_core::EventKind::Diagnostic, None, None);
    event.severity = severity;
    event.message = Some(message.to_owned());
    event
}

#[test]
pub(crate) fn diagnostic_echo_labels_by_severity_and_prints_each_text_once() {
    // DR-1 §3.5: core has no stderr, so the human sink is the only channel a
    // warning reaches a person by. It must survive a repeated emission without
    // spamming the terminal, and must not echo progress or debug chatter.
    let echo = DiagnosticEcho::default();
    let warning = "crash recovery is unsupported on btrfs (no durable filesystem identity). \
Merge will continue. Use --filesystem-strict to refuse.";
    // M5d (`GwzM5-8M5d-Charter.md` §3): on a volume that also fails the handle
    // probe, core appends the reverse-door limit to that SAME diagnostic
    // rather than raising a second warning class. Ship (1)'s sentence stays
    // byte-identical at the head, so the pin above is unchanged and this is an
    // additional pin, not a weakening of it.
    let handle_fail_warning = "crash recovery is unsupported on overlay (no durable filesystem \
identity). Merge will continue. Use --filesystem-strict to refuse. Selected-root and --preserve \
abort may refuse until the workspace is on a handle-capable volume.";

    assert_eq!(
        echo.line_for(&diagnostic_event(gwz_core::Severity::Warn, warning)),
        Some(format!("warning: {warning}"))
    );
    // The same text a second time in one invocation prints nothing.
    assert_eq!(
        echo.line_for(&diagnostic_event(gwz_core::Severity::Warn, warning)),
        None
    );
    // The appended form is a different string and must not be swallowed by the
    // dedup of the sentence it extends.
    assert_eq!(
        echo.line_for(&diagnostic_event(
            gwz_core::Severity::Warn,
            handle_fail_warning
        )),
        Some(format!("warning: {handle_fail_warning}"))
    );
    assert_eq!(
        echo.line_for(&diagnostic_event(
            gwz_core::Severity::Warn,
            handle_fail_warning
        )),
        None
    );
    assert_eq!(
        echo.line_for(&diagnostic_event(gwz_core::Severity::Error, "probe failed")),
        Some("error: probe failed".to_owned())
    );
    // De-duplication is per text, not per severity or per kind.
    assert_eq!(
        echo.line_for(&diagnostic_event(gwz_core::Severity::Warn, "probe failed")),
        Some("warning: probe failed".to_owned())
    );
    assert_eq!(
        echo.line_for(&diagnostic_event(
            gwz_core::Severity::Info,
            "just so you know"
        )),
        None
    );
    let mut member = progress_event(gwz_core::EventKind::MemberStarted, Some("repos/foo"), None);
    member.severity = gwz_core::Severity::Warn;
    member.message = Some("started".to_owned());
    assert_eq!(echo.line_for(&member), None);
    let mut empty = diagnostic_event(gwz_core::Severity::Warn, "");
    empty.message = None;
    assert_eq!(echo.line_for(&empty), None);
}

#[test]
pub(crate) fn render_progress_line_shows_counts_and_receiving_detail() {
    let model = ProgressModel {
        label: "cloning".to_owned(),
        started: 3,
        finished: 1,
        current_path: Some("repos/app.git".to_owned()),
        current_progress: Some(receiving(1234, 2730, 3_400_000)),
    };
    assert_eq!(
        render_progress_line(&model, 0),
        "⠋ cloning: 1 done, 2 active · app receiving 45% (1234/2730), 3.2 MiB"
    );
}
