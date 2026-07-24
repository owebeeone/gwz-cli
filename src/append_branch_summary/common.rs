pub(crate) fn is_unmaterialized(member: &gwz_core::MemberResponse) -> bool {
    member
        .state
        .as_ref()
        .is_some_and(|state| !state.materialized)
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
