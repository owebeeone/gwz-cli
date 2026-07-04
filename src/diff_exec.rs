//! `gwz diff` execution (D5, cli half) — the in-process producer/consumer pump.
//!
//! Unlike every other `gwz` command, `diff` does not return a rendered
//! [`CliResponse`] envelope: it streams patch bytes (AD4/AD5). This module owns
//! the whole lifecycle:
//!
//! 1. Build the [`DiffRequest`](gwz_core::DiffRequest) (done by `DiffArgs`).
//! 2. Create a [`DiffLogRegistry`](gwz_core::DiffLogRegistry) and run
//!    [`handle_diff`](gwz_core::diff::handle_diff) with the **producer on a
//!    spawned thread** while the **main thread reads the log** and streams patch
//!    bytes to stdout / the pager in manifest order (plan §"CLI plan").
//! 3. On pager quit / broken pipe, [`end_stream`](gwz_core::DiffLog) the log so
//!    the producer's `should_stop` poll halts rendering (ProducerStop path), then
//!    exit cleanly.
//! 4. Compute the exit code from the summary for `--exit-code`/`--quiet` (AD8).
//! 5. Render `--json`/`--jsonl` machine output and manifest-only text formats
//!    (`--name-only`, `--stat`, …) client-side from the manifest.
//!
//! `stale_file` records are surfaced per plan: a warning to stderr (non-fatal),
//! omitted from human patch bytes, surfaced in `--jsonl`.

use std::io::IsTerminal;
use std::sync::mpsc;
use std::thread;

use gwz_core::diff::{DiffLogRegistry, LogReadRequest, LogReadState};
use gwz_core::{DiffManifestResponse, DiffOutputRecord, DiffOutputRecordKind};

use crate::diffargs::DiffInvocation;
use crate::pager::{self, PagerContext, PagerDecision, PatchSink, SinkError};
use crate::{CliError, OutputMode};

/// The result of a diff run: what the process should exit with. Diff owns its own
/// exit status (it never flows through `exit_code_for_response`).
pub(crate) struct DiffExit {
    pub(crate) code: i32,
}

/// Run a `gwz diff` invocation end to end and return the process exit code.
///
/// `start` is the physical start directory (used to resolve the workspace and, in
/// the local transport, hand core an absolute anchor). `output` is the global
/// output mode. Human patch output pages on a TTY; machine modes never do.
pub(crate) fn run_diff(
    invocation: &DiffInvocation,
    output: OutputMode,
    start: &std::path::Path,
    operation_id: String,
) -> Result<DiffExit, CliError> {
    let registry = DiffLogRegistry::new();

    // Run the planner + byte producer on a worker thread; it mints the log into
    // the shared registry and (for byte formats) produces into it synchronously.
    // The main thread reads the log concurrently so a pager quit can stop the
    // producer mid-stream (ProducerStop) rather than waiting for it to finish.
    let (tx, rx) = mpsc::channel::<Result<PlanHandoff, gwz_core::model::ModelError>>();
    let producer_registry = registry.clone();
    let request = invocation.request.clone();
    let start = start.to_path_buf();
    let worker = thread::spawn(move || {
        let outcome =
            gwz_core::diff::handle_diff(&start, request, operation_id, &producer_registry);
        match outcome {
            Ok(outcome) => {
                let _ = tx.send(Ok(PlanHandoff {
                    response: outcome.response,
                    log_id: outcome.log_id,
                }));
            }
            Err(err) => {
                let _ = tx.send(Err(err));
            }
        }
    });

    // The planner sends the response as soon as planning is done, then continues
    // producing bytes into the log. For byte formats we start reading immediately.
    let handoff = rx
        .recv()
        .map_err(|_| CliError::new("diff worker exited before producing a response"))?;
    let handoff = handoff.map_err(CliError::from_model)?;
    // Join the worker (producer has finished or stopped by the time the log seals
    // / closes; we join at the end after draining to guarantee no leak).
    let response = handoff.response;
    let log_id = handoff.log_id;

    let has_differences = response
        .summary
        .as_ref()
        .map(|summary| summary.has_differences)
        .unwrap_or(false);

    // ── machine output ───────────────────────────────────────────────────────
    // Machine modes emit summary/manifest metadata even under --quiet (the plan's
    // `gwz --json diff --quiet` returns summary metadata), so this runs first.
    match output {
        OutputMode::Json => {
            print!("{}", crate::diff_render::manifest_json(&response));
            drain_join(worker, &registry, log_id.as_deref(), None);
            return Ok(DiffExit {
                code: exit_status(invocation, has_differences),
            });
        }
        OutputMode::Jsonl => {
            // Manifest entries first, then output records (with bytes base64'd) if
            // the mode has a byte log. Under --quiet there is no byte log.
            crate::diff_render::print_manifest_jsonl(&response);
            if let Some(log_id) = &log_id {
                stream_jsonl_records(&registry, log_id);
            }
            drain_join(worker, &registry, log_id.as_deref(), None);
            return Ok(DiffExit {
                code: exit_status(invocation, has_differences),
            });
        }
        OutputMode::Human | OutputMode::Porcelain => {}
    }

    // ── --quiet short-circuit (human) ────────────────────────────────────────
    // Human --quiet emits nothing; exit status only.
    if invocation.quiet {
        drain_join(worker, &registry, log_id.as_deref(), None);
        return Ok(DiffExit {
            code: exit_status(invocation, has_differences),
        });
    }

    // ── human output ─────────────────────────────────────────────────────────
    // Manifest-only formats render client-side from the manifest (no byte log).
    if log_id.is_none() {
        let null_terminated = invocation
            .request
            .options
            .as_ref()
            .and_then(|o| o.null_terminated)
            .unwrap_or(false);
        let text = crate::diff_render::render_manifest_text(
            &response,
            invocation.display_format,
            null_terminated,
        );
        // These text formats are small; page them like patch output on a TTY.
        pump_bytes_to_sink(invocation, output, &text)?;
        drain_join(worker, &registry, None, None);
        return Ok(DiffExit {
            code: exit_status(invocation, has_differences),
        });
    }

    // Patch/raw byte formats: read the log and stream bytes through the pager.
    let log_id = log_id.expect("byte format has a log id");
    let decision = pager_decision(invocation, output);
    let pager_command = match decision {
        PagerDecision::Pager => pager::resolve_pager_command(
            std::env::var("GIT_PAGER").ok().as_deref(),
            std::env::var("PAGER").ok().as_deref(),
        ),
        PagerDecision::Direct => None,
    };
    let mut sink =
        PatchSink::open(decision, pager_command).map_err(|err| CliError::new(err.to_string()))?;

    let stop_reason = stream_patch_log(&registry, &log_id, &mut sink);

    // Whatever happened (drained, broken pipe, error), finish the sink and stop
    // the producer, then join.
    let _ = sink.finish();
    drain_join(worker, &registry, Some(&log_id), stop_reason);

    Ok(DiffExit {
        code: exit_status(invocation, has_differences),
    })
}

/// The response + optional log handle handed from the worker to the reader.
struct PlanHandoff {
    response: DiffManifestResponse,
    log_id: Option<String>,
}

/// Why the reader stopped early (for logging / clean shutdown).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StopReason {
    /// The consumer (pager/pipe) closed — end the stream to trigger ProducerStop.
    ConsumerGone,
}

/// The pager decision for this invocation, reading the real TTY state.
fn pager_decision(invocation: &DiffInvocation, output: OutputMode) -> PagerDecision {
    pager::decide(PagerContext {
        stdout_is_tty: std::io::stdout().is_terminal(),
        no_pager: invocation.no_pager,
        machine_output: matches!(output, OutputMode::Json | OutputMode::Jsonl),
        quiet: invocation.quiet,
        null_terminated: invocation
            .request
            .options
            .as_ref()
            .and_then(|o| o.null_terminated)
            .unwrap_or(false),
    })
}

/// Read the byte log in order and pump `patch_bytes` to the sink. Surfaces
/// `stale_file` records as stderr warnings (non-fatal). Returns `Some` when the
/// consumer went away so the caller can end the stream (ProducerStop).
fn stream_patch_log(
    registry: &DiffLogRegistry,
    log_id: &str,
    sink: &mut PatchSink,
) -> Option<StopReason> {
    let stream_id = format!("diff-out-{}", std::process::id());
    let mut cursor: Option<u64> = None;
    loop {
        let response = match registry.read(
            log_id,
            &LogReadRequest {
                stream_id: stream_id.clone(),
                cursor,
                max_records: Some(64),
                max_bytes: None,
                // Block until data/terminal (v0 honors None exactly).
                timeout_ms: None,
            },
        ) {
            Ok(response) => response,
            Err(_) => return None,
        };

        for record in &response.records {
            let decoded = gwz_core::diff::decode_record(&record.payload);
            if let Some(reason) = deliver_record(&decoded, sink) {
                // Consumer closed mid-stream: end this reader's stream so that,
                // under stop_when=last_reader, the producer sees ProducerStop.
                if let Ok(log) = registry.get(log_id) {
                    log.end_stream(&stream_id);
                }
                return Some(reason);
            }
        }
        cursor = Some(response.next_cursor);

        match response.state {
            LogReadState::Data | LogReadState::WouldBlock => continue,
            LogReadState::Eof | LogReadState::Closed => return None,
            LogReadState::Failed => {
                eprintln!(
                    "gwz: diff output failed: {}",
                    response.error.as_deref().unwrap_or("producer error")
                );
                return None;
            }
            LogReadState::Expired => {
                // Resume from the earliest available position (should not happen
                // in the operation-scoped v0 window, but honor the contract).
                cursor = Some(response.next_cursor);
            }
        }
    }
}

/// Write one output record's payload to the sink. `patch_bytes` → bytes;
/// `stale_file` → a stderr warning (omitted from human output, per plan);
/// `diagnostic` → a stderr note; boundaries are ignored for human patch output.
fn deliver_record(record: &DiffOutputRecord, sink: &mut PatchSink) -> Option<StopReason> {
    match record.kind {
        DiffOutputRecordKind::PatchBytes => {
            if let Some(data) = &record.data {
                match sink.write_all(data) {
                    Ok(()) => None,
                    Err(SinkError::BrokenPipe) => Some(StopReason::ConsumerGone),
                    Err(SinkError::Io(err)) => {
                        eprintln!("gwz: diff write error: {err}");
                        Some(StopReason::ConsumerGone)
                    }
                }
            } else {
                None
            }
        }
        DiffOutputRecordKind::StaleFile => {
            let path = record.file_id.as_deref().unwrap_or("<unknown>");
            eprintln!(
                "gwz: warning: {} changed during diff and was skipped (stale)",
                record.diagnostic.as_deref().unwrap_or(path)
            );
            None
        }
        DiffOutputRecordKind::Diagnostic => {
            if let Some(message) = &record.diagnostic {
                eprintln!("gwz: {message}");
            }
            None
        }
        DiffOutputRecordKind::FileStarted | DiffOutputRecordKind::FileFinished => None,
    }
}

/// Stream every output record as a `--jsonl` line (manifest entries were already
/// printed). `patch_bytes` are base64-expanded per D5; `stale_file` is surfaced.
fn stream_jsonl_records(registry: &DiffLogRegistry, log_id: &str) {
    let stream_id = format!("diff-jsonl-{}", std::process::id());
    let mut cursor: Option<u64> = None;
    loop {
        let response = match registry.read(
            log_id,
            &LogReadRequest {
                stream_id: stream_id.clone(),
                cursor,
                max_records: Some(64),
                max_bytes: None,
                timeout_ms: None,
            },
        ) {
            Ok(response) => response,
            Err(_) => return,
        };
        for record in &response.records {
            let decoded = gwz_core::diff::decode_record(&record.payload);
            println!("{}", crate::diff_render::output_record_json(&decoded));
        }
        cursor = Some(response.next_cursor);
        match response.state {
            LogReadState::Data | LogReadState::WouldBlock => continue,
            LogReadState::Eof | LogReadState::Closed | LogReadState::Failed => return,
            LogReadState::Expired => cursor = Some(response.next_cursor),
        }
    }
}

/// Pump a fully-assembled text blob (manifest-only formats) through the pager /
/// stdout, honoring the same pager policy as patch output.
fn pump_bytes_to_sink(
    invocation: &DiffInvocation,
    output: OutputMode,
    bytes: &[u8],
) -> Result<(), CliError> {
    let decision = pager_decision(invocation, output);
    let pager_command = match decision {
        PagerDecision::Pager => pager::resolve_pager_command(
            std::env::var("GIT_PAGER").ok().as_deref(),
            std::env::var("PAGER").ok().as_deref(),
        ),
        PagerDecision::Direct => None,
    };
    let mut sink =
        PatchSink::open(decision, pager_command).map_err(|err| CliError::new(err.to_string()))?;
    match sink.write_all(bytes) {
        Ok(()) | Err(SinkError::BrokenPipe) => {}
        Err(SinkError::Io(err)) => return Err(CliError::new(err.to_string())),
    }
    let _ = sink.finish();
    Ok(())
}

/// Exit status per AD8: `--exit-code`/`--quiet` → 1 on differences, else 0; plain
/// `gwz diff` → always 0.
fn exit_status(invocation: &DiffInvocation, has_differences: bool) -> i32 {
    if invocation.exit_code && has_differences {
        1
    } else {
        0
    }
}

/// End any dangling stream, release the log, and join the producer thread so no
/// worker outlives the command. `stop` records why the reader stopped (for future
/// diagnostics); the release + join is what actually unblocks a parked producer.
fn drain_join(
    worker: thread::JoinHandle<()>,
    registry: &DiffLogRegistry,
    log_id: Option<&str>,
    _stop: Option<StopReason>,
) {
    if let Some(log_id) = log_id {
        // Release the operation-scoped log; under stop_when=last_reader this is
        // the last reader leaving, so the producer's should_stop trips and it
        // returns from run_producer without finishing the remaining files.
        registry.release(log_id);
    }
    let _ = worker.join();
}
