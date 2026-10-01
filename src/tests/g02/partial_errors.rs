//! A `Partial` result repeats each failed member's error in `errors` for
//! machine readers (TR2.3; docs/MachineOutput.md, "Partial results"). The human
//! reports print no copy, so they read exactly as they did before.

use gwz_core::MemberStatus::{Failed, Rejected};

use super::*;

fn row(
    member_id: &str,
    member_path: &str,
    status: gwz_core::MemberStatus,
    error: Option<&str>,
) -> gwz_core::MemberResponse {
    let target_kind = if member_id == "@root" {
        gwz_core::TargetKind::Root
    } else {
        gwz_core::TargetKind::Member
    };
    gwz_core::MemberResponse {
        member_id: member_id.to_owned(),
        member_path: member_path.to_owned(),
        status,
        error: error.map(|message| gwz_core::GwzError {
            code: gwz_core::GwzErrorCode::RemoteRejected,
            message: message.to_owned(),
            member_id: Some(member_id.to_owned()),
            member_path: Some(member_path.to_owned()),
            target_kind: Some(target_kind),
            ..Default::default()
        }),
        target_kind: Some(target_kind),
        ..Default::default()
    }
}

/// A `Partial` envelope as core builds one: each error on a row is copied
/// into `errors`.
fn partial(
    action: gwz_core::ActionKind,
    members: Vec<gwz_core::MemberResponse>,
) -> gwz_core::ResponseEnvelope {
    let errors = members.iter().filter_map(|row| row.error.clone()).collect();
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            action,
            aggregate_status: gwz_core::AggregateStatus::Partial,
            ..Default::default()
        },
        members,
        errors,
    }
}

/// The human report with the copies, and without them.
fn human_with_and_without_copies(mut response: CliResponse) -> (String, String) {
    let with = render_response(&response, OutputMode::Human);
    response.envelope.errors.clear();
    (with, render_response(&response, OutputMode::Human))
}

#[test]
fn a_partial_push_prints_each_rows_error_once() {
    let members = vec![
        row("mem_app", "app", Failed, Some("cannot push")),
        row("mem_lib", "lib", gwz_core::MemberStatus::Ok, None),
        row(
            "@root",
            ".",
            Rejected,
            Some("root publication was not attempted"),
        ),
    ];
    let response = CliResponse::envelope(partial(gwz_core::ActionKind::Push, members));

    let (with, without) = human_with_and_without_copies(response);

    assert_eq!(with, without);
    assert_eq!(with.matches("cannot push").count(), 1, "{with}");
    assert_eq!(with.matches("was not attempted").count(), 1, "{with}");
}

#[test]
fn a_partial_stash_prints_no_copy() {
    let members = vec![
        row("mem_app", "app", gwz_core::MemberStatus::Ok, None),
        row("mem_lib", "lib", Failed, Some("stash failed")),
    ];
    let response = CliResponse::stash(gwz_core::StashResponse {
        response: partial(gwz_core::ActionKind::Stash, members),
        bundles: Some(Vec::new()),
    });

    let (with, without) = human_with_and_without_copies(response);

    assert_eq!(with, without);
}
