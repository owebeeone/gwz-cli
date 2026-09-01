use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

mod commands;

pub(crate) use commands::*;

#[test]
pub(crate) fn parses_init_workspace_with_root() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/tmp/gwz-test", "init"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Human);
    let CliRequest::CreateWorkspace(request) = invocation.request else {
        panic!("expected create workspace");
    };
    assert_eq!(request.workspace_root, "/tmp/gwz-test");
    assert_eq!(request.meta.request_id, "req_test");
}

#[test]
pub(crate) fn parses_init_update_bootstrap_with_root() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/tmp/gwz-test", "init", "--update"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Human);
    let CliRequest::UpdateBootstrap { meta } = invocation.request else {
        panic!("expected update bootstrap");
    };
    assert_eq!(meta.request_id, "req_test");
    assert_eq!(
        meta.workspace.unwrap().root,
        Some("/tmp/gwz-test".to_owned())
    );
}

#[test]
pub(crate) fn init_update_rejects_sources_and_path_prefix() {
    let with_source = parse_args_with_request_id(
        strings(["init", "--update", "git@github.com:org/repo.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        with_source
            .message
            .contains("--update cannot be combined with source URLs")
    );

    let with_path = parse_args_with_request_id(
        strings(["init", "--update", "--path", "repos"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        with_path
            .message
            .contains("--update cannot be combined with --path")
    );
}

#[test]
pub(crate) fn parses_init_sources_from_plain_urls() {
    let invocation = parse_args_with_request_id(
        strings([
            "init",
            "git@github.com:org/repo-a.git",
            "https://github.com/org/repo-b",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::InitFromSources(request) = invocation.request else {
        panic!("expected init from sources");
    };
    assert_eq!(request.workspace_root, "/cwd");
    assert_eq!(request.sources[0].url, "git@github.com:org/repo-a.git");
    assert_eq!(request.sources[0].path, None);
    assert_eq!(request.sources[1].url, "https://github.com/org/repo-b");
}

#[test]
pub(crate) fn parses_clone_with_explicit_and_derived_target() {
    let with_dir = parse_args_with_request_id(
        strings(["clone", "git@github.com:org/workspace.git", "work/demo"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::CloneWorkspace { url, target, .. } = with_dir.request else {
        panic!("expected clone workspace");
    };
    assert_eq!(url, "git@github.com:org/workspace.git");
    assert_eq!(target, "work/demo");

    let derived = parse_args_with_request_id(
        strings(["clone", "https://github.com/org/workspace.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::CloneWorkspace { target, .. } = derived.request else {
        panic!("expected clone workspace");
    };
    assert_eq!(target, "workspace");
}

#[test]
pub(crate) fn parses_repo_lifecycle_commands_and_identity_overrides() {
    let cloned = parse(strings([
        "--dry-run",
        "repo",
        "clone",
        "git@example.invalid:org/shared.git",
        "libs/shared",
        "--member-id",
        "mem_shared_v2",
        "--source-id",
        "src_shared",
    ]));
    let CliRequest::CloneRepoMember(request) = cloned.request else {
        panic!("expected clone repo member");
    };
    assert_eq!(request.source.url, "git@example.invalid:org/shared.git");
    assert_eq!(request.source.path.as_deref(), Some("libs/shared"));
    assert_eq!(request.member_id.as_deref(), Some("mem_shared_v2"));
    assert_eq!(request.source_id.as_deref(), Some("src_shared"));
    assert_eq!(request.meta.dry_run, Some(true));
    assert_eq!(
        operation_label(&CliRequest::CloneRepoMember(request)),
        "cloning"
    );

    let detached = parse(strings(["repo", "detach", "libs/shared"]));
    let CliRequest::DetachRepoMember(request) = detached.request else {
        panic!("expected detach repo member");
    };
    assert_eq!(request.meta.selection.unwrap().targets, vec!["libs/shared"]);

    let attached = parse(strings(["repo", "attach", "mem_shared"]));
    let CliRequest::AttachRepoMember(request) = attached.request else {
        panic!("expected attach repo member");
    };
    assert_eq!(request.meta.selection.unwrap().targets, vec!["mem_shared"]);

    let added = parse(strings([
        "repo",
        "add",
        "libs/shared",
        "--member-id",
        "mem_shared_v2",
        "--source-id",
        "src_shared",
    ]));
    let CliRequest::AddExistingRepo(request) = added.request else {
        panic!("expected add existing repo");
    };
    assert_eq!(request.member_id.as_deref(), Some("mem_shared_v2"));
    assert_eq!(request.source_id.as_deref(), Some("src_shared"));

    let created = parse(strings([
        "repo",
        "create",
        "libs/shared",
        "--member-id",
        "mem_shared_v2",
        "--source-id",
        "src_shared",
    ]));
    let CliRequest::CreateRepo(request) = created.request else {
        panic!("expected create repo");
    };
    assert_eq!(request.member_id.as_deref(), Some("mem_shared_v2"));
    assert_eq!(request.source_id.as_deref(), Some("src_shared"));
}

#[test]
pub(crate) fn repo_detach_and_attach_reject_global_selection() {
    for args in [
        strings(["--member", "mem_other", "repo", "detach", "mem_shared"]),
        strings(["--no-target", "@root", "repo", "detach", "mem_shared"]),
        strings(["--all", "repo", "attach", "mem_shared"]),
        strings([
            "--member-path",
            "libs/other",
            "repo",
            "attach",
            "mem_shared",
        ]),
    ] {
        let error = parse_result(args).unwrap_err();
        assert!(
            error
                .message
                .contains("cannot be combined with global selection")
        );
    }
}

#[test]
pub(crate) fn repo_attach_rejects_non_member_id_operand() {
    let error = parse_result(strings(["repo", "attach", "libs/shared"])).unwrap_err();
    assert!(error.message.contains("member id"));
}

#[test]
pub(crate) fn clone_rejects_dry_run() {
    let error = parse_args_with_request_id(
        strings(["--dry-run", "clone", "https://github.com/org/workspace.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        error
            .message
            .contains("--dry-run is not supported for clone")
    );
}

#[test]
pub(crate) fn parses_init_path_prefix_for_initial_sources() {
    let invocation = parse_args_with_request_id(
        strings([
            "init",
            "--path",
            "repos",
            "git@github.com:org/repo-a.git",
            "https://github.com/org/repo-b",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::InitFromSources(request) = invocation.request else {
        panic!("expected init from sources");
    };
    assert_eq!(request.sources[0].path, Some("repos/repo-a".to_owned()));
    assert_eq!(request.sources[1].path, Some("repos/repo-b".to_owned()));
}

#[test]
pub(crate) fn parses_global_selection_policy_and_output_flags() {
    let invocation = parse_args_with_request_id(
        strings([
            "--root",
            "/ws",
            "--member",
            "mem_app",
            "--member-path",
            "repos/lib",
            "--dry-run",
            "--partial",
            "--force",
            "--sync",
            "reset",
            "--remote",
            "origin",
            "--jobs",
            "4",
            "--json",
            "status",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Json);
    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    let workspace = request.meta.workspace.unwrap();
    assert_eq!(workspace.root, Some("/ws".to_owned()));
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["mem_app", "repos/lib"]);
    assert!(selection.exclude_targets.is_empty());
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
    let policy = request.meta.policy.unwrap();
    assert_eq!(policy.partial, Some(gwz_core::PartialBehavior::Partial));
    assert_eq!(
        policy.destructive,
        Some(gwz_core::DestructiveBehavior::Allow)
    );
    assert_eq!(policy.sync, Some(gwz_core::SyncBehavior::Reset));
    assert_eq!(policy.remote, Some("origin".to_owned()));
    assert_eq!(policy.concurrency, Some(4));
    assert_eq!(request.meta.dry_run, Some(true));
}

#[test]
pub(crate) fn capture_verb_parses_with_selection() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/ws", "--member", "mem_app", "capture"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Capture(request) = invocation.request else {
        panic!("expected capture");
    };
    assert_eq!(request.meta.workspace.unwrap().root, Some("/ws".to_owned()));
    assert_eq!(request.meta.selection.unwrap().targets, vec!["mem_app"]);
}

#[test]
pub(crate) fn commit_marker_flags_parse_to_tristate() {
    let default = parse_args_with_request_id(
        strings(["commit", "-m", "message"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::Commit(request) = default.request else {
        panic!("expected commit");
    };
    assert_eq!(request.message, "message");
    assert_eq!(request.commit_marker, None);

    let enabled = parse_args_with_request_id(
        strings(["commit", "-m", "message", "--commit-marker"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::Commit(request) = enabled.request else {
        panic!("expected commit");
    };
    assert_eq!(request.commit_marker, Some(true));

    let disabled = parse_args_with_request_id(
        strings(["commit", "-m", "message", "--no-commit-marker"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::Commit(request) = disabled.request else {
        panic!("expected commit");
    };
    assert_eq!(request.commit_marker, Some(false));

    assert!(
        parse_args_with_request_id(
            strings([
                "commit",
                "-m",
                "message",
                "--commit-marker",
                "--no-commit-marker",
            ]),
            "req_test",
            Path::new("/cwd"),
        )
        .is_err()
    );
}

// `add -A` and `commit -a` share clap's `all` id with the global `--all` target selector.
// The git-style flag must stay a git-style flag: it may not inject `@all` and silently widen
// an explicit `--target` back to the whole workspace.

#[test]
pub(crate) fn stage_all_flag_does_not_widen_an_explicit_target() {
    let invocation = parse_args_with_request_id(
        strings(["add", "-A", "--target", "mem_x"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Stage(request) = invocation.request else {
        panic!("expected stage");
    };
    assert_eq!(request.all, Some(true));
    assert_eq!(request.meta.selection.unwrap().targets, vec!["mem_x"]);
}

#[test]
pub(crate) fn commit_all_flag_does_not_widen_an_explicit_target() {
    let invocation = parse_args_with_request_id(
        strings(["commit", "-a", "--target", "mem_x", "-m", "msg"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Commit(request) = invocation.request else {
        panic!("expected commit");
    };
    assert_eq!(request.all, Some(true));
    assert_eq!(request.meta.selection.unwrap().targets, vec!["mem_x"]);
}

#[test]
pub(crate) fn global_all_still_selects_every_target_for_other_verbs() {
    for args in [strings(["--all", "status"]), strings(["status", "--all"])] {
        let invocation = parse_args_with_request_id(args, "req_test", Path::new("/cwd")).unwrap();
        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.meta.selection.unwrap().targets, vec!["@all"]);
    }
}

#[test]
pub(crate) fn parses_all_with_target_exclusion_for_ls() {
    let invocation = parse_args_with_request_id(
        strings(["--all", "--no-target", "@root", "ls"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Ls { request, .. } = invocation.request else {
        panic!("expected ls");
    };
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["@all"]);
    assert_eq!(selection.exclude_targets, vec!["@root"]);
    assert_eq!(selection.all, None);
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
}

#[test]
pub(crate) fn parses_target_aliases_into_selector_fields() {
    let invocation = parse_args_with_request_id(
        strings([
            "--target",
            "@root",
            "--member",
            "mem_app",
            "--member-path",
            "repos/lib",
            "--no-target",
            "@default",
            "--no-member",
            "mem_old",
            "--no-member-path",
            "repos/old",
            "status",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["@root", "mem_app", "repos/lib"]);
    assert_eq!(
        selection.exclude_targets,
        vec!["@default", "mem_old", "repos/old"]
    );
    assert_eq!(selection.all, None);
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
}

#[test]
pub(crate) fn parses_combined_status_flags() {
    let invocation = parse_args_with_request_id(
        strings(["status", "--porcelain", "--no-branches"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Porcelain);
    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(false));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}

#[test]
pub(crate) fn parses_status_as_combined_by_default() {
    let invocation =
        parse_args_with_request_id(strings(["status"]), "req_test", Path::new("/cwd")).unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(true));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}

#[test]
pub(crate) fn parses_no_combined_status_as_summary_mode() {
    let invocation = parse_args_with_request_id(
        strings(["status", "--no-combined"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Summary));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(true));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}
