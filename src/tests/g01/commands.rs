use super::*;

#[test]
pub(crate) fn parses_command_matrix() {
    assert!(matches!(
        parse(strings(["repo", "add", "repos/app"])).request,
        CliRequest::AddExistingRepo(_)
    ));
    assert!(matches!(
        parse(strings(["add", "src/foo.rs"])).request,
        CliRequest::Stage(_)
    ));
    assert!(matches!(
        parse(strings(["tag"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::List)
    ));
    assert!(matches!(
        parse(strings(["tag", "--list"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::List)
    ));
    assert!(matches!(
        parse(strings(["tag", "v1"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Create)
    ));
    assert!(matches!(
        parse(strings(["tag", "--delete", "v1"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Delete)
    ));
    assert!(matches!(
        parse(strings(["tag", "--push"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Push)
    ));
    assert!(matches!(
        parse(strings(["tag", "--fetch"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Fetch)
    ));
    assert!(matches!(
        parse(strings(["tag", "--list", "--remote", "origin"])).request,
        CliRequest::Tag(ref r) if r.remote.as_deref() == Some("origin")
    ));
    assert!(matches!(
        parse(strings(["branch"])).request,
        CliRequest::Branch(ref r) if matches!(r.op, gwz_core::BranchOp::List)
    ));
    assert!(matches!(
        parse(strings(["branch", "--list"])).request,
        CliRequest::Branch(ref r) if matches!(r.op, gwz_core::BranchOp::List)
    ));
    assert!(matches!(
        parse(strings(["branch", "--create", "feature/login"])).request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Create)
                && r.name.as_deref() == Some("feature/login")
                && r.start_ref.as_deref() == Some("HEAD")
                && r.switch_after_create.is_none()
    ));
    assert!(matches!(
        parse(strings([
            "branch",
            "--create",
            "feature/login",
            "--from",
            "main",
            "--switch"
        ]))
        .request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Create)
                && r.name.as_deref() == Some("feature/login")
                && r.start_ref.as_deref() == Some("main")
                && r.switch_after_create == Some(true)
    ));
    assert!(matches!(
        parse(strings(["branch", "--delete", "feature/login"])).request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Delete)
                && r.name.as_deref() == Some("feature/login")
    ));
    assert!(matches!(
        parse(strings(["branch", "--merge", "feature/source"])).request,
        CliRequest::Merge(ref r)
            if matches!(r.op, gwz_core::MergeOp::Start)
                && r.source_ref.as_deref() == Some("feature/source")
    ));
    assert!(matches!(
        parse(strings(["stash", "push"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_untracked.is_none()
                && r.include_ignored.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "push", "-u", "-m", "wip"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_untracked == Some(true)
                && r.message.as_deref() == Some("wip")
    ));
    assert!(matches!(
        parse(strings(["stash", "push", "-a"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_ignored == Some(true)
                && r.include_untracked.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "list", "--expanded"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::List)
                && r.expanded == Some(true)
    ));
    assert!(matches!(
        parse(strings(["stash", "apply"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Apply)
                && r.stash_id.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "pop", "stash_one"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Pop)
                && r.stash_id.as_deref() == Some("stash_one")
    ));
    assert!(matches!(
        parse(strings(["stash", "drop", "stash_one"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Drop)
                && r.stash_id.as_deref() == Some("stash_one")
    ));
    assert!(matches!(
        parse(strings(["snapshot"])).request,
        CliRequest::ListSnapshots(_)
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap"])).request,
        CliRequest::Snapshot(ref r) if r.source.is_none()
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap", "--branch"])).request,
        CliRequest::Snapshot(ref r)
            if matches!(
                r.source.as_ref(),
                Some(gwz_core::SnapshotSource {
                    kind: gwz_core::SnapshotSourceKind::Current,
                    branch: None,
                })
            )
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap", "--branch", "main"])).request,
        CliRequest::Snapshot(ref r)
            if matches!(
                r.source.as_ref(),
                Some(gwz_core::SnapshotSource {
                    kind: gwz_core::SnapshotSourceKind::Branch,
                    branch: Some(branch),
                }) if branch == "main"
            )
    ));
    assert!(matches!(
        parse(strings(["ls"])).request,
        CliRequest::Ls { local: false, ref request } if request.include_unmaterialized.is_none()
    ));
    assert!(matches!(
        parse(strings(["ls", "--local"])).request,
        CliRequest::Ls { local: true, .. }
    ));
    assert!(matches!(
        parse(strings(["ls", "--unmaterialized"])).request,
        CliRequest::Ls { ref request, .. } if request.include_unmaterialized == Some(true)
    ));
    assert!(matches!(
        parse(strings(["forall", "--", "git", "status"])).request,
        CliRequest::Forall { mode: gwz_core::ExecMode::Argv, ref command, .. } if command.len() == 2
    ));
    assert!(matches!(
        parse(strings(["forall", "-c", "git status"])).request,
        CliRequest::Forall {
            mode: gwz_core::ExecMode::Shell,
            ..
        }
    ));
    assert!(matches!(
        parse(strings(["forall", "app", "lib", "--", "git"])).request,
        CliRequest::Forall { ref projects, .. } if projects.len() == 2
    ));
    assert!(matches!(
        parse(strings(["repo", "create", "repos/app"])).request,
        CliRequest::CreateRepo(_)
    ));
    assert!(matches!(
        parse(strings(["repo", "sync"])).request,
        CliRequest::RepoSync(_)
    ));
    assert!(matches!(
        parse(strings(["repo", "sync", "repos/app"])).request,
        CliRequest::RepoSync(ref request)
            if request.meta.selection.as_ref().unwrap().targets == vec!["repos/app"]
    ));
    assert!(matches!(
        parse(strings(["materialize", "--lock"])).request,
        CliRequest::Materialize(_)
    ));
    assert!(matches!(
        parse(strings(["materialize", "--snapshot", "snap_one"])).request,
        CliRequest::Materialize(_)
    ));
    assert!(matches!(
        parse(strings(["materialize", "--switch", "feature/login"])).request,
        CliRequest::Materialize(ref r)
            if r.target.kind == gwz_core::MaterializeTargetKind::Branch
                && r.target.name.as_deref() == Some("feature/login")
                && r.target.commit.is_none()
    ));
    assert!(matches!(
        parse(strings(["pull", "--head"])).request,
        CliRequest::PullHead(_)
    ));
    assert!(matches!(
        parse(strings(["pull", "--snapshot", "snap_one"])).request,
        CliRequest::PullSnapshot(_)
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap_one"])).request,
        CliRequest::Snapshot(_)
    ));
    assert!(matches!(
        parse(strings(["tag", "release_one"])).request,
        CliRequest::Tag(_)
    ));
    assert!(matches!(
        parse(strings(["push"])).request,
        CliRequest::Push(_)
    ));
}

#[test]
pub(crate) fn parses_first_class_merge_and_reserved_forms() {
    let invocation = parse(strings([
        "merge",
        "feature/source",
        "--dry-run",
        "--target",
        "mem_app",
    ]));
    let CliRequest::Merge(request) = invocation.request else {
        panic!("expected merge request");
    };
    assert_eq!(request.op, gwz_core::MergeOp::Start);
    assert_eq!(request.source_ref.as_deref(), Some("feature/source"));
    assert_eq!(request.meta.dry_run, Some(true));
    assert_eq!(request.meta.policy.unwrap().progress_min_interval_ms, None);

    assert!(matches!(
        parse(strings(["merge", "--continue"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Resume
                && r.source_ref.is_none()
                && r.merge_id.is_none()
    ));
    assert!(matches!(
        parse(strings(["merge", "--abort"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Abort
                && r.source_ref.is_none()
                && r.merge_id.is_none()
    ));
    assert!(matches!(
        parse(strings(["merge", "--status"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Status
                && r.source_ref.is_none()
                && r.merge_id.is_none()
    ));
    assert!(matches!(
        parse(strings(["merge", "--status", "merge_closed"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Status
                && r.merge_id.as_deref() == Some("merge_closed")
    ));
    assert!(matches!(
        parse(strings(["merge", "--abort", "--preserve"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Abort && r.preserve == Some(true)
    ));
    assert!(matches!(
        parse(strings(["merge", "--gc", "merge_closed"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Gc
                && r.merge_id.as_deref() == Some("merge_closed")
    ));
    assert!(matches!(
        parse(strings(["merge", "feature/source", "--ff-only"])).request,
        CliRequest::Merge(ref r) if r.mode == Some(gwz_core::MergeMode::FfOnly)
    ));
    assert!(matches!(
        parse(strings(["merge", "feature/source", "-m", "coordinated change"])).request,
        CliRequest::Merge(ref r) if r.message.as_deref() == Some("coordinated change")
    ));
    assert!(matches!(
        parse(strings(["merge", "feature/source", "--partial"])).request,
        CliRequest::Merge(ref r)
            if r.meta.policy.as_ref().and_then(|p| p.partial)
                == Some(gwz_core::PartialBehavior::Partial)
    ));
    assert!(matches!(
        parse(strings(["merge", "feature/source", "--preserve"])).request,
        CliRequest::Merge(ref r) if r.preserve == Some(true)
    ));

    for args in [
        strings(["merge", "--continue", "--abort"]),
        strings(["merge", "feature/source", "--ff-only", "--no-ff"]),
    ] {
        assert_eq!(
            parse_result(args).unwrap_err().code,
            Some(gwz_core::model::ErrorCode::InvalidRequest)
        );
    }
}

#[test]
pub(crate) fn merge_filesystem_strict_is_a_start_only_request_flag() {
    // DR-1: the crash-recovery decision belongs to the start that opens the
    // attempt. Absent, the request carries no opinion and core warns-and-
    // continues below the bar; present, core refuses before any lease.
    assert!(matches!(
        parse(strings(["merge", "feature/source"])).request,
        CliRequest::Merge(ref r) if r.filesystem_strict.is_none()
    ));
    assert!(matches!(
        parse(strings(["merge", "feature/source", "--filesystem-strict"])).request,
        CliRequest::Merge(ref r)
            if r.op == gwz_core::MergeOp::Start && r.filesystem_strict == Some(true)
    ));
    assert!(matches!(
        parse(strings([
            "merge",
            "feature/source",
            "--no-ff",
            "--filesystem-strict"
        ]))
        .request,
        CliRequest::Merge(ref r)
            if r.mode == Some(gwz_core::MergeMode::NoFf) && r.filesystem_strict == Some(true)
    ));

    // Every lifecycle op uses what its own start opened and never consults the
    // flag, so offering it there is a request error, not a silent no-op.
    for args in [
        strings(["merge", "--continue", "--filesystem-strict"]),
        strings(["merge", "--abort", "--filesystem-strict"]),
        strings(["merge", "--status", "--filesystem-strict"]),
        strings(["merge", "--gc", "--filesystem-strict"]),
    ] {
        let error = parse_result(args).unwrap_err();
        assert_eq!(
            error.code,
            Some(gwz_core::model::ErrorCode::InvalidRequest),
            "{error:?}"
        );
        assert!(
            error
                .message
                .contains("--filesystem-strict is accepted only when starting a merge"),
            "{error:?}"
        );
    }
}

#[test]
pub(crate) fn merge_help_exposes_status_and_recovery_flags() {
    use clap::CommandFactory;

    let mut command = Cli::command();
    let help = command
        .find_subcommand_mut("merge")
        .unwrap()
        .render_long_help()
        .to_string();
    assert!(help.contains("--status"), "{help}");
    assert!(help.contains("--continue"), "{help}");
    assert!(help.contains("--abort"), "{help}");
    assert!(help.contains("--preserve"), "{help}");
    assert!(help.contains("--gc"), "{help}");
    assert!(help.contains("--ff-only"), "{help}");
    // A1 unhid `--no-ff`: the flag was `hide = true` while the v1 record
    // lifecycle was a compile-gated boundary, and the activation made it a
    // public surface. Its mutual exclusion with `--ff-only` is unchanged.
    assert!(help.contains("--no-ff"), "{help}");
    assert!(help.contains("Always create a merge commit"), "{help}");
    assert!(help.contains("--message"), "{help}");
    assert!(help.contains("custom merge commit-message body"), "{help}");
}

#[test]
pub(crate) fn rejects_invalid_command_combinations_before_core_execution() {
    assert!(parse_result(strings(["--json", "--jsonl", "status"])).is_err());
    assert!(parse_result(strings(["--all", "--member", "mem_app", "status"])).is_ok());
    assert!(parse_result(strings(["--path", "repos/lib", "status"])).is_err());
    assert!(parse_result(strings(["status", "--no-files", "--no-branches"])).is_err());
    assert!(parse_result(strings(["status", "--combined", "--no-combined"])).is_err());
    assert!(parse_result(strings(["status", "--porcelain", "--no-combined"])).is_err());
    assert!(parse_result(strings(["status", "--no-combined", "--no-files"])).is_err());
    assert!(parse_result(strings(["push", "--combined"])).is_err());
    assert!(parse_result(strings(["forall"])).is_err());
    assert!(parse_result(strings(["forall", "-c", "x", "--", "y"])).is_err());
    assert!(parse_result(strings(["--json", "forall", "--", "echo"])).is_err());
    assert!(parse_result(strings(["--jsonl", "forall", "--", "echo"])).is_err());
    assert!(parse_result(strings(["push", "--no-combined"])).is_err());
    assert!(
        parse_result(strings([
            "--member",
            "mem_app",
            "repo",
            "sync",
            "repos/app"
        ]))
        .is_err()
    );
    assert!(parse_result(strings(["materialize", "--snapshot"])).is_err());
    assert!(
        parse_result(strings([
            "materialize",
            "--lock",
            "--switch",
            "feature/login"
        ]))
        .is_err()
    );
    assert!(parse_result(strings(["materialize", "--switch"])).is_err());
    assert!(parse_result(strings(["snapshot", "--branch"])).is_err());
    assert!(parse_result(strings(["snapshot", "--list", "--branch"])).is_err());
    assert!(parse_result(strings(["branch", "--list", "--create", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--create", "work", "--delete", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--create", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--delete", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--list"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--switch"])).is_err());
    assert!(parse_result(strings(["branch", "--delete", "work", "--switch"])).is_err());
    assert!(parse_result(strings(["branch", "--from", "main"])).is_err());
    assert!(parse_result(strings(["stash", "push", "-u", "-a"])).is_err());
    assert!(parse_result(strings(["stash", "drop"])).is_err());
    assert!(parse_result(strings(["pull", "--lock"])).is_err());
    assert!(parse_result(strings(["unknown"])).is_err());
}

#[test]
pub(crate) fn can_call_core_status_in_process() {
    let temp = TempDir::new("cli-status");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    let invocation = parse_args_with_request_id(
        strings([
            "--root",
            temp.path().to_str().unwrap(),
            "status",
            "--no-combined",
        ]),
        "req_status",
        temp.path(),
    )
    .unwrap();

    let response = execute_invocation(&invocation).unwrap();

    assert_eq!(
        response.envelope.meta.aggregate_status,
        gwz_core::AggregateStatus::Ok
    );
    assert!(response.envelope.members.is_empty());
}

pub(crate) fn parse(args: Vec<String>) -> CliInvocation {
    parse_result(args).unwrap()
}

pub(crate) fn parse_result(args: Vec<String>) -> Result<CliInvocation, CliError> {
    parse_args_with_request_id(args, "req_test", Path::new("/cwd"))
}

pub(crate) fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

pub(crate) fn request_meta(request_id: &str) -> gwz_core::RequestMeta {
    gwz_core::RequestMeta {
        request_id: request_id.to_owned(),
        schema_version: "gwz.protocol/v0".to_owned(),
        ..Default::default()
    }
}

pub(crate) struct TempDir {
    pub(crate) path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("gwz-cli-{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
