pub(crate) fn append_branch_summary(
    lines: &mut Vec<String>,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let mut groups = workspace_status
        .branch_groups
        .iter()
        .map(|group| (group.label.clone(), group.member_paths.clone()))
        .collect::<Vec<_>>();

    let Some(root_status) = workspace_status.root_status.as_ref() else {
        if groups.is_empty() {
            lines.push("Workspace status".to_owned());
        } else if groups.len() == 1 {
            lines.push(branch_group_sentence(&groups[0].0));
        } else {
            append_branch_groups(lines, &groups);
        }
        return;
    };

    if let Some(label) = root_branch_label(root_status) {
        add_branch_group_path(&mut groups, label, ".".to_owned());
    }

    if groups.is_empty() {
        lines.push("Workspace status".to_owned());
    } else {
        if groups.len() == 1 {
            lines.push(branch_group_sentence(&groups[0].0));
        } else {
            append_branch_groups(lines, &groups);
        }
    }

    if root_status.unborn {
        lines.push("No commits yet".to_owned());
    }
}

pub(crate) fn root_branch_label(root_status: &gwz_core::WorkspaceRootGitStatus) -> Option<String> {
    if let Some(branch) = &root_status.branch {
        Some(branch.clone())
    } else if root_status.detached {
        Some(
            root_status
                .head
                .as_ref()
                .map(|head| format!("detached@{}", head.chars().take(12).collect::<String>()))
                .unwrap_or_else(|| "detached".to_owned()),
        )
    } else if root_status.unborn {
        Some("unborn".to_owned())
    } else {
        None
    }
}

pub(crate) fn add_branch_group_path(
    groups: &mut Vec<(String, Vec<String>)>,
    label: String,
    path: String,
) {
    if let Some(index) = groups
        .iter()
        .position(|(group_label, _)| group_label == &label)
    {
        let (label, mut paths) = groups.remove(index);
        paths.insert(0, path);
        groups.insert(0, (label, paths));
    } else {
        groups.insert(0, (label, vec![path]));
    }
}

pub(crate) fn append_branch_groups(lines: &mut Vec<String>, groups: &[(String, Vec<String>)]) {
    for (label, paths) in groups {
        lines.push(format!(
            "{} {}",
            paths.join(", "),
            branch_group_phrase(label)
        ));
    }
}

pub(crate) fn branch_group_sentence(label: &str) -> String {
    let phrase = branch_group_phrase(label);
    let mut chars = phrase.chars();
    let Some(first) = chars.next() else {
        return phrase;
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

pub(crate) fn branch_group_phrase(label: &str) -> String {
    if label == "unborn" {
        "have no commits yet".to_owned()
    } else if label == "detached" {
        "HEAD detached".to_owned()
    } else if let Some(commit) = label.strip_prefix("detached@") {
        format!("detached at {commit}")
    } else {
        format!("on branch {label}")
    }
}
