//! `gwz hook claude-code worktree-remove` (D3, D8, D10; S1.1).
//!
//! The remove hook decides by inspection of `worktree_path`, canonicalised
//! first, and deletes only what that path names. It never passes `--force`
//! and never `--keep`: a refusal keeps the lane and the session, which is
//! the safe outcome.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::create::canonical;
use super::env::HookEnv;
use super::family::{dispose_lane, is_absent, is_family_busy, read_family};
use super::input::RemoveInput;
use super::integration::WaitOwner;
use super::{Classification, HookContext, HookFailure, HookSuccess, fallback};

const ATTEMPT_SLEEP: Duration = Duration::from_millis(500);

pub(crate) fn run_worktree_remove(
    env: &dyn HookEnv,
    context: &HookContext,
    input: &RemoveInput,
) -> Result<HookSuccess, HookFailure> {
    let requested = PathBuf::from(&input.worktree_path);
    if !requested.exists() {
        // The other placement's handler, or a hand retirement, removed it
        // first (D5).
        return Ok(HookSuccess {
            path: requested,
            class: Classification::RefusedByHook,
            outcome: "absent",
        });
    }
    let path = canonical(&requested);

    // A registered git worktree of the project is Git's to remove (D8).
    if let Some(repository) = super::ignore::enclosing_repository(&context.project_dir) {
        let repository = canonical(&repository);
        if fallback::registered_worktrees(&repository).contains(&path) {
            fallback::remove_worktree(&repository, &path)?;
            return Ok(HookSuccess {
                path,
                class: Classification::FallbackWorktree,
                outcome: "removed",
            });
        }
    }

    let lane_root = gwz_core::workspace::discover_workspace_root(&path).map_err(|_| {
        HookFailure::refused(
            format!(
                "{} is neither a registered worktree of the project nor a GWZ lane",
                path.display()
            ),
            "remove it by hand if it is no longer wanted",
        )
    })?;
    let lane_root = canonical(&lane_root);
    dispose(env, context, &lane_root, &path)
}

fn dispose(
    env: &dyn HookEnv,
    context: &HookContext,
    lane_root: &Path,
    requested: &Path,
) -> Result<HookSuccess, HookFailure> {
    let deadline = env.now() + Duration::from_secs(context.options.wait_secs);
    loop {
        let family = match read_family(lane_root) {
            Ok(family) => family,
            Err(error) if is_absent(&error) => {
                return Err(HookFailure::refused(
                    format!("{} is in no local clone family", lane_root.display()),
                    "remove it by hand if it is no longer wanted",
                ));
            }
            Err(error) if is_family_busy(&error) => {
                if env.now() >= deadline {
                    return Err(HookFailure::busy(format!(
                        "the family lock was still held after {}s",
                        context.options.wait_secs
                    )));
                }
                env.sleep(ATTEMPT_SLEEP);
                continue;
            }
            Err(error) => {
                return Err(HookFailure::refused(
                    error.message,
                    "resolve the workspace refusal above, then retry",
                ));
            }
        };
        let family_root = family
            .root_path
            .as_deref()
            .map(|root| canonical(Path::new(root)))
            .ok_or_else(|| {
                HookFailure::refused(
                    format!("{} names no family root", lane_root.display()),
                    "remove it by hand if it is no longer wanted",
                )
            })?;
        // Exactly one `ready` row of that family must have a canonical path
        // equal to the lane root (D8). A basename that happens to match a
        // family name decides nothing.
        let matching: Vec<&gwz_core::LocalFamilyMemberEntry> = family
            .members
            .iter()
            .filter(|entry| entry.recorded_state == gwz_core::LocalMemberState::Ready)
            .filter(|entry| canonical(&family.absolute_path(entry)) == *lane_root)
            .collect();
        let [row] = matching.as_slice() else {
            if matching.is_empty() {
                // A row that disappeared between classification and dispose
                // is a removal somebody else completed (D8).
                return Ok(HookSuccess {
                    path: requested.to_path_buf(),
                    class: Classification::RefusedByHook,
                    outcome: "absent",
                });
            }
            return Err(HookFailure::refused(
                format!(
                    "{} matches {} ready family rows",
                    lane_root.display(),
                    matching.len()
                ),
                "retire it by hand: `gwz local list`, then the retirement procedure in \
                 docs/ClaudeCode.md",
            ));
        };
        let name = row.name.clone();
        let wait = Duration::from_secs(context.options.wait_secs);
        let (owner, result) = dispose_lane(&family_root, &name, wait);
        match result {
            Ok(_) => {
                return Ok(HookSuccess {
                    path: requested.to_path_buf(),
                    class: Classification::Lane,
                    outcome: "disposed",
                });
            }
            Err(error) if is_absent(&error) => {
                return Ok(HookSuccess {
                    path: requested.to_path_buf(),
                    class: Classification::RefusedByHook,
                    outcome: "absent",
                });
            }
            Err(error) if is_family_busy(&error) => {
                // Until R21 the hook owns the wait; after it, core waited
                // already and a busy answer is final.
                if owner == WaitOwner::Core || env.now() >= deadline {
                    return Err(HookFailure::busy(format!(
                        "the family lock was still held after {}s",
                        context.options.wait_secs
                    )));
                }
                env.sleep(ATTEMPT_SLEEP);
            }
            Err(error) => {
                return Err(HookFailure::hazard(
                    error.message,
                    format!(
                        "integrate it first: `gwz merge --remote {name}` from the main \
                         workspace, then `gwz local dispose {name}`"
                    ),
                ));
            }
        }
    }
}
