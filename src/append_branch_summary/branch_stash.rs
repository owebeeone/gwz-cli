use crate::*;
use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub(crate) fn render_branch_response(
    response: &CliResponse,
    repos: &[gwz_core::BranchRepoSummary],
) -> String {
    if repos
        .iter()
        .all(|repo| repo.result == gwz_core::BranchActionResult::Listed)
    {
        return render_branch_listing_response(response, repos);
    }

    let mut lines = vec![format!(
        "status: {:?}",
        response.envelope.meta.aggregate_status
    )];
    for repo in repos {
        let branch = repo
            .branch
            .as_deref()
            .or(repo.current_branch.as_deref())
            .unwrap_or("(detached)");
        let mut line = format!(
            "{} {} {:?} {}",
            repo.member_id, repo.member_path, repo.result, branch
        );
        if let Some(head) = &repo.head {
            line.push_str(&format!(" {head}"));
        }
        if let Some(source_ref) = &repo.source_ref {
            line.push_str(&format!(" from {source_ref}"));
        }
        if let Some(resulting_commit) = &repo.resulting_commit {
            line.push_str(&format!(" -> {resulting_commit}"));
        }
        if !repo.conflict_paths.is_empty() {
            line.push_str(&format!(" conflicts: {}", repo.conflict_paths.join(",")));
        }
        lines.push(line);
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

pub(crate) fn render_branch_listing_response(
    response: &CliResponse,
    repos: &[gwz_core::BranchRepoSummary],
) -> String {
    let mut lines = branch_listing_lines(repos);
    if response.envelope.meta.aggregate_status != gwz_core::AggregateStatus::Ok {
        lines.insert(
            0,
            format!("status: {:?}", response.envelope.meta.aggregate_status),
        );
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

pub(crate) fn branch_listing_lines(repos: &[gwz_core::BranchRepoSummary]) -> Vec<String> {
    if repos.is_empty() {
        return vec!["no branches".to_owned()];
    }

    let short_name_counts = branch_repo_short_name_counts(repos);
    let mut groups: BTreeMap<(String, bool), BTreeSet<String>> = BTreeMap::new();
    for repo in repos {
        let branch = repo
            .branch
            .as_deref()
            .or(repo.current_branch.as_deref())
            .unwrap_or("(detached)");
        let is_current = repo.current_branch.as_deref() == Some(branch);
        groups
            .entry((branch.to_owned(), is_current))
            .or_default()
            .insert(branch_repo_label(repo, &short_name_counts));
    }

    let mut grouped = groups.into_iter().collect::<Vec<_>>();
    grouped.sort_by(
        |((left_branch, left_current), _), ((right_branch, right_current), _)| {
            left_branch
                .cmp(right_branch)
                .then_with(|| right_current.cmp(left_current))
        },
    );
    grouped
        .into_iter()
        .map(|((branch, is_current), labels)| {
            format!(
                "{}{}: {}",
                if is_current { "*" } else { "" },
                branch,
                labels.into_iter().collect::<Vec<_>>().join(" ")
            )
        })
        .collect()
}

fn branch_repo_short_name_counts(repos: &[gwz_core::BranchRepoSummary]) -> BTreeMap<String, usize> {
    let mut paths = BTreeSet::new();
    for repo in repos {
        paths.insert(repo.member_path.as_str());
    }

    let mut counts = BTreeMap::new();
    for path in paths {
        *counts
            .entry(member_short_name(path).to_owned())
            .or_insert(0) += 1;
    }
    counts
}

fn branch_repo_label(
    repo: &gwz_core::BranchRepoSummary,
    short_name_counts: &BTreeMap<String, usize>,
) -> String {
    let short = member_short_name(&repo.member_path);
    if short_name_counts.get(short).copied().unwrap_or(0) > 1 {
        repo.member_path.clone()
    } else {
        short.to_owned()
    }
}

pub(crate) fn render_stash_response(
    response: &CliResponse,
    bundles: &[gwz_core::StashBundle],
) -> String {
    let mut lines = vec![format!(
        "status: {:?}",
        response.envelope.meta.aggregate_status
    )];
    for bundle in bundles {
        lines.push(format!(
            "{} {} ({} member{})",
            bundle.stash_id,
            bundle.created_at,
            bundle.members.len(),
            if bundle.members.len() == 1 { "" } else { "s" }
        ));
    }
    for member in &response.envelope.members {
        lines.push(format!(
            "{} {} {:?}",
            member.member_id, member.member_path, member.status
        ));
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}
