use crate::*;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ArtifactListing {
    Tags(Vec<gwz_core::TagInfo>),
    Snapshots(Vec<gwz_core::SnapshotInfo>),
    Members {
        entries: Vec<gwz_core::MemberEntry>,
        local: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CliResponse {
    pub(crate) envelope: gwz_core::ResponseEnvelope,
    pub(crate) workspace_git_status: Option<gwz_core::WorkspaceGitStatus>,
    pub(crate) status_mode: Option<gwz_core::StatusMode>,
    pub(crate) listing: Option<ArtifactListing>,
    pub(crate) branch_repos: Option<Vec<gwz_core::BranchRepoSummary>>,
    pub(crate) merge_response: Option<gwz_core::MergeResponse>,
    pub(crate) stash_bundles: Option<Vec<gwz_core::StashBundle>>,
    /// forall's trailing summary — rendered verbatim (it already streamed member output live).
    pub(crate) summary: Option<String>,
}

impl CliResponse {
    pub(crate) fn envelope(response: gwz_core::ResponseEnvelope) -> Self {
        Self {
            envelope: response,
            workspace_git_status: None,
            status_mode: None,
            listing: None,
            branch_repos: None,
            merge_response: None,
            stash_bundles: None,
            summary: None,
        }
    }

    pub(crate) fn branch(response: gwz_core::BranchResponse) -> Self {
        Self {
            envelope: response.response,
            workspace_git_status: None,
            status_mode: None,
            listing: None,
            branch_repos: response.repos,
            merge_response: None,
            stash_bundles: None,
            summary: None,
        }
    }

    pub(crate) fn merge(response: gwz_core::MergeResponse) -> Self {
        Self {
            envelope: response.response.clone(),
            workspace_git_status: None,
            status_mode: None,
            listing: None,
            branch_repos: None,
            merge_response: Some(response),
            stash_bundles: None,
            summary: None,
        }
    }

    pub(crate) fn stash(response: gwz_core::StashResponse) -> Self {
        Self {
            envelope: response.response,
            workspace_git_status: None,
            status_mode: None,
            listing: None,
            branch_repos: None,
            merge_response: None,
            stash_bundles: response.bundles,
            summary: None,
        }
    }

    pub(crate) fn listing(response: gwz_core::ResponseEnvelope, listing: ArtifactListing) -> Self {
        Self {
            envelope: response,
            workspace_git_status: None,
            status_mode: None,
            listing: Some(listing),
            branch_repos: None,
            merge_response: None,
            stash_bundles: None,
            summary: None,
        }
    }
}

/// Human/porcelain text for a tag/snapshot listing.
pub(crate) fn render_listing_text(listing: &ArtifactListing) -> String {
    let plural = |count: usize| if count == 1 { "" } else { "s" };
    match listing {
        ArtifactListing::Tags(tags) => {
            if tags.is_empty() {
                return "no tags".to_owned();
            }
            let mut lines = vec![format!("{} tag{}:", tags.len(), plural(tags.len()))];
            for tag in tags {
                lines.push(format!(
                    "  {}\t({} member{})",
                    tag.name,
                    tag.members,
                    plural(tag.members as usize)
                ));
            }
            lines.join("\n")
        }
        ArtifactListing::Snapshots(snapshots) => {
            if snapshots.is_empty() {
                return "no snapshots".to_owned();
            }
            let mut lines = vec![format!(
                "{} snapshot{}:",
                snapshots.len(),
                plural(snapshots.len())
            )];
            for snapshot in snapshots {
                lines.push(format!(
                    "  {}\t{}\t{}\t({} member{})",
                    snapshot.name,
                    snapshot.created_at,
                    snapshot.created_by,
                    snapshot.members,
                    plural(snapshot.members as usize)
                ));
            }
            lines.join("\n")
        }
        // Members render as raw paths, one per line (no header) — for `for i in $(gwz ls)`.
        ArtifactListing::Members { entries, local } => entries
            .iter()
            .map(|member| {
                if *local {
                    member.path.clone()
                } else {
                    member.abspath.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// JSON for a tag/snapshot listing.
pub(crate) fn listing_json(listing: &ArtifactListing) -> serde_json::Value {
    use serde_json::json;
    match listing {
        ArtifactListing::Tags(tags) => json!({
            "kind": "tags",
            "entries": tags
                .iter()
                .map(|tag| json!({ "name": tag.name, "members": tag.members }))
                .collect::<Vec<_>>(),
        }),
        ArtifactListing::Snapshots(snapshots) => json!({
            "kind": "snapshots",
            "entries": snapshots
                .iter()
                .map(|snapshot| json!({
                    "name": snapshot.name,
                    "created_at": snapshot.created_at,
                    "created_by": snapshot.created_by,
                    "members": snapshot.members,
                }))
                .collect::<Vec<_>>(),
        }),
        ArtifactListing::Members { entries, .. } => json!({
            "kind": "members",
            "entries": entries
                .iter()
                .map(|member| json!({
                    "id": member.id,
                    "path": member.path,
                    "abspath": member.abspath,
                    "materialized": member.materialized,
                }))
                .collect::<Vec<_>>(),
        }),
    }
}

pub(crate) fn render_human_response(response: &CliResponse) -> String {
    if let Some(merge) = &response.merge_response {
        return render_merge_response(merge);
    }
    if let Some(workspace_status) = &response.workspace_git_status {
        return render_human_status_response(response, workspace_status);
    }
    if let Some(repos) = &response.branch_repos {
        return render_branch_response(response, repos);
    }
    if let Some(bundles) = &response.stash_bundles {
        return render_stash_response(response, bundles);
    }

    let mut lines = vec![format!(
        "status: {:?}",
        response.envelope.meta.aggregate_status
    )];
    if let Some(message) = &response.envelope.meta.message {
        lines.push(message.clone());
    }
    for member in &response.envelope.members {
        let mut line = format!(
            "{} {} {:?}",
            member.member_id, member.member_path, member.status
        );
        if let Some(error) = &member.error {
            line.push_str(&format!(" {:?}: {}", error.code, error.message));
        }
        if let Some(message) = member
            .planned
            .as_ref()
            .and_then(|planned| planned.message.as_ref())
        {
            line.push_str(&format!(" {message}"));
        }
        lines.push(line);
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}
