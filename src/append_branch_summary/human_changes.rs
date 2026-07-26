use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HumanChangeSection {
    Staged,
    Unstaged,
    Untracked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HumanChange {
    pub(crate) section: HumanChangeSection,
    pub(crate) status: String,
    pub(crate) path: String,
}

pub(crate) fn root_human_changes(
    workspace_status: &gwz_core::WorkspaceGitStatus,
) -> Vec<HumanChange> {
    workspace_status
        .root_file_changes
        .iter()
        .map(|change| {
            human_change(
                &change.index_status,
                &change.worktree_status,
                &change.workspace_path,
            )
        })
        .collect()
}

pub(crate) fn member_human_changes(
    workspace_status: &gwz_core::WorkspaceGitStatus,
    member_id: Option<&str>,
) -> Vec<HumanChange> {
    workspace_status
        .file_changes
        .iter()
        .filter(|change| member_id.is_none_or(|member_id| change.member_id == member_id))
        .map(|change| {
            human_change(
                &change.index_status,
                &change.worktree_status,
                &change.workspace_path,
            )
        })
        .collect()
}

pub(crate) fn human_change(index_status: &str, worktree_status: &str, path: &str) -> HumanChange {
    let section = if index_status == " " && worktree_status == "?" {
        HumanChangeSection::Untracked
    } else if index_status != " " {
        HumanChangeSection::Staged
    } else {
        HumanChangeSection::Unstaged
    };
    HumanChange {
        section,
        status: format_status_pair(index_status, worktree_status),
        path: path.to_owned(),
    }
}

pub(crate) fn append_change_sections(lines: &mut Vec<String>, changes: &[HumanChange]) {
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Staged,
        "Changes to be committed:",
    );
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Unstaged,
        "Changes not staged for commit:",
    );
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Untracked,
        "Untracked files:",
    );
}

pub(crate) fn append_change_section(
    lines: &mut Vec<String>,
    changes: &[HumanChange],
    section: HumanChangeSection,
    header: &str,
) {
    let section_changes = changes
        .iter()
        .filter(|change| change.section == section)
        .collect::<Vec<_>>();
    if section_changes.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push(header.to_owned());
    lines.extend(
        section_changes
            .into_iter()
            .map(|change| format!("  {} {}", change.status, change.path)),
    );
}

pub(crate) fn render_porcelain_response(response: &CliResponse) -> String {
    if let Some(workspace_status) = &response.workspace_git_status
        && !(workspace_status.root_file_changes.is_empty()
            && workspace_status.file_changes.is_empty())
    {
        return workspace_status
            .root_file_changes
            .iter()
            .map(format_root_file_change)
            .chain(workspace_status.file_changes.iter().map(format_file_change))
            .collect::<Vec<_>>()
            .join("\n");
    }
    response
        .envelope
        .members
        .iter()
        .filter(|member| member.status != gwz_core::MemberStatus::Ok)
        .map(|member| format!("!! {}", member.member_path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn format_file_change(change: &gwz_core::GitFileChange) -> String {
    let status = format_status_pair(&change.index_status, &change.worktree_status);
    format!("{status} {}", change.workspace_path)
}

pub(crate) fn format_root_file_change(change: &gwz_core::WorkspaceRootFileChange) -> String {
    let status = format_status_pair(&change.index_status, &change.worktree_status);
    format!("{status} {}", change.workspace_path)
}
