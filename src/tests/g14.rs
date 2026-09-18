//! S1.2: `gwz hook claude-code setup` (D1, D5), and its `--remove` (A2).
//!
//! The writer edits another program's configuration, so every test here is
//! about what it does to a file: what it inserts, what it leaves alone, and
//! what it refuses. Nothing here touches a Claude settings file on this
//! machine; every target is a temporary directory.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::g01::TempDir;

use crate::hook::env::{HookEnv, ShareProbe};
use crate::hook::setup::{SetupPlacement, SetupRequest, handlers, render_block, run_setup};
use crate::hook::{DEFAULT_WAIT_SECS, HookOptions};

struct SetupEnv {
    home: PathBuf,
    warnings: RefCell<Vec<String>>,
}

impl SetupEnv {
    fn new(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            warnings: RefCell::new(Vec::new()),
        }
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.borrow().clone()
    }
}

impl HookEnv for SetupEnv {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    fn sleep(&self, _duration: Duration) {}

    fn free_bytes(&self, _dir: &Path) -> Option<u64> {
        Some(1 << 40)
    }

    fn probe_share(&self, _source: &Path, _destination_parent: &Path) -> ShareProbe {
        ShareProbe {
            cloned: true,
            same_filesystem: true,
            fs_type: Some("apfs".to_owned()),
        }
    }

    fn build_busy(&self, _root: &Path) -> bool {
        false
    }

    fn home_dir(&self) -> Option<PathBuf> {
        Some(self.home.clone())
    }

    fn warn(&self, line: &str) {
        self.warnings.borrow_mut().push(line.to_owned());
    }
}

fn request(write: bool) -> SetupRequest {
    SetupRequest {
        placement: SetupPlacement::User,
        write,
        remove: false,
        command: None,
        options: HookOptions::default(),
    }
}

fn removal() -> SetupRequest {
    SetupRequest {
        remove: true,
        ..request(false)
    }
}

fn settings(temp: &TempDir) -> PathBuf {
    temp.path().join(".claude/settings.json")
}

fn document(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn commands(document: &serde_json::Value, event: &str) -> Vec<String> {
    document["hooks"][event]
        .as_array()
        .expect("the event is an array")
        .iter()
        .flat_map(|group| group["hooks"].as_array().expect("a handler list").iter())
        .map(|handler| handler["command"].as_str().unwrap().to_owned())
        .collect()
}

/// A fresh file: the block is written, and it is the whole file.
#[test]
fn setup_writes_a_fresh_settings_file() {
    let home = TempDir::new("setup-fresh");
    let env = SetupEnv::new(home.path());
    let output = run_setup(&env, None, &request(true)).expect("the file is written");
    assert_eq!(output.target, settings(&home));
    let document = document(&settings(&home));
    assert_eq!(
        commands(&document, "WorktreeCreate"),
        vec!["gwz hook claude-code worktree-create".to_owned()]
    );
    assert_eq!(
        commands(&document, "WorktreeRemove"),
        vec!["gwz hook claude-code worktree-remove".to_owned()]
    );
    // Without --write nothing is created.
    let other = TempDir::new("setup-print");
    let env = SetupEnv::new(other.path());
    let printed = run_setup(&env, None, &request(false)).expect("the block is printed");
    assert!(!settings(&other).exists());
    assert!(
        printed.block.contains("WorktreeCreate"),
        "{}",
        printed.block
    );
    assert!(printed.note.contains("--write"), "{}", printed.note);
}

/// A file with unrelated hooks, unknown top-level keys and unusual
/// formatting: every byte outside the insertion is the file's own, and a
/// second run changes nothing at all.
#[test]
fn setup_merges_without_touching_a_byte_outside_the_block() {
    let home = TempDir::new("setup-merge");
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    let original = "{\n  \"unknownTopLevel\":   [1,2,3],\n\t\"hooks\": {\n    \"PreToolUse\": [{\"hooks\":[{\"type\":\"command\",\"command\":\"true\"}]}]\n  }\n}\n";
    std::fs::write(settings(&home), original).unwrap();
    let env = SetupEnv::new(home.path());

    run_setup(&env, None, &request(true)).expect("the block is merged");
    let merged = std::fs::read_to_string(settings(&home)).unwrap();
    let document = document(&settings(&home));
    assert_eq!(document["unknownTopLevel"], serde_json::json!([1, 2, 3]));
    assert_eq!(commands(&document, "PreToolUse"), vec!["true".to_owned()]);
    assert_eq!(commands(&document, "WorktreeCreate").len(), 1);
    // Every original byte survives, in order: the merge is an insertion.
    let mut remaining = merged.as_str();
    for fragment in ["\"unknownTopLevel\":   [1,2,3]", "\"PreToolUse\""] {
        let at = remaining.find(fragment).unwrap_or_else(|| {
            panic!("{fragment} is missing from the merged file:\n{merged}");
        });
        remaining = &remaining[at + fragment.len()..];
    }

    // A second run is a no-op, byte for byte.
    let note = run_setup(&env, None, &request(true)).expect("the second run is served");
    assert!(note.note.contains("already carries"), "{}", note.note);
    assert_eq!(std::fs::read_to_string(settings(&home)).unwrap(), merged);
}

/// A file that does not parse is refused and left exactly as it was, and so
/// is one this writer cannot replace: neither leaves a half-written file.
#[test]
fn setup_refuses_a_file_it_cannot_parse_or_replace() {
    let home = TempDir::new("setup-refuse");
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    let broken = "{ \"hooks\": [ }\n";
    std::fs::write(settings(&home), broken).unwrap();
    let env = SetupEnv::new(home.path());
    let failure = run_setup(&env, None, &request(true)).expect_err("a broken file is refused");
    assert!(failure.cause.contains("is not JSON"), "{}", failure.cause);
    assert_eq!(std::fs::read_to_string(settings(&home)).unwrap(), broken);

    // A write that cannot reach its rename leaves the original bytes: the
    // temporary file is written beside the target, so a directory standing
    // in its place stops the write before anything is replaced.
    let good = "{\n  \"permissions\": {}\n}\n";
    std::fs::write(settings(&home), good).unwrap();
    let temporary = home
        .path()
        .join(".claude")
        .join(format!(".settings.json.gwz-{}", std::process::id()));
    std::fs::create_dir_all(&temporary).unwrap();
    let failure = run_setup(&env, None, &request(true)).expect_err("the write cannot complete");
    assert!(
        failure.cause.contains("could not be written"),
        "{}",
        failure.cause
    );
    assert_eq!(std::fs::read_to_string(settings(&home)).unwrap(), good);
    std::fs::remove_dir_all(&temporary).unwrap();
}

/// D5: another placement carrying a differing handler is warned about by
/// name, and never refused.
#[test]
fn setup_warns_about_a_differing_handler_in_another_placement() {
    let home = TempDir::new("setup-warn-home");
    let project = TempDir::new("setup-warn-project");
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    std::fs::write(
        settings(&home),
        "{\"hooks\":{\"WorktreeCreate\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"/opt/gwz hook claude-code worktree-create\"}]}]}}\n",
    )
    .unwrap();
    let env = SetupEnv::new(home.path());
    let mut request = request(true);
    request.placement = SetupPlacement::Project { local: true };
    let output =
        run_setup(&env, Some(project.path()), &request).expect("the project file is written");
    assert_eq!(
        output.target,
        project.path().join(".claude/settings.local.json")
    );
    let warnings = env.warnings();
    assert!(
        warnings.iter().any(
            |line| line.contains("/opt/gwz hook claude-code worktree-create")
                && line.contains(".claude/settings.json")
        ),
        "{warnings:?}"
    );
}

/// `--command` and the hook options appear in the handler text, and both
/// handlers' timeouts are present and consistent with the baked
/// `--wait-secs` (S1.2).
#[test]
fn the_handler_text_carries_the_command_and_the_options() {
    let home = TempDir::new("setup-handlers");
    let env = SetupEnv::new(home.path());
    let mut request = request(false);
    request.command = Some("/usr/local/bin/gwz".to_owned());
    request.options = HookOptions {
        min_free_gb: Some(20.0),
        max_lanes: 3,
        wait_secs: DEFAULT_WAIT_SECS,
        base_ref: Some("main".to_owned()),
        log: None,
    };
    let outside = handlers(&env, None, &request);
    assert_eq!(
        outside.create_command,
        "/usr/local/bin/gwz hook claude-code worktree-create --min-free-gb 20 --max-lanes 3 \
         --base-ref main"
    );
    // The remove leaf obeys neither guard nor the fallback base, so its
    // handler carries none of them.
    assert_eq!(
        outside.remove_command,
        "/usr/local/bin/gwz hook claude-code worktree-remove"
    );
    // Outside a workspace both timeouts are the documented default.
    assert_eq!(outside.create_timeout, 600);
    assert_eq!(outside.remove_timeout, 600);

    let block: serde_json::Value = serde_json::from_str(&render_block(&outside)).unwrap();
    assert_eq!(
        block["hooks"]["WorktreeCreate"][0]["hooks"][0]["timeout"],
        600
    );
    assert_eq!(
        block["hooks"]["WorktreeRemove"][0]["hooks"][0]["timeout"],
        600
    );

    // Inside a workspace the wait is the estimated copy time, and the two
    // timeouts are one wait plus one estimated copy plus 60 s, and one wait
    // plus 60 s.
    let workspace = TempDir::new("setup-workspace");
    let root = workspace.path().join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("file.txt"), b"x").unwrap();
    let inside = handlers(&env, Some(&root), &request);
    let wait = inside
        .create_command
        .split("--wait-secs ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .map(|value| value.parse::<u64>().unwrap())
        .expect("the baked wait");
    assert_eq!(inside.create_timeout, wait * 2 + 60);
    assert_eq!(inside.remove_timeout, wait + 60);
    assert!(
        inside
            .remove_command
            .contains(&format!("--wait-secs {wait}"))
    );
}

/// `--remove` is `--write` run backwards: for a fresh file, for a file with
/// unrelated hooks, and for a file with unknown keys and unusual formatting,
/// the bytes after the removal are the bytes before the write.
#[test]
fn remove_restores_the_file_write_started_from() {
    for original in [
        "{}\n",
        "{\n  \"hooks\": {\n    \"PreToolUse\": [{\"hooks\":[{\"type\":\"command\",\"command\":\"true\"}]}]\n  }\n}\n",
        "{\n  \"unknownTopLevel\":   [1,2,3],\n\t\"hooks\": {\n    \"PreToolUse\": [{\"hooks\":[{\"type\":\"command\",\"command\":\"true\"}]}]\n  }\n}\n",
    ] {
        let home = TempDir::new("setup-remove");
        std::fs::create_dir_all(home.path().join(".claude")).unwrap();
        std::fs::write(settings(&home), original).unwrap();
        let env = SetupEnv::new(home.path());

        run_setup(&env, None, &request(true)).expect("the block is merged");
        let merged = std::fs::read_to_string(settings(&home)).unwrap();
        assert_ne!(merged, original);
        assert_eq!(
            commands(&document(&settings(&home)), "WorktreeCreate").len(),
            1
        );

        let output = run_setup(&env, None, &removal()).expect("the block is removed");
        assert!(output.note.contains("removed"), "{}", output.note);
        assert_eq!(
            std::fs::read_to_string(settings(&home)).unwrap(),
            original,
            "removal did not restore\n{merged}"
        );
        // The file is never deleted, and what it does carry still parses.
        assert!(settings(&home).exists());
    }
}

/// A file the tool never wrote to is left byte for byte as it was, and the
/// command says so; so is a missing file.
#[test]
fn remove_leaves_a_file_without_the_block_alone() {
    let home = TempDir::new("setup-remove-absent");
    let env = SetupEnv::new(home.path());
    let output = run_setup(&env, None, &removal()).expect("a missing file is served");
    assert!(output.note.contains("does not exist"), "{}", output.note);
    assert!(!settings(&home).exists());

    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    let original = "{\n  \"hooks\": {\n    \"PreToolUse\": [{\"hooks\":[{\"type\":\"command\",\"command\":\"true\"}]}]\n  }\n}\n";
    std::fs::write(settings(&home), original).unwrap();
    let output = run_setup(&env, None, &removal()).expect("a file without the block is served");
    assert!(
        output.note.contains("does not carry the block"),
        "{}",
        output.note
    );
    assert_eq!(std::fs::read_to_string(settings(&home)).unwrap(), original);
}

/// `--remove` refuses what `--write` refuses: a file that does not parse, and
/// one that is not a regular file. Neither is touched.
#[test]
fn remove_refuses_a_file_it_cannot_parse_or_is_not_a_file() {
    let home = TempDir::new("setup-remove-refuse");
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    let broken = "{ \"hooks\": [ }\n";
    std::fs::write(settings(&home), broken).unwrap();
    let env = SetupEnv::new(home.path());
    let failure = run_setup(&env, None, &removal()).expect_err("a broken file is refused");
    assert!(failure.cause.contains("is not JSON"), "{}", failure.cause);
    assert_eq!(std::fs::read_to_string(settings(&home)).unwrap(), broken);

    std::fs::remove_file(settings(&home)).unwrap();
    std::fs::create_dir_all(settings(&home)).unwrap();
    let failure = run_setup(&env, None, &removal()).expect_err("a directory is refused");
    assert!(
        failure.cause.contains("not a regular file"),
        "{}",
        failure.cause
    );
}

/// A settings path that is a symbolic link is refused rather than followed.
#[cfg(unix)]
mod symlinks {
    use super::*;

    #[test]
    fn setup_refuses_a_symlinked_settings_file() {
        let home = TempDir::new("setup-symlink");
        std::fs::create_dir_all(home.path().join(".claude")).unwrap();
        let real = home.path().join("real-settings.json");
        std::fs::write(&real, "{}\n").unwrap();
        std::os::unix::fs::symlink(&real, settings(&home)).unwrap();
        let env = SetupEnv::new(home.path());
        let failure = run_setup(&env, None, &request(true)).expect_err("a symlink is refused");
        assert!(failure.cause.contains("symbolic link"), "{}", failure.cause);
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "{}\n");

        let failure = run_setup(&env, None, &removal()).expect_err("a symlink is refused");
        assert!(failure.cause.contains("symbolic link"), "{}", failure.cause);
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "{}\n");
    }
}
