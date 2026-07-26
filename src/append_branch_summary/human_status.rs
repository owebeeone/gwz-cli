use super::*;

pub(crate) fn render_human_status_response(
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) -> String {
    let per_repo = response.status_mode == Some(gwz_core::StatusMode::Summary);
    let mut lines = Vec::new();
    append_branch_summary(&mut lines, workspace_status);
    if per_repo {
        append_per_repo_status(&mut lines, response, workspace_status);
    } else {
        let mut changes = root_human_changes(workspace_status);
        changes.extend(member_human_changes(workspace_status, None));
        append_change_sections(&mut lines, &changes);
    }
    append_unmaterialized_notice(&mut lines, response);
    append_status_issues(&mut lines, response);
    append_suppressed_dirty_summary(&mut lines, response, workspace_status);
    if lines.is_empty() {
        lines.push("nothing to commit, working tree clean".to_owned());
    }
    lines.join("\n")
}

pub(crate) fn append_unmaterialized_notice(lines: &mut Vec<String>, response: &CliResponse) {
    let unmaterialized = response
        .envelope
        .members
        .iter()
        .filter(|member| is_unmaterialized(member))
        .collect::<Vec<_>>();
    if unmaterialized.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push(
        "Members not materialized (run `gwz materialize --lock` to complete the clone):".to_owned(),
    );
    lines.extend(
        unmaterialized
            .into_iter()
            .map(|member| format!("  {}", member.member_path)),
    );
}

pub(crate) fn append_per_repo_status(
    lines: &mut Vec<String>,
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let root_changes = root_human_changes(workspace_status);
    if !root_changes.is_empty() {
        push_blank(lines);
        lines.push("Workspace root".to_owned());
        append_change_sections(lines, &root_changes);
    }

    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        let changes = member_human_changes(workspace_status, Some(&member.member_id));
        if changes.is_empty() && member.status == gwz_core::MemberStatus::Ok {
            continue;
        }
        push_blank(lines);
        lines.push(format_member_status_heading(member));
        append_change_sections(lines, &changes);
    }
}

/// F16: a dirty tree whose per-file detail was suppressed (`--no-files`) must not vanish
/// from the human output — the counts (`dirty`/staged/unstaged/untracked) are first-class
/// on `GitStatus`, independent of the file list. Surface a count summary for the root and
/// each member that is dirty but produced no rendered file changes.
pub(crate) fn append_suppressed_dirty_summary(
    lines: &mut Vec<String>,
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let mut summary = Vec::new();
    if let Some(root) = workspace_status.root_status.as_ref()
        && root.dirty
        && root_human_changes(workspace_status).is_empty()
    {
        summary.push(format!(
            "  workspace root: {} staged, {} unstaged, {} untracked",
            root.staged, root.unstaged, root.untracked
        ));
    }
    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        let Some(status) = member.git_status.as_ref() else {
            continue;
        };
        if status.dirty
            && member_human_changes(workspace_status, Some(&member.member_id)).is_empty()
        {
            summary.push(format!(
                "  {}: {} staged, {} unstaged, {} untracked",
                member.member_path, status.staged, status.unstaged, status.untracked
            ));
        }
    }
    if summary.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push("Uncommitted changes (file detail omitted; re-run without --no-files):".to_owned());
    lines.extend(summary);
}

pub(crate) fn append_status_issues(lines: &mut Vec<String>, response: &CliResponse) {
    let mut issues = Vec::new();
    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        if member.status != gwz_core::MemberStatus::Ok || member.error.is_some() {
            let mut issue = format!("{}: {:?}", member.member_path, member.status);
            if let Some(error) = &member.error {
                issue.push_str(&format!(" {:?}: {}", error.code, error.message));
            }
            issues.push(issue);
        }
    }
    issues.extend(
        response
            .envelope
            .errors
            .iter()
            .map(|error| format!("{:?}: {}", error.code, error.message)),
    );
    if issues.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push("Issues:".to_owned());
    lines.extend(issues.into_iter().map(|issue| format!("  {issue}")));
}

pub(crate) fn format_member_status_heading(member: &gwz_core::MemberResponse) -> String {
    let Some(git_status) = &member.git_status else {
        return member.member_path.clone();
    };
    if let Some(branch) = &git_status.branch {
        format!("{} on branch {}", member.member_path, branch)
    } else if git_status.detached {
        format!("{} detached", member.member_path)
    } else {
        member.member_path.clone()
    }
}
