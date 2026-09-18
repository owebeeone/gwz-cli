//! D10's log: one line per decision, to a fixed file, because the desktop
//! app cannot pass an environment variable to switch logging on.
//!
//! A line carries exactly a timestamp, the event, `name`, `session_id`, the
//! resolved classification, the path printed or acted on, the outcome and
//! the exit code, plus the one stderr line of a non-zero exit. Never
//! `transcript_path`, never `cwd`.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::env::HookEnv;
use super::{Classification, HookFailure};

/// The log file for a workspace, relative to its root.
pub(crate) const WORKSPACE_LOG: &str = ".gwz/claude-hooks.log";
/// The user-level log, relative to `HOME`: the fallback's log, and the
/// substitute for a workspace location that would show in `git status`.
pub(crate) const USER_LOG: &str = ".claude/gwz-lane-hooks.log";
/// D10's bound. Past it the file is truncated to its newest half.
pub(crate) const LOG_LIMIT_BYTES: u64 = 1 << 20;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogLocation {
    pub(crate) path: Option<PathBuf>,
    /// The stderr note D10 requires when a location that would appear in
    /// `git status` was declined in favour of the user log.
    pub(crate) note: Option<String>,
}

/// Choose the log location: `--log` when the handler carries it, else the
/// workspace file, else the user file. Whichever is chosen, a location that
/// would appear in `git status` is not used (D10).
pub(crate) fn resolve_log_location(
    env: &dyn HookEnv,
    option: Option<&str>,
    workspace_root: Option<&Path>,
) -> LogLocation {
    let user = env.home_dir().map(|home| home.join(USER_LOG));
    let requested = match (option, workspace_root) {
        (Some(path), _) => Some(PathBuf::from(path)),
        (None, Some(root)) => Some(root.join(WORKSPACE_LOG)),
        (None, None) => user.clone(),
    };
    let Some(requested) = requested else {
        return LogLocation {
            path: None,
            note: None,
        };
    };
    if super::ignore::path_is_ignored(&requested) {
        return LogLocation {
            path: Some(requested),
            note: None,
        };
    }
    match user {
        Some(user) if user != requested => LogLocation {
            note: Some(format!(
                "gwz: {} would appear in `git status`; logging to {} instead",
                requested.display(),
                user.display()
            )),
            path: Some(user),
        },
        _ => LogLocation {
            path: None,
            note: Some(format!(
                "gwz: {} would appear in `git status` and there is no user log location; \
                 this decision was not logged",
                requested.display()
            )),
        },
    }
}

/// One decision, in D10's exact field set.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogRecord {
    pub(crate) event: &'static str,
    pub(crate) name: String,
    pub(crate) session_id: String,
    pub(crate) class: Classification,
    pub(crate) path: String,
    pub(crate) outcome: String,
    pub(crate) exit: i32,
    pub(crate) message: Option<String>,
}

impl LogRecord {
    /// A refusal, with the name and the path the hook itself learned
    /// preferred over what the payload spelled (F4): the remove payload
    /// carries no `name` at all, and a refusal whose path was known should
    /// not log `path=-`.
    pub(crate) fn refusal(
        event: &'static str,
        name: &str,
        session_id: &str,
        failure: &HookFailure,
    ) -> Self {
        Self {
            event,
            name: failure.name.clone().unwrap_or_else(|| name.to_owned()),
            session_id: session_id.to_owned(),
            class: failure.class,
            path: failure
                .path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            outcome: "refused".to_owned(),
            exit: 1,
            message: Some(failure.line()),
        }
    }

    pub(crate) fn line(&self, env: &dyn HookEnv) -> String {
        let timestamp = env
            .now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or_default();
        let mut line = format!(
            "ts={timestamp} event={} name={} session={} class={} path={} outcome={} exit={}",
            self.event,
            field(&self.name),
            field(&self.session_id),
            self.class.word(),
            field(&self.path),
            field(&self.outcome),
            self.exit
        );
        if let Some(message) = &self.message {
            line.push_str(" message=");
            line.push_str(&field(message));
        }
        line
    }
}

/// One field, with whitespace and newlines made harmless so a line stays a
/// line.
fn field(value: &str) -> String {
    let flattened: String = value
        .chars()
        .map(|value| if value.is_whitespace() { ' ' } else { value })
        .collect();
    if flattened.is_empty() {
        "-".to_owned()
    } else if flattened.contains(' ') {
        format!("\"{}\"", flattened.replace('"', "'"))
    } else {
        flattened
    }
}

/// Append one line, then hold the file to its bound.
pub(crate) fn write_log(env: &dyn HookEnv, location: &LogLocation, record: &LogRecord) {
    if let Some(note) = &location.note {
        env.warn(note);
    }
    let Some(path) = &location.path else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format!("{}\n", record.line(env));
    let appended = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(line.as_bytes())
        });
    if appended.is_err() {
        // A log that cannot be written never decides the hook's outcome.
        return;
    }
    bound(path);
}

fn bound(path: &Path) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.len() <= LOG_LIMIT_BYTES {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = text.lines().collect();
    let keep = lines.len() / 2;
    let mut kept = lines[keep..].join("\n");
    kept.push('\n');
    let _ = std::fs::write(path, kept);
}
