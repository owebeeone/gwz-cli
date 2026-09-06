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

    // Operator ruling 4 (2026-09-06, design §7 and §11 item 20): an empty
    // `--name` is refused here, typed, rather than encoded for core to reject
    // after a workspace discovery and a family read.
    let empty_name = refusal(&["clone", "--local", "--name", ""]);
    assert!(empty_name.contains("--name"), "{empty_name}");

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

/// `--remote` keeps its existing meaning on pull -- the token stays in
/// `OperationPolicy.remote` and core resolves it -- and on push it is
/// encoded exactly once, in `PushRequest.remote` (operator ruling 4 of
/// 2026-09-06, design §7 and §11 item 20). Push used to set both fields;
/// core reads the request field first, so only the bytes changed.
#[test]
fn pull_keeps_the_policy_binding_and_push_encodes_the_token_once() {
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

    // Both §7 push rows: the family name and the Git remote encode
    // identically, and neither leaves a second copy in the policy.
    for token in ["hub", "origin"] {
        let CliRequest::Push(push) = parse(&["push", "--remote", token]).unwrap().request else {
            panic!("expected a push request");
        };
        assert_eq!(push.remote.as_deref(), Some(token));
        assert_eq!(
            push.meta.policy.as_ref().and_then(|p| p.remote.clone()),
            None,
            "the push token travels only in PushRequest.remote"
        );
        // Nothing else moved out of the policy with it.
        assert_eq!(
            push.meta
                .policy
                .as_ref()
                .and_then(|policy| policy.progress_min_interval_ms),
            Some(DEFAULT_PROGRESS_MIN_INTERVAL_MS)
        );
    }

    // A push without `--remote` is unchanged: no token in either place.
    let CliRequest::Push(push) = parse(&["push"]).unwrap().request else {
        panic!("expected a push request");
    };
    assert_eq!(push.remote, None);
    assert_eq!(
        push.meta.policy.as_ref().and_then(|p| p.remote.clone()),
        None
    );
}

// ---------------------------------------------------------------------------
// dispatch and refusal presentation
// ---------------------------------------------------------------------------

/// The mutating family verbs reach their core entry point and come back
/// with what core answers today for a workspace in no family (LCM1.1):
/// ordinary `dispose`, with or without a hazard waiver, is still the typed
/// `unsupported_operation` LCM2.1 owes; `dispose --keep` is wired and refuses
/// `member_not_found`, naming the verb and the workspace; `disband` is served
/// as a `Noop` that writes nothing. Until LCM1.1 every one of these answered
/// `unsupported_operation`; these rows moved with core, as they were written
/// to.
#[test]
fn local_family_verbs_dispatch_and_answer_for_a_workspace_in_no_family() {
    let temp = workspace("cli-local-family");

    assert_unsupported(&run_in(&temp, &["local", "dispose", "C"]), "local dispose");
    assert_unsupported(
        &run_in(&temp, &["local", "dispose", "C", "--force", "dirty"]),
        "local dispose",
    );

    let keep = run_in(&temp, &["local", "dispose", "C", "--keep"]);
    assert_eq!(
        keep.code,
        Some(gwz_core::model::ErrorCode::MemberNotFound),
        "{}",
        keep.message
    );
    assert!(
        keep.message.contains("local dispose --keep"),
        "{}",
        keep.message
    );
    assert!(
        keep.message.contains("is in no local family"),
        "{}",
        keep.message
    );
    assert_eq!(
        keep.human_message(),
        format!("MemberNotFound: {}", keep.message)
    );
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&keep)).unwrap();
    assert_eq!(json["errors"][0]["code"], "MemberNotFound");

    let disband = dispatch_in(&temp, &["local", "disband"]).expect("disband outside a family");
    assert_eq!(
        disband.envelope.meta.aggregate_status,
        gwz_core::AggregateStatus::Noop
    );
    assert!(
        disband
            .envelope
            .meta
            .message
            .as_deref()
            .is_some_and(|message| message.contains("nothing to disband")),
        "{:?}",
        disband.envelope.meta.message
    );
    assert_eq!(exit_code_for_response(&disband.envelope), 0);
    assert!(
        !temp.path().join(".gwz/local-family.yml").exists(),
        "a no-op disband founds nothing"
    );
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

/// `gwz clone --local` reaches `handle_clone_local_workspace`. Verbatim is
/// served (LCM1.1): the response names the destination, the recorded path
/// and the dest-complete walk, the tree stands with its pointer and manifest,
/// and the root founds a family. Clean and bare still come back as the typed
/// `unsupported_operation` naming the mode, and `--from` naming the flag.
#[test]
fn clone_local_dispatches_verbatim_and_refuses_the_unbuilt_modes_typed() {
    let temp = workspace("cli-local-clone");
    // A destination this test owns, so a served create never lands beside
    // the temporary directory in the system temp root.
    let lanes = TempDir::new("cli-local-clone-lanes");
    let dest = lanes.path().join("dest-a");
    let dest_arg = dest.to_string_lossy().into_owned();
    let created = dispatch_in(&temp, &["clone", "--local", "--name", "A", &dest_arg])
        .expect("a verbatim local clone is served");
    assert_eq!(
        created.envelope.meta.aggregate_status,
        gwz_core::AggregateStatus::Ok
    );
    let message = created
        .envelope
        .meta
        .message
        .clone()
        .expect("the create reports what it did");
    assert!(message.contains("created local clone `A`"), "{message}");
    assert!(message.contains("dest-complete:"), "{message}");
    assert!(dest.join("gwz.conf/gwz.yml").is_file(), "{message}");
    assert!(dest.join(".gwz/family-root").is_file(), "{message}");
    assert!(
        temp.path().join(".gwz/local-family.yml").is_file(),
        "the root founded a family"
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
    family_response(gwz_core::LocalFamilyOp::List, None, members)
}

/// The same listing, with the observed root the response carries beside the
/// rows (`LocalFamilyResponse.root_path`, design §7 tag 3).
fn rooted_list_response(
    root_path: &str,
    members: Vec<gwz_core::LocalFamilyMemberEntry>,
) -> CliResponse {
    family_response(gwz_core::LocalFamilyOp::List, Some(root_path), members)
}

fn family_response(
    op: gwz_core::LocalFamilyOp,
    root_path: Option<&str>,
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
            // LCM1.0c follow-up 3 (operator ruling 3 of 2026-09-06): the
            // family root's path, joined with each member's root-relative
            // `path` by the renderer. `None` is the listing that has no root
            // to join against, and its column stays root-relative.
            root_path: root_path.map(ToOwned::to_owned),
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

/// Design §8.1's absolute paths are literal, not the driver's guess: the
/// response carries the observed root (`root_path`, tag 3, operator ruling 3
/// of 2026-09-06) and each member's root-relative `path`, and the human table
/// joins the two. A listing run in a clone still names root's own directory,
/// because `root_path` is the index's directory reached through that clone's
/// pointer.
#[test]
fn local_list_joins_the_observed_root_with_each_relative_path() {
    let members = || {
        vec![
            ready("root", gwz_core::LocalMemberKind::Checkout, "."),
            ready("A", gwz_core::LocalMemberKind::Checkout, "../gwz-dev-A"),
            ready("hub", gwz_core::LocalMemberKind::Bare, "../gwz-dev-hub"),
        ]
    };
    let response = rooted_list_response("/Users/gianni/limbo/gwz-dev", members());
    assert_eq!(
        render_response(&response, OutputMode::Human),
        "\
root  checkout  ready  /Users/gianni/limbo/gwz-dev
A     checkout  ready  /Users/gianni/limbo/gwz-dev-A
hub   bare      ready  /Users/gianni/limbo/gwz-dev-hub"
    );

    // No root to join against is not a licence to guess one: the column is
    // the wire's own root-relative path, unchanged.
    assert!(
        render_response(&list_response(members()), OutputMode::Human)
            .contains("A     checkout  ready  ../gwz-dev-A")
    );

    // The join is lexical, never a disk lookup -- a listing is
    // observation-only, and it must name a member whose directory is missing
    // exactly as the index recorded it. A path that is already absolute is
    // left alone, and the alignment is computed on what is printed.
    let response = rooted_list_response(
        "/ws/root",
        vec![
            entry(
                "gone",
                gwz_core::LocalMemberKind::Checkout,
                gwz_core::LocalMemberState::Ready,
                gwz_core::LocalObservedState::Missing,
                "../ws-gone",
                None,
            ),
            ready(
                "elsewhere",
                gwz_core::LocalMemberKind::Checkout,
                "/mnt/ws-e",
            ),
        ],
    );
    assert_eq!(
        render_response(&response, OutputMode::Human),
        "\
gone       checkout  ready/missing  /ws/ws-gone
elsewhere  checkout  ready          /mnt/ws-e"
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
    // Operator ruling 2 (2026-09-06, design §7 and §11 item 18): machine
    // output spells every enum value in the protocol's own snake_case, the
    // same words the human table uses and the same convention the error codes
    // follow — never the generated Rust variant name.
    assert_eq!(
        members[0],
        serde_json::json!({
            "name": "root",
            "kind": "checkout",
            "recorded_state": "ready",
            "observed_state": "ready",
            "path": ".",
            "last_error": null,
        })
    );
    assert_eq!(
        members[1],
        serde_json::json!({
            "name": "B",
            "kind": "checkout",
            "recorded_state": "creating",
            "observed_state": "incomplete",
            "path": "../ws-B",
            "last_error": "copy interrupted at src/",
        })
    );
    // The machine spelling is the human column's word, not a second mapping:
    // a variant that gained a `{:?}` rendering again would show up here.
    for member in members {
        for key in ["kind", "recorded_state", "observed_state"] {
            let value = member[key].as_str().unwrap();
            assert!(
                !value.contains(|letter: char| letter.is_ascii_uppercase()),
                "`{key}` must be the protocol's snake_case word, got `{value}`"
            );
        }
    }

    // Both wire fields travel faithfully (operator ruling 3 of 2026-09-06):
    // `path` stays root-relative, exactly as the response carries it, and the
    // root travels once beside the rows rather than being folded into every
    // row. Only the human table joins them.
    assert!(
        json["local_family_root_path"].is_null(),
        "a listing with no observed root must not invent one: {json}"
    );
    let rooted = rooted_list_response(
        "/ws/root",
        vec![ready("A", gwz_core::LocalMemberKind::Checkout, "../ws-A")],
    );
    let rooted_json: serde_json::Value =
        serde_json::from_str(&render_response(&rooted, OutputMode::Json)).unwrap();
    assert_eq!(rooted_json["local_family_root_path"], "/ws/root");
    assert_eq!(rooted_json["local_family_members"][0]["path"], "../ws-A");

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
        let mutation = family_response(op, None, Vec::new());
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
// the four local-create codes (LCM1.1 fix 1: 63-66)
// ---------------------------------------------------------------------------

/// LCM1.1 fix 1 (lane C, 2026-09-06): a design §4.0 source-layout hazard, a
/// stopped copy, a moved source and an incomplete destination are their own
/// codes -- no longer folded into `unsupported_operation` and `io_error` --
/// and the driver presents each like every other typed refusal: the code
/// prefixes the human line, `--json`/`--jsonl` carry it structured, and
/// core's message travels unedited. Nothing here is code the driver chose:
/// the labels are the model enum's own names, as for `UnknownLocal`.
#[test]
fn the_four_local_create_codes_are_presented_as_typed_refusals() {
    use gwz_core::model::ErrorCode;
    for (code, wire, label, message) in [
        (
            ErrorCode::UnsupportedSourceLayout,
            63,
            "UnsupportedSourceLayout",
            "local clone `A` -> /ws-A: inventory source failed: /ws/app: unsupported layout: \
             Alternates; nothing was reserved",
        ),
        (
            ErrorCode::CopyFailed,
            64,
            "CopyFailed",
            "local clone `A` -> /ws-A: copy tree failed: copy failed at app/locked.txt: \
             SourceUnreadable: Permission denied; effects: [RowAllocated, \
             DestinationAllocated, ErrorRecorded]; the `creating` row `A` and /ws-A are \
             retained for inspection",
        ),
        (
            ErrorCode::SourceDrift,
            65,
            "SourceDrift",
            "local clone `A` -> /ws-A: recheck source failed: source drift: repository @root \
             changed; effects: [RowAllocated, DestinationAllocated, TreeCopied, \
             DestinationGitInstalled, PointerInstalled, ErrorRecorded]",
        ),
        (
            ErrorCode::DestinationIncomplete,
            66,
            "DestinationIncomplete",
            "local clone `A` -> /ws-A: check destination failed: destination is incomplete: \
             mem_app: objects missing from the destination store (7 objects in the store, \
             3 roots): 3f6b4a59 (below 8ec04f1d)",
        ),
    ] {
        let error =
            CliError::from_model(gwz_core::model::ModelError::new(code, message.to_owned()));
        assert_eq!(error.code, Some(code), "{label}");
        assert_eq!(
            error.human_message(),
            format!("{label}: {message}"),
            "the human line names the code, like every other typed refusal"
        );
        let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
        assert_eq!(json["errors"][0]["code"], label);
        assert_eq!(json["errors"][0]["message"], message);
        // The wire value, and the two codes it is no longer folded into.
        let wired = gwz_core::GwzErrorCode::from(code);
        assert_eq!(wired.wire(), wire, "{label}");
        for folded in [ErrorCode::UnsupportedOperation, ErrorCode::IoError] {
            assert_ne!(wired, gwz_core::GwzErrorCode::from(folded), "{label}");
        }
    }
}

/// LCM1.2 (lane C, 2026-09-06): the two family-merge import outcomes,
/// `pairing_mismatch` (67, refused before any fetch, nothing written) and
/// `import_incomplete` (68, the import stopped before the engine; the
/// retained import refs travel in core's message), presented like every
/// other typed refusal and distinct from the codes they would have folded
/// into. No rendering code changed: the labels are the model enum's names.
#[test]
fn the_two_family_merge_import_codes_are_presented_as_typed_refusals() {
    use gwz_core::model::ErrorCode;
    for (code, wire, label, message) in [
        (
            ErrorCode::PairingMismatch,
            67,
            "PairingMismatch",
            "local family merge from `A`: import of HEAD from `A` as \
             refs/gwz/local-imports/xfer_0123456789abcdef0123456789abcdef failed: import \
             pairing is incomplete; unpaired: mem_extra; no import ref was created; nothing \
             was written; the merge engine was not entered; a retry mints a fresh transfer id",
        ),
        (
            ErrorCode::ImportIncomplete,
            68,
            "ImportIncomplete",
            "local family merge from `A`: import of HEAD from `A` as \
             refs/gwz/local-imports/xfer_0123456789abcdef0123456789abcdef failed: mem_lib: \
             transfer failed: /ws/lib: failed to update refs; retained import refs (ordinary \
             Git refs, never pruned by gwz): mem_app \
             refs/gwz/local-imports/xfer_0123456789abcdef0123456789abcdef = \
             2f44c2f5f825e785ae19c1b85bc358fb4e89d61d; the merge engine was not entered; a \
             retry mints a fresh transfer id",
        ),
    ] {
        let error =
            CliError::from_model(gwz_core::model::ModelError::new(code, message.to_owned()));
        assert_eq!(error.code, Some(code), "{label}");
        assert_eq!(error.human_message(), format!("{label}: {message}"));
        let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
        assert_eq!(json["errors"][0]["code"], label);
        assert_eq!(json["errors"][0]["message"], message);
        let wired = gwz_core::GwzErrorCode::from(code);
        assert_eq!(wired.wire(), wire, "{label}");
        for folded in [
            ErrorCode::InvalidRequest,
            ErrorCode::MemberNotFound,
            ErrorCode::GitCommandFailed,
            ErrorCode::IoError,
        ] {
            assert_ne!(wired, gwz_core::GwzErrorCode::from(folded), "{label}");
        }
    }
}

/// The first of the four, end to end through the driver on a real workspace:
/// a source repository borrowing another object store (design §4.0
/// "objects/info/alternates") refuses `gwz clone --local` as
/// `unsupported_source_layout` before anything is reserved -- not as
/// `unsupported_operation`, which now means only "not built yet".
#[test]
fn a_source_layout_hazard_reaches_the_driver_as_unsupported_source_layout() {
    let temp = TempDir::new("cli-local-hazard");
    git2::Repository::init(temp.path()).expect("a root repository");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: super::g01::request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_hazard".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    let info = temp.path().join(".git/objects/info");
    std::fs::create_dir_all(&info).unwrap();
    std::fs::write(
        info.join("alternates"),
        format!("{}\n", temp.path().join("borrowed").display()),
    )
    .unwrap();

    let error = run_in(&temp, &["clone", "--local", "--name", "A"]);
    assert_eq!(
        error.code,
        Some(gwz_core::model::ErrorCode::UnsupportedSourceLayout),
        "{}",
        error.message
    );
    assert!(error.message.contains("Alternates"), "{}", error.message);
    assert!(
        error.message.contains("nothing was reserved"),
        "{}",
        error.message
    );
    assert_eq!(
        error.human_message(),
        format!("UnsupportedSourceLayout: {}", error.message)
    );
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
    assert_eq!(json["errors"][0]["code"], "UnsupportedSourceLayout");
    assert!(
        !temp.path().join(".gwz/local-family.yml").exists(),
        "refused before reservation: no family index was founded"
    );
}

// ---------------------------------------------------------------------------
// the shared listing-path rendering fixture (LCM1.1 fix 3)
// ---------------------------------------------------------------------------

/// The cross-driver rendering fixture for the `gwz local list` path column,
/// beside the argv fixture in gwz-core (LCM1.1 fix 3, lane C, 2026-09-06):
/// every case is a `root_path`, a member `path` and the display path BOTH
/// drivers must render. This driver's join is `local_list_render::member_path`;
/// gwz-py asserts the same file against its own. A disagreement is a finding,
/// never a reason to bend the expected value.
const LISTING_FIXTURE: &str =
    include_str!("../../../gwz-core/protocol/fixtures/cli_parity/local_family_listing_cases.json");

/// The shapes the fixture must cover (the fix 3 brief), by case id.
const REQUIRED_LISTING_CASES: [&str; 7] = [
    "plain-child",
    "sibling-through-parent",
    "nested-member",
    "two-level-escape",
    "absent-root",
    "empty-root",
    "already-absolute-member-path",
];

#[test]
fn the_listing_fixture_cases_render_the_expected_display_path() {
    let document: serde_json::Value = serde_json::from_str(LISTING_FIXTURE).unwrap();
    let cases = document["cases"].as_array().expect("`cases` is a list");
    let mut ids: Vec<&str> = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("a case has an id");
        assert!(!ids.contains(&id), "case ids are unique: {id}");
        ids.push(id);
        // `root_path` is a string or null, exactly as the wire carries it.
        let root_path = match &case["root_path"] {
            serde_json::Value::Null => None,
            serde_json::Value::String(root) => Some(root.as_str()),
            other => panic!("{id}: root_path is a string or null, got {other}"),
        };
        let path = case["path"].as_str().expect("a case has a path");
        let expected = case["expected"]
            .as_str()
            .expect("a case has an expected path");
        // The join is lexical and never touches a filesystem, so the
        // fixture's paths need not exist. `expected` is spelled with `/`; a
        // host whose separator differs is compared after mapping it
        // (fixture `_schema.separators`).
        let rendered = crate::local_list_render::member_path(root_path, path).replace('\\', "/");
        assert_eq!(
            rendered,
            expected,
            "{id}: {}",
            case["note"].as_str().unwrap_or("")
        );
    }
    for required in REQUIRED_LISTING_CASES {
        assert!(ids.contains(&required), "the fixture covers {required}");
    }
}

// ---------------------------------------------------------------------------
// the shared argv -> request parity fixture
// ---------------------------------------------------------------------------

/// The one cross-driver parity fixture, and it lives in `gwz-core` beside the
/// merge fixtures (operator ruling 1 of 2026-09-06; design §7 and §11 item 17):
/// both drivers read *this* file, neither restates its cases inline and
/// neither keeps a copy, so a case is added there rather than here. The
/// sibling path is the one `gwz-core` already occupies as this crate's path
/// dependency, and `src/tests/g02.rs` already reads two fixtures across it.
const PARITY_FIXTURE: &str =
    include_str!("../../../gwz-core/protocol/fixtures/cli_parity/local_family_cases.json");

/// This driver's name in the fixture's `drivers` scoping and in the
/// per-driver `driver_message_contains` needles.
const DRIVER: &str = "rust";

/// The fixture's projection of a built request: `message` names the type and
/// every field a case can address by a dotted path is rendered, with optionals
/// spelled `null` rather than omitted.
fn request_document(request: &CliRequest) -> serde_json::Value {
    use serde_json::json;
    // A request that carries no policy object at all renders `policy: null`,
    // so `meta.policy.remote` resolves to null through the absent
    // intermediate — exactly what the fixture's dotted paths mean.
    let meta_json = |meta: &gwz_core::RequestMeta| {
        json!({
            "dry_run": meta.dry_run,
            "policy": meta
                .policy
                .as_ref()
                .map(|policy| json!({ "remote": policy.remote })),
        })
    };
    match request {
        CliRequest::CloneLocalWorkspace(request) => json!({
            "message": "CloneLocalWorkspaceRequest",
            "name": request.name,
            "dest": request.dest,
            "mode": match request.mode {
                gwz_core::LocalCloneMode::Verbatim => "verbatim",
                gwz_core::LocalCloneMode::Clean => "clean",
                gwz_core::LocalCloneMode::Bare => "bare",
            },
            "branch": request.branch,
            "copy_source": request.copy_source,
            "meta": meta_json(&request.meta),
        }),
        CliRequest::LocalFamily(request) => json!({
            "message": "LocalFamilyRequest",
            "op": match request.op {
                gwz_core::LocalFamilyOp::List => "list",
                gwz_core::LocalFamilyOp::Dispose => "dispose",
                gwz_core::LocalFamilyOp::Disband => "disband",
            },
            "name": request.name,
            "keep": request.keep,
            "force_hazards": request.force_hazards,
            "meta": meta_json(&request.meta),
        }),
        CliRequest::Merge(request) => json!({
            "message": "MergeRequest",
            "op": format!("{:?}", request.op).to_lowercase(),
            "source_ref": request.source_ref,
            "local_source_name": request.local_source_name,
            "meta": meta_json(&request.meta),
        }),
        CliRequest::PullHead(request) => json!({
            "message": "PullHeadRequest",
            "meta": meta_json(&request.meta),
        }),
        CliRequest::Push(request) => json!({
            "message": "PushRequest",
            "remote": request.remote,
            "meta": meta_json(&request.meta),
        }),
        other => panic!("the parity fixture covers no request of this kind: {other:?}"),
    }
}

/// The fixture's dotted field paths. A path resolves through an absent
/// (`null`) intermediate to `null`, but its *first* segment must exist in the
/// projection above, so a case naming a field this driver never renders fails
/// loudly instead of quietly matching a `null`.
fn field(document: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut segments = path.split('.');
    let first = segments.next().expect("a field path is never empty");
    let mut current = document.get(first).unwrap_or_else(|| {
        panic!("the parity projection renders no `{first}` field: {document}");
    });
    for segment in segments {
        match current.get(segment) {
            Some(value) => current = value,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

/// `drivers` scopes a case to the drivers that can express it; the default is
/// both. One refusal is `rust`-only (`local dispose C dirty`, which argparse
/// rejects as an unrecognized operand before gwz-py's handler runs).
fn applies_here(case: &serde_json::Value) -> bool {
    match case.get("drivers") {
        None => true,
        Some(drivers) => drivers
            .as_array()
            .expect("`drivers` is a list")
            .iter()
            .any(|driver| driver == DRIVER),
    }
}

fn argv_of(case: &serde_json::Value) -> Vec<&str> {
    case["argv"]
        .as_array()
        .expect("`argv` is a list")
        .iter()
        .map(|item| item.as_str().expect("argv holds strings"))
        .collect()
}

#[test]
fn the_shared_parity_fixture_pins_every_design_row_argv_to_request() {
    let fixture: serde_json::Value = serde_json::from_str(PARITY_FIXTURE).unwrap();

    let mut cases = 0;
    for case in fixture["message_cases"].as_array().unwrap() {
        if !applies_here(case) {
            continue;
        }
        cases += 1;
        let id = case["id"].as_str().unwrap();
        let argv = argv_of(case);
        let invocation = parse(&argv).unwrap_or_else(|error| {
            panic!("{id}: {argv:?} must parse, got {}", error.message);
        });
        let document = request_document(&invocation.request);
        assert_eq!(
            document["message"], case["message"],
            "{id}: {argv:?} built the wrong message"
        );
        for (path, expected) in case["fields"].as_object().expect("`fields` is an object") {
            assert_eq!(&field(&document, path), expected, "{id}: {argv:?} `{path}`");
        }
    }

    let mut refusals = 0;
    for case in fixture["message_refusals"].as_array().unwrap() {
        if !applies_here(case) {
            continue;
        }
        refusals += 1;
        let id = case["id"].as_str().unwrap();
        assert_eq!(case["refused_by"], "cli", "{id}: not this driver's refusal");
        let argv = argv_of(case);
        // The fixture pins that the driver refuses before encoding, and the
        // wording; the typed `InvalidRequest` code the family surface answers
        // with is pinned by this module's own refusal cases through
        // `refusal()`. `url-clone-dry-run` is the pre-existing plain usage
        // error `Cli::validate` has always raised, and this lane does not
        // re-type it.
        let message = match parse(&argv) {
            Ok(invocation) => panic!(
                "{id}: {argv:?} must refuse before encoding, it built {:?}",
                invocation.request
            ),
            Err(error) => error.message,
        };
        // The shared needles both drivers' wordings carry, plus this driver's
        // own, where the two messages say the same thing differently.
        let mut needles: Vec<&str> = case["message_contains"]
            .as_array()
            .expect("`message_contains` is a list")
            .iter()
            .map(|needle| needle.as_str().expect("needles are strings"))
            .collect();
        if let Some(mine) = case
            .get("driver_message_contains")
            .and_then(|per_driver| per_driver.get(DRIVER))
        {
            needles.extend(
                mine.as_array()
                    .expect("`driver_message_contains` holds lists")
                    .iter()
                    .map(|needle| needle.as_str().expect("needles are strings")),
            );
        }
        assert!(
            !needles.is_empty(),
            "{id}: a refusal with no needle pins nothing"
        );
        for needle in needles {
            assert!(
                message.contains(needle),
                "{id}: {argv:?} must name `{needle}`, got: {message}"
            );
        }
    }

    // The fixture is the §7 table, not a sample of it. `gwz-core` owns the
    // file and may append to it (a python-only case would not raise these
    // counts), so the pin is the floor this driver ran at follow-up 3: every
    // one of the 30 message cases and all 23 refusals are this driver's.
    assert!(
        cases >= 30,
        "a §7 message case stopped running here: {cases}"
    );
    assert!(
        refusals >= 23,
        "a refusal case stopped running here: {refusals}"
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
        // The binding behind the name is core's and is not landed, so the page
        // must not read as though it already works.
        assert!(page.contains("`missing_remote` in this build"));
    }
}
