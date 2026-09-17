//! D8's fallback: the project is not a GWZ workspace, so Claude Code's own
//! default is reproduced as far as a hook can -- a worktree under
//! `.claude/worktrees/<name>` on branch `worktree-<name>`, from
//! `origin/<default-branch>` when it resolves and `HEAD` otherwise.
//!
//! This is the one place gwz-cli drives Git itself. There is no workspace
//! here and so no gwz-core semantic to delegate to: a plain repository's
//! worktrees are Git's own concept, and the hook is standing in for Claude
//! Code's `git worktree add`. What the fallback cannot reproduce is the
//! automatic sweep, which never runs for a hook-created worktree (S4.1).

use std::path::{Path, PathBuf};
use std::process::Command;

use super::create::canonical;
use super::env::HookEnv;
use super::ignore::{Patterns, enclosing_repository};
use super::{Classification, HookContext, HookFailure, HookSuccess};

fn git(repository: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(|error| format!("git {} failed to start: {error}", args.join(" ")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

/// The repository the session started in.
fn project_repository(project_dir: &Path) -> Result<PathBuf, HookFailure> {
    enclosing_repository(project_dir).ok_or_else(|| {
        HookFailure::refused(
            format!(
                "{} is neither a GWZ workspace nor a Git repository",
                project_dir.display()
            ),
            "run the session from a workspace or a repository",
        )
    })
}

/// Every worktree the repository has registered, by canonical path.
pub(crate) fn registered_worktrees(repository: &Path) -> Vec<PathBuf> {
    let Ok(listing) = git(repository, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    listing
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(|path| canonical(Path::new(path)))
        .collect()
}

fn branch_of(repository: &Path, worktree: &Path) -> Option<String> {
    git(worktree, &["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|_| registered_worktrees(repository).contains(&canonical(worktree)))
}

/// The documented default base: `origin/<default-branch>` when it resolves,
/// else `HEAD` (O3). `--base-ref` overrides it.
fn base_ref(repository: &Path, option: Option<&str>) -> String {
    if let Some(base) = option {
        return base.to_owned();
    }
    if let Ok(head) = git(
        repository,
        &["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"],
    ) {
        let name = head.trim();
        if !name.is_empty() && git(repository, &["rev-parse", "--verify", "--quiet", name]).is_ok()
        {
            return name.to_owned();
        }
    }
    "HEAD".to_owned()
}

pub(crate) fn create_worktree(
    env: &dyn HookEnv,
    context: &HookContext,
    name: &str,
) -> Result<HookSuccess, HookFailure> {
    let repository = project_repository(&context.project_dir)?;
    let worktree = repository.join(".claude/worktrees").join(name);
    let branch = format!("worktree-{name}");
    if worktree.exists() {
        // A registered worktree of this project on this branch is reused and
        // printed rather than failed, with no session check (D8): a plain
        // worktree is cheap, and `git worktree remove` protects a dirty or
        // locked one.
        if branch_of(&repository, &worktree).as_deref() == Some(branch.as_str()) {
            report_missing_includes(env, &repository, &worktree);
            return Ok(HookSuccess {
                path: canonical(&worktree),
                class: Classification::FallbackWorktree,
                outcome: "reused",
            });
        }
        return Err(HookFailure::refused(
            format!(
                "{} exists but is not this project's `{branch}` worktree",
                worktree.display()
            ),
            "remove that directory, or choose another worktree name",
        ));
    }
    if let Some(parent) = worktree.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            HookFailure::refused(
                format!("{} could not be created: {error}", parent.display()),
                "check the permissions on the project directory",
            )
        })?;
    }
    let base = base_ref(&repository, context.options.base_ref.as_deref());
    let path = worktree.to_string_lossy().into_owned();
    git(
        &repository,
        &["worktree", "add", "-b", &branch, &path, &base],
    )
    .map_err(|error| {
        HookFailure::refused(
            format!("git worktree add failed: {error}"),
            "resolve the Git refusal above, then retry",
        )
    })?;
    // The `.worktreeinclude` copy Claude skips when a hook owns creation is
    // performed only for a worktree this invocation created (D8).
    copy_includes(env, &repository, &worktree);
    Ok(HookSuccess {
        path: canonical(&worktree),
        class: Classification::FallbackWorktree,
        outcome: "created",
    })
}

/// `git worktree remove`, never `--force`: a dirty worktree makes the hook
/// fail and keeps the session, matching D3, and a locked worktree -- what
/// Claude holds on a running agent's -- is refused too.
pub(crate) fn remove_worktree(repository: &Path, worktree: &Path) -> Result<(), HookFailure> {
    let path = worktree.to_string_lossy().into_owned();
    git(repository, &["worktree", "remove", &path]).map_err(|error| {
        HookFailure::refused(
            format!(
                "git worktree remove refused {}: {error}",
                worktree.display()
            ),
            "commit or discard the changes in that worktree, then remove it by hand",
        )
    })?;
    Ok(())
}

fn include_patterns(repository: &Path) -> Patterns {
    Patterns::read(&repository.join(".worktreeinclude"))
}

/// The project-root files a `.worktreeinclude` names. Files already present
/// in the worktree are left exactly as they are: a created worktree holds
/// the tracked files of its base, so what is missing from it is what the
/// include list is for.
fn included_files(repository: &Path, patterns: &Patterns) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![repository.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(repository) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            if relative == ".git" || relative.starts_with(".git/") {
                continue;
            }
            let is_directory = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
            if is_directory {
                stack.push(path);
            } else if patterns.matches(&relative, false) {
                found.push(PathBuf::from(relative));
            }
        }
    }
    found.sort();
    found
}

fn copy_includes(env: &dyn HookEnv, repository: &Path, worktree: &Path) {
    let patterns = include_patterns(repository);
    if patterns.is_empty() {
        return;
    }
    for relative in included_files(repository, &patterns) {
        let target = worktree.join(&relative);
        if target.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::copy(repository.join(&relative), &target).is_err() {
            env.warn(&format!(
                "gwz: {} could not be copied into the worktree",
                relative.display()
            ));
        }
    }
}

/// On reuse nothing is written, and a listed file missing from the reused
/// worktree is reported on stderr rather than supplied (D8).
fn report_missing_includes(env: &dyn HookEnv, repository: &Path, worktree: &Path) {
    let patterns = include_patterns(repository);
    if patterns.is_empty() {
        return;
    }
    for relative in included_files(repository, &patterns) {
        if !worktree.join(&relative).exists() {
            env.warn(&format!(
                "gwz: {} is listed in .worktreeinclude and missing from the reused worktree",
                relative.display()
            ));
        }
    }
}
