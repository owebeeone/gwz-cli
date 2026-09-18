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

/// The longest cause the hook builds from a dispose refusal. Past it the
/// text is cut on a word boundary: D10's line is a line, and Claude Code
/// puts it in the user's terminal wrapped in its own prefix.
const MAX_CAUSE_BYTES: usize = 160;

/// D10's one line, from `gwz local dispose`'s full report.
///
/// The report names every hazard of every repository -- 3.5 KB of it on
/// gwz-dev (the probe of 2026-09-18, §6, F3). That report is
/// `gwz local dispose`'s to print; what the hook says is how much there is
/// and of what class, and its remedy is the command that prints the rest.
/// A message of any other shape is cut to one readable line rather than
/// guessed at.
pub(crate) fn summarise(message: &str) -> String {
    match hazard_counts(message) {
        Some((hazards, repositories, classes)) => {
            format!(
                "{hazards} hazard{} across {repositories} repositor{} ({classes})",
                plural(hazards),
                if repositories == 1 { "y" } else { "ies" }
            )
        }
        None => shortened(message),
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// The hazard count, the repository count and the distinct waiver classes of
/// a dispose hazard report, or `None` when the message is not one.
///
/// A scan of the text gwz-core rendered, because the hazards reach the hook
/// only as that message: `unwaived hazard(s): \`<repo>\` <class>: <item>,
/// <item>, and <n> more; \`<repo>\` <class>: ...; name each accepted loss
/// with ...`.
fn hazard_counts(message: &str) -> Option<(usize, usize, String)> {
    let listing = message.split_once("unwaived hazard(s): ")?.1;
    let listing = listing
        .split_once("; name each accepted loss")
        .map(|(head, _)| head)
        .unwrap_or(listing);
    let mut hazards = 0_usize;
    let mut repositories = 0_usize;
    let mut classes: Vec<&str> = Vec::new();
    for finding in listing.split("; ") {
        let Some(rest) = finding.strip_prefix('`') else {
            continue;
        };
        let Some((_repository, rest)) = rest.split_once("` <") else {
            continue;
        };
        let Some((class, items)) = rest.split_once(">: ") else {
            continue;
        };
        repositories += 1;
        if !classes.contains(&class) {
            classes.push(class);
        }
        hazards += counted(items);
    }
    if repositories == 0 {
        return None;
    }
    Some((hazards, repositories, classes.join(", ")))
}

/// How many hazards one finding's rendered item list stands for, including
/// the ones `and <n> more` stands in for.
fn counted(items: &str) -> usize {
    let mut total = 0_usize;
    for item in items.split(", ") {
        let more = item
            .strip_prefix("and ")
            .and_then(|rest| rest.strip_suffix(" more"))
            .and_then(|count| count.parse::<usize>().ok());
        match more {
            Some(count) => {
                total += count;
            }
            None => {
                total += 1;
            }
        }
    }
    total
}

/// One readable line from a message of an unexpected shape: its first clause,
/// cut on a word boundary.
fn shortened(message: &str) -> String {
    let flattened = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if flattened.len() <= MAX_CAUSE_BYTES {
        return flattened;
    }
    let mut kept = String::new();
    for word in flattened.split(' ') {
        if kept.len() + word.len() + 1 > MAX_CAUSE_BYTES {
            break;
        }
        if !kept.is_empty() {
            kept.push(' ');
        }
        kept.push_str(word);
    }
    format!("{kept}...")
}

pub(crate) fn run_worktree_remove(
    env: &dyn HookEnv,
    context: &HookContext,
    input: &RemoveInput,
) -> Result<HookSuccess, HookFailure> {
    let requested = PathBuf::from(&input.worktree_path);
    if !requested.exists() {
        // The other placement's handler, or a hand retirement, removed it
        // first (D5).
        // Exit zero, and not a refusal: the other placement's handler, or
        // a hand retirement, removed it first (D5, F4).
        return Ok(HookSuccess {
            path: requested,
            class: Classification::AlreadyAbsent,
            outcome: "absent",
            name: None,
        });
    }
    let path = canonical(&requested);

    // A registered git worktree of the project is Git's to remove (D8).
    if let Some(repository) = super::ignore::enclosing_repository(&context.project_dir) {
        let repository = canonical(&repository);
        if fallback::registered_worktrees(&repository).contains(&path) {
            fallback::remove_worktree(&repository, &path)?;
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            return Ok(HookSuccess {
                path,
                class: Classification::FallbackWorktree,
                outcome: "removed",
                name,
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
        .at(&path)
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
                )
                .at(requested));
            }
            Err(error) if is_family_busy(&error) => {
                if env.now() >= deadline {
                    return Err(HookFailure::busy(format!(
                        "the family lock was still held after {}s",
                        context.options.wait_secs
                    ))
                    .at(requested));
                }
                env.sleep(ATTEMPT_SLEEP);
                continue;
            }
            Err(error) => {
                return Err(HookFailure::refused(
                    error.message,
                    "resolve the workspace refusal above, then retry",
                )
                .at(requested));
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
                .at(requested)
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
                    class: Classification::AlreadyAbsent,
                    outcome: "absent",
                    name: None,
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
            )
            .at(requested));
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
                    name: Some(name),
                });
            }
            Err(error) if is_absent(&error) => {
                return Ok(HookSuccess {
                    path: requested.to_path_buf(),
                    class: Classification::AlreadyAbsent,
                    outcome: "absent",
                    name: Some(name),
                });
            }
            Err(error) if is_family_busy(&error) => {
                // Until R21 the hook owns the wait; after it, core waited
                // already and a busy answer is final.
                if owner == WaitOwner::Core || env.now() >= deadline {
                    return Err(HookFailure::busy(format!(
                        "the family lock was still held after {}s",
                        context.options.wait_secs
                    ))
                    .at(requested)
                    .named(&name));
                }
                env.sleep(ATTEMPT_SLEEP);
            }
            Err(error) => {
                return Err(HookFailure::hazard(
                    summarise(&error.message),
                    format!(
                        "integrate it first: `gwz merge --remote {name}` from the main \
                         workspace, then `gwz local dispose {name}`"
                    ),
                )
                .at(requested)
                .named(&name));
            }
        }
    }
}
