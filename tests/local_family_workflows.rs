//! End-to-end coverage for the local clone family surface (lane CR): the real
//! binary, real exit codes, and the channel each kind of refusal lands on.
//!
//! The driver must reach core's entry points and present what comes back,
//! rather than answering for them. Since LCM1.1 (gwz-core `81fcaf2`) the
//! verbatim create, `local list`, `dispose --keep` and `disband` are served
//! end to end; clean and bare, `--from`, ordinary `dispose` and a family
//! `--dry-run` still answer the typed `UnsupportedOperation` this build owes,
//! and the rows here moved with core (LCM1.1 fix 1, lane C, 2026-09-06).

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
/// A surface this build parses and dispatches but cannot yet serve.
const UNSUPPORTED: &str = "UnsupportedOperation";
/// Design §7 / §11 item 13: `merge --remote <name>` named no ready member.
const UNKNOWN_LOCAL: &str = "UnknownLocal";
/// `dispose --keep` in a workspace that is in no family (served since LCM1.1).
const MEMBER_NOT_FOUND: &str = "MemberNotFound";

#[test]
fn family_verbs_reach_core_and_report_its_typed_refusal() {
    let temp = TempDir::new("family-dispatch");
    init_workspace(&temp);

    for (args, code) in [
        (vec!["local", "dispose", "C"], UNSUPPORTED),
        (vec!["local", "dispose", "C", "--keep"], MEMBER_NOT_FOUND),
        (
            vec!["local", "dispose", "C", "--force", "dirty"],
            UNSUPPORTED,
        ),
        (
            vec!["clone", "--local", "--clean", "--name", "C", "../dest-c"],
            UNSUPPORTED,
        ),
        (
            vec!["clone", "--local", "--bare", "--name", "hub", "../dest-hub"],
            UNSUPPORTED,
        ),
        // `--from` is `copy_source` on the wire now (design §7, §11 item 11):
        // the driver encodes it and core owns the refusal, naming the flag.
        (
            vec![
                "clone",
                "--local",
                "--from",
                "A",
                "--name",
                "B",
                "../dest-b",
            ],
            UNSUPPORTED,
        ),
        // Likewise `--dry-run` on a local create: a family operation, so core
        // answers rather than the driver. The URL clone keeps its own gate,
        // asserted below.
        (
            vec!["--dry-run", "clone", "--local", "--name", "A", "../dest-a"],
            UNSUPPORTED,
        ),
        // Design §7/§11 item 13: a family-only merge miss is its own code.
        (vec!["merge", "--remote", "A"], UNKNOWN_LOCAL),
        (
            vec!["merge", "--remote", "C", "lane/agent-17"],
            UNKNOWN_LOCAL,
        ),
        (vec!["merge", "--remote", "origin"], UNKNOWN_LOCAL),
    ] {
        let human = run(&temp, &args);
        assert_eq!(exit(&human), DISPATCHED, "{args:?}: {}", stderr(&human));
        assert!(
            stderr(&human).starts_with(&format!("gwz: {code}: ")),
            "{args:?}: {}",
            stderr(&human)
        );
        assert!(human.stdout.is_empty(), "{args:?} wrote to stdout");

        let mut machine = vec!["--json"];
        machine.extend(args.iter().copied());
        let machine = run(&temp, &machine);
        assert_eq!(exit(&machine), DISPATCHED, "{args:?}");
        let json: Value = serde_json::from_slice(&machine.stdout).unwrap();
        assert_eq!(json["errors"][0]["code"], code, "{args:?}");
        assert!(machine.stderr.is_empty(), "{args:?} wrote to stderr");
    }

    // No directory was created by any of that: a refusing create must not
    // allocate its destination.
    for dest in ["dest-b", "dest-c", "dest-hub"] {
        assert!(
            !temp.path().parent().unwrap().join(dest).exists(),
            "{dest} was allocated by a refused create"
        );
    }
    assert!(!temp.path().join(".gwz/local-family.yml").exists());
    assert!(!temp.path().join(".gwz/local-family.lock").exists());

    // `disband` outside a family is served, as a no-op that writes nothing.
    let human = run(&temp, &["local", "disband"]);
    assert_eq!(exit(&human), 0, "{}", stderr(&human));
    assert!(
        String::from_utf8_lossy(&human.stdout).contains("nothing to disband"),
        "{}",
        String::from_utf8_lossy(&human.stdout)
    );
    let machine = run(&temp, &["--json", "local", "disband"]);
    assert_eq!(exit(&machine), 0, "{}", stderr(&machine));
    let json: Value = serde_json::from_slice(&machine.stdout).unwrap();
    assert_eq!(json["meta"]["aggregate_status"], "Noop");
    assert_eq!(json["errors"], Value::Array(Vec::new()));
    assert!(!temp.path().join(".gwz/local-family.yml").exists());
}

/// The served lifecycle, end to end through the real binary (LCM1.1): a
/// verbatim `clone --local` into a destination this test owns, `local list`
/// naming the root and the clone with the observed root beside them,
/// `dispose --keep` detaching the clone with every file retained, and
/// `disband` removing the index. Exit 0 and a message on every step.
#[test]
fn the_verbatim_lifecycle_is_served_end_to_end() {
    let temp = TempDir::new("family-lifecycle");
    init_workspace(&temp);
    let lanes = TempDir::new("family-lifecycle-lanes");
    let dest = lanes.path().join("dest-a");
    let dest_arg = dest.to_string_lossy().into_owned();

    let created = run(
        &temp,
        &["--json", "clone", "--local", "--name", "A", &dest_arg],
    );
    assert_eq!(exit(&created), 0, "{}", stderr(&created));
    let json: Value = serde_json::from_slice(&created.stdout).unwrap();
    assert_eq!(json["meta"]["aggregate_status"], "Ok");
    let message = json["meta"]["message"].as_str().unwrap_or("");
    assert!(message.contains("created local clone `A`"), "{message}");
    assert!(message.contains("dest-complete:"), "{message}");
    assert!(dest.join("gwz.conf/gwz.yml").is_file());
    assert!(dest.join(".gwz/family-root").is_file());

    let listed = run(&temp, &["--json", "local", "list"]);
    assert_eq!(exit(&listed), 0, "{}", stderr(&listed));
    let json: Value = serde_json::from_slice(&listed.stdout).unwrap();
    let members = json["local_family_members"].as_array().unwrap();
    let names: Vec<&str> = members
        .iter()
        .map(|member| member["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["root", "A"]);
    assert_eq!(members[0]["path"], ".");
    assert_eq!(members[1]["observed_state"], "ready");
    assert!(
        members[1]["path"].as_str().unwrap().ends_with("/dest-a"),
        "{}",
        members[1]["path"]
    );
    assert_eq!(
        json["local_family_root_path"].as_str().map(PathBuf::from),
        Some(temp.path().canonicalize().unwrap()),
        "the observed root travels beside the rows"
    );

    let detached = run(&temp, &["local", "dispose", "A", "--keep"]);
    assert_eq!(exit(&detached), 0, "{}", stderr(&detached));
    assert!(
        String::from_utf8_lossy(&detached.stdout).contains("detached local clone `A`"),
        "{}",
        String::from_utf8_lossy(&detached.stdout)
    );
    assert!(
        dest.join("gwz.conf/gwz.yml").is_file(),
        "every file is retained"
    );
    assert!(
        !dest.join(".gwz/family-root").exists(),
        "the pointer is gone"
    );

    let disbanded = run(&temp, &["local", "disband"]);
    assert_eq!(exit(&disbanded), 0, "{}", stderr(&disbanded));
    assert!(
        String::from_utf8_lossy(&disbanded.stdout).contains("disbanded local family"),
        "{}",
        String::from_utf8_lossy(&disbanded.stdout)
    );
    assert!(!temp.path().join(".gwz/local-family.yml").exists());
    assert!(dest.is_dir(), "every tree is retained");
}

/// LCM1.2, the plan's MVP loop through the real binary: a member committed
/// at the root, `clone --local --name A`, work committed in A's member, and
/// `gwz merge --remote A` integrating it at the root -- the engine's own
/// merge response with the retained import ref as its `source_ref`, the
/// import summarised in `meta.message`, one operation lifecycle on the
/// `--jsonl` stream, and a second merge after more work in A finding a
/// fresh import name beside the retained one.
#[test]
fn a_family_merge_by_name_integrates_the_clones_commits_end_to_end() {
    let temp = TempDir::new("family-merge");
    init_workspace(&temp);
    let created = run(&temp, &["repo", "create", "app"]);
    assert_eq!(exit(&created), 0, "{}", stderr(&created));
    let app = temp.path().join("app");
    let base = commit_file(&app, "README.md", "one\n", "initial");
    commit_workspace_root(temp.path());
    let lanes = TempDir::new("family-merge-lanes");
    let dest = lanes.path().join("dest-a");
    let dest_arg = dest.to_string_lossy().into_owned();
    let cloned = run(&temp, &["clone", "--local", "--name", "A", &dest_arg]);
    assert_eq!(exit(&cloned), 0, "{}", stderr(&cloned));
    assert_eq!(repo_ref(&dest.join("app"), "HEAD"), Some(base.clone()));

    let work = commit_file(&dest.join("app"), "feature.txt", "from A\n", "work in A");
    let merged = run(&temp, &["--json", "merge", "--remote", "A"]);
    assert_eq!(exit(&merged), 0, "{}", stderr(&merged));
    let json: Value = serde_json::from_slice(&merged.stdout).unwrap();
    assert_eq!(json["meta"]["aggregate_status"], "Ok");
    assert_eq!(json["merge"]["state"], "Completed");
    assert_eq!(json["merge"]["open"], false);
    let repo = &json["merge"]["repos"][0];
    assert_eq!(repo["target_id"], "mem_app");
    assert_eq!(repo["state"], "FastForwarded");
    let import_ref = repo["source_ref"].as_str().unwrap().to_owned();
    assert!(
        import_ref.starts_with("refs/gwz/local-imports/xfer_"),
        "{import_ref}"
    );
    assert_eq!(repo["source_commit"], work);
    assert_eq!(repo["resulting_commit"], work);
    let message = json["meta"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains(&format!(
            "imported HEAD of family member `A` as {import_ref} (mem_app={work})"
        )),
        "{message}"
    );
    assert_eq!(repo_ref(&app, "HEAD"), Some(work.clone()));
    assert_eq!(repo_ref(&app, &import_ref), Some(work.clone()));
    assert!(
        git2::Repository::open(&app)
            .unwrap()
            .remotes()
            .unwrap()
            .is_empty(),
        "no family remote is persisted"
    );

    // More work in A; the stream carries one lifecycle and the retained
    // ref stays beside the fresh one.
    let more = commit_file(
        &dest.join("app"),
        "feature.txt",
        "more from A\n",
        "more work in A",
    );
    let streamed = run(&temp, &["--jsonl", "merge", "--remote", "A"]);
    assert_eq!(exit(&streamed), 0, "{}", stderr(&streamed));
    let lines: Vec<Value> = String::from_utf8_lossy(&streamed.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let event_kinds: Vec<&str> = lines
        .iter()
        .filter(|line| line["kind"] == "event")
        .map(|line| line["event_kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        event_kinds
            .iter()
            .filter(|kind| **kind == "OperationStarted")
            .count(),
        1,
        "{event_kinds:?}"
    );
    assert_eq!(
        event_kinds
            .iter()
            .filter(|kind| **kind == "OperationFinished")
            .count(),
        1,
        "{event_kinds:?}"
    );
    assert_eq!(event_kinds.first(), Some(&"OperationStarted"));
    assert_eq!(event_kinds.last(), Some(&"OperationFinished"));
    let response = lines.last().unwrap();
    assert_eq!(response["kind"], "response");
    assert_eq!(response["merge"]["state"], "Completed");
    let fresh = response["merge"]["repos"][0]["source_ref"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(fresh, import_ref, "a retry mints a fresh transfer id");
    assert_eq!(repo_ref(&app, "HEAD"), Some(more.clone()));
    assert_eq!(repo_ref(&app, &fresh), Some(more));
    assert_eq!(
        repo_ref(&app, &import_ref),
        Some(work),
        "the earlier import ref is retained, never pruned"
    );

    // The human channel, up to date now: exit 0 and the engine's own table,
    // whose source column names the retained import ref.
    let human = run(&temp, &["merge", "--remote", "A"]);
    assert_eq!(exit(&human), 0, "{}", stderr(&human));
    let stdout = String::from_utf8_lossy(&human.stdout);
    assert!(stdout.contains("app (mem_app)  up-to-date"), "{stdout}");
    assert!(
        stdout.contains("source: refs/gwz/local-imports/xfer_"),
        "{stdout}"
    );
}

/// `gwz local list` end to end. A workspace that holds no family index lists
/// nothing, and says so: exit 0 on stdout, not a refusal, and no fabricated
/// row. The columns themselves are pinned by the unit tests, which can supply
/// a populated `LocalFamilyResponse` while the store is still landing.
#[test]
fn local_list_reports_an_empty_family_without_inventing_one() {
    let temp = TempDir::new("family-list");
    init_workspace(&temp);

    let human = run(&temp, &["local", "list"]);
    assert_eq!(exit(&human), 0, "{}", stderr(&human));
    assert_eq!(
        String::from_utf8_lossy(&human.stdout).trim_end(),
        "no local clone family members"
    );

    let machine = run(&temp, &["--json", "local", "list"]);
    assert_eq!(exit(&machine), 0, "{}", stderr(&machine));
    let json: Value = serde_json::from_slice(&machine.stdout).unwrap();
    assert_eq!(json["errors"], Value::Array(Vec::new()));
    assert_eq!(json["meta"]["aggregate_status"], "Ok");
    assert_eq!(json["local_family_members"], Value::Array(Vec::new()));

    // Listing takes no lock and writes nothing (design §8.1).
    assert!(!temp.path().join(".gwz/local-family.yml").exists());
    assert!(!temp.path().join(".gwz/local-family.lock").exists());
}

/// A refusal never carries a listing: the family rows are absent from the
/// error envelope rather than rendered as an empty table. Ordinary `dispose`
/// is the verb this build still refuses (`disband` is served since LCM1.1).
#[test]
fn a_refused_family_verb_carries_no_listing() {
    let temp = TempDir::new("family-list-refusal");
    init_workspace(&temp);

    let machine = run(&temp, &["--json", "local", "dispose", "C"]);
    assert_eq!(exit(&machine), DISPATCHED);
    let json: Value = serde_json::from_slice(&machine.stdout).unwrap();
    assert_eq!(json["errors"][0]["code"], UNSUPPORTED);
    assert!(
        json.get("local_family_members").is_none(),
        "a refusal carries no family rows: {json}"
    );
}

#[test]
fn jsonl_streams_the_operation_lifecycle_then_the_refusal() {
    let temp = TempDir::new("family-jsonl");
    init_workspace(&temp);

    let output = run(&temp, &["--jsonl", "local", "dispose", "C"]);
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
            vec!["clone", "--local", "--from", "", "--name", "A"],
            "must not be empty",
        ),
        (
            vec!["--dry-run", "clone", "https://example.invalid/ws.git"],
            "--dry-run is not supported for clone",
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

fn commit_file(repo_path: &Path, relative_path: &str, content: &str, message: &str) -> String {
    let repo = git2::Repository::open(repo_path).unwrap();
    let workdir = repo.workdir().unwrap();
    std::fs::write(workdir.join(relative_path), content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(relative_path)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
    let parents: Vec<git2::Commit<'_>> = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| repo.find_commit(oid).unwrap())
        .into_iter()
        .collect();
    let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parent_refs,
    )
    .unwrap()
    .to_string()
}

fn commit_workspace_root(root: &Path) {
    let repo = git2::Repository::open(root).unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
    let parents: Vec<git2::Commit<'_>> = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| repo.find_commit(oid).unwrap())
        .into_iter()
        .collect();
    let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "init workspace",
        &tree,
        &parent_refs,
    )
    .unwrap();
}

fn repo_ref(repo_path: &Path, ref_name: &str) -> Option<String> {
    git2::Repository::open(repo_path)
        .unwrap()
        .revparse_single(ref_name)
        .ok()
        .map(|object| object.id().to_string())
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
