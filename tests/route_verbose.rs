//! `--verbose` says why a commit ran the Git CLI instead of the native route
//! (gwz-core `GwzNoFallbackPlan.md` section 6: every refusal names its reason),
//! and no other output mode says it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A directory this test created, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let path =
            std::env::temp_dir().join(format!("gwz-cli-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `gwz` in `root`, with no user or machine Git configuration.
fn gwz(root: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gwz"))
        .current_dir(root)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_CONFIG_GLOBAL")
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A workspace whose member repository has a `pre-commit` hook and a staged change.
fn workspace_with_a_hook(scratch: &Scratch) -> PathBuf {
    let root = scratch.0.join("workspace");
    let home = scratch.0.join("home");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&home).unwrap();
    succeeded(&gwz(&root, &home, &["init"]));
    let identity = |repo: &git2::Repository| {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "GWZ Test").unwrap();
        config.set_str("user.email", "gwz@example.invalid").unwrap();
    };
    identity(&git2::Repository::open(&root).unwrap());
    let member = root.join("repos").join("app");
    fs::create_dir_all(&member).unwrap();
    let repo = git2::Repository::init(&member).unwrap();
    identity(&repo);
    let who = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
    fs::write(member.join("README.md"), "one\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    repo.commit(Some("HEAD"), &who, &who, "one", &tree, &[])
        .unwrap();
    succeeded(&gwz(
        &root,
        &home,
        &["repo", "add", member.to_str().unwrap()],
    ));
    let hooks = member.join(".git").join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    let hook = hooks.join("pre-commit");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    root
}

fn stage_a_change(root: &Path, text: &str) {
    let member = root.join("repos").join("app");
    fs::write(member.join("README.md"), text).unwrap();
    let repo = git2::Repository::open(&member).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();
}

#[test]
fn verbose_names_the_hook_that_sent_a_commit_to_git_and_other_modes_do_not() {
    let scratch = Scratch::new("route-verbose");
    let root = workspace_with_a_hook(&scratch);
    let home = scratch.0.join("home");
    let reason = "ran the Git CLI (a hook would run: pre-commit)";
    for (args, said) in [
        (vec!["commit", "-m", "plain"], false),
        (vec!["commit", "-m", "json", "--json", "--verbose"], false),
        (vec!["commit", "-m", "jsonl", "--jsonl", "--verbose"], false),
        (vec!["commit", "-m", "verbose", "--verbose"], true),
    ] {
        stage_a_change(&root, &format!("{args:?}\n"));
        let output = gwz(&root, &home, &args);
        succeeded(&output);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.contains(reason), said, "{args:?}: {stderr}");
        if said {
            assert!(
                stderr.contains("note: commit in "),
                "the line names the operation: {stderr}"
            );
        }
    }
}
