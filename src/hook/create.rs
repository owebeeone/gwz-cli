//! `gwz hook claude-code worktree-create` (D2, D6, D8, D10; S1.1).
//!
//! One test decides the branch: is the project a GWZ workspace (D8). A
//! workspace gets a lane of itself; anything else gets Claude Code's own
//! default git worktree, reproduced as far as a hook can.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::env::HookEnv;
use super::estimate::{check_free_space, check_lane_ceiling, estimate};
use super::family::{FamilyRead, clone_lane, is_absent, is_family_busy, read_family};
use super::input::{CreateInput, validate_lane_name, validate_session_token};
use super::{Classification, HookContext, HookFailure, HookSuccess, fallback};

/// How long an attempt sleeps before re-reading the index. An attempt that
/// reaches the clone pays the clone's own pre-lock source inventory before
/// it can learn the lock is busy, so against an unrelated family command the
/// effective poll period is that inventory, not this sleep (D6).
const ATTEMPT_SLEEP: Duration = Duration::from_millis(500);

/// What the reuse table said about this attempt (D6).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Decision {
    Create,
    Reuse,
    Wait,
    Refuse(Box<HookFailure>),
}

pub(crate) fn run_worktree_create(
    env: &dyn HookEnv,
    context: &HookContext,
    input: &CreateInput,
) -> Result<HookSuccess, HookFailure> {
    validate_lane_name(&input.name)?;
    validate_session_token(&input.session_id)?;
    match gwz_core::workspace::discover_workspace_root(&context.project_dir) {
        Ok(root) => create_lane(env, context, input, &root),
        // Not a workspace: Claude Code's default, as far as a hook can (D8).
        Err(_) => fallback::create_worktree(env, context, &input.name),
    }
}

fn create_lane(
    env: &dyn HookEnv,
    context: &HookContext,
    input: &CreateInput,
    root: &Path,
) -> Result<HookSuccess, HookFailure> {
    let root = canonical(root);
    let destination = default_destination(&root, &input.name)?;
    let destination_parent = destination
        .parent()
        .ok_or_else(|| {
            HookFailure::refused(
                format!("{} has no parent directory", destination.display()),
                "run the session from a workspace that is not at the filesystem root",
            )
        })?
        .to_path_buf();
    // D2: the parent of the workspace must not itself sit inside a
    // repository, because Claude Code refuses a directory that git resolves
    // to another repository's checkout (section 1).
    if let Some(repository) = super::ignore::enclosing_repository(&destination_parent) {
        return Err(HookFailure::refused(
            format!(
                "the lane would land inside the repository at {}",
                repository.display()
            ),
            "move the workspace out of that repository, or create the lane by hand with \
             `gwz local clone <name> <dest>`",
        ));
    }
    warn_about_worktrees(env, &root);
    if env.build_busy(&root) {
        env.warn(
            "gwz: a build holds the source's target directory open; the lane may need a rebuild",
        );
    }

    let deadline = env.now() + Duration::from_secs(context.options.wait_secs);
    // What the deadline message will name: a sibling handler mid-copy, or
    // the family lock itself.
    let mut waiting_for_sibling = false;
    loop {
        let family = match read_family(&root) {
            Ok(family) => family,
            Err(error) if is_absent(&error) => FamilyRead {
                root_path: Some(root.to_string_lossy().into_owned()),
                members: Vec::new(),
            },
            Err(error) if is_family_busy(&error) => {
                // The index could not be read this instant; the attempt
                // loop's next read is the retry.
                FamilyRead {
                    root_path: Some(root.to_string_lossy().into_owned()),
                    members: Vec::new(),
                }
            }
            Err(error) => {
                return Err(HookFailure::refused(
                    error.message,
                    "resolve the workspace refusal above, then retry",
                ));
            }
        };
        match decide(&family, &input.name, &destination, &input.session_id) {
            Decision::Reuse => {
                return Ok(success(&root, &destination, context, "reused"));
            }
            Decision::Refuse(failure) => {
                return Err(*failure);
            }
            Decision::Wait => {
                waiting_for_sibling = true;
            }
            Decision::Create => {
                // The guards are evaluated only for an attempt that will
                // create, and re-evaluated at every attempt (D6).
                let estimate = estimate(env, &root, &destination_parent);
                check_free_space(
                    env,
                    &estimate,
                    &destination_parent,
                    context.options.min_free_gb,
                )?;
                check_lane_ceiling(family.ready_rows(), context.options.max_lanes)?;
                match clone_lane(&root, &input.name, &input.session_id) {
                    Ok(()) => {
                        return Ok(success(&root, &destination, context, "created"));
                    }
                    Err(error) if is_family_busy(&error) => {
                        // Someone else holds the lock: loop and re-read.
                        let _ = error;
                    }
                    Err(error) => {
                        return Err(HookFailure::refused(
                            error.message,
                            "resolve the refusal above, then retry",
                        ));
                    }
                }
            }
        }
        if env.now() >= deadline {
            let reason = if waiting_for_sibling {
                format!(
                    "another handler for this session is still creating `{}`",
                    input.name
                )
            } else {
                "the family lock is held".to_owned()
            };
            return Err(HookFailure::busy(format!(
                "waited {}s for the family: {reason}",
                context.options.wait_secs
            )));
        }
        env.sleep(ATTEMPT_SLEEP);
    }
}

/// D6's table, evaluated on every attempt.
pub(crate) fn decide(
    family: &FamilyRead,
    name: &str,
    destination: &Path,
    session_id: &str,
) -> Decision {
    let Some(row) = family.row(name) else {
        // No row. Nothing at the destination is the only state that creates.
        if destination.exists() {
            return Decision::Refuse(Box::new(HookFailure::refused(
                format!(
                    "{} exists but the family has no row for `{name}`",
                    destination.display()
                ),
                "choose another name, or move that directory aside",
            )));
        }
        return Decision::Create;
    };
    let recorded = family.absolute_path(row);
    let same_place = canonical(&recorded) == canonical(destination);
    let owner = family.owner_of(row);
    match row.recorded_state {
        gwz_core::LocalMemberState::Creating => {
            if owner.as_deref() == Some(session_id) {
                Decision::Wait
            } else {
                Decision::Refuse(Box::new(HookFailure::refused(
                    format!("`{name}` has an unfinished create in the family index"),
                    "retire it: `gwz local list`, then the retirement procedure in \
                     docs/ClaudeCode.md",
                )))
            }
        }
        gwz_core::LocalMemberState::Disposing => Decision::Refuse(Box::new(HookFailure::refused(
            format!("`{name}` is part-way through disposal"),
            "retire it: `gwz local list`, then the retirement procedure in docs/ClaudeCode.md",
        ))),
        gwz_core::LocalMemberState::Ready => {
            if !same_place {
                return Decision::Refuse(Box::new(HookFailure::refused(
                    format!("`{name}` is already a lane at {}", recorded.display()),
                    "choose another worktree name",
                )));
            }
            match owner.as_deref() {
                None => Decision::Refuse(Box::new(HookFailure::refused(
                    format!("the lane `{name}` records no owning session"),
                    "retire it, or choose another name: `gwz local list`, then the retirement \
                     procedure in docs/ClaudeCode.md",
                ))),
                Some(other) if other != session_id => {
                    Decision::Refuse(Box::new(HookFailure::refused(
                        format!("the lane `{name}` belongs to session {other}"),
                        "choose another worktree name",
                    )))
                }
                Some(_) => {
                    if row.observed_state == gwz_core::LocalObservedState::Ready {
                        Decision::Reuse
                    } else {
                        Decision::Refuse(Box::new(HookFailure::refused(
                            format!(
                                "the lane `{name}` is incomplete ({})",
                                crate::observed_state_word(row.observed_state)
                            ),
                            "retire it: `gwz local list`, then the retirement procedure in \
                             docs/ClaudeCode.md",
                        )))
                    }
                }
            }
        }
    }
}

/// The path the hook prints: the lane, or the session's own member
/// directory inside it when the session started inside a member (D8).
fn success(
    root: &Path,
    destination: &Path,
    context: &HookContext,
    outcome: &'static str,
) -> HookSuccess {
    let relative = canonical(&context.project_dir)
        .strip_prefix(canonical(root))
        .ok()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if relative.as_os_str().is_empty() {
        HookSuccess {
            path: canonical(destination),
            class: Classification::Lane,
            outcome,
        }
    } else {
        HookSuccess {
            path: canonical(&destination.join(relative)),
            class: Classification::MemberInLane,
            outcome,
        }
    }
}

/// GWZ's default sibling destination, `../<root-dirname>-<name>` (D2).
fn default_destination(root: &Path, name: &str) -> Result<PathBuf, HookFailure> {
    let directory = root
        .file_name()
        .ok_or_else(|| {
            HookFailure::refused(
                format!("{} has no directory name", root.display()),
                "run the session from a named workspace directory",
            )
        })?
        .to_string_lossy()
        .into_owned();
    let parent = root.parent().ok_or_else(|| {
        HookFailure::refused(
            format!("{} has no parent directory", root.display()),
            "run the session from a workspace that is not at the filesystem root",
        )
    })?;
    Ok(parent.join(format!("{directory}-{name}")))
}

/// D2: a verbatim copy carries the root's own git worktrees' `.git`
/// pointers into the lane, so a non-empty `.claude/worktrees/` is worth one
/// stderr line.
fn warn_about_worktrees(env: &dyn HookEnv, root: &Path) {
    let worktrees = root.join(".claude/worktrees");
    let occupied = std::fs::read_dir(&worktrees)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false);
    if occupied {
        env.warn(&format!(
            "gwz: {} is not empty; its worktree pointers travel into the lane",
            worktrees.display()
        ));
    }
}

pub(crate) fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
