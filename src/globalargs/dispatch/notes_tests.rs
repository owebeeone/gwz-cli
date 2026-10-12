//! Who is told the transport's notes (the adaptive concurrency design's §9):
//! only a person reading human output. Every machine-readable mode gets none.
use super::*;
use crate::tests::notes::sink;
use std::sync::Arc;

#[test]
fn human_output_prints_each_note_as_a_line_above_the_progress_block() {
    let (progress, kept) = sink(false);
    let progress = Arc::new(progress);
    let reader = note_sink(OutputMode::Human, false, &progress)
        .expect("a person reading human output is told");
    reader.tell("note: github.com:443: now using 12 connections");
    assert_eq!(
        kept.told(),
        ["line:note: github.com:443: now using 12 connections"]
    );
}

#[test]
fn json_jsonl_and_porcelain_get_no_reader_so_nothing_is_ever_printed() {
    let (progress, kept) = sink(false);
    let progress = Arc::new(progress);
    for output in [OutputMode::Json, OutputMode::Jsonl, OutputMode::Porcelain] {
        assert!(note_sink(output, true, &progress).is_none(), "{output:?}");
    }
    assert_eq!(kept.told(), Vec::<String>::new());
}

#[test]
fn a_followed_redirect_is_printed_only_with_verbose() {
    // U9: the transport tells a followed discovery redirect as a detail line; only `--verbose` prints it.
    for (verbose, printed) in [(false, false), (true, true)] {
        let (progress, kept) = sink(false);
        let progress = Arc::new(progress);
        let reader = note_sink(OutputMode::Human, verbose, &progress).unwrap();
        reader.tell_detail(
            "note: https://a.example/repo redirected to https://b.example/repo; following it",
        );
        reader.tell("note: github.com:443: now using 12 connections");
        let told = kept.told();
        assert_eq!(
            told.iter().any(|line| line.contains("redirected to")),
            printed,
            "verbose={verbose}: {told:?}"
        );
        // An ordinary note is printed either way.
        assert!(
            told.iter()
                .any(|line| line.contains("now using 12 connections")),
            "{told:?}"
        );
    }
}
