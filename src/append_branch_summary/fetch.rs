use crate::*;

/// Abbreviate an object id the way `git` renders one: the first seven hex
/// characters. Anything shorter is shown whole rather than padded.
fn short(commit: &str) -> &str {
    let end = commit
        .char_indices()
        .nth(7)
        .map_or(commit.len(), |(i, _)| i);
    &commit[..end]
}

/// What one repository's remote-tracking ref did, in the words the report
/// uses for it.
fn movement(repo: &gwz_core::FetchRepoSummary) -> String {
    match repo.result {
        gwz_core::FetchResult::Updated => match (&repo.before, &repo.after) {
            (Some(before), Some(after)) => format!("{}..{}", short(before), short(after)),
            // A tracking ref that did not exist before this fetch: there is no
            // left-hand side to show, so say where it now points.
            (None, Some(after)) => format!("new {}", short(after)),
            _ => "updated".to_owned(),
        },
        gwz_core::FetchResult::Unchanged => "no change".to_owned(),
        gwz_core::FetchResult::NoUpstream => "no upstream".to_owned(),
        gwz_core::FetchResult::Failed => "failed".to_owned(),
        // `--dry-run` only: the repository was not contacted, so the row says
        // what would happen rather than what did.
        gwz_core::FetchResult::Planned => match &repo.remote {
            Some(remote) => format!("would contact {remote}"),
            None => "would contact".to_owned(),
        },
    }
}

/// `gwz fetch`'s human report: one line per repository, in envelope order
/// (GwzFetchPlan.md §3.3). The columns are the repository's id and path, what
/// its remote-tracking ref did, and the ref it tracks with `+ahead -behind`.
pub(crate) fn render_fetch_response(
    response: &CliResponse,
    repos: &[gwz_core::FetchRepoSummary],
) -> String {
    let mut lines = vec![format!(
        "status: {:?}",
        response.envelope.meta.aggregate_status
    )];
    if let Some(message) = &response.envelope.meta.message {
        lines.push(message.clone());
    }
    let id_width = repos
        .iter()
        .map(|repo| repo.member_id.chars().count())
        .max()
        .unwrap_or(0);
    let path_width = repos
        .iter()
        .map(|repo| repo.member_path.chars().count())
        .max()
        .unwrap_or(0);
    let movements: Vec<String> = repos.iter().map(movement).collect();
    let movement_width = movements
        .iter()
        .map(|movement| movement.chars().count())
        .max()
        .unwrap_or(0);
    for (repo, movement) in repos.iter().zip(&movements) {
        let mut line = format!(
            "{:<id_width$}  {:<path_width$}  {movement:<movement_width$}",
            repo.member_id, repo.member_path
        );
        if let (Some(upstream), Some(ahead), Some(behind)) =
            (&repo.upstream, repo.ahead, repo.behind)
        {
            let tracked = upstream.strip_prefix("refs/remotes/").unwrap_or(upstream);
            line.push_str(&format!("  ({tracked}, +{ahead} -{behind})"));
        }
        // A failed row's reason lives on the envelope, where every other verb
        // puts it; repeat it here so one line per repository stays true.
        if repo.result == gwz_core::FetchResult::Failed
            && let Some(error) = response
                .envelope
                .members
                .iter()
                .find(|member| member.member_id == repo.member_id)
                .and_then(|member| member.error.as_ref())
        {
            line.push_str(&format!("  {:?}: {}", error.code, error.message));
        }
        // The movement column is padded so the tracking column lines up; a row
        // with nothing after it must not carry that padding off the end.
        lines.push(line.trim_end().to_owned());
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

pub(crate) fn fetch_repo_json(repo: &gwz_core::FetchRepoSummary) -> serde_json::Value {
    serde_json::json!({
        "member_id": repo.member_id,
        "member_path": repo.member_path,
        "source_kind": format!("{:?}", repo.source_kind),
        "result": format!("{:?}", repo.result),
        "remote": repo.remote,
        "branch": repo.branch,
        "before": repo.before,
        "after": repo.after,
        "upstream": repo.upstream,
        "ahead": repo.ahead,
        "behind": repo.behind,
    })
}
