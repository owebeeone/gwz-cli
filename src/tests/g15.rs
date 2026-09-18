//! `gwz fetch` (gwz-cli dev-docs/GwzFetchPlan.md step 1.4): the verb parses,
//! `--target` reaches the request's selection, the human report is one line per
//! repository, and `--json` carries the structured rows.

use std::path::Path;

use super::*;

fn parse_fetch(args: &[&str]) -> Result<CliInvocation, CliError> {
    let owned = args.iter().map(|item| (*item).to_owned()).collect();
    parse_args_with_request_id(owned, "req_fetch", Path::new("/cwd"))
}

fn fetch_request(args: &[&str]) -> gwz_core::FetchRequest {
    match parse_fetch(args).expect("parses").request {
        CliRequest::Fetch(request) => request,
        other => panic!("expected a fetch request, got {other:?}"),
    }
}

fn row(
    member_id: &str,
    member_path: &str,
    result: gwz_core::FetchResult,
) -> gwz_core::FetchRepoSummary {
    gwz_core::FetchRepoSummary {
        member_id: member_id.to_owned(),
        member_path: member_path.to_owned(),
        source_kind: gwz_core::SourceKind::Git,
        result,
        remote: Some("origin".to_owned()),
        branch: Some("main".to_owned()),
        before: None,
        after: None,
        upstream: Some("refs/remotes/origin/main".to_owned()),
        ahead: Some(0),
        behind: Some(0),
    }
}

fn member(
    member_id: &str,
    member_path: &str,
    status: gwz_core::MemberStatus,
) -> gwz_core::MemberResponse {
    gwz_core::MemberResponse {
        member_id: member_id.to_owned(),
        member_path: member_path.to_owned(),
        source_kind: gwz_core::SourceKind::Git,
        status,
        error: None,
        planned: None,
        state: None,
        git_status: None,
        lock_match: None,
        target_kind: Some(if member_id == "@root" {
            gwz_core::TargetKind::Root
        } else {
            gwz_core::TargetKind::Member
        }),
        lock_difference_reasons: None,
        url_resolution: None,
    }
}

fn fetch_response(
    aggregate_status: gwz_core::AggregateStatus,
    members: Vec<gwz_core::MemberResponse>,
    repos: Vec<gwz_core::FetchRepoSummary>,
) -> CliResponse {
    CliResponse::fetch(gwz_core::FetchResponse {
        response: gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                transport: None,
                request_id: "req_fetch".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Fetch,
                aggregate_status,
                operation_id: Some("op_fetch".to_owned()),
                message: None,
                attribution: None,
            },
            members,
            errors: Vec::new(),
        },
        repos: Some(repos),
    })
}

#[test]
fn fetch_parses_and_takes_no_options_of_its_own() {
    let plain = fetch_request(&["fetch"]);
    assert_eq!(plain.meta.selection, None);
    assert_eq!(plain.meta.dry_run, None);

    // Plan D4: push's knob is deliberately absent, so it must not parse here.
    assert!(parse_fetch(&["fetch", "--check-remotes"]).is_err());
    assert!(parse_fetch(&["fetch", "--prune"]).is_err(), "Phase 2");
}

/// `--target` semantics are push's, and `--remote` rides in the policy rather
/// than on the request, because `FetchRequest` has no `remote` field.
#[test]
fn selection_and_remote_reach_the_request() {
    let one = fetch_request(&["--target", "mem_app", "fetch"]);
    let selection = one.meta.selection.expect("a selection");
    assert_eq!(selection.targets, ["mem_app"]);
    assert!(selection.exclude_targets.is_empty());

    let members_only = fetch_request(&["--all", "--no-target", "@root", "fetch"]);
    let selection = members_only.meta.selection.expect("a selection");
    assert_eq!(selection.targets, ["@all"]);
    assert_eq!(selection.exclude_targets, ["@root"]);

    let remote = fetch_request(&["--remote", "upstream", "fetch"]);
    assert_eq!(
        remote
            .meta
            .policy
            .and_then(|policy| policy.remote)
            .as_deref(),
        Some("upstream")
    );

    let dry = fetch_request(&["--dry-run", "fetch"]);
    assert_eq!(dry.meta.dry_run, Some(true));
}

/// One line per repository (plan §3.3): what the tracking ref did, then the
/// ref it tracks with `+ahead -behind`. A `no upstream` row carries neither.
#[test]
fn the_human_report_is_one_line_per_repository() {
    let mut moved = row("mem_core", "gwz-core", gwz_core::FetchResult::Updated);
    moved.before = Some("a1b2c3d4e5f60718293a4b5c6d7e8f9012345678".to_owned());
    moved.after = Some("9f8e7d6c5b4a39281706f5e4d3c2b1a098765432".to_owned());
    moved.behind = Some(3);
    let mut local = row("mem_local", "local", gwz_core::FetchResult::NoUpstream);
    local.remote = None;
    local.branch = None;
    local.upstream = None;
    local.ahead = None;
    local.behind = None;

    let rendered = render_response(
        &fetch_response(
            gwz_core::AggregateStatus::Ok,
            vec![
                member("@root", ".", gwz_core::MemberStatus::Noop),
                member("mem_core", "gwz-core", gwz_core::MemberStatus::Ok),
                member("mem_local", "local", gwz_core::MemberStatus::Noop),
            ],
            vec![
                row("@root", ".", gwz_core::FetchResult::Unchanged),
                moved,
                local,
            ],
        ),
        OutputMode::Human,
    );

    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines[0], "status: Ok");
    assert_eq!(lines.len(), 4, "one line per repository: {rendered}");
    assert!(
        lines[1].starts_with("@root"),
        "the root is the first row: {rendered}"
    );
    assert!(lines[1].contains("no change"), "{rendered}");
    assert!(lines[1].contains("(origin/main, +0 -0)"), "{rendered}");
    assert!(
        lines[2].contains("a1b2c3d..9f8e7d6"),
        "abbreviated to seven, as git renders one: {rendered}"
    );
    assert!(lines[2].contains("(origin/main, +0 -3)"), "{rendered}");
    assert!(lines[3].contains("no upstream"), "{rendered}");
    assert!(
        !lines[3].contains('('),
        "a no-upstream row tracks nothing to count against: {rendered}"
    );
}

/// A failed row says which repository and why, on its own line, and the
/// aggregate says the report is incomplete.
#[test]
fn a_failed_row_carries_its_reason() {
    let mut failed = member("mem_broken", "broken", gwz_core::MemberStatus::Failed);
    failed.error = Some(gwz_core::GwzError {
        code: gwz_core::model::ErrorCode::RemoteRejected.into(),
        message: "connection refused".to_owned(),
        member_id: Some("mem_broken".to_owned()),
        member_path: Some("broken".to_owned()),
        ..Default::default()
    });
    let mut broken_row = row("mem_broken", "broken", gwz_core::FetchResult::Failed);
    broken_row.upstream = None;
    broken_row.ahead = None;
    broken_row.behind = None;

    let response = fetch_response(
        gwz_core::AggregateStatus::Partial,
        vec![
            member("mem_good", "good", gwz_core::MemberStatus::Noop),
            failed,
        ],
        vec![
            row("mem_good", "good", gwz_core::FetchResult::Unchanged),
            broken_row,
        ],
    );
    let rendered = render_response(&response, OutputMode::Human);
    assert!(rendered.starts_with("status: Partial"), "{rendered}");
    assert!(rendered.contains("failed"), "{rendered}");
    assert!(rendered.contains("connection refused"), "{rendered}");
    assert_eq!(exit_code_for_response(&response.envelope), 1);
}

/// `--json` carries the structured rows under `fetch_repos`, and they appear
/// on a fetch response only -- the envelope's pinned key set is untouched.
#[test]
fn json_carries_the_structured_rows() {
    let mut moved = row("mem_core", "gwz-core", gwz_core::FetchResult::Updated);
    moved.before = Some("a1b2c3d4e5f60718293a4b5c6d7e8f9012345678".to_owned());
    moved.after = Some("9f8e7d6c5b4a39281706f5e4d3c2b1a098765432".to_owned());
    moved.behind = Some(3);

    let rendered = render_response(
        &fetch_response(
            gwz_core::AggregateStatus::Ok,
            vec![member("mem_core", "gwz-core", gwz_core::MemberStatus::Ok)],
            vec![moved],
        ),
        OutputMode::Json,
    );
    let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
    assert_eq!(value["meta"]["action"], "Fetch");
    let repos = value["fetch_repos"].as_array().expect("fetch_repos rows");
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0]["member_id"], "mem_core");
    assert_eq!(repos[0]["result"], "Updated");
    assert_eq!(repos[0]["remote"], "origin");
    assert_eq!(repos[0]["branch"], "main");
    assert_eq!(
        repos[0]["before"],
        "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678"
    );
    assert_eq!(
        repos[0]["after"],
        "9f8e7d6c5b4a39281706f5e4d3c2b1a098765432"
    );
    assert_eq!(repos[0]["upstream"], "refs/remotes/origin/main");
    assert_eq!(repos[0]["ahead"], 0);
    assert_eq!(repos[0]["behind"], 3);

    // A non-fetch response must not grow the key.
    let other = render_response(
        &CliResponse::envelope(gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                transport: None,
                request_id: "req_status".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Status,
                aggregate_status: gwz_core::AggregateStatus::Ok,
                operation_id: Some("op_status".to_owned()),
                message: None,
                attribution: None,
            },
            members: Vec::new(),
            errors: Vec::new(),
        }),
        OutputMode::Json,
    );
    let other: serde_json::Value = serde_json::from_str(&other).expect("valid json");
    assert!(other.get("fetch_repos").is_none());
}
