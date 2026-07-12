//! D5 acceptance: `gwz diff` end-to-end against the real `gwz` binary and a real
//! workspace (root + a member). Covers the plan's D5 CLI acceptance list:
//! stdout patch correctness vs `git`, no-pager-when-piped, exit codes, `-z`,
//! prefixes, `--json`/`--jsonl`, and manifest-only formats.
//!
//! These tests pipe output (stdout is not a TTY under the test harness), which is
//! exactly the "no pager when piped" path — a pager would hang the test, so its
//! absence is asserted implicitly by every test completing.

use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

// ── acceptance tests ─────────────────────────────────────────────────────────

#[test]
fn diff_writes_unified_workspace_relative_patch_to_stdout() {
    let ws = Workspace::new("patch");
    ws.dirty_member("one\ntwo\n");
    ws.dirty_root("roottop\nmore\n");

    let out = ws.diff(&[]);
    assert_success(&out);
    let stdout = stdout(&out);

    // Root first, then member with the `lib/` workspace prefix.
    assert!(
        stdout.contains("diff --git a/top.txt b/top.txt"),
        "{stdout}"
    );
    assert!(
        stdout.contains("diff --git a/lib/x.txt b/lib/x.txt"),
        "member path must be workspace-relative:\n{stdout}"
    );
    let root_pos = stdout.find("a/top.txt").unwrap();
    let member_pos = stdout.find("a/lib/x.txt").unwrap();
    assert!(
        root_pos < member_pos,
        "root must sort before member:\n{stdout}"
    );
}

#[test]
fn diff_patch_body_matches_git_for_each_repo() {
    let ws = Workspace::new("git-parity");
    ws.dirty_member("one\ntwo\n");

    let gwz_out = stdout(&ws.diff(&[]));
    // The member hunk should be byte-identical to `git diff` in the member (only
    // the header path differs: git prints x.txt, gwz prints lib/x.txt).
    let git_member = git_stdout(&ws.member_path(), &["diff", "x.txt"]);
    let git_hunk = git_member.split_once("@@").map(|(_, rest)| rest).unwrap();
    let gwz_hunk = gwz_out.split_once("@@").map(|(_, rest)| rest).unwrap();
    assert_eq!(gwz_hunk, git_hunk, "hunk bytes must match git");
}

#[test]
fn diff_piped_does_not_launch_a_pager() {
    // Piped (non-TTY) output must go straight to stdout; if a pager were launched
    // this call would block or fail. Completing + producing patch text proves it.
    let ws = Workspace::new("no-pager");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&[]);
    assert_success(&out);
    assert!(stdout(&out).contains("diff --git"));
    assert!(
        out.stderr.is_empty(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn diff_no_pager_flag_is_accepted_and_writes_directly() {
    let ws = Workspace::new("no-pager-flag");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--no-pager"]);
    assert_success(&out);
    assert!(stdout(&out).contains("diff --git a/lib/x.txt"));
}

#[test]
fn plain_diff_exits_zero_even_with_differences() {
    let ws = Workspace::new("plain-exit");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&[]);
    assert_eq!(out.status.code(), Some(0), "plain diff exits 0 (like git)");
}

#[test]
fn quiet_exit_code_returns_one_on_differences_and_zero_when_clean() {
    let ws = Workspace::new("quiet-exit");

    // Clean: no differences → exit 0, no output.
    let clean = ws.diff(&["--quiet", "--exit-code"]);
    assert_eq!(clean.status.code(), Some(0));
    assert!(clean.stdout.is_empty());

    // Dirty: differences → exit 1, still no patch output.
    ws.dirty_member("one\ntwo\n");
    let dirty = ws.diff(&["--quiet", "--exit-code"]);
    assert_eq!(dirty.status.code(), Some(1));
    assert!(dirty.stdout.is_empty(), "quiet emits no patch");
}

#[test]
fn exit_code_alone_prints_patch_and_exits_one() {
    let ws = Workspace::new("exit-code-only");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--exit-code"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout(&out).contains("diff --git"),
        "--exit-code still prints the patch"
    );
}

#[test]
fn json_quiet_returns_summary_metadata() {
    let ws = Workspace::new("json-quiet");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff_global(&["--json"], &["--quiet"]);
    // --quiet implies --exit-code, so with differences the process exits 1 even in
    // JSON mode; the summary metadata is still written to stdout.
    assert_eq!(out.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["kind"], "diff");
    assert_eq!(value["summary"]["has_differences"], true);
    // --quiet uses the any_difference fast path: no file list.
    assert!(value["files"].as_array().unwrap().is_empty());
}

#[test]
fn json_diff_emits_manifest_without_patch_bytes() {
    let ws = Workspace::new("json-manifest");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff_global(&["--json"], &[]);
    assert_success(&out);
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let files = value["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["new_path"], "lib/x.txt");
    assert_eq!(files[0]["status"], "Modified");
    // No patch bytes leak into the metadata document (AD5).
    assert!(value.get("data").is_none());
    assert!(!value.to_string().contains("data_base64"));
}

#[test]
fn jsonl_diff_emits_manifest_then_output_records_with_base64_bytes() {
    let ws = Workspace::new("jsonl");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff_global(&["--jsonl"], &[]);
    assert_success(&out);
    let lines: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(lines.iter().any(|l| l["kind"] == "diff_file"));
    // Patch bytes ride an output record, base64-expanded (BYTES has no JSON type).
    let patch = lines
        .iter()
        .find(|l| l["record_kind"] == "patch_bytes")
        .expect("a patch_bytes output record");
    assert!(patch["data_base64"].is_string());
}

#[test]
fn z_name_only_uses_nul_separators() {
    let ws = Workspace::new("z-name-only");
    ws.dirty_member("one\ntwo\n");
    ws.dirty_root("roottop\nmore\n");
    let out = ws.diff(&["-z", "--name-only"]);
    assert_success(&out);
    // Records are NUL-terminated, not newline-terminated.
    assert!(out.stdout.contains(&0u8), "expected NUL separators");
    assert!(!out.stdout.contains(&b'\n'), "no newlines under -z");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("top.txt"));
    assert!(text.contains("lib/x.txt"));
}

#[test]
fn name_status_reports_status_and_workspace_paths() {
    let ws = Workspace::new("name-status");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--name-status"]);
    assert_success(&out);
    assert_eq!(stdout(&out).trim(), "M\tlib/x.txt");
}

#[test]
fn numstat_matches_git_counts_with_workspace_paths() {
    let ws = Workspace::new("numstat");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--numstat"]);
    assert_success(&out);
    assert_eq!(stdout(&out).trim(), "1\t0\tlib/x.txt");
}

#[test]
fn custom_src_and_dst_prefixes_apply_over_the_member_prefix() {
    let ws = Workspace::new("prefixes");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--src-prefix", "x/", "--dst-prefix", "y/"]);
    assert_success(&out);
    let stdout = stdout(&out);
    // The custom prefix wraps the member path.
    assert!(stdout.contains("--- x/lib/x.txt"), "{stdout}");
    assert!(stdout.contains("+++ y/lib/x.txt"), "{stdout}");
}

#[test]
fn no_prefix_still_keeps_the_member_path() {
    let ws = Workspace::new("no-prefix");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--no-prefix"]);
    assert_success(&out);
    let stdout = stdout(&out);
    assert!(stdout.contains("--- lib/x.txt"), "{stdout}");
    assert!(stdout.contains("+++ lib/x.txt"), "{stdout}");
}

#[test]
fn cached_diffs_the_staged_index() {
    let ws = Workspace::new("cached");
    // Stage a change in the member but leave the worktree matching the index.
    ws.write_member("one\ntwo\n");
    git(&ws.member_path(), &["add", "-A"]);
    // Plain diff (worktree vs index) is now empty; --cached shows the staged change.
    let plain = ws.diff(&[]);
    assert!(stdout(&plain).is_empty(), "worktree==index → empty");
    let cached = ws.diff(&["--cached"]);
    assert!(
        stdout(&cached).contains("diff --git a/lib/x.txt"),
        "{}",
        stdout(&cached)
    );
}

#[test]
fn stat_reports_a_diffstat_summary_line() {
    let ws = Workspace::new("stat");
    ws.dirty_member("one\ntwo\n");
    let out = ws.diff(&["--stat"]);
    assert_success(&out);
    let stdout = stdout(&out);
    assert!(stdout.contains("lib/x.txt"), "{stdout}");
    assert!(stdout.contains("1 file changed"), "{stdout}");
}

#[test]
fn whitespace_flag_ignores_whitespace_only_changes() {
    let ws = Workspace::new("ignore-space");
    ws.dirty_member("one   \n");

    let plain = ws.diff(&[]);
    assert_success(&plain);
    assert!(stdout(&plain).contains("diff --git a/lib/x.txt"));

    let ignored = ws.diff(&["-w"]);
    assert_success(&ignored);
    assert!(
        stdout(&ignored).is_empty(),
        "-w should suppress whitespace-only hunks:\n{}",
        stdout(&ignored)
    );
}

#[test]
fn inter_hunk_context_merges_nearby_zero_context_hunks() {
    let ws = Workspace::new("inter-hunk");
    ws.write_member("a\nb\nc\nd\ne\n");
    git(&ws.member_path(), &["add", "-A"]);
    git_commit(&ws.member_path(), "expand member baseline");

    ws.dirty_member("a\nB\nc\nd\nE\n");

    let split = stdout(&ws.diff(&["-U0"]));
    assert_eq!(hunk_header_count(&split), 2, "{split}");

    let merged = stdout(&ws.diff(&["-U0", "--inter-hunk-context=2"]));
    assert_eq!(hunk_header_count(&merged), 1, "{merged}");
}

#[test]
fn binary_flag_emits_binary_patch_marker_without_large_golden() {
    let ws = Workspace::new("binary");
    std::fs::write(ws.member_path().join("bin.dat"), [0, 1, 2, 3, 0]).unwrap();
    git(&ws.member_path(), &["add", "-A"]);
    git_commit(&ws.member_path(), "seed binary");

    std::fs::write(ws.member_path().join("bin.dat"), [0, 1, 9, 3, 0, 4]).unwrap();

    let out = ws.diff(&["--binary"]);
    assert_success(&out);
    let stdout = stdout(&out);
    assert!(stdout.contains("diff --git a/lib/bin.dat b/lib/bin.dat"));
    assert!(stdout.contains("GIT binary patch"), "{stdout}");
}

// ── D5 bare-operand classification (git's rev/path split) ────────────────────

#[test]
fn bare_file_operand_diffs_the_file_without_dashdash() {
    // The user's exact repro shape: `gwz diff <path>` with no `--` must classify
    // the path as a pathspec, not a revspec, and diff that file.
    let ws = Workspace::new("bare-file");
    ws.dirty_member("one\ntwo\n");
    ws.dirty_root("roottop\nmore\n");

    let out = ws.diff(&["lib/x.txt"]);
    assert_success(&out);
    let stdout = stdout(&out);
    // Only the named member file diffs; the root change is out of scope.
    assert!(
        stdout.contains("diff --git a/lib/x.txt b/lib/x.txt"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("top.txt"),
        "root file out of scope:\n{stdout}"
    );
}

#[test]
fn bare_file_operand_matches_dashdash_form() {
    let ws = Workspace::new("bare-eq-dd");
    ws.dirty_member("one\ntwo\n");

    let bare = stdout(&ws.diff(&["lib/x.txt"]));
    let dashdash = stdout(&ws.diff(&["--", "lib/x.txt"]));
    assert_eq!(
        bare, dashdash,
        "bare operand must equal the `-- <file>` form"
    );
}

#[test]
fn mixed_revision_and_bare_file_operand() {
    // `gwz diff HEAD lib/x.txt` — a revision then a bare file, no `--`.
    let ws = Workspace::new("mixed-cli");
    ws.dirty_member("one\ntwo\n");

    let out = ws.diff(&["HEAD", "lib/x.txt"]);
    assert_success(&out);
    assert!(stdout(&out).contains("diff --git a/lib/x.txt b/lib/x.txt"));
}

#[test]
fn member_name_and_dot_pathspecs_diff_the_whole_member() {
    // Regression: `gwz diff <member>` and `gwz diff .` must diff the whole
    // member, not print nothing and exit 0. A pathspec naming a member root
    // routes to the repo-root primitive `.`; the plan's whole-repo narrowing
    // must become the *empty* pathspec list, because libgit2's diff matcher
    // treats a literal "." as a path that matches no file. In v0.9.1 this
    // printed nothing on a dirty member.
    let ws = Workspace::new("member-name-pathspec");
    ws.dirty_member("one\ntwo\n");

    // `gwz diff lib` — bare operand naming the member root.
    let by_name = ws.diff(&["lib"]);
    assert_success(&by_name);
    assert!(
        stdout(&by_name).contains("diff --git a/lib/x.txt b/lib/x.txt"),
        "`gwz diff lib` must diff the whole member, got:\n{}",
        stdout(&by_name)
    );

    // `gwz diff .` at the workspace root — fans out to root + members.
    let by_dot = ws.diff(&["."]);
    assert_success(&by_dot);
    assert!(
        stdout(&by_dot).contains("diff --git a/lib/x.txt b/lib/x.txt"),
        "`gwz diff .` must diff the member, got:\n{}",
        stdout(&by_dot)
    );

    // The bare member-name operand equals the explicit `-- lib` form.
    let dashdash = ws.diff(&["--", "lib"]);
    assert_eq!(
        stdout(&by_name),
        stdout(&dashdash),
        "bare member-name operand must equal the `-- lib` form"
    );
}

#[test]
fn nonexistent_bare_operand_fails_with_dashdash_hint() {
    let ws = Workspace::new("bad-operand");
    let out = ws.diff(&["definitely-not-a-thing"]);
    assert!(!out.status.success(), "unknown operand must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown revision or path") && stderr.contains("--"),
        "stderr should hint at `--`:\n{stderr}"
    );
}

// ── workspace fixture ────────────────────────────────────────────────────────

/// A materialized GWZ workspace: root repo + one Git member `lib`, each with a
/// committed baseline file so `gwz diff` has a HEAD to compare against.
struct Workspace {
    root: std::path::PathBuf,
}

impl Workspace {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "gwz-cli-diff-{prefix}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let ws = Workspace { root };
        assert_success(&ws.gwz(&["--root", ws.root_str(), "init"]));
        assert_success(&ws.gwz(&["--root", ws.root_str(), "repo", "create", "lib"]));

        // Seed + commit the member baseline.
        std::fs::write(ws.member_path().join("x.txt"), "one\n").unwrap();
        git(&ws.member_path(), &["add", "-A"]);
        git_commit(&ws.member_path(), "member init");

        // Seed + commit the root baseline (repo::create already `git init`ed root).
        std::fs::write(ws.root.join("top.txt"), "roottop\n").unwrap();
        git(&ws.root, &["add", "-A"]);
        git_commit(&ws.root, "root init");

        ws
    }

    fn root_str(&self) -> &str {
        self.root.to_str().unwrap()
    }

    fn member_path(&self) -> std::path::PathBuf {
        self.root.join("lib")
    }

    /// Overwrite the member file (worktree only).
    fn write_member(&self, content: &str) {
        std::fs::write(self.member_path().join("x.txt"), content).unwrap();
    }

    /// Dirty the member worktree (unstaged change).
    fn dirty_member(&self, content: &str) {
        self.write_member(content);
    }

    /// Dirty the root worktree (unstaged change).
    fn dirty_root(&self, content: &str) {
        std::fs::write(self.root.join("top.txt"), content).unwrap();
    }

    /// Run `gwz --root <root> diff <args>` with cwd at the workspace root.
    fn diff(&self, args: &[&str]) -> Output {
        self.diff_global(&[], args)
    }

    /// Run `gwz <global> --root <root> diff <diff-args>`.
    fn diff_global(&self, global: &[&str], diff_args: &[&str]) -> Output {
        let mut argv: Vec<&str> = Vec::new();
        argv.extend_from_slice(global);
        argv.push("--root");
        argv.push(self.root_str());
        argv.push("diff");
        argv.extend_from_slice(diff_args);
        self.gwz(&argv)
    }

    fn gwz(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gwz"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?} in {}", dir.display());
}

fn git_commit(dir: &Path, message: &str) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=GWZ Test",
            "-c",
            "user.email=gwz@example.invalid",
        ])
        .args(["commit", "-q", "-m", message])
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git commit in {}", dir.display());
}

fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn hunk_header_count(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("@@ ")).count()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "exit={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
