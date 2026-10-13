//! Who is told why a commit or tag ran the Git CLI: a person reading human
//! output who asked for `--verbose`, and nobody else (gwz-core's
//! `GwzNoFallbackPlan.md` section 6: every refusal of the native route names
//! its reason, so an operator can see why Git ran).
use super::*;
use crate::tests::notes::sink;
use crate::tests::temp_dir::TempDir;
use gwz_core::git::GitRepository as _;
use std::sync::Arc;

/// A repository with one commit, an identity, a `pre-commit` hook and a
/// staged change, so a commit through the backend runs Git.
fn repository_with_a_hook(temp: &TempDir) -> std::path::PathBuf {
    let path = temp.path().join("repo");
    let repo = git2::Repository::init(&path).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "GWZ Test").unwrap();
    config.set_str("user.email", "gwz@example.invalid").unwrap();
    for (name, text) in [("a.txt", "one\n"), ("a.txt", "two\n")] {
        std::fs::write(path.join(name), text).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(name)).unwrap();
        index.write().unwrap();
        if text == "one\n" {
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
            let who = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
            repo.commit(Some("HEAD"), &who, &who, "one", &tree, &[])
                .unwrap();
        }
    }
    let hooks = path.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let hook = hooks.join("pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    path
}

#[test]
fn verbose_human_output_prints_why_a_commit_ran_git() {
    let temp = TempDir::new("route-notes");
    let path = repository_with_a_hook(&temp);
    let (progress, kept) = sink(false);
    let progress = Arc::new(progress);
    let notes = route_notes(OutputMode::Human, true, &progress)
        .expect("a person who asked for detail is told");
    let backend = gwz_core::git::Git2Backend::new().with_route_notes(notes);
    backend.commit(&path, "with a hook", false).unwrap();
    let told = kept.told();
    assert_eq!(told.len(), 1, "{told:?}");
    assert!(told[0].starts_with("line:note: commit in "), "{told:?}");
    // The reason depends on this machine's system configuration as much as on
    // the hook (a Homebrew `git` with its own system file goes to Git first),
    // so only its shape is pinned here; the process-level row in
    // `tests/route_verbose.rs` pins the hook as the reason.
    assert!(told[0].contains(": ran the Git CLI ("), "{told:?}");
    assert!(told[0].ends_with(')'), "{told:?}");
}

#[test]
fn without_verbose_and_in_machine_modes_nothing_is_told() {
    let (progress, _kept) = sink(false);
    let progress = Arc::new(progress);
    assert!(route_notes(OutputMode::Human, false, &progress).is_none());
    for output in [OutputMode::Json, OutputMode::Jsonl, OutputMode::Porcelain] {
        assert!(route_notes(output, true, &progress).is_none(), "{output:?}");
    }
}
