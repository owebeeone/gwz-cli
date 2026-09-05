//! LCM1.1c/LCM1.2 CLI coverage for the local clone family (lane CR):
//! `gwz clone --local`, `gwz local list|dispose|disband`, and the
//! `gwz merge --remote <name> [<ref>]` family selector.
//!
//! Every §7 row of `dev-docs/GwzLocalCloneDesign.md` is pinned twice: once as
//! argument -> exact request, and once as the refusal a malformed invocation
//! earns *before* any request is encoded. The dispatch cases run against a real
//! workspace and pin the typed refusal core returns today, without pinning
//! core's own wording (lanes S and X move it).

use std::path::Path;

use clap::CommandFactory;

use super::g01::{TempDir, strings};
use super::*;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn parse(args: &[&str]) -> Result<CliInvocation, CliError> {
    let owned = args.iter().map(|item| (*item).to_owned()).collect();
    parse_args_with_request_id(owned, "req_test", Path::new("/cwd"))
}

fn clone_local(args: &[&str]) -> gwz_core::CloneLocalWorkspaceRequest {
    match parse(args).expect("parses").request {
        CliRequest::CloneLocalWorkspace(request) => request,
        other => panic!("expected a local clone request, got {other:?}"),
    }
}

fn local_family(args: &[&str]) -> gwz_core::LocalFamilyRequest {
    match parse(args).expect("parses").request {
        CliRequest::LocalFamily(request) => request,
        other => panic!("expected a local family request, got {other:?}"),
    }
}

fn merge(args: &[&str]) -> gwz_core::MergeRequest {
    match parse(args).expect("parses").request {
        CliRequest::Merge(request) => request,
        other => panic!("expected a merge request, got {other:?}"),
    }
}

/// A typed CLI refusal: the message, asserted to carry `InvalidRequest` so
/// `--json`/`--jsonl` render it structured rather than as a bare string.
fn refusal(args: &[&str]) -> String {
    let error = parse(args).expect_err("must refuse");
    assert_eq!(
        error.code,
        Some(gwz_core::model::ErrorCode::InvalidRequest),
        "{}: expected a typed InvalidRequest, got {error:?}",
        args.join(" ")
    );
    error.message
}

/// A workspace root with nothing else: enough for `resolve_workspace_root`,
/// and no family index anywhere (the family verbs must still refuse typed).
fn workspace(prefix: &str) -> TempDir {
    let temp = TempDir::new(prefix);
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: super::g01::request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_local".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    temp
}

fn run_in(temp: &TempDir, args: &[&str]) -> CliError {
    let mut owned = strings(["--root"]);
    owned.push(temp.path().to_string_lossy().into_owned());
    owned.extend(args.iter().map(|item| (*item).to_owned()));
    let invocation = parse_args_with_request_id(owned, "req_dispatch", temp.path())
        .unwrap_or_else(|error| panic!("{}: {}", args.join(" "), error.message));
    execute_invocation(&invocation).expect_err("every local family verb refuses today")
}

/// The refusal presentation: human prefixes the code, `--json` carries it as a
/// structured field, and both carry core's message unchanged.
fn assert_unsupported(error: &CliError, names: &str) {
    assert_eq!(
        error.code,
        Some(gwz_core::model::ErrorCode::UnsupportedOperation),
        "{}",
        error.message
    );
    assert!(
        error.message.contains(names),
        "refusal must name `{names}`: {}",
        error.message
    );
    assert_eq!(
        error.human_message(),
        format!("UnsupportedOperation: {}", error.message)
    );
    let json: serde_json::Value = serde_json::from_str(&render_error_json(error)).unwrap();
    assert_eq!(json["errors"][0]["code"], "UnsupportedOperation");
    assert_eq!(json["errors"][0]["message"], error.message);
}

// ---------------------------------------------------------------------------
// §7 create rows
// ---------------------------------------------------------------------------

/// `gwz clone --local --name A ../gwz-dev-A` ->
/// `CloneLocalWorkspaceRequest` mode=verbatim name=A dest=...
#[test]
fn clone_local_verbatim_carries_the_name_and_the_destination() {
    let request = clone_local(&["clone", "--local", "--name", "A", "../gwz-dev-A"]);
    assert_eq!(request.name, "A");
    assert_eq!(request.dest.as_deref(), Some("../gwz-dev-A"));
    assert_eq!(request.mode, gwz_core::LocalCloneMode::Verbatim);
    assert_eq!(request.branch, None);
    assert_eq!(request.meta.request_id, "req_test");

    // Design §4: the default destination `../<root-dirname>-<Name>` is core's,
    // so an omitted dest travels as absent rather than computed here.
    assert_eq!(clone_local(&["clone", "--local", "--name", "A"]).dest, None);

    // `--verbatim` is the explicit spelling of the default mode.
    let explicit = clone_local(&["clone", "--local", "--verbatim", "--name", "A"]);
    assert_eq!(explicit.mode, gwz_core::LocalCloneMode::Verbatim);
}

/// `gwz clone --local --clean -b lane/x --name C dest` -> mode=clean,
/// branch=lane/x.
#[test]
fn clone_local_clean_carries_the_branch() {
    let request = clone_local(&[
        "clone", "--local", "--clean", "-b", "lane/x", "--name", "C", "dest",
    ]);
    assert_eq!(request.name, "C");
    assert_eq!(request.dest.as_deref(), Some("dest"));
    assert_eq!(request.mode, gwz_core::LocalCloneMode::Clean);
    assert_eq!(request.branch.as_deref(), Some("lane/x"));

    let no_branch = clone_local(&["clone", "--local", "--clean", "--name", "D", "dest"]);
    assert_eq!(no_branch.mode, gwz_core::LocalCloneMode::Clean);
    assert_eq!(no_branch.branch, None);
}

/// `gwz clone --local --bare --name hub dest` -> mode=bare. `--bare` implies
/// `--clean` (design §4.2), so it accepts `-b` and the redundant `--clean`.
#[test]
fn clone_local_bare_implies_clean() {
    let request = clone_local(&["clone", "--local", "--bare", "--name", "hub", "dest"]);
    assert_eq!(request.name, "hub");
    assert_eq!(request.mode, gwz_core::LocalCloneMode::Bare);

    let with_branch = clone_local(&[
        "clone", "--local", "--bare", "-b", "lane/x", "--name", "hub", "dest",
    ]);
    assert_eq!(with_branch.mode, gwz_core::LocalCloneMode::Bare);
    assert_eq!(with_branch.branch.as_deref(), Some("lane/x"));

    let redundant = clone_local(&[
        "clone", "--local", "--bare", "--clean", "--name", "hub", "dest",
    ]);
    assert_eq!(redundant.mode, gwz_core::LocalCloneMode::Bare);
}

/// The URL clone is untouched: same request, same derived target, and the
/// local-only flags are refused when `--local` is absent.
#[test]
fn url_clone_is_unchanged_and_local_flags_require_local() {
    let CliRequest::CloneWorkspace { url, target, .. } =
        parse(&["clone", "https://example.com/org/ws.git", "work/demo"])
            .unwrap()
            .request
    else {
        panic!("expected the url clone");
    };
    assert_eq!(url, "https://example.com/org/ws.git");
    assert_eq!(target, "work/demo");

    for (flag, args) in [
        ("--name", vec!["clone", "url", "--name", "A"]),
        ("--verbatim", vec!["clone", "url", "--verbatim"]),
        ("--clean", vec!["clone", "url", "--clean"]),
        ("--bare", vec!["clone", "url", "--bare"]),
        ("-b", vec!["clone", "url", "-b", "lane/x"]),
        ("--from", vec!["clone", "url", "--from", "A"]),
    ] {
        let message = refusal(&args);
        assert!(
            message.contains(flag) && message.contains("--local"),
            "{flag}: {message}"
        );
    }

    // A missing URL without `--local` is still Clap's own required-argument
    // error (untyped), not one of the local refusals.
    let error = parse(&["clone"]).expect_err("url is required");
    assert_eq!(error.code, None, "{}", error.message);
}

/// Every CLI-level create refusal, all before a request is encoded.
#[test]
fn clone_local_refuses_malformed_flag_combinations() {
    assert!(
        refusal(&["clone", "--local", "dest"]).contains("--name"),
        "--local requires a name"
    );

    // `--local` is mutually exclusive with a URL: only a destination is taken.
    let with_url = refusal(&["clone", "--local", "--name", "A", "url", "dest"]);
    assert!(
        with_url.contains("destination") && with_url.contains("URL"),
        "{with_url}"
    );

    let both_modes = refusal(&["clone", "--local", "--verbatim", "--clean", "--name", "A"]);
    assert!(
        both_modes.contains("--verbatim") && both_modes.contains("--clean"),
        "{both_modes}"
    );

    let verbatim_bare = refusal(&["clone", "--local", "--verbatim", "--bare", "--name", "A"]);
    assert!(
        verbatim_bare.contains("--verbatim") && verbatim_bare.contains("--bare"),
        "{verbatim_bare}"
    );

    let stray_branch = refusal(&["clone", "--local", "-b", "lane/x", "--name", "A"]);
    assert!(
        stray_branch.contains("--clean") && stray_branch.contains("--bare"),
        "{stray_branch}"
    );
}

/// Design §7 holds `CloneLocalWorkspaceRequest` tag 6 (`from`) unallocated
/// pending an operator decision, so `--from` parses and refuses rather than
/// being silently dropped or encoded into an invented field.
#[test]
fn clone_local_from_parses_and_refuses_as_unsupported() {
    let error = parse(&["clone", "--local", "--from", "A", "--name", "B", "dest"])
        .expect_err("--from is not wired");
    assert_eq!(
        error.code,
        Some(gwz_core::model::ErrorCode::UnsupportedOperation),
        "{}",
        error.message
    );
    assert!(error.message.contains("--from"), "{}", error.message);
    assert!(
        error.message.contains("not yet supported"),
        "{}",
        error.message
    );
}

// ---------------------------------------------------------------------------
// §7 family rows
// ---------------------------------------------------------------------------

/// `gwz local list` -> `LocalFamilyRequest` op=list, and
/// `gwz local disband` -> op=disband; both carry no other field.
#[test]
fn local_list_and_disband_carry_only_the_op() {
    let list = local_family(&["local", "list"]);
    assert_eq!(list.op, gwz_core::LocalFamilyOp::List);
    assert_eq!(list.name, None);
    assert_eq!(list.keep, None);
    assert!(list.force_hazards.is_empty());

    let disband = local_family(&["local", "disband"]);
    assert_eq!(disband.op, gwz_core::LocalFamilyOp::Disband);
    assert_eq!(disband.name, None);
    assert_eq!(disband.keep, None);
    assert!(disband.force_hazards.is_empty());
}

/// The four dispose rows of the §7 table, field for field.
#[test]
fn local_dispose_carries_the_name_keep_and_hazards() {
    let plain = local_family(&["local", "dispose", "C"]);
    assert_eq!(plain.op, gwz_core::LocalFamilyOp::Dispose);
    assert_eq!(plain.name.as_deref(), Some("C"));
    assert_eq!(plain.keep, None);
    assert!(plain.force_hazards.is_empty());

    let keep = local_family(&["local", "dispose", "C", "--keep"]);
    assert_eq!(keep.keep, Some(true));
    assert!(keep.force_hazards.is_empty());

    let one = local_family(&["local", "dispose", "C", "--force", "unpreserved-history"]);
    assert_eq!(one.keep, None);
    assert_eq!(one.force_hazards, vec!["unpreserved-history".to_owned()]);

    let many = local_family(&[
        "local",
        "dispose",
        "C",
        "--force",
        "open-merge,dirty,unpreserved-history",
    ]);
    assert_eq!(
        many.force_hazards,
        vec![
            "open-merge".to_owned(),
            "dirty".to_owned(),
            "unpreserved-history".to_owned(),
        ]
    );

    // The hazard vocabulary is core's (`HazardWaiver::parse_all`), so an
    // unknown name travels rather than being second-guessed here.
    let unknown = local_family(&["local", "dispose", "C", "--force", "wat"]);
    assert_eq!(unknown.force_hazards, vec!["wat".to_owned()]);
}

/// Design §8.4: `gwz local dispose C --force` refuses for want of hazard
/// names, and it refuses *here* — an empty list on the wire means "no force",
/// so encoding one would silently turn an authorization into its opposite.
#[test]
fn local_dispose_refuses_force_without_hazard_names() {
    for args in [
        vec!["local", "dispose", "C", "--force"],
        vec!["local", "dispose", "C", "--force", ""],
        vec!["local", "dispose", "C", "--force", "   "],
    ] {
        let message = refusal(&args);
        assert!(
            message.contains("--force") && message.contains("hazard"),
            "{args:?}: {message}"
        );
    }

    let empty_segment = refusal(&["local", "dispose", "C", "--force", "dirty,"]);
    assert!(
        empty_segment.contains("empty"),
        "an empty segment is not a hazard: {empty_segment}"
    );

    // The other half of the pairing: names without the switch never become a
    // waiver by accident.
    let unauthorized = refusal(&["local", "dispose", "C", "dirty"]);
    assert!(
        unauthorized.contains("--force"),
        "hazard names need the switch: {unauthorized}"
    );
}

/// §5.2: keep and force are mutually exclusive; the CLI says so before the
/// request is built (core repeats the rule).
#[test]
fn local_dispose_refuses_keep_with_force() {
    let message = refusal(&["local", "dispose", "C", "--keep", "--force", "dirty"]);
    assert!(
        message.contains("--keep") && message.contains("--force"),
        "{message}"
    );
}

/// `--dry-run` is a global flag and travels on the request meta; core refuses
/// it before any write.
#[test]
fn local_family_passes_dry_run_through_to_core() {
    assert_eq!(
        local_family(&["local", "list", "--dry-run"]).meta.dry_run,
        Some(true)
    );
    assert_eq!(local_family(&["local", "list"]).meta.dry_run, None);
}

// ---------------------------------------------------------------------------
// §7 merge rows
// ---------------------------------------------------------------------------

/// `gwz merge --remote A` -> `local_source_name=A`;
/// `gwz merge --remote C lane/agent-17` -> `local_source_name=C`,
/// `source_ref=lane/agent-17`.
#[test]
fn merge_remote_sets_the_family_selector() {
    let head = merge(&["merge", "--remote", "A"]);
    assert_eq!(head.op, gwz_core::MergeOp::Start);
    assert_eq!(head.local_source_name.as_deref(), Some("A"));
    assert_eq!(head.source_ref, None);

    let with_ref = merge(&["merge", "--remote", "C", "lane/agent-17"]);
    assert_eq!(with_ref.local_source_name.as_deref(), Some("C"));
    assert_eq!(with_ref.source_ref.as_deref(), Some("lane/agent-17"));

    // `--remote origin` is a family miss, not a Git remote: it is encoded as
    // the selector and core answers (design §6).
    assert_eq!(
        merge(&["merge", "--remote", "origin"])
            .local_source_name
            .as_deref(),
        Some("origin")
    );
}

/// The global `--remote` is `policy.remote` for every other verb, and the
/// merge engine refuses a request that still carries it. Moving the token into
/// the selector must therefore clear the policy field.
#[test]
fn merge_remote_moves_out_of_the_operation_policy() {
    let request = merge(&["merge", "--remote", "A"]);
    assert_eq!(
        request.meta.policy.as_ref().and_then(|p| p.remote.clone()),
        None,
        "policy.remote is rejected by the merge engine"
    );
}

/// `gwz merge feature/x` and `gwz merge A` stay Git refs: the selector is
/// absent, so the entry switch delegates to the unchanged engine.
#[test]
fn merge_without_remote_is_unchanged() {
    for source in ["feature/x", "A"] {
        let request = merge(&["merge", source]);
        assert_eq!(request.op, gwz_core::MergeOp::Start);
        assert_eq!(request.source_ref.as_deref(), Some(source));
        assert_eq!(request.local_source_name, None);
    }

    for args in [
        vec!["merge", "--abort"],
        vec!["merge", "--continue"],
        vec!["merge", "--status"],
        vec!["merge", "--gc"],
    ] {
        assert_eq!(
            merge(&args).local_source_name,
            None,
            "{args:?} carries no selector"
        );
    }
}

/// `MergeRequest.local_source_name` is start-only (§7), so a lifecycle op with
/// `--remote` is refused by the driver before the request is built.
#[test]
fn merge_remote_is_refused_for_lifecycle_operations() {
    for args in [
        vec!["merge", "--remote", "A", "--abort"],
        vec!["merge", "--remote", "A", "--continue"],
        vec!["merge", "--remote", "A", "--status"],
        vec!["merge", "--remote", "A", "--gc"],
    ] {
        let message = refusal(&args);
        assert!(
            message.contains("--remote") && message.contains("starting a merge"),
            "{args:?}: {message}"
        );
    }
}

/// `--remote` on pull and push keeps its existing meaning: the token stays in
/// `OperationPolicy.remote` / `PushRequest.remote` and core resolves it.
#[test]
fn pull_and_push_remote_are_untouched() {
    let CliRequest::PullHead(pull) = parse(&["pull", "--head", "--remote", "A"]).unwrap().request
    else {
        panic!("expected a pull head request");
    };
    assert_eq!(
        pull.meta.policy.as_ref().and_then(|p| p.remote.clone()),
        Some("A".to_owned())
    );

    let CliRequest::Push(push) = parse(&["push", "--remote", "hub"]).unwrap().request else {
        panic!("expected a push request");
    };
    assert_eq!(push.remote.as_deref(), Some("hub"));
    assert_eq!(
        push.meta.policy.as_ref().and_then(|p| p.remote.clone()),
        Some("hub".to_owned())
    );
}

// ---------------------------------------------------------------------------
// dispatch and refusal presentation
// ---------------------------------------------------------------------------

/// Every family verb reaches its core entry point and comes back as the typed
/// refusal this build owes: `unsupported_operation`, naming the verb.
#[test]
fn local_family_verbs_dispatch_and_refuse_typed() {
    let temp = workspace("cli-local-family");

    assert_unsupported(&run_in(&temp, &["local", "list"]), "local family list");
    assert_unsupported(&run_in(&temp, &["local", "dispose", "C"]), "local dispose");
    assert_unsupported(
        &run_in(&temp, &["local", "dispose", "C", "--keep"]),
        "local dispose --keep",
    );
    assert_unsupported(&run_in(&temp, &["local", "disband"]), "local disband");
}

/// Core refuses a family `dry_run` before workspace discovery; the driver
/// passes the flag through and presents that refusal unchanged.
#[test]
fn local_family_dry_run_is_refused_by_core() {
    let temp = workspace("cli-local-dry-run");
    assert_unsupported(&run_in(&temp, &["local", "list", "--dry-run"]), "dry_run");
}

/// `gwz clone --local` reaches `handle_clone_local_workspace`, which names the
/// mode in its refusal.
#[test]
fn clone_local_dispatches_and_refuses_typed() {
    let temp = workspace("cli-local-clone");
    assert_unsupported(
        &run_in(&temp, &["clone", "--local", "--name", "A", "../dest-a"]),
        "local clone (verbatim mode)",
    );
    assert_unsupported(
        &run_in(
            &temp,
            &["clone", "--local", "--clean", "--name", "C", "../dest-c"],
        ),
        "local clone (clean mode)",
    );
    assert_unsupported(
        &run_in(
            &temp,
            &["clone", "--local", "--bare", "--name", "hub", "../dest-hub"],
        ),
        "local clone (bare mode)",
    );
}

/// The one pre-existing clone gate still stands for the local family: the
/// driver refuses `--dry-run` for every clone before dispatch.
#[test]
fn clone_local_keeps_the_existing_dry_run_gate() {
    let error = parse(&["clone", "--local", "--name", "A", "--dry-run"]).expect_err("gated");
    assert_eq!(error.message, "--dry-run is not supported for clone");
}

/// The merge entry switch: with a selector the request takes the family
/// wrapper (which names the source), and without one it reaches the unchanged
/// engine — the same refusal an unmodified driver produced.
#[test]
fn merge_entry_switch_routes_only_the_family_selector() {
    let temp = workspace("cli-local-merge");

    assert_unsupported(&run_in(&temp, &["merge", "--remote", "A"]), "from `A`");
    assert_unsupported(
        &run_in(&temp, &["merge", "--remote", "C", "lane/agent-17"]),
        "from `C`",
    );

    // No selector: the engine answers for the missing ref, and nothing in the
    // message comes from the family wrapper.
    let plain = run_in(&temp, &["merge", "feature/x"]);
    assert_ne!(
        plain.code,
        Some(gwz_core::model::ErrorCode::UnsupportedOperation),
        "{}",
        plain.message
    );
    assert!(
        !plain.message.contains("local family"),
        "a plain merge never reaches the family wrapper: {}",
        plain.message
    );
}

// ---------------------------------------------------------------------------
// help and generated reference
// ---------------------------------------------------------------------------

fn long_help(path: &[&str]) -> String {
    let mut command = Cli::command();
    let mut current = &mut command;
    for segment in path {
        current = current.find_subcommand_mut(segment).unwrap_or_else(|| {
            panic!("missing command path segment `{segment}`");
        });
    }
    current.render_long_help().to_string()
}

#[test]
fn local_and_clone_help_describe_the_family_surface() {
    let root = usage_text();
    assert!(root.contains("local"), "{root}");

    let local = long_help(&["local"]);
    for phrase in ["list", "dispose", "disband"] {
        assert!(local.contains(phrase), "missing `{phrase}` in:\n{local}");
    }

    let dispose = long_help(&["local", "dispose"]);
    for phrase in [
        "--keep",
        "--force",
        "open-merge",
        "dirty",
        "unpreserved-history",
    ] {
        assert!(
            dispose.contains(phrase),
            "missing `{phrase}` in:\n{dispose}"
        );
    }

    let clone = long_help(&["clone"]);
    for phrase in ["--local", "--name", "--clean", "--bare", "--from"] {
        assert!(clone.contains(phrase), "missing `{phrase}` in:\n{clone}");
    }

    let merge = long_help(&["merge"]);
    assert!(
        merge.contains("--remote"),
        "the merge page must explain the family selector:\n{merge}"
    );
}

#[test]
fn generated_reference_and_command_page_cover_local() {
    let reference = cli_reference_markdown();
    assert!(reference.contains("Command page: [local](commands/local.md)."));
    assert!(reference.contains("### `gwz local dispose`"));

    let page = include_str!("../../docs/commands/local.md");
    for needle in [
        "gwz local list",
        "gwz local dispose",
        "gwz local disband",
        "--keep",
        "open-merge,dirty,unpreserved-history",
    ] {
        assert!(
            page.contains(needle),
            "local command page is missing `{needle}`"
        );
    }

    let clone_page = include_str!("../../docs/commands/clone.md");
    assert!(clone_page.contains("gwz clone --local"));

    // The merge page used to list `--remote` among the operation policies
    // merge rejects. It is the family selector now, and the page must not
    // still say the opposite.
    let merge_page = include_str!("../../docs/commands/merge.md");
    assert!(merge_page.contains("gwz merge --remote <name> [<ref>]"));
    assert!(merge_page.contains("gwz merge --remote C lane/agent-17"));
    assert!(
        !merge_page.contains("`--sync`, `--remote`,"),
        "the rejected-policy list must no longer name --remote"
    );

    // Family names on pull and push keep their existing flag and gain the
    // family meaning; both pages say so and both keep the `origin` fallback.
    for page in [
        include_str!("../../docs/commands/pull.md"),
        include_str!("../../docs/commands/push.md"),
    ] {
        assert!(page.contains("local clone family](local.md)"));
        assert!(page.contains("`--remote origin` keeps its usual meaning"));
    }
}
