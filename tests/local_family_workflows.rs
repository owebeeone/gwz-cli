//! End-to-end coverage for the local clone family surface (lane CR): the real
//! binary, real exit codes, and the channel each kind of refusal lands on.
//!
//! The engine behind the family is still landing, so every dispatched verb
//! answers `UnsupportedOperation`. That is the point of these cases: the driver
//! must reach core's entry points and present what comes back, rather than
//! answering for them.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// A dispatched refusal: core answered, so the process exits 1 and machine
/// modes carry the typed code on stdout.
const DISPATCHED: i32 = 1;
/// A refusal the driver made before building a request: exit 2, the same code
/// every other rejected `gwz` invocation uses.
const REJECTED: i32 = 2;

#[test]
fn family_verbs_reach_core_and_report_its_typed_refusal() {
    let temp = TempDir::new("family-dispatch");
    init_workspace(&temp);

    for args in [
        vec!["local", "list"],
        vec!["local", "dispose", "C"],
        vec!["local", "dispose", "C", "--keep"],
        vec!["local", "dispose", "C", "--force", "dirty"],
        vec!["local", "disband"],
        vec!["clone", "--local", "--name", "A", "../dest-a"],
        vec!["clone", "--local", "--clean", "--name", "C", "../dest-c"],
        vec!["clone", "--local", "--bare", "--name", "hub", "../dest-hub"],
        vec!["merge", "--remote", "A"],
        vec!["merge", "--remote", "C", "lane/agent-17"],
    ] {
        let human = run(&temp, &args);
        assert_eq!(exit(&human), DISPATCHED, "{args:?}: {}", stderr(&human));
        assert!(
            stderr(&human).starts_with("gwz: UnsupportedOperation: "),
            "{args:?}: {}",
            stderr(&human)
        );
        assert!(human.stdout.is_empty(), "{args:?} wrote to stdout");

        let mut machine = vec!["--json"];
        machine.extend(args.iter().copied());
        let machine = run(&temp, &machine);
        assert_eq!(exit(&machine), DISPATCHED, "{args:?}");
        let json: Value = serde_json::from_slice(&machine.stdout).unwrap();
        assert_eq!(
            json["errors"][0]["code"], "UnsupportedOperation",
            "{args:?}"
        );
        assert!(machine.stderr.is_empty(), "{args:?} wrote to stderr");
    }

    // No directory was created by any of that: a refusing create must not
    // allocate its destination.
    for dest in ["dest-a", "dest-c", "dest-hub"] {
        assert!(
            !temp.path().parent().unwrap().join(dest).exists(),
            "{dest} was allocated by a refused create"
        );
    }
    assert!(!temp.path().join(".gwz/local-family.yml").exists());
    assert!(!temp.path().join(".gwz/local-family.lock").exists());
}

#[test]
fn jsonl_streams_the_operation_lifecycle_then_the_refusal() {
    let temp = TempDir::new("family-jsonl");
    init_workspace(&temp);

    let output = run(&temp, &["--jsonl", "local", "list"]);
    assert_eq!(exit(&output), DISPATCHED);
    let lines: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let kinds: Vec<&str> = lines
        .iter()
        .map(|line| line["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, vec!["event", "event", "response"], "{lines:?}");
    assert_eq!(lines[0]["event_kind"], "OperationStarted");
    assert_eq!(lines[1]["event_kind"], "OperationFinished");
    assert_eq!(lines[2]["errors"][0]["code"], "UnsupportedOperation");
}

#[test]
fn driver_refusals_are_rejected_before_anything_is_dispatched() {
    let temp = TempDir::new("family-rejected");
    init_workspace(&temp);

    for (args, needle) in [
        (vec!["clone", "--local", "dest"], "--name"),
        (
            vec!["clone", "--local", "--name", "A", "url", "dest"],
            "not a workspace URL",
        ),
        (
            vec!["clone", "--local", "--verbatim", "--clean", "--name", "A"],
            "mutually exclusive",
        ),
        (
            vec!["clone", "--local", "-b", "lane/x", "--name", "A"],
            "--clean or --bare",
        ),
        (
            vec!["clone", "--local", "--from", "B", "--name", "A"],
            "--from",
        ),
        (vec!["clone", "url", "--clean"], "only with --local"),
        (vec!["local", "dispose", "C", "--force"], "hazard names"),
        (
            vec!["local", "dispose", "C", "--keep", "--force", "dirty"],
            "mutually exclusive",
        ),
        (vec!["local", "dispose", "C", "dirty"], "only with --force"),
        (
            vec!["merge", "--remote", "A", "--abort"],
            "starting a merge",
        ),
    ] {
        let output = run(&temp, &args);
        assert_eq!(exit(&output), REJECTED, "{args:?}: {}", stderr(&output));
        assert!(
            stderr(&output).contains(needle),
            "{args:?}: expected `{needle}`, got: {}",
            stderr(&output)
        );
        assert!(output.stdout.is_empty(), "{args:?} wrote to stdout");
    }
}

/// The URL clone keeps its own required-argument error, and the local flags
/// did not make `<url>` optional for it.
#[test]
fn the_url_clone_still_requires_its_url() {
    let temp = TempDir::new("family-url-clone");
    let output = run(&temp, &["clone"]);
    assert_eq!(exit(&output), REJECTED);
    assert!(stderr(&output).contains("<url>"), "{}", stderr(&output));
}

// ---------------------------------------------------------------------------
// harness
// ---------------------------------------------------------------------------

fn run(temp: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gwz"))
        .current_dir(temp.path())
        .args(args)
        .output()
        .unwrap()
}

fn init_workspace(temp: &TempDir) {
    let output = run(temp, &["init"]);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn exit(output: &Output) -> i32 {
    output.status.code().unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gwz-cli-it-{prefix}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
