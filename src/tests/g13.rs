//! S1.1: `gwz hook claude-code worktree-create|worktree-remove`
//! (gwz-cli `dev-docs/GwzClaudeIntegrationPlan.md`, D1, D2, D6, D8, D10).
//!
//! The fixtures are built in temporary directories: a GWZ workspace for the
//! lane cases and a plain Git repository for D8's fallback. Nothing here
//! reads or writes a Claude settings file on this machine.
//!
//! Tests that need R20's owner token or R21's `--wait` are marked
//! `#[ignore = "needs R20/R21"]`: until the owner is recorded on the family
//! row, every row reads back no owner, which is the fail-closed side of D6's
//! table.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::g01::{TempDir, request_meta};
use super::*;

use crate::hook::create::{Decision, decide, run_worktree_create};
use crate::hook::env::{HookEnv, ShareProbe};
use crate::hook::estimate::{SourceWalk, check_free_space, check_lane_ceiling, estimate_from};
use crate::hook::family::FamilyRead;
use crate::hook::ignore::path_is_ignored;
use crate::hook::input::{parse_create_input, parse_remove_input, validate_lane_name};
use crate::hook::logging::{LogRecord, resolve_log_location, write_log};
use crate::hook::remove::run_worktree_remove;
use crate::hook::{Classification, CreateInput, HookContext, HookOptions, RemoveInput};

// ---------------------------------------------------------------------------
// the injected environment
// ---------------------------------------------------------------------------

/// Everything the hook learns from the machine, forced. Injection rather
/// than environment variables: a process-global variable would bleed between
/// tests that run in parallel.
struct TestEnv {
    free: Option<u64>,
    probe: ShareProbe,
    build_busy: bool,
    home: Option<PathBuf>,
    now: RefCell<SystemTime>,
    warnings: RefCell<Vec<String>>,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            free: Some(1 << 40),
            probe: ShareProbe {
                cloned: true,
                same_filesystem: true,
                fs_type: Some("apfs".to_owned()),
            },
            build_busy: false,
            home: None,
            now: RefCell::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000)),
            warnings: RefCell::new(Vec::new()),
        }
    }

    fn with_home(mut self, home: &Path) -> Self {
        self.home = Some(home.to_path_buf());
        self
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.borrow().clone()
    }

    fn warned(&self, fragment: &str) -> bool {
        self.warnings().iter().any(|line| line.contains(fragment))
    }
}

impl HookEnv for TestEnv {
    fn now(&self) -> SystemTime {
        *self.now.borrow()
    }

    fn sleep(&self, duration: Duration) {
        let advanced = *self.now.borrow() + duration;
        *self.now.borrow_mut() = advanced;
    }

    fn free_bytes(&self, _dir: &Path) -> Option<u64> {
        self.free
    }

    fn probe_share(&self, _source: &Path, _destination_parent: &Path) -> ShareProbe {
        self.probe.clone()
    }

    fn build_busy(&self, _root: &Path) -> bool {
        self.build_busy
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn warn(&self, line: &str) {
        self.warnings.borrow_mut().push(line.to_owned());
    }
}

// ---------------------------------------------------------------------------
// fixtures
// ---------------------------------------------------------------------------

/// A container holding a workspace root, so every lane the hook creates at
/// the default sibling destination is removed with the fixture.
struct Fixture {
    container: TempDir,
}

impl Fixture {
    fn new(prefix: &str) -> Self {
        let container = TempDir::new(prefix);
        let root = container.path().join("ws");
        std::fs::create_dir_all(&root).unwrap();
        gwz_core::workspace_ops::handle_create_workspace(
            gwz_core::CreateWorkspaceRequest {
                meta: request_meta("req_hook_setup"),
                workspace_root: root.to_string_lossy().into_owned(),
                workspace_id: Some("ws_hook".to_owned()),
            },
            "op_hook_setup",
        )
        .unwrap();
        let repository = git2::Repository::open(&root).expect("the root is a repository");
        let mut config = repository.config().unwrap();
        config.set_str("user.name", "GWZ Fixture").unwrap();
        config
            .set_str("user.email", "fixture@example.invalid")
            .unwrap();
        let invocation = parse_args_with_request_id(
            vec![
                "--root".to_owned(),
                root.to_string_lossy().into_owned(),
                "--target".to_owned(),
                "@root".to_owned(),
                "add".to_owned(),
                "-A".to_owned(),
            ],
            "req_hook_add",
            &root,
        )
        .unwrap();
        execute_invocation(&invocation).unwrap();
        let invocation = parse_args_with_request_id(
            vec![
                "--root".to_owned(),
                root.to_string_lossy().into_owned(),
                "--target".to_owned(),
                "@root".to_owned(),
                "commit".to_owned(),
                "-m".to_owned(),
                "initial workspace".to_owned(),
            ],
            "req_hook_commit",
            &root,
        )
        .unwrap();
        execute_invocation(&invocation).unwrap();
        Self { container }
    }

    fn root(&self) -> PathBuf {
        self.container.path().join("ws")
    }

    fn lane(&self, name: &str) -> PathBuf {
        self.container.path().join(format!("ws-{name}"))
    }
}

fn context(project_dir: &Path) -> HookContext {
    HookContext {
        project_dir: project_dir.to_path_buf(),
        options: HookOptions::default(),
    }
}

fn create_input(name: &str, session: &str) -> CreateInput {
    CreateInput {
        name: name.to_owned(),
        session_id: session.to_owned(),
    }
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// A plain Git repository with one commit, for D8's fallback.
fn plain_repository(prefix: &str) -> TempDir {
    let temp = TempDir::new(prefix);
    let root = temp.path();
    run_git(root, &["init", "-q", "-b", "main", "."]);
    std::fs::write(root.join("README.md"), b"fixture\n").unwrap();
    run_git(root, &["add", "README.md"]);
    run_git(
        root,
        &[
            "-c",
            "user.name=GWZ Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-q",
            "-m",
            "initial",
        ],
    );
    temp
}

fn run_git(directory: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn git_status(directory: &Path) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["status", "--porcelain"])
        .output()
        .expect("git runs");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn exclude_block(root: &Path) -> String {
    std::fs::read_to_string(root.join(".git/info/exclude")).unwrap_or_default()
}

fn ready_row(name: &str, path: &str) -> gwz_core::LocalFamilyMemberEntry {
    gwz_core::LocalFamilyMemberEntry {
        name: name.to_owned(),
        kind: gwz_core::LocalMemberKind::Checkout,
        recorded_state: gwz_core::LocalMemberState::Ready,
        observed_state: gwz_core::LocalObservedState::Ready,
        path: path.to_owned(),
        last_error: None,
        owner: None,
    }
}

/// A `ready` row carrying an owner token (R20).
fn owned_row(name: &str, path: &str, owner: &str) -> gwz_core::LocalFamilyMemberEntry {
    let mut row = ready_row(name, path);
    row.owner = Some(owner.to_owned());
    row
}

fn family(root: &Path, members: Vec<gwz_core::LocalFamilyMemberEntry>) -> FamilyRead {
    FamilyRead {
        root_path: Some(root.to_string_lossy().into_owned()),
        members,
    }
}

// ---------------------------------------------------------------------------
// the input contract
// ---------------------------------------------------------------------------

/// Malformed or missing stdin fields exit non-zero with a message naming the
/// field (S1.1).
#[test]
fn hook_input_refusals_name_the_field() {
    let failure = parse_create_input("not json").unwrap_err();
    assert!(failure.cause.contains("not JSON"), "{}", failure.cause);
    let failure = parse_create_input("{\"session_id\":\"s1\"}").unwrap_err();
    assert!(failure.cause.contains("`name`"), "{}", failure.cause);
    let failure = parse_create_input("{\"name\":1,\"session_id\":\"s1\"}").unwrap_err();
    assert!(failure.cause.contains("`name`"), "{}", failure.cause);
    let failure = parse_remove_input("{}").unwrap_err();
    assert!(
        failure.cause.contains("`worktree_path`"),
        "{}",
        failure.cause
    );

    let input =
        parse_create_input("{\"name\":\"a\",\"session_id\":\"s1\",\"cwd\":\"/x\"}").unwrap();
    assert_eq!(input, create_input("a", "s1"));
    // The remove payload carries no `name`, and a missing session id is not
    // a refusal: the remove hook classifies by path (D8).
    let input = parse_remove_input("{\"worktree_path\":\"/x/y\"}").unwrap();
    assert_eq!(
        input,
        RemoveInput {
            worktree_path: "/x/y".to_owned(),
            session_id: String::new(),
        }
    );
}

/// D2: the name is validated against GWZ's refused names and characters
/// outside `[A-Za-z0-9._-]`; D6: the session id against R20's grammar.
#[test]
fn the_name_and_the_session_token_are_validated_before_anything_is_read() {
    for name in ["fix-123", "a.b_c", "A1"] {
        assert!(validate_lane_name(name).is_ok(), "{name}");
    }
    for name in ["root", "origin", "HEAD", "FETCH_HEAD", ".", ".."] {
        let failure = validate_lane_name(name).unwrap_err();
        assert!(
            failure.cause.contains("reserves"),
            "{name}: {}",
            failure.cause
        );
    }
    for name in ["a/b", "a:b", "a b", "a*", ""] {
        assert!(validate_lane_name(name).is_err(), "{name}");
    }

    let fixture = Fixture::new("hook-token");
    let context = context(&fixture.root());
    let failure = run_worktree_create(&TestEnv::new(), &context, &create_input("ok", "bad token"))
        .unwrap_err();
    assert!(
        failure.cause.contains("`session_id` is not an owner token"),
        "{}",
        failure.cause
    );
    assert_eq!(failure.class, Classification::RefusedByHook);
}

// ---------------------------------------------------------------------------
// D6: the estimate and the two guards
// ---------------------------------------------------------------------------

/// The estimate takes its share from the table on a destination whose probe
/// clones, and zero on one whose probe fails or lies on another filesystem
/// (D6, amendment A1).
#[test]
fn the_cost_estimate_reads_the_share_table() {
    let walk = SourceWalk {
        apparent_bytes: 100_000_000_000,
        files: 1_000,
    };
    let sharing = estimate_from(
        walk,
        ShareProbe {
            cloned: true,
            same_filesystem: true,
            fs_type: Some("apfs".to_owned()),
        },
    );
    assert!((sharing.share - 0.94).abs() < 1e-9);

    let unknown = estimate_from(
        walk,
        ShareProbe {
            cloned: true,
            same_filesystem: true,
            fs_type: None,
        },
    );
    assert!((unknown.share - 0.70).abs() < 1e-9);

    for probe in [
        ShareProbe {
            cloned: false,
            same_filesystem: true,
            fs_type: Some("ext4".to_owned()),
        },
        ShareProbe {
            cloned: true,
            same_filesystem: false,
            fs_type: Some("apfs".to_owned()),
        },
    ] {
        let none = estimate_from(walk, probe);
        assert_eq!(none.share, 0.0);
        // Share zero means the guard fires on apparent size.
        assert!(none.cost_bytes > walk.apparent_bytes);
    }
    assert!(sharing.cost_bytes < unknown.cost_bytes);
    // A clone's wall time is bound by file count, not bytes.
    assert!(
        sharing.copy_time
            < estimate_from(
                SourceWalk {
                    apparent_bytes: 0,
                    files: 10_000
                },
                ShareProbe::default()
            )
            .copy_time
    );
}

/// The free-space guard fires against the estimate, and `--min-free-gb` is a
/// floor applied on top of it (D6).
#[test]
fn the_free_space_guard_and_the_min_free_floor() {
    let walk = SourceWalk {
        apparent_bytes: 10_000_000_000,
        files: 100,
    };
    let estimate = estimate_from(
        walk,
        ShareProbe {
            cloned: false,
            same_filesystem: true,
            fs_type: Some("ext4".to_owned()),
        },
    );
    let mut env = TestEnv::new();
    env.free = Some(1_000_000_000);
    let failure =
        check_free_space(&env, &estimate, Path::new("/tmp"), None).expect_err("must refuse");
    assert!(failure.cause.contains("free at"), "{}", failure.cause);
    assert!(
        failure.remedy.contains("gwz local list"),
        "{}",
        failure.remedy
    );

    let mut roomy = TestEnv::new();
    roomy.free = Some(20_000_000_000);
    assert!(check_free_space(&roomy, &estimate, Path::new("/tmp"), None).is_ok());
    // The floor applies on top of the estimate, never in place of it.
    assert!(check_free_space(&roomy, &estimate, Path::new("/tmp"), Some(100.0)).is_err());
    assert!(check_free_space(&roomy, &estimate, Path::new("/tmp"), Some(1.0)).is_ok());

    // A platform that will not say does not invent a refusal.
    let mut silent = TestEnv::new();
    silent.free = None;
    assert!(check_free_space(&silent, &estimate, Path::new("/tmp"), Some(100.0)).is_ok());
}

/// The ceiling guard counts the family's ready rows (D6).
#[test]
fn the_lane_ceiling_guard_counts_ready_rows() {
    assert!(check_lane_ceiling(7, 8).is_ok());
    let failure = check_lane_ceiling(8, 8).expect_err("must refuse");
    assert!(failure.cause.contains("ceiling is 8"), "{}", failure.cause);
    assert_eq!(failure.class, Classification::RefusedByHook);
}

/// The guards apply only to an attempt that will create: a reuse consumes
/// nothing and is never refused by one (D6). The table is evaluated first,
/// which is what makes that true.
#[test]
fn the_reuse_rule_is_evaluated_before_the_guards() {
    let fixture = Fixture::new("hook-order");
    let root = fixture.root();
    let destination = fixture.lane("a");
    let read = family(&root, vec![ready_row("a", "../ws-a")]);
    // Today every row reads back no owner, so the same-session reuse is
    // refused fail-closed rather than admitted (R20).
    assert!(matches!(
        decide(&read, "a", &destination, "s1"),
        Decision::Refuse(_)
    ));
}

/// Every refusing row of D6's table, and the one row that creates.
#[test]
fn the_reuse_table_refuses_every_state_but_a_fresh_name() {
    let fixture = Fixture::new("hook-table");
    let root = fixture.root();
    let destination = fixture.lane("a");

    // No row, nothing at the destination: attempt the create.
    assert_eq!(
        decide(&family(&root, Vec::new()), "a", &destination, "s1"),
        Decision::Create
    );

    // No row, a directory at the destination.
    std::fs::create_dir_all(&destination).unwrap();
    let Decision::Refuse(failure) = decide(&family(&root, Vec::new()), "a", &destination, "s1")
    else {
        panic!("an unrelated directory at the destination must refuse");
    };
    assert!(failure.cause.contains("has no row"), "{}", failure.cause);
    std::fs::remove_dir_all(&destination).unwrap();

    let mut creating = ready_row("a", "../ws-a");
    creating.recorded_state = gwz_core::LocalMemberState::Creating;
    creating.observed_state = gwz_core::LocalObservedState::Incomplete;
    let Decision::Refuse(failure) = decide(&family(&root, vec![creating]), "a", &destination, "s1")
    else {
        panic!("a creating row of no session of ours must refuse");
    };
    assert!(
        failure.cause.contains("unfinished create"),
        "{}",
        failure.cause
    );

    let mut disposing = ready_row("a", "../ws-a");
    disposing.recorded_state = gwz_core::LocalMemberState::Disposing;
    let Decision::Refuse(failure) =
        decide(&family(&root, vec![disposing]), "a", &destination, "s1")
    else {
        panic!("a disposing row must refuse");
    };
    assert!(failure.cause.contains("disposal"), "{}", failure.cause);

    // A ready row whose path points elsewhere.
    let Decision::Refuse(failure) = decide(
        &family(&root, vec![ready_row("a", "../elsewhere-a")]),
        "a",
        &destination,
        "s1",
    ) else {
        panic!("a row at another path must refuse");
    };
    assert!(
        failure.cause.contains("already a lane at"),
        "{}",
        failure.cause
    );

    // A ready row at the destination with no owner: fail-closed.
    let Decision::Refuse(failure) = decide(
        &family(&root, vec![ready_row("a", "../ws-a")]),
        "a",
        &destination,
        "s1",
    ) else {
        panic!("a row with no owner must refuse");
    };
    assert!(
        failure.cause.contains("no owning session"),
        "{}",
        failure.cause
    );
    assert_eq!(failure.class, Classification::RefusedByHook);
    assert!(!failure.remedy.is_empty());
}

/// The rows that need an owner token to exercise: a reuse by this session, a
/// refusal naming another session, an incomplete lane of this session, and a
/// `creating` row of this session that makes the second handler wait.
#[test]
fn the_owner_keyed_rows_of_the_reuse_table() {
    let fixture = Fixture::new("hook-owner");
    let root = fixture.root();
    let destination = fixture.lane("a");
    assert_eq!(
        decide(
            &family(&root, vec![owned_row("a", "../ws-a", "s1")]),
            "a",
            &destination,
            "s1"
        ),
        Decision::Reuse,
        "a ready row at the destination owned by this session reuses"
    );
    assert!(
        matches!(
            decide(
                &family(&root, vec![owned_row("a", "../ws-a", "s2")]),
                "a",
                &destination,
                "s1"
            ),
            Decision::Refuse(_)
        ),
        "a ready row at the destination owned by another session refuses"
    );
    let mut creating = owned_row("a", "../ws-a", "s1");
    creating.recorded_state = gwz_core::LocalMemberState::Creating;
    creating.observed_state = gwz_core::LocalObservedState::Incomplete;
    assert_eq!(
        decide(&family(&root, vec![creating]), "a", &destination, "s1"),
        Decision::Wait,
        "a creating row of this session means another handler is mid-copy: wait"
    );
}

// ---------------------------------------------------------------------------
// the workspace branch
// ---------------------------------------------------------------------------

/// A workspace: the lane path is printed, canonical, and the lane is a
/// workspace of its own. The root's managed exclude block is asserted before
/// and after, and `git status --porcelain` is unchanged (D10).
#[test]
fn a_workspace_creation_prints_the_canonical_lane_path() {
    let fixture = Fixture::new("hook-create");
    let root = fixture.root();
    let before_exclude = exclude_block(&root);
    let before_status = git_status(&root);
    let env = TestEnv::new().with_home(fixture.container.path());
    let context = context(&root);

    let success = run_worktree_create(&env, &context, &create_input("lane1", "s1"))
        .expect("a lane is created");
    assert_eq!(success.class, Classification::Lane);
    assert_eq!(success.outcome, "created");
    assert_eq!(success.path, canonical(&fixture.lane("lane1")));
    assert!(success.path.join("gwz.conf/gwz.yml").is_file());
    assert!(success.path.join(".gwz/family-root").is_file());

    assert_eq!(
        exclude_block(&root),
        before_exclude,
        "the managed exclude block is regenerated idempotently"
    );
    assert_eq!(
        git_status(&root),
        before_status,
        "a creation leaves nothing in the root's git status"
    );
}

/// A session that started inside a member lands in that member's directory
/// inside the lane (D8).
#[test]
fn a_member_rooted_project_dir_resolves_to_the_member_inside_the_lane() {
    let fixture = Fixture::new("hook-member");
    let root = fixture.root();
    let member = root.join("docs");
    std::fs::create_dir_all(&member).unwrap();
    std::fs::write(member.join("note.md"), b"note\n").unwrap();
    let env = TestEnv::new().with_home(fixture.container.path());
    let context = context(&member);

    let success = run_worktree_create(&env, &context, &create_input("lane2", "s1"))
        .expect("a lane is created");
    assert_eq!(success.class, Classification::MemberInLane);
    assert_eq!(success.path, canonical(&fixture.lane("lane2").join("docs")));
    assert!(success.path.join("note.md").is_file());
}

/// D10: a build holding the source's target directory open is one stderr
/// line, and a non-empty `.claude/worktrees/` is another (D2).
#[test]
fn the_creation_warnings_reach_stderr() {
    let fixture = Fixture::new("hook-warn");
    let root = fixture.root();
    std::fs::create_dir_all(root.join(".claude/worktrees/old")).unwrap();
    let mut env = TestEnv::new();
    env.build_busy = true;
    env.home = Some(fixture.container.path().to_path_buf());
    let context = context(&root);

    run_worktree_create(&env, &context, &create_input("lane3", "s1")).expect("a lane is created");
    assert!(env.warned("target directory open"), "{:?}", env.warnings());
    assert!(env.warned(".claude/worktrees"), "{:?}", env.warnings());
}

/// The guards refuse before anything is copied, and the refusal names the
/// retirement procedure (D6).
#[test]
fn a_guard_refusal_creates_nothing() {
    let fixture = Fixture::new("hook-guard");
    let root = fixture.root();
    let mut env = TestEnv::new();
    env.free = Some(0);
    env.home = Some(fixture.container.path().to_path_buf());
    let context = context(&root);

    let failure = run_worktree_create(&env, &context, &create_input("lane4", "s1"))
        .expect_err("the guard refuses");
    assert_eq!(failure.class, Classification::RefusedByHook);
    assert!(
        failure.remedy.contains("gwz local list"),
        "{}",
        failure.remedy
    );
    assert!(!fixture.lane("lane4").exists(), "nothing was copied");

    let ceiling = HookOptions {
        max_lanes: 0,
        ..HookOptions::default()
    };
    let context = HookContext {
        project_dir: root.clone(),
        options: ceiling,
    };
    let roomy = TestEnv::new().with_home(fixture.container.path());
    let failure = run_worktree_create(&roomy, &context, &create_input("lane5", "s1"))
        .expect_err("the ceiling refuses");
    assert!(failure.cause.contains("ceiling"), "{}", failure.cause);
    assert!(!fixture.lane("lane5").exists(), "nothing was copied");
}

/// The attempt loop gives up at its deadline and says so as "family busy",
/// never as a hazard (D3, D6).
#[test]
fn the_attempt_loop_refuses_at_its_deadline_behind_a_held_lock() {
    let fixture = Fixture::new("hook-busy");
    let root = fixture.root();
    // Found the family, so the lock file exists to be held.
    let env = TestEnv::new().with_home(fixture.container.path());
    run_worktree_create(&env, &context(&root), &create_input("lane6", "s1"))
        .expect("the first lane founds the family");

    let lock = std::fs::File::open(root.join(".gwz/local-family.lock")).expect("the lock exists");
    lock.lock().expect("the stub takes the lock");

    let options = HookOptions {
        wait_secs: 2,
        ..HookOptions::default()
    };
    let context = HookContext {
        project_dir: root.clone(),
        options,
    };
    let failure = run_worktree_create(&env, &context, &create_input("lane7", "s1"))
        .expect_err("a held lock outlives the deadline");
    assert_eq!(failure.class, Classification::FamilyBusy);
    assert_eq!(failure.remedy, "family busy; retry");
    assert!(failure.cause.contains("waited 2s"), "{}", failure.cause);
    assert!(!fixture.lane("lane7").exists(), "nothing was created");
    let _ = lock.unlock();
}

// ---------------------------------------------------------------------------
// removal
// ---------------------------------------------------------------------------

/// A lane gwz refuses to dispose keeps the lane and the session, and the
/// refusal is classified as a hazard, not as the hook's own (D3, D8, D10).
#[test]
fn a_dispose_refusal_is_classified_as_a_hazard_and_keeps_the_lane() {
    let fixture = Fixture::new("hook-remove");
    let root = fixture.root();
    let env = TestEnv::new().with_home(fixture.container.path());
    let created = run_worktree_create(&env, &context(&root), &create_input("lane8", "s1"))
        .expect("a lane is created");

    let outcome = run_worktree_remove(
        &env,
        &context(&root),
        &RemoveInput {
            worktree_path: created.path.to_string_lossy().into_owned(),
            session_id: "s1".to_owned(),
        },
    );
    match outcome {
        Ok(success) => {
            // A lane that disposes in one command is the S3.5 world; until
            // then every verbatim lane refuses (section 1).
            assert_eq!(success.outcome, "disposed");
            assert!(!created.path.exists());
        }
        Err(failure) => {
            assert_eq!(failure.class, Classification::RefusedByHazard);
            assert!(
                failure.remedy.contains("gwz merge --remote lane8"),
                "{}",
                failure.remedy
            );
            // F3: one line, not the dispose report. The full report stays
            // `gwz local dispose`'s, which the remedy names.
            let line = failure.line();
            assert!(line.len() < 300, "{} bytes: {line}", line.len());
            assert_eq!(line.lines().count(), 1, "{line}");
            assert!(created.path.exists(), "the lane is kept");
        }
    }
}

/// F3 (the probe of 2026-09-18, §6): a dispose hazard report of any size
/// becomes one short line naming how many hazards there are, across how many
/// repositories, and of which class.
#[test]
fn a_hazard_report_is_summarised_to_one_line() {
    let report = "local dispose `probe` at /tmp/ws-probe: unwaived hazard(s): \
        `@root` <dirty>: ignored user data (ignored does not mean disposable) \
        (.claude/.cc-writes/), text content (.claude/settings.local.json), \
        ignored user data (.cursor/), and 3 more; \
        `mem_gwz_cli` <dirty>: ignored user data (__pycache__/); \
        `mem_gwz_core` <unpreserved-history>: reflog-only commit; \
        name each accepted loss with --force <hazard,...> to delete, or --keep to \
        detach and retain every file; nothing was removed";
    let summary = crate::hook::remove::summarise(report);
    assert_eq!(
        summary,
        "8 hazards across 3 repositories (dirty, unpreserved-history)"
    );

    // One repository, one hazard, singular.
    let single = "local dispose `a` at /tmp/ws-a: unwaived hazard(s): `@root` <dirty>: \
        untracked file (note.txt); name each accepted loss with --force";
    assert_eq!(
        crate::hook::remove::summarise(single),
        "1 hazard across 1 repository (dirty)"
    );

    // A refusal of another shape is cut to one readable line, never guessed
    // at and never passed through at length.
    let other = format!(
        "removal stopped (permission denied); remaining: {}",
        "x/".repeat(400)
    );
    let summary = crate::hook::remove::summarise(&other);
    assert!(summary.len() < 200, "{} bytes: {summary}", summary.len());
    assert!(summary.starts_with("removal stopped"), "{summary}");
}

/// A `worktree_path` that no longer exists exits zero and is logged; one
/// that is neither a registered worktree nor a lane is refused; and a
/// basename that matches a family name decides nothing (D8).
#[test]
fn removal_classifies_by_canonical_path_and_not_by_name() {
    let fixture = Fixture::new("hook-classify");
    let root = fixture.root();
    let env = TestEnv::new().with_home(fixture.container.path());
    run_worktree_create(&env, &context(&root), &create_input("lane9", "s1"))
        .expect("a lane is created");

    let absent = run_worktree_remove(
        &env,
        &context(&root),
        &RemoveInput {
            worktree_path: fixture
                .container
                .path()
                .join("gone")
                .to_string_lossy()
                .into_owned(),
            session_id: "s1".to_owned(),
        },
    )
    .expect("an absent path exits zero");
    assert_eq!(absent.outcome, "absent");

    // A directory whose basename matches the family name but whose canonical
    // path does not.
    let impostor = fixture.container.path().join("decoy").join("ws-lane9");
    std::fs::create_dir_all(&impostor).unwrap();
    let failure = run_worktree_remove(
        &env,
        &context(&root),
        &RemoveInput {
            worktree_path: impostor.to_string_lossy().into_owned(),
            session_id: "s1".to_owned(),
        },
    )
    .expect_err("a decoy path must refuse");
    assert_eq!(failure.class, Classification::RefusedByHook);
    assert!(fixture.lane("lane9").exists(), "the real lane is untouched");
}

/// The removal runs from the family root, never from inside the lane, so a
/// process working directory inside the lane still disposes (D8).
#[test]
#[ignore = "needs GwzLaneCleanFixes R0: a verbatim lane cannot dispose without a waiver yet"]
fn a_removal_from_inside_the_lane_still_disposes() {
    // Until an integrated lane disposes in one command (GwzLaneCleanFixes
    // R0) every verbatim lane refuses, so the positive half of this row
    // cannot be asserted; the hook's own `--root` and working directory are
    // covered by `dispose_lane`.
}

// ---------------------------------------------------------------------------
// D8's fallback
// ---------------------------------------------------------------------------

/// A plain Git repository gets Claude Code's own default, and the
/// `.worktreeinclude` copy runs only for a worktree this invocation created.
#[test]
fn the_fallback_creates_reuses_and_removes_a_plain_worktree() {
    let temp = plain_repository("hook-fallback");
    let root = temp.path().to_path_buf();
    std::fs::write(root.join(".worktreeinclude"), b"secrets.env\n").unwrap();
    std::fs::write(root.join("secrets.env"), b"TOKEN=1\n").unwrap();
    let env = TestEnv::new().with_home(temp.path());
    let context = context(&root);

    let created = run_worktree_create(&env, &context, &create_input("probe", "s1"))
        .expect("a worktree is created");
    assert_eq!(created.class, Classification::FallbackWorktree);
    assert_eq!(
        created.path,
        canonical(&root.join(".claude/worktrees/probe"))
    );
    assert_eq!(
        run_git(&created.path, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "worktree-probe"
    );
    assert!(
        created.path.join("secrets.env").is_file(),
        "the .worktreeinclude entry is copied into a created worktree"
    );

    // Reuse by a second session prints the same path and writes nothing: an
    // included file modified in the worktree survives byte-identical.
    std::fs::write(created.path.join("secrets.env"), b"TOKEN=edited\n").unwrap();
    let reused = run_worktree_create(&env, &context, &create_input("probe", "s2"))
        .expect("the worktree is reused");
    assert_eq!(reused.path, created.path);
    assert_eq!(reused.outcome, "reused");
    assert_eq!(
        std::fs::read_to_string(created.path.join("secrets.env")).unwrap(),
        "TOKEN=edited\n"
    );

    // An included file deleted from the worktree is reported, not supplied.
    std::fs::remove_file(created.path.join("secrets.env")).unwrap();
    run_worktree_create(&env, &context, &create_input("probe", "s3")).expect("reused again");
    assert!(
        env.warned("missing from the reused worktree"),
        "{:?}",
        env.warnings()
    );
    assert!(!created.path.join("secrets.env").exists());

    // A clean worktree is removed; a dirty one is refused and kept.
    std::fs::write(created.path.join("README.md"), b"edited\n").unwrap();
    let failure = run_worktree_remove(
        &env,
        &context,
        &RemoveInput {
            worktree_path: created.path.to_string_lossy().into_owned(),
            session_id: "s1".to_owned(),
        },
    )
    .expect_err("a dirty worktree is kept");
    assert_eq!(failure.class, Classification::RefusedByHook);
    assert!(created.path.exists());

    run_git(&created.path, &["checkout", "--", "README.md"]);
    let removed = run_worktree_remove(
        &env,
        &context,
        &RemoveInput {
            worktree_path: created.path.to_string_lossy().into_owned(),
            session_id: "s1".to_owned(),
        },
    )
    .expect("a clean worktree is removed");
    assert_eq!(removed.class, Classification::FallbackWorktree);
    assert!(!created.path.exists());
}

/// F7 (the probe of 2026-09-18, §9): the `.worktreeinclude` enumeration is
/// scoped to the project's own files, so a second worktree never receives the
/// first worktree's copies and a reuse never reports a present file missing.
#[test]
fn the_fallback_never_copies_one_worktrees_includes_into_another() {
    let temp = plain_repository("hook-include-scope");
    let root = temp.path().to_path_buf();
    std::fs::write(root.join(".gitignore"), b"secrets.env\n").unwrap();
    std::fs::write(root.join(".worktreeinclude"), b"secrets.env\n").unwrap();
    std::fs::write(root.join("secrets.env"), b"TOKEN=abc\n").unwrap();
    let env = TestEnv::new().with_home(temp.path());
    let context = context(&root);

    let first = run_worktree_create(&env, &context, &create_input("one", "s1"))
        .expect("the first worktree is created");
    assert!(first.path.join("secrets.env").is_file());

    // The second creation walks a project root that now holds the first
    // worktree, including its copy of the included file.
    let second = run_worktree_create(&env, &context, &create_input("two", "s1"))
        .expect("the second worktree is created");
    assert!(
        second.path.join("secrets.env").is_file(),
        "the project's own included file is copied"
    );
    assert!(
        !second.path.join(".claude").exists(),
        "no other worktree's copy travels into this one: {:?}",
        std::fs::read_dir(&second.path)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name())
            .collect::<Vec<_>>()
    );
    let copies = std::process::Command::new("find")
        .arg(&second.path)
        .args(["-name", "secrets.env"])
        .output()
        .expect("find runs");
    assert_eq!(
        String::from_utf8_lossy(&copies.stdout).lines().count(),
        1,
        "exactly one copy, at the top level: {}",
        String::from_utf8_lossy(&copies.stdout)
    );

    // Reuse of the first worktree, whose included file is present, warns
    // about nothing.
    let reused = run_worktree_create(&env, &context, &create_input("one", "s1"))
        .expect("the first worktree is reused");
    assert_eq!(reused.outcome, "reused");
    assert!(
        !env.warned("missing from the reused worktree"),
        "{:?}",
        env.warnings()
    );
}

/// `--base-ref` is honoured, and a locked worktree -- what Claude holds on a
/// running agent's -- is refused rather than forced (D8).
#[test]
fn the_fallback_honours_base_ref_and_refuses_a_locked_worktree() {
    let temp = plain_repository("hook-base");
    let root = temp.path().to_path_buf();
    run_git(&root, &["branch", "other"]);
    std::fs::write(root.join("second.txt"), b"second\n").unwrap();
    run_git(&root, &["add", "second.txt"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=GWZ Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-q",
            "-m",
            "second",
        ],
    );
    let env = TestEnv::new().with_home(temp.path());
    let options = HookOptions {
        base_ref: Some("other".to_owned()),
        ..HookOptions::default()
    };
    let context = HookContext {
        project_dir: root.clone(),
        options,
    };

    let created = run_worktree_create(&env, &context, &create_input("based", "s1"))
        .expect("a worktree is created");
    assert!(
        !created.path.join("second.txt").exists(),
        "the worktree was branched from `other`, not from HEAD"
    );

    run_git(
        &root,
        &["worktree", "lock", &created.path.to_string_lossy()],
    );
    let failure = run_worktree_remove(
        &env,
        &context,
        &RemoveInput {
            worktree_path: created.path.to_string_lossy().into_owned(),
            session_id: "s1".to_owned(),
        },
    )
    .expect_err("a locked worktree is kept");
    assert_eq!(failure.class, Classification::RefusedByHook);
    assert!(created.path.exists());
    run_git(
        &root,
        &["worktree", "unlock", &created.path.to_string_lossy()],
    );
}

// ---------------------------------------------------------------------------
// D10: the log
// ---------------------------------------------------------------------------

/// The log line's exact field set, the three refusal classes, the 1 MB bound
/// and the ignore check.
#[test]
fn the_log_line_carries_exactly_the_documented_fields() {
    let temp = TempDir::new("hook-log");
    let env = TestEnv::new().with_home(temp.path());
    let location = crate::hook::logging::LogLocation {
        path: Some(temp.path().join("hooks.log")),
        note: None,
    };
    write_log(
        &env,
        &location,
        &LogRecord {
            event: "worktree-create",
            name: "lane".to_owned(),
            session_id: "s1".to_owned(),
            class: Classification::Lane,
            path: "/tmp/ws-lane".to_owned(),
            outcome: "created".to_owned(),
            exit: 0,
            message: None,
        },
    );
    let text = std::fs::read_to_string(temp.path().join("hooks.log")).unwrap();
    let line = text.lines().next().unwrap();
    let keys: Vec<&str> = line
        .split_whitespace()
        .filter_map(|field| field.split('=').next())
        .collect();
    assert_eq!(
        keys,
        vec![
            "ts", "event", "name", "session", "class", "path", "outcome", "exit"
        ]
    );
    assert!(!line.contains("transcript"), "{line}");
    assert!(!line.contains("cwd"), "{line}");

    // The three refusal classes D8 names are distinguished.
    for (failure, word) in [
        (
            crate::hook::HookFailure::refused("cause", "remedy"),
            "refused-by-hook",
        ),
        (
            crate::hook::HookFailure::hazard("cause", "remedy"),
            "refused-by-hazard",
        ),
        (crate::hook::HookFailure::busy("cause"), "family-busy"),
    ] {
        write_log(
            &env,
            &location,
            &LogRecord::refusal("worktree-remove", "lane", "s1", &failure),
        );
        let text = std::fs::read_to_string(temp.path().join("hooks.log")).unwrap();
        let last = text.lines().next_back().unwrap();
        assert!(last.contains(&format!("class={word}")), "{last}");
        assert!(last.contains("exit=1"), "{last}");
        assert!(last.contains(&failure.line()), "{last}");
    }

    // The bound: past 1 MB the file keeps its newest half.
    let filler = "x".repeat(200);
    for _ in 0..6_000 {
        write_log(
            &env,
            &location,
            &LogRecord {
                event: "worktree-create",
                name: filler.clone(),
                session_id: "s1".to_owned(),
                class: Classification::Lane,
                path: "/tmp/ws-lane".to_owned(),
                outcome: "created".to_owned(),
                exit: 0,
                message: None,
            },
        );
    }
    let size = std::fs::metadata(temp.path().join("hooks.log"))
        .unwrap()
        .len();
    assert!(size <= crate::hook::logging::LOG_LIMIT_BYTES, "{size}");
}

/// A log location that would appear in `git status` is not used: the
/// user-level log is used instead and a note says so (D10).
#[test]
fn a_log_location_that_would_show_in_git_status_is_redirected() {
    let temp = plain_repository("hook-log-ignored");
    let root = temp.path().to_path_buf();
    let home = TempDir::new("hook-log-home");
    let env = TestEnv::new().with_home(home.path());

    let visible = root.join("hooks.log");
    let location = resolve_log_location(&env, Some(&visible.to_string_lossy()), Some(&root));
    assert_eq!(
        location.path.as_deref(),
        Some(home.path().join(".claude/gwz-lane-hooks.log").as_path())
    );
    assert!(location.note.is_some());

    // An ignored location is used as it is.
    std::fs::write(root.join(".git/info/exclude"), b"/.gwz/\n").unwrap();
    let ignored = root.join(".gwz/claude-hooks.log");
    assert!(path_is_ignored(&ignored));
    let location = resolve_log_location(&env, None, Some(&root));
    assert_eq!(location.path.as_deref(), Some(ignored.as_path()));
    assert!(location.note.is_none());
}
