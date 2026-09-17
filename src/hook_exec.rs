//! The hook's own lifecycle: stdin, the one line of stdout, the one line of
//! stderr, the log and the exit code (D1, D10).
//!
//! Like diff and log, the hook commands never flow through the response
//! renderer: their output contract is Claude Code's, not GWZ's.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::hook::env::HookEnv;
use crate::hook::{
    CreateInput, HookContext, HookFailure, HookSuccess, LogRecord, RemoveInput, SystemEnv,
    parse_create_input, parse_remove_input, resolve_log_location, run_setup, run_worktree_create,
    run_worktree_remove, write_log,
};
use crate::*;

/// `CLAUDE_PROJECT_DIR`, the project root the session started from, falling
/// back to the process's working directory when the variable is absent so
/// the command can be run by hand (S1.1).
pub(crate) fn project_dir(start_dir: &Path) -> PathBuf {
    std::env::var_os("CLAUDE_PROJECT_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| start_dir.to_path_buf())
}

fn read_stdin() -> Result<String, HookFailure> {
    let mut body = String::new();
    std::io::stdin()
        .read_to_string(&mut body)
        .map_err(|error| {
            HookFailure::refused(
                format!("the hook input could not be read: {error}"),
                "check the hooks block written by `gwz hook claude-code setup`",
            )
        })?;
    Ok(body)
}

/// Run one hook and return its exit code. Stdout carries the created or
/// verified path and nothing else; every other byte is stderr's.
pub(crate) fn run_hook(invocation: &HookInvocation, start_dir: &Path) -> i32 {
    let env = SystemEnv;
    let context = HookContext {
        project_dir: project_dir(start_dir),
        options: invocation.options.clone(),
    };
    let workspace_root = gwz_core::workspace::discover_workspace_root(&context.project_dir).ok();
    let location = resolve_log_location(
        &env,
        context.options.log.as_deref(),
        workspace_root.as_deref(),
    );
    let event = invocation.event.word();
    let body = match read_stdin() {
        Ok(body) => body,
        Err(failure) => {
            return report(&env, &location, event, "", "", Err(failure));
        }
    };
    match invocation.event {
        HookEvent::WorktreeCreate => {
            let input = match parse_create_input(&body) {
                Ok(input) => input,
                Err(failure) => {
                    return report(&env, &location, event, "", "", Err(failure));
                }
            };
            let CreateInput { name, session_id } = input.clone();
            let outcome = run_worktree_create(&env, &context, &input);
            report(&env, &location, event, &name, &session_id, outcome)
        }
        HookEvent::WorktreeRemove => {
            let input = match parse_remove_input(&body) {
                Ok(input) => input,
                Err(failure) => {
                    return report(&env, &location, event, "", "", Err(failure));
                }
            };
            let RemoveInput {
                worktree_path,
                session_id,
            } = input.clone();
            let name = Path::new(&worktree_path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let outcome = run_worktree_remove(&env, &context, &input);
            report(&env, &location, event, &name, &session_id, outcome)
        }
    }
}

/// The one place a hook writes stdout, stderr and its log.
fn report(
    env: &dyn HookEnv,
    location: &crate::hook::logging::LogLocation,
    event: &'static str,
    name: &str,
    session_id: &str,
    outcome: Result<HookSuccess, HookFailure>,
) -> i32 {
    match outcome {
        Ok(success) => {
            // Only the create hook has a path to print; the remove hook's
            // path is recorded in the log alone.
            if event == "worktree-create" {
                println!("{}", success.path.display());
            }
            write_log(
                env,
                location,
                &LogRecord {
                    event,
                    name: name.to_owned(),
                    session_id: session_id.to_owned(),
                    class: success.class,
                    path: success.path.to_string_lossy().into_owned(),
                    outcome: success.outcome.to_owned(),
                    exit: 0,
                    message: None,
                },
            );
            0
        }
        Err(failure) => {
            eprintln!("{}", failure.line());
            write_log(
                env,
                location,
                &LogRecord::refusal(event, name, session_id, &failure),
            );
            1
        }
    }
}

/// `gwz hook claude-code setup`: ordinary output on stdout, refusals on
/// stderr.
pub(crate) fn run_claude_code_setup(
    request: &crate::hook::setup::SetupRequest,
    start_dir: &Path,
) -> i32 {
    let env = SystemEnv;
    let root = gwz_core::workspace::discover_workspace_root(start_dir)
        .ok()
        .or_else(|| crate::hook::ignore::enclosing_repository(start_dir));
    match run_setup(&env, root.as_deref(), request) {
        Ok(output) => {
            println!("{}", output.block);
            eprintln!("gwz: {}", output.note);
            if !request.remove {
                eprintln!(
                    "gwz: install or refresh the agent skill too: copy `skills/gwz/SKILL.md` to \
                     `~/.claude/skills/gwz/`"
                );
            }
            0
        }
        Err(failure) => {
            eprintln!("{}", failure.line());
            1
        }
    }
}
