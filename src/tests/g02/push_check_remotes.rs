//! Push plan step 3.7 (`dev-docs/GwzUrlSchemePushPlan.md`): `gwz push
//! --check-remotes` and the human rendering of push `Noop` rows. Core does not
//! classify repositories until step 3.5, so the responses here are synthetic.

use std::path::Path;

use clap::CommandFactory;

use super::*;

const UP_TO_DATE: &str = "up to date with origin/main as of the last fetch or push";
const BEHIND: &str = "behind origin/main as of the last fetch or push";
const ON_ORIGIN: &str = "already on origin";
const ONE_UNCHECKED: &str = "1 repository unchanged since the last fetch or push was not checked for changes; --check-remotes to verify";
const TWO_UNCHECKED: &str = "2 repositories unchanged since the last fetch or push were not checked for changes; --check-remotes to verify";

fn parse_push(args: &[&str]) -> Result<CliInvocation, CliError> {
    let owned = args.iter().map(|item| (*item).to_owned()).collect();
    parse_args_with_request_id(owned, "req_push", Path::new("/cwd"))
}

fn push_request(args: &[&str]) -> gwz_core::PushRequest {
    match parse_push(args).expect("parses").request {
        CliRequest::Push(request) => request,
        other => panic!("expected a push request, got {other:?}"),
    }
}

fn noop_row(member_id: &str, member_path: &str, reason: &str) -> gwz_core::MemberResponse {
    gwz_core::MemberResponse {
        member_id: member_id.to_owned(),
        member_path: member_path.to_owned(),
        source_kind: gwz_core::SourceKind::Git,
        status: gwz_core::MemberStatus::Noop,
        error: None,
        planned: Some(gwz_core::PlannedChange {
            action: gwz_core::PlannedAction::Noop,
            from_ref: None,
            to_ref: None,
            message: Some(reason.to_owned()),
        }),
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

fn rows_response(
    action: gwz_core::ActionKind,
    aggregate_status: gwz_core::AggregateStatus,
    members: Vec<gwz_core::MemberResponse>,
) -> CliResponse {
    CliResponse::envelope(gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            transport: None,
            request_id: "req_push".to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            action,
            aggregate_status,
            operation_id: Some("op_push".to_owned()),
            message: None,
            attribution: None,
        },
        members,
        errors: Vec::new(),
    })
}

#[test]
fn check_remotes_maps_to_always_and_is_otherwise_unset() {
    let plain = push_request(&["push"]);
    assert_eq!(plain.remote_check, None);
    assert_eq!(plain.refspec, None);

    let checked = push_request(&["push", "--check-remotes"]);
    assert_eq!(checked.remote_check, Some(gwz_core::RemoteCheck::Always));
    assert_eq!(checked.refspec, None);

    let combined = push_request(&["--dry-run", "--remote", "origin", "push", "--check-remotes"]);
    assert_eq!(combined.remote_check, Some(gwz_core::RemoteCheck::Always));
    assert_eq!(combined.remote.as_deref(), Some("origin"));
    assert_eq!(combined.meta.dry_run, Some(true));
}

/// Plan §2.1 and §3.5: the global `--force` allows destructive behaviour. It
/// is not a forced push and does not change which remotes are checked.
#[test]
fn force_keeps_a_plain_refspec_and_leaves_remote_check_unset() {
    for args in [["--force", "push"], ["push", "--force"]] {
        let request = push_request(&args);
        assert_eq!(request.refspec, None, "{args:?}");
        assert_eq!(request.remote_check, None, "{args:?}");
        assert_eq!(
            request.meta.policy.and_then(|policy| policy.destructive),
            Some(gwz_core::DestructiveBehavior::Allow),
            "{args:?}"
        );
    }
    let both = push_request(&["push", "--force", "--check-remotes"]);
    assert_eq!(both.refspec, None);
    assert_eq!(both.remote_check, Some(gwz_core::RemoteCheck::Always));
}

#[test]
fn check_remotes_belongs_to_push_alone() {
    for args in [
        vec!["--check-remotes", "push"],
        vec!["pull", "--check-remotes"],
        vec!["tag", "--push", "--check-remotes"],
    ] {
        assert!(parse_push(&args).is_err(), "{args:?} must not parse");
    }
}

#[test]
fn push_help_names_the_option_and_how_publication_is_checked() {
    let command = Cli::command();
    let push = command.find_subcommand("push").expect("push command");
    let option = push
        .get_arguments()
        .find(|arg| arg.get_long() == Some("check-remotes"))
        .expect("--check-remotes");
    assert!(!option.is_global_set());
    assert_eq!(
        option.get_help().map(ToString::to_string).as_deref(),
        Some(
            "Read every selected remote and every root dependency instead of skipping repositories that are unchanged since the last fetch or push"
        )
    );
    // Compare words, not the help's line wrapping.
    let about = push
        .get_long_about()
        .map(|about| {
            about
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    for phrase in [
        "accepted pushes",
        "not checked for changes or pushed",
        "still reads each dependency",
        "`--check-remotes` reads",
    ] {
        assert!(
            about.contains(phrase),
            "push help lacks `{phrase}`:\n{about}"
        );
    }

    let page = include_str!("../../../docs/commands/push.md");
    assert!(page.contains("gwz push --check-remotes"));
    assert!(!page.contains("has no command-specific options"));
}

#[test]
fn human_push_output_counts_unchecked_rows_and_verbose_shows_every_reason() {
    let response = rows_response(
        gwz_core::ActionKind::Push,
        gwz_core::AggregateStatus::Noop,
        vec![
            noop_row("@root", ".", UP_TO_DATE),
            noop_row("mem_app", "repos/app", BEHIND),
            noop_row("mem_lib", "repos/lib", ON_ORIGIN),
        ],
    );
    assert_eq!(
        render_response(&response, OutputMode::Human),
        format!(
            "status: Noop\n@root . Noop\nmem_app repos/app Noop\nmem_lib repos/lib Noop\n{TWO_UNCHECKED}"
        )
    );
    assert_eq!(
        render_response_with_transport(&response, OutputMode::Human, true),
        format!(
            "status: Noop\n@root . Noop {UP_TO_DATE}\nmem_app repos/app Noop {BEHIND}\nmem_lib repos/lib Noop {ON_ORIGIN}\n{TWO_UNCHECKED}"
        )
    );
}

#[test]
fn one_unchecked_row_reads_singular_and_checked_rows_add_no_summary() {
    let one = rows_response(
        gwz_core::ActionKind::Push,
        gwz_core::AggregateStatus::Noop,
        vec![
            noop_row("@root", ".", ON_ORIGIN),
            noop_row("mem_app", "repos/app", UP_TO_DATE),
        ],
    );
    assert_eq!(
        render_response(&one, OutputMode::Human),
        format!("status: Noop\n@root . Noop\nmem_app repos/app Noop\n{ONE_UNCHECKED}")
    );

    // `--check-remotes` reads every destination, so no row is unchecked.
    let checked = rows_response(
        gwz_core::ActionKind::Push,
        gwz_core::AggregateStatus::Noop,
        vec![
            noop_row("@root", ".", ON_ORIGIN),
            noop_row("mem_app", "repos/app", ON_ORIGIN),
        ],
    );
    assert_eq!(
        render_response(&checked, OutputMode::Human),
        "status: Noop\n@root . Noop\nmem_app repos/app Noop"
    );
}

#[test]
fn rows_other_than_push_noops_render_as_before() {
    // A push dry run's planned row keeps its message without `--verbose`.
    let mut planned = noop_row("mem_app", "repos/app", "push to origin");
    planned.status = gwz_core::MemberStatus::Planned;
    if let Some(change) = &mut planned.planned {
        change.action = gwz_core::PlannedAction::Push;
    }
    let dry_run = rows_response(
        gwz_core::ActionKind::Push,
        gwz_core::AggregateStatus::Ok,
        vec![planned],
    );
    assert_eq!(
        render_response(&dry_run, OutputMode::Human),
        "status: Ok\nmem_app repos/app Planned push to origin"
    );

    // Another command's `Noop` row keeps its message and earns no summary,
    // even when the message ends with the push phrase.
    let other = rows_response(
        gwz_core::ActionKind::Materialize,
        gwz_core::AggregateStatus::Noop,
        vec![noop_row("mem_app", "repos/app", UP_TO_DATE)],
    );
    assert_eq!(
        render_response(&other, OutputMode::Human),
        format!("status: Noop\nmem_app repos/app Noop {UP_TO_DATE}")
    );
}

#[test]
fn machine_output_carries_each_reason_without_the_summary_line() {
    let response = rows_response(
        gwz_core::ActionKind::Push,
        gwz_core::AggregateStatus::Noop,
        vec![noop_row("mem_app", "repos/app", BEHIND)],
    );
    let rendered = render_response(&response, OutputMode::Json);
    let json: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(json["meta"]["aggregate_status"], "Noop");
    assert_eq!(json["members"][0]["status"], "Noop");
    assert_eq!(json["members"][0]["planned"]["action"], "Noop");
    assert_eq!(json["members"][0]["planned"]["message"], BEHIND);
    assert!(!rendered.contains("not checked for changes"), "{rendered}");
}

#[test]
fn only_reasons_decided_without_a_read_count_as_unchecked() {
    assert!(push_reason_is_unchecked(UP_TO_DATE));
    assert!(push_reason_is_unchecked(
        "behind origin/release as of the last fetch or push"
    ));
    assert!(!push_reason_is_unchecked(ON_ORIGIN));
}
