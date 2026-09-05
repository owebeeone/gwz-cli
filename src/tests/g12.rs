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

fn dispatch_in(temp: &TempDir, args: &[&str]) -> Result<CliResponse, CliError> {
    let mut owned = strings(["--root"]);
    owned.push(temp.path().to_string_lossy().into_owned());
    owned.extend(args.iter().map(|item| (*item).to_owned()));
    let invocation = parse_args_with_request_id(owned, "req_dispatch", temp.path())
        .unwrap_or_else(|error| panic!("{}: {}", args.join(" "), error.message));
    execute_invocation(&invocation)
}

fn run_in(temp: &TempDir, args: &[&str]) -> CliError {
    dispatch_in(temp, args)
        .err()
        .unwrap_or_else(|| panic!("{} must be refused by core", args.join(" ")))
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

/// Design §7 tag 6 is `copy_source` (§11 item 11, operator 2026-09-05):
/// `--from <name|path>` travels in that field, and the token itself is core's
/// to resolve — a family name, a path, or neither.
#[test]
fn clone_local_from_travels_as_copy_source() {
    let by_name = clone_local(&[
        "clone",
        "--local",
        "--clean",
        "--from",
        "A",
        "--name",
        "B",
        "../gwz-dev-B",
    ]);
    assert_eq!(by_name.copy_source.as_deref(), Some("A"));
    assert_eq!(by_name.name, "B");
    assert_eq!(by_name.mode, gwz_core::LocalCloneMode::Clean);

    let by_path = clone_local(&[
        "clone",
        "--local",
        "--from",
        "../gwz-dev-C",
        "--name",
        "D",
        "../gwz-dev-D",
    ]);
    assert_eq!(by_path.copy_source.as_deref(), Some("../gwz-dev-C"));
    assert_eq!(by_path.mode, gwz_core::LocalCloneMode::Verbatim);

    // Absent means "copy the workspace this command ran in" (design §4), so
    // nothing is invented for the ordinary create.
    assert_eq!(
        clone_local(&["clone", "--local", "--name", "A"]).copy_source,
        None
    );

    // An empty value is indistinguishable from absent once encoded, so it is
    // refused here rather than silently becoming "copy this workspace".
    let empty = refusal(&["clone", "--local", "--from", "", "--name", "A"]);
    assert!(
        empty.contains("--from") && empty.contains("must not be empty"),
        "{empty}"
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

    // `--force` is the global switch, so its other legal position works too.
    let global_first = local_family(&["--force", "local", "dispose", "C", "unpreserved-history"]);
    assert_eq!(global_first.force_hazards, one.force_hazards);

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

/// Parity with `gwz-py` (lane CP): the hazard list is split on `,` and on
/// nothing else. A waiver is the operator's authorization token, so the
/// driver neither trims nor folds it — a name with surrounding whitespace is
/// not in core's vocabulary and core says so, one layer down, in both drivers.
#[test]
fn local_dispose_splits_hazards_on_commas_only_and_never_rewrites_them() {
    let spaced = local_family(&["local", "dispose", "C", "--force", "dirty, open-merge"]);
    assert_eq!(
        spaced.force_hazards,
        vec!["dirty".to_owned(), " open-merge".to_owned()],
        "the token travels exactly as typed"
    );

    // Whitespace is a name core does not know, not an empty list: it travels
    // and core refuses it, rather than the driver answering for a vocabulary
    // it does not own.
    let blank = local_family(&["local", "dispose", "C", "--force", "   "]);
    assert_eq!(blank.force_hazards, vec!["   ".to_owned()]);
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

    // The §7 rows that pair a family name with an ordinary policy, and the
    // `origin` rows that must keep resolving as Git remotes.
    for (args, token) in [
        (vec!["pull", "--head", "--remote", "origin"], "origin"),
        (
            vec!["--sync", "ff-only", "pull", "--head", "--remote", "root"],
            "root",
        ),
    ] {
        let CliRequest::PullHead(request) = parse(&args).unwrap().request else {
            panic!("expected a pull head request for {args:?}");
        };
        assert_eq!(
            request.meta.policy.as_ref().and_then(|p| p.remote.clone()),
            Some(token.to_owned())
        );
    }

    for token in ["hub", "origin"] {
        let CliRequest::Push(push) = parse(&["push", "--remote", token]).unwrap().request else {
            panic!("expected a push request");
        };
        assert_eq!(push.remote.as_deref(), Some(token));
        assert_eq!(
            push.meta.policy.as_ref().and_then(|p| p.remote.clone()),
            Some(token.to_owned())
        );
    }
}

// ---------------------------------------------------------------------------
// dispatch and refusal presentation
// ---------------------------------------------------------------------------

/// The mutating family verbs reach their core entry point and come back as
/// the typed refusal this build owes: `unsupported_operation`, naming the verb.
#[test]
fn local_family_verbs_dispatch_and_refuse_typed() {
    let temp = workspace("cli-local-family");

    assert_unsupported(&run_in(&temp, &["local", "dispose", "C"]), "local dispose");
    assert_unsupported(
        &run_in(&temp, &["local", "dispose", "C", "--keep"]),
        "local dispose --keep",
    );
    assert_unsupported(&run_in(&temp, &["local", "disband"]), "local disband");
}

/// `local list` is served: a workspace holding no family index lists nothing,
/// and that is an ordinary `Ok` answer the driver renders as a listing — not
/// a refusal, and not an invented row.
#[test]
fn local_list_dispatches_and_renders_an_empty_family() {
    let temp = workspace("cli-local-list");

    let response = dispatch_in(&temp, &["local", "list"]).expect("list is served");
    assert_eq!(
        response.envelope.meta.aggregate_status,
        gwz_core::AggregateStatus::Ok
    );
    assert_eq!(exit_code_for_response(&response.envelope), 0);
    assert_eq!(
        render_response(&response, OutputMode::Human),
        "no local clone family members"
    );
    let json: serde_json::Value =
        serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();
    assert_eq!(json["local_family_members"], serde_json::json!([]));
}

/// Core refuses a family `dry_run` before workspace discovery; the driver
/// passes the flag through and presents that refusal unchanged — for a local
/// create as well, which no longer meets a driver-side gate of its own.
#[test]
fn local_family_dry_run_is_refused_by_core() {
    let temp = workspace("cli-local-dry-run");
    assert_unsupported(&run_in(&temp, &["local", "list", "--dry-run"]), "dry_run");
    assert_unsupported(
        &run_in(
            &temp,
            &["clone", "--local", "--name", "A", "../dest-a", "--dry-run"],
        ),
        "dry_run",
    );
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

    // `--from` reaches core in `copy_source`: core names the flag in its own
    // refusal, which is the proof the token travelled rather than being
    // answered for by the driver.
    assert_unsupported(
        &run_in(
            &temp,
            &[
                "clone",
                "--local",
                "--from",
                "A",
                "--name",
                "B",
                "../dest-b",
            ],
        ),
        "--from <name|path>",
    );
}

/// The URL clone keeps its pre-existing `--dry-run` refusal untouched. A
/// *local* clone is a family operation, so — as with every `gwz local` verb,
/// and as `gwz-py` already does (lane CP) — the flag travels and core answers
/// for it, before any destination is allocated.
#[test]
fn the_dry_run_gate_is_the_url_clones_and_the_family_passes_it_through() {
    let error = parse(&["clone", "https://example.invalid/ws.git", "--dry-run"])
        .expect_err("the url clone is gated");
    assert_eq!(error.message, "--dry-run is not supported for clone");

    let local = clone_local(&["clone", "--local", "--name", "A", "--dry-run"]);
    assert_eq!(local.meta.dry_run, Some(true));
    assert_eq!(
        clone_local(&["clone", "--local", "--name", "A"])
            .meta
            .dry_run,
        None
    );
}

/// The merge entry switch: with a selector the request takes the family
/// wrapper, and a token that names no ready member comes back as
/// `unknown_local` (design §7, §11 item 13) carrying the state detail — the
/// driver presents it like any other typed refusal, with no Git-remote
/// fallback of its own.
#[test]
fn merge_entry_switch_routes_only_the_family_selector() {
    let temp = workspace("cli-local-merge");

    for (args, token) in [
        (vec!["merge", "--remote", "A"], "A"),
        (vec!["merge", "--remote", "C", "lane/agent-17"], "C"),
        // §7: `origin` is a family miss on merge, never a fetch.
        (vec!["merge", "--remote", "origin"], "origin"),
    ] {
        let error = run_in(&temp, &args);
        assert_eq!(
            error.code,
            Some(gwz_core::model::ErrorCode::UnknownLocal),
            "{args:?}: {}",
            error.message
        );
        assert!(
            error.message.contains(&format!("`{token}`")),
            "{args:?}: the refusal names the token: {}",
            error.message
        );
        assert!(
            error.message.contains("never falls back to a Git remote"),
            "{args:?}: {}",
            error.message
        );
        assert_eq!(
            error.human_message(),
            format!("UnknownLocal: {}", error.message)
        );
        let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
        assert_eq!(json["errors"][0]["code"], "UnknownLocal", "{args:?}");
    }

    // No selector: the engine answers for the missing ref, and nothing in the
    // message comes from the family wrapper.
    let plain = run_in(&temp, &["merge", "feature/x"]);
    assert_ne!(
        plain.code,
        Some(gwz_core::model::ErrorCode::UnknownLocal),
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
// `gwz local list` rendering (design §8.1, §7, §11 item 12)
// ---------------------------------------------------------------------------

fn entry(
    name: &str,
    kind: gwz_core::LocalMemberKind,
    recorded: gwz_core::LocalMemberState,
    observed: gwz_core::LocalObservedState,
    path: &str,
    last_error: Option<&str>,
) -> gwz_core::LocalFamilyMemberEntry {
    gwz_core::LocalFamilyMemberEntry {
        name: name.to_owned(),
        kind,
        recorded_state: recorded,
        observed_state: observed,
        path: path.to_owned(),
        last_error: last_error.map(ToOwned::to_owned),
    }
}

/// A ready member, the shape every row in design §8.1's sample listing has.
fn ready(
    name: &str,
    kind: gwz_core::LocalMemberKind,
    path: &str,
) -> gwz_core::LocalFamilyMemberEntry {
    entry(
        name,
        kind,
        gwz_core::LocalMemberState::Ready,
        gwz_core::LocalObservedState::Ready,
        path,
        None,
    )
}

/// A `LocalFamilyResponse` carrying `members`, as core will answer op=list
/// once lane S's store lands. Constructed here because core refuses the read
/// today (design §11: the store is still being built).
fn list_response(members: Vec<gwz_core::LocalFamilyMemberEntry>) -> CliResponse {
    family_response(gwz_core::LocalFamilyOp::List, members)
}

fn family_response(
    op: gwz_core::LocalFamilyOp,
    members: Vec<gwz_core::LocalFamilyMemberEntry>,
) -> CliResponse {
    CliResponse::local_family(
        op,
        gwz_core::LocalFamilyResponse {
            response: gwz_core::ResponseEnvelope {
                meta: gwz_core::ResponseMeta {
                    request_id: "req_test".to_owned(),
                    schema_version: "gwz.protocol/v0".to_owned(),
                    action: gwz_core::ActionKind::LocalFamily,
                    aggregate_status: gwz_core::AggregateStatus::Ok,
                    operation_id: Some("op_test".to_owned()),
                    message: None,
                    attribution: None,
                },
                members: Vec::new(),
                errors: Vec::new(),
            },
            members,
            // LCM1.0c follow-up 3 (operator ruling 2026-09-06): the family
            // root's path, joined with each member's root-relative `path` by
            // the renderer (lane CR). Absent here; the listing tests above
            // pin the root-relative column, not the join.
            root_path: None,
        },
    )
}

/// Design §8.1: the family of the worked example, byte for byte — four
/// columns (name, kind, state, path), aligned, and no header.
#[test]
fn local_list_renders_the_design_sample_listing() {
    let response = list_response(vec![
        ready("root", gwz_core::LocalMemberKind::Checkout, "."),
        ready("A", gwz_core::LocalMemberKind::Checkout, "../gwz-dev-A"),
        ready("B", gwz_core::LocalMemberKind::Checkout, "../gwz-dev-B"),
        ready("C", gwz_core::LocalMemberKind::Checkout, "../gwz-dev-C"),
        ready("D", gwz_core::LocalMemberKind::Checkout, "../gwz-dev-D"),
        ready("hub", gwz_core::LocalMemberKind::Bare, "../gwz-dev-hub"),
    ]);
    assert_eq!(
        render_response(&response, OutputMode::Human),
        "\
root  checkout  ready  .
A     checkout  ready  ../gwz-dev-A
B     checkout  ready  ../gwz-dev-B
C     checkout  ready  ../gwz-dev-C
D     checkout  ready  ../gwz-dev-D
hub   bare      ready  ../gwz-dev-hub"
    );
}

/// The whole point of carrying both states: a lane whose recorded row and
/// whose disk disagree says so in the `state` column, and a recorded
/// diagnostic is shown rather than dropped.
#[test]
fn local_list_shows_the_observed_state_when_it_differs_and_any_recorded_error() {
    let response = list_response(vec![
        ready("root", gwz_core::LocalMemberKind::Checkout, "."),
        entry(
            "B",
            gwz_core::LocalMemberKind::Checkout,
            gwz_core::LocalMemberState::Creating,
            gwz_core::LocalObservedState::Incomplete,
            "../ws-B",
            Some("copy interrupted at src/"),
        ),
        entry(
            "C",
            gwz_core::LocalMemberKind::Checkout,
            gwz_core::LocalMemberState::Disposing,
            gwz_core::LocalObservedState::InterruptedDisposal,
            "../ws-C",
            None,
        ),
        entry(
            "hub",
            gwz_core::LocalMemberKind::Bare,
            gwz_core::LocalMemberState::Ready,
            gwz_core::LocalObservedState::PointerRemoved,
            "../ws-hub",
            None,
        ),
    ]);
    assert_eq!(
        render_response(&response, OutputMode::Human),
        "\
root  checkout  ready                           .
B     checkout  creating/incomplete             ../ws-B    copy interrupted at src/
C     checkout  disposing/interrupted_disposal  ../ws-C
hub   bare      ready/pointer_removed           ../ws-hub"
    );
}

/// A recorded diagnostic is one row: a newline inside it must not forge a
/// member of its own in a listing consumers read line by line.
#[test]
fn local_list_keeps_a_multi_line_diagnostic_on_its_own_row() {
    let response = list_response(vec![entry(
        "B",
        gwz_core::LocalMemberKind::Checkout,
        gwz_core::LocalMemberState::Creating,
        gwz_core::LocalObservedState::Malformed,
        "../ws-B",
        Some("index unreadable\nA  checkout  ready  ../forged"),
    )]);
    let rendered = render_response(&response, OutputMode::Human);
    assert_eq!(rendered.lines().count(), 1, "{rendered}");
    assert!(
        rendered.contains("index unreadable A  checkout"),
        "{rendered}"
    );
}

/// Every field of every entry survives `--json` and the `--jsonl` response
/// record, and a response that is not a family listing carries `null` rather
/// than an invented empty list.
#[test]
fn local_list_json_carries_every_field_of_every_entry() {
    let response = list_response(vec![
        ready("root", gwz_core::LocalMemberKind::Checkout, "."),
        entry(
            "B",
            gwz_core::LocalMemberKind::Checkout,
            gwz_core::LocalMemberState::Creating,
            gwz_core::LocalObservedState::Incomplete,
            "../ws-B",
            Some("copy interrupted at src/"),
        ),
    ]);

    let json: serde_json::Value =
        serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();
    let members = json["local_family_members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0],
        serde_json::json!({
            "name": "root",
            "kind": "Checkout",
            "recorded_state": "Ready",
            "observed_state": "Ready",
            "path": ".",
            "last_error": null,
        })
    );
    assert_eq!(
        members[1],
        serde_json::json!({
            "name": "B",
            "kind": "Checkout",
            "recorded_state": "Creating",
            "observed_state": "Incomplete",
            "path": "../ws-B",
            "last_error": "copy interrupted at src/",
        })
    );

    // `--jsonl` streams the same response record.
    let first = render_response(&response, OutputMode::Jsonl);
    let streamed: serde_json::Value = serde_json::from_str(first.lines().next().unwrap()).unwrap();
    assert_eq!(streamed["kind"], "response");
    assert_eq!(
        streamed["local_family_members"],
        json["local_family_members"]
    );

    // A workspace that holds no family index lists nothing, and that is an
    // answer rather than an error: the human line says so instead of printing
    // a bare status.
    let no_family = list_response(Vec::new());
    let json: serde_json::Value =
        serde_json::from_str(&render_response(&no_family, OutputMode::Json)).unwrap();
    assert_eq!(json["local_family_members"], serde_json::json!([]));
    assert_eq!(
        render_response(&no_family, OutputMode::Human),
        "no local clone family members"
    );

    // dispose/disband carry no rows (design §7), so they are not a listing at
    // all: the human rendering is the ordinary envelope, and the machine
    // envelope still reports the empty list faithfully.
    for op in [
        gwz_core::LocalFamilyOp::Dispose,
        gwz_core::LocalFamilyOp::Disband,
    ] {
        let mutation = family_response(op, Vec::new());
        assert_eq!(render_response(&mutation, OutputMode::Human), "status: Ok");
        let json: serde_json::Value =
            serde_json::from_str(&render_response(&mutation, OutputMode::Json)).unwrap();
        assert_eq!(json["local_family_members"], serde_json::json!([]));
    }

    // Every other verb is not a family response at all. Its envelope key set
    // is pinned by the canonical cross-driver fixture in gwz-core, so the
    // field is absent there rather than rendered as `null`.
    let other = CliResponse::envelope(no_family.envelope.clone());
    let json: serde_json::Value =
        serde_json::from_str(&render_response(&other, OutputMode::Json)).unwrap();
    assert!(
        json.get("local_family_members").is_none(),
        "a non-family response must not gain a family key: {json}"
    );
}

/// Every wire enum has exactly one word, and the words are the design's.
#[test]
fn every_listed_enum_variant_renders_its_design_spelling() {
    assert_eq!(
        [
            gwz_core::LocalMemberKind::Checkout,
            gwz_core::LocalMemberKind::Bare
        ]
        .map(member_kind_word),
        ["checkout", "bare"]
    );
    assert_eq!(
        [
            gwz_core::LocalMemberState::Creating,
            gwz_core::LocalMemberState::Ready,
            gwz_core::LocalMemberState::Disposing,
        ]
        .map(recorded_state_word),
        ["creating", "ready", "disposing"]
    );
    assert_eq!(
        [
            gwz_core::LocalObservedState::Ready,
            gwz_core::LocalObservedState::Incomplete,
            gwz_core::LocalObservedState::InterruptedDisposal,
            gwz_core::LocalObservedState::Missing,
            gwz_core::LocalObservedState::PointerRemoved,
            gwz_core::LocalObservedState::Mismatched,
            gwz_core::LocalObservedState::Malformed,
            gwz_core::LocalObservedState::Unobserved,
        ]
        .map(observed_state_word),
        [
            "ready",
            "incomplete",
            "interrupted_disposal",
            "missing",
            "pointer_removed",
            "mismatched",
            "malformed",
            "unobserved",
        ]
    );
}

// ---------------------------------------------------------------------------
// unknown_local = 62 (design §7, §11 item 13)
// ---------------------------------------------------------------------------

/// The family-only merge miss is presented like any other typed refusal: the
/// code prefixes the human line, `--json`/`--jsonl` carry it structured, and
/// core's state detail travels unedited in both.
#[test]
fn unknown_local_is_presented_as_a_typed_refusal_with_the_state_detail() {
    for detail in [
        "no ready family member is named `origin`; `merge --remote` resolves family \
         names only and never falls back to a Git remote",
        "`B` is a family member whose row is creating, not ready; only a ready member \
         is a merge source",
    ] {
        let message = format!("local family merge: {detail}");
        let error = CliError::from_model(gwz_core::model::ModelError::new(
            gwz_core::model::ErrorCode::UnknownLocal,
            message.clone(),
        ));
        assert_eq!(
            error.code,
            Some(gwz_core::model::ErrorCode::UnknownLocal),
            "{message}"
        );
        assert_eq!(
            error.human_message(),
            format!("UnknownLocal: {message}"),
            "the human line names the code, like every other typed refusal"
        );

        let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
        assert_eq!(json["errors"][0]["code"], "UnknownLocal");
        assert_eq!(json["errors"][0]["message"], message);
    }

    // Design §7 pins the number: it is not folded into `missing_remote`, which
    // pull and push keep for their own "neither" case.
    assert_eq!(
        gwz_core::GwzErrorCode::from(gwz_core::model::ErrorCode::UnknownLocal).wire(),
        62
    );
    assert_ne!(
        gwz_core::GwzErrorCode::from(gwz_core::model::ErrorCode::UnknownLocal),
        gwz_core::GwzErrorCode::from(gwz_core::model::ErrorCode::MissingRemote)
    );
}

// ---------------------------------------------------------------------------
// the shared argv -> request parity fixture
// ---------------------------------------------------------------------------

/// The same fixture `gwz-py` can read (see the file's own note): one row per
/// design §7 CLI -> message row, argv on one side and the encoded request on
/// the other, so a driver that quietly changes an encoding fails here rather
/// than diverging silently from its sibling.
const PARITY_FIXTURE: &str =
    include_str!("../../tests/fixtures/cli_parity/local_family_cases.json");

/// The fixture's projection of a built request: the design's wire spellings,
/// with every optional field spelled `null` rather than omitted.
fn request_json(request: &CliRequest) -> serde_json::Value {
    use serde_json::json;
    let policy_remote = |meta: &gwz_core::RequestMeta| {
        meta.policy
            .as_ref()
            .and_then(|policy| policy.remote.clone())
    };
    match request {
        CliRequest::CloneLocalWorkspace(request) => json!({
            "kind": "CloneLocalWorkspaceRequest",
            "name": request.name,
            "dest": request.dest,
            "mode": match request.mode {
                gwz_core::LocalCloneMode::Verbatim => "verbatim",
                gwz_core::LocalCloneMode::Clean => "clean",
                gwz_core::LocalCloneMode::Bare => "bare",
            },
            "branch": request.branch,
            "copy_source": request.copy_source,
        }),
        CliRequest::LocalFamily(request) => json!({
            "kind": "LocalFamilyRequest",
            "op": match request.op {
                gwz_core::LocalFamilyOp::List => "list",
                gwz_core::LocalFamilyOp::Dispose => "dispose",
                gwz_core::LocalFamilyOp::Disband => "disband",
            },
            "name": request.name,
            "keep": request.keep,
            "force_hazards": request.force_hazards,
        }),
        CliRequest::Merge(request) => json!({
            "kind": "MergeRequest",
            "op": format!("{:?}", request.op).to_lowercase(),
            "source_ref": request.source_ref,
            "local_source_name": request.local_source_name,
            "policy_remote": policy_remote(&request.meta),
        }),
        CliRequest::PullHead(request) => json!({
            "kind": "PullHeadRequest",
            "policy_remote": policy_remote(&request.meta),
        }),
        CliRequest::Push(request) => json!({
            "kind": "PushRequest",
            "remote": request.remote,
            "policy_remote": policy_remote(&request.meta),
        }),
        other => panic!("the parity fixture covers no request of this kind: {other:?}"),
    }
}

#[test]
fn the_shared_parity_fixture_pins_every_design_row_argv_to_request() {
    let fixture: serde_json::Value = serde_json::from_str(PARITY_FIXTURE).unwrap();

    let accept = fixture["accept"].as_array().unwrap();
    for case in accept {
        let argv: Vec<&str> = case["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap())
            .collect();
        let row = case["design_row"].as_str().unwrap();
        let invocation = parse(&argv).unwrap_or_else(|error| {
            panic!("{row}: {argv:?} must parse, got {}", error.message);
        });
        assert_eq!(
            request_json(&invocation.request),
            case["request"],
            "{row}: {argv:?}"
        );
    }

    for case in fixture["reject"].as_array().unwrap() {
        let argv: Vec<&str> = case["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap())
            .collect();
        let why = case["why"].as_str().unwrap();
        let needle = case["rust_needle"].as_str().unwrap();
        let message = refusal(&argv);
        assert!(
            message.contains(needle),
            "{why}: {argv:?} must name `{needle}`, got: {message}"
        );
    }

    // The fixture is the §7 table, not a sample of it: every row of the design
    // table has a case, and the two `copy_source` rows the message block adds.
    assert_eq!(accept.len(), 24, "a §7 row lost its parity case");
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
        // The binding behind the name is core's and is not landed, so the page
        // must not read as though it already works.
        assert!(page.contains("`missing_remote` in this build"));
    }
}
