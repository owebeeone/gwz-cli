//! Client-side pager policy for `gwz diff` (AD6 — this lives in the client, never
//! in headless core).
//!
//! The pager decision (§"Pager policy" of the plan):
//!
//! - Page only **human patch output** on a **TTY**.
//! - Never page `--json`, `--jsonl`, `--quiet`, or NUL-terminated (`-z`) output.
//! - Honor `--no-pager`, `$GIT_PAGER`, then `$PAGER`, else `less -FRX`.
//! - A pager that cannot spawn is a client-side error, not a core diff failure.
//!
//! The pump ([`PatchSink`]) writes assembled patch bytes to either the pager's
//! stdin or straight to stdout. A pager quit / closed pipe surfaces as
//! [`SinkError::BrokenPipe`] on the next write; the diff driver turns that into
//! the taut-shape ProducerStop path (end the log stream, stop cleanly) — this is
//! the pager-quit → ProducerStop path the plan calls out.

use std::io::{self, Write};
use std::process::{Child, Command, Stdio};

/// Whether the client should attempt to launch a pager for this invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PagerDecision {
    /// Spawn the resolved pager and pump bytes through it.
    Pager,
    /// Write straight to stdout (piped, non-TTY, `--no-pager`, or a machine mode).
    Direct,
}

/// Inputs to the pager decision, isolated from the environment so the decision is
/// unit-testable without a real terminal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PagerContext {
    /// stdout is a terminal (the client checks `IsTerminal` at the call site).
    pub(crate) stdout_is_tty: bool,
    /// `--no-pager` was passed.
    pub(crate) no_pager: bool,
    /// The output is a machine mode (`--json`/`--jsonl`) — never paged.
    pub(crate) machine_output: bool,
    /// `--quiet` — no output at all, so never paged.
    pub(crate) quiet: bool,
    /// `-z` NUL-terminated output — never paged (it is for machine consumers).
    pub(crate) null_terminated: bool,
}

/// The pure pager decision. Page only human, non-quiet, non-`-z` patch output on
/// a TTY without `--no-pager`.
pub(crate) fn decide(ctx: PagerContext) -> PagerDecision {
    if ctx.no_pager || ctx.machine_output || ctx.quiet || ctx.null_terminated || !ctx.stdout_is_tty
    {
        return PagerDecision::Direct;
    }
    PagerDecision::Pager
}

/// The pager command line resolved from the environment: `$GIT_PAGER`, then
/// `$PAGER`, else `less -FRX`. An empty/whitespace value or the literal `cat`
/// means "no pager" (git treats `PAGER=cat` as an effective passthrough; we
/// return `None` so the driver writes to stdout directly).
pub(crate) fn resolve_pager_command(
    git_pager: Option<&str>,
    pager: Option<&str>,
) -> Option<String> {
    let candidate = git_pager
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| pager.map(str::trim).filter(|value| !value.is_empty()));
    match candidate {
        Some("cat") => None,
        Some(command) => Some(command.to_owned()),
        None => Some("less -FRX".to_owned()),
    }
}

/// A write error while pumping patch bytes.
#[derive(Debug)]
pub(crate) enum SinkError {
    /// The consumer (pager or downstream pipe) went away — pager quit or the
    /// stdout pipe closed. This is the ProducerStop trigger: stop cleanly.
    BrokenPipe,
    /// Any other I/O error on the sink.
    Io(io::Error),
}

impl SinkError {
    fn from_io(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::BrokenPipe {
            SinkError::BrokenPipe
        } else {
            SinkError::Io(error)
        }
    }
}

/// Where assembled patch bytes are written: either a spawned pager's stdin or the
/// process's stdout. Owns the pager child so the pump can wait on it at the end.
pub(crate) enum PatchSink {
    /// A spawned pager and the handle to its stdin.
    Pager { child: Child },
    /// Direct to stdout.
    Stdout,
}

impl PatchSink {
    /// Build the sink for a decision. On [`PagerDecision::Pager`] this spawns the
    /// resolved pager; a spawn failure falls back to `Ok(None)` so the caller can
    /// report a client-side error (never a core failure).
    pub(crate) fn open(
        decision: PagerDecision,
        pager_command: Option<String>,
    ) -> Result<PatchSink, io::Error> {
        match (decision, pager_command) {
            (PagerDecision::Pager, Some(command)) => {
                let child = spawn_pager(&command)?;
                Ok(PatchSink::Pager { child })
            }
            // Pager wanted but resolved to no-pager (e.g. PAGER=cat), or Direct.
            _ => Ok(PatchSink::Stdout),
        }
    }

    /// Write a chunk of patch bytes. A closed consumer surfaces as
    /// [`SinkError::BrokenPipe`].
    pub(crate) fn write_all(&mut self, bytes: &[u8]) -> Result<(), SinkError> {
        match self {
            PatchSink::Pager { child } => {
                let stdin = child.stdin.as_mut().ok_or(SinkError::BrokenPipe)?;
                stdin.write_all(bytes).map_err(SinkError::from_io)
            }
            PatchSink::Stdout => {
                let mut out = io::stdout().lock();
                out.write_all(bytes).map_err(SinkError::from_io)
            }
        }
    }

    /// Flush and, for a pager, close its stdin and wait for it to exit. Called
    /// once the output log is drained (or a ProducerStop was seen).
    pub(crate) fn finish(self) -> Result<(), io::Error> {
        match self {
            PatchSink::Pager { mut child } => {
                // Drop stdin so the pager sees EOF, then wait for it to quit.
                drop(child.stdin.take());
                child.wait()?;
                Ok(())
            }
            PatchSink::Stdout => io::stdout().lock().flush(),
        }
    }
}

/// Spawn a pager command via the platform shell so `$GIT_PAGER`/`$PAGER` strings
/// with arguments (e.g. `less -FRX`, `less -R`) work verbatim. `LESS`/`LV` are
/// left to the user's environment; we set `LESS=FRX`-style flags inline in the
/// default command instead.
fn spawn_pager(command: &str) -> Result<Child, io::Error> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };
    cmd.stdin(Stdio::piped());
    // Give `less` sane defaults if the user did not set LESS.
    if std::env::var_os("LESS").is_none() {
        cmd.env("LESS", "FRX");
    }
    cmd.spawn()
}
