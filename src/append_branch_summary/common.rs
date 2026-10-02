pub(crate) fn is_unmaterialized(member: &gwz_core::MemberResponse) -> bool {
    member
        .state
        .as_ref()
        .is_some_and(|state| !state.materialized)
}

/// The top-level errors a human report prints after its rows: every one that is
/// not a copy of a member entry's error. A `Partial`, `Failed` or `Rejected`
/// result copies each failed or refused member's error into `errors` for
/// machine readers (docs/MachineOutput.md, "Failed, rejected and partial
/// results"); a report prints no copy, so a member's error stays on its row, or
/// stays out of a report whose rows never showed it.
pub(crate) fn errors_not_on_members(
    envelope: &gwz_core::ResponseEnvelope,
) -> impl Iterator<Item = &gwz_core::GwzError> {
    envelope.errors.iter().filter(|error| {
        !envelope
            .members
            .iter()
            .any(|member| member.error.as_ref() == Some(*error))
    })
}

pub(crate) fn push_blank(lines: &mut Vec<String>) {
    if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
        lines.push(String::new());
    }
}

pub(crate) fn format_status_pair(index_status: &str, worktree_status: &str) -> String {
    if index_status == " " && worktree_status == "?" {
        "??".to_owned()
    } else {
        format!("{index_status}{worktree_status}")
    }
}
