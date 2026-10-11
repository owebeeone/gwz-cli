//! Notes above the progress line (the adaptive concurrency design's §9): a
//! note is a line of its own, printed whether or not stderr is a terminal,
//! and on a terminal the live progress block is redrawn below it.
use super::*;
use progress_detail::Screen;
use std::sync::{Arc, Mutex, atomic::AtomicUsize};

/// A terminal that keeps what it is told, in order: `clear`, `line:<text>`
/// and `draw:<text>`.
#[derive(Clone, Default)]
pub(crate) struct Kept(Arc<Mutex<Vec<String>>>);

impl Screen for Kept {
    fn clear_line(&self) {
        self.0.lock().unwrap().push("clear".into());
    }
    fn write_line(&self, line: &str) {
        self.0.lock().unwrap().push(format!("line:{line}"));
    }
    fn write_str(&self, text: &str) {
        self.0.lock().unwrap().push(format!("draw:{text}"));
    }
    fn columns(&self) -> usize {
        0
    }
}

impl Kept {
    /// What the screen was told so far, in order.
    pub(crate) fn told(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// A sink over a screen that keeps what it is told; `enabled` is whether it
/// draws a live line, as on a terminal.
pub(crate) fn sink(enabled: bool) -> (StderrProgressSink, Kept) {
    let kept = Kept::default();
    let sink = StderrProgressSink {
        screen: Box::new(kept.clone()),
        enabled,
        state: Mutex::new(ProgressModel::new("fetching")),
        tick: AtomicUsize::new(0),
        diagnostics: DiagnosticEcho::default(),
    };
    (sink, kept)
}

#[test]
fn a_note_is_its_own_line_when_stderr_is_not_a_terminal() {
    let (sink, kept) = sink(false);
    sink.println_above("note: github.com:443 asked gwz to wait 20 s; waiting");
    assert_eq!(
        kept.told(),
        ["line:note: github.com:443 asked gwz to wait 20 s; waiting"],
        "nothing is cleared or redrawn when there is no live line"
    );
}

#[test]
fn on_a_terminal_a_note_clears_the_progress_line_and_the_block_is_redrawn_below_it() {
    let (sink, kept) = sink(true);
    sink.println_above("note: github.com:443: now using 12 connections");
    let told = kept.told();
    assert_eq!(told.len(), 3, "{told:?}");
    assert_eq!(told[0], "clear", "the live line is cleared first");
    assert_eq!(told[1], "line:note: github.com:443: now using 12 connections");
    assert!(
        told[2].starts_with("draw:") && told[2].contains("fetching"),
        "and drawn again after it: {told:?}"
    );
}

#[test]
fn a_diagnostic_is_printed_the_same_way() {
    // The diagnostic echo and the notes share one path, so neither lands
    // inside the rewritten status line.
    let (sink, kept) = sink(false);
    let event = super::g03::progress_event(gwz_core::EventKind::Diagnostic, None, None);
    let warning = gwz_core::OperationEvent {
        severity: gwz_core::Severity::Warn,
        message: Some("careful".into()),
        ..event
    };
    gwz_core::operation::EventSink::deliver(&sink, warning);
    assert_eq!(kept.told(), ["line:warning: careful"]);
}
