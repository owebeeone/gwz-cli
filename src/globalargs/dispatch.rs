use super::open_merge_gate::open_merge_gate_request;
use crate::*;

pub(crate) fn execute_invocation(invocation: &CliInvocation) -> Result<CliResponse, CliError> {
    let backend = gwz_core::git::Git2Backend::new();
    let operation_id = new_operation_id();
    let start = invocation.start_dir.as_path();
    // --jsonl streams machine records to stdout; Human renders a live progress
    // line to stderr (TTY-gated); Json/Porcelain stay quiet.
    let jsonl_sink = JsonlSink;
    let null_sink = gwz_core::operation::NullSink;
    let progress_sink = StderrProgressSink::new(operation_label(&invocation.request));
    let events: &dyn gwz_core::operation::EventSink = match invocation.output {
        OutputMode::Jsonl => &jsonl_sink,
        OutputMode::Human => &progress_sink,
        OutputMode::Json | OutputMode::Porcelain => &null_sink,
    };
    // Most mutations are guarded authoritatively by their public core handler.
    // `forall` executes arbitrary commands in this driver, so it retains the
    // core workspace guard itself across the complete dispatch.
    let _forall_guard = if let CliRequest::Forall { meta, .. } = &invocation.request {
        Some(
            gwz_core::workspace_ops::acquire_workspace_mutation_guard(
                start,
                meta.workspace.as_ref(),
                gwz_core::operation::OpenMergeCommand::Forall,
            )
            .map_err(CliError::from_model)?,
        )
    } else {
        if let Some((workspace, command)) = open_merge_gate_request(&invocation.request) {
            gwz_core::workspace_ops::enforce_workspace_open_merge_gate(start, workspace, command)
                .map_err(CliError::from_model)?;
        }
        None
    };
    let response = match &invocation.request {
        CliRequest::CloneWorkspace { meta, url, target } => {
            gwz_core::workspace_ops::handle_clone_workspace(
                &backend,
                meta.clone(),
                url,
                target,
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::CreateWorkspace(request) => {
            gwz_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::UpdateBootstrap { meta } => {
            gwz_core::workspace_ops::handle_update_workspace_bootstrap(
                &backend,
                start,
                meta.clone(),
                operation_id,
            )
            .map(CliResponse::envelope)
        }
        CliRequest::InitFromSources(request) => gwz_core::workspace_ops::handle_init_from_sources(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::AddExistingRepo(request) => gwz_core::workspace_ops::handle_add_existing_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::CreateRepo(request) => gwz_core::workspace_ops::handle_create_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::RepoSync(request) => gwz_core::workspace_ops::handle_repo_sync(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::CloneRepoMember(request) => gwz_core::workspace_ops::handle_clone_repo_member(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::DetachRepoMember(request) => {
            gwz_core::workspace_ops::handle_detach_repo_member(
                &backend,
                start,
                request.clone(),
                operation_id,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::AttachRepoMember(request) => {
            gwz_core::workspace_ops::handle_attach_repo_member(
                &backend,
                start,
                request.clone(),
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Materialize(request) => gwz_core::workspace_ops::handle_materialize(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Status(request) => {
            gwz_core::status::handle_status(&backend, start, request.clone(), operation_id).map(
                |response| CliResponse {
                    envelope: response.response,
                    workspace_git_status: response.workspace_git_status,
                    status_mode: request.mode,
                    listing: None,
                    branch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    summary: None,
                },
            )
        }
        CliRequest::Ls { request, local } => {
            gwz_core::workspace_ops::handle_ls(start, request.clone(), operation_id).map(
                |response| CliResponse {
                    envelope: response.response,
                    workspace_git_status: None,
                    status_mode: None,
                    listing: Some(ArtifactListing::Members {
                        entries: response.members.unwrap_or_default(),
                        local: *local,
                    }),
                    branch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    summary: None,
                },
            )
        }
        CliRequest::Forall {
            meta,
            projects,
            mode,
            command,
            continue_on_fail,
            no_banner,
        } => execute_forall(
            start,
            meta,
            projects,
            *mode,
            command,
            *continue_on_fail,
            *no_banner,
            operation_id,
        ),
        CliRequest::Snapshot(request) => {
            gwz_core::workspace_ops::handle_snapshot(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Tag(request) => {
            gwz_core::workspace_ops::handle_tag(&backend, start, request.clone(), operation_id).map(
                |response| match response.tags {
                    Some(tags) => {
                        CliResponse::listing(response.response, ArtifactListing::Tags(tags))
                    }
                    None => CliResponse::envelope(response.response),
                },
            )
        }
        CliRequest::Branch(request) => {
            gwz_core::workspace_ops::handle_branch(&backend, start, request.clone(), operation_id)
                .map(CliResponse::branch)
        }
        CliRequest::Merge(request) => gwz_core::workspace_ops::handle_merge_with_events(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(CliResponse::merge),
        CliRequest::Stash(request) => {
            gwz_core::workspace_ops::handle_stash(&backend, start, request.clone(), operation_id)
                .map(CliResponse::stash)
        }
        CliRequest::PullHead(request) => gwz_core::workspace_ops::handle_pull_head_with_events(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::PullSnapshot(request) => gwz_core::workspace_ops::handle_pull_snapshot(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Push(request) => gwz_core::workspace_ops::handle_push_with_events(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Capture(request) => {
            gwz_core::workspace_ops::handle_capture(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Commit(request) => {
            gwz_core::workspace_ops::handle_commit(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Stage(request) => {
            gwz_core::workspace_ops::handle_stage(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Diff(_) => {
            // Diff is dispatched in `run()` before this function (it streams patch
            // bytes and owns its exit code); it never reaches the envelope path.
            unreachable!("diff is handled by diff_exec::run_diff, not execute_invocation")
        }
        CliRequest::ListSnapshots(request) => {
            gwz_core::workspace_ops::handle_list_snapshots(start, request.clone(), operation_id)
                .map(|response| CliResponse {
                    envelope: response.response,
                    workspace_git_status: None,
                    status_mode: None,
                    listing: Some(ArtifactListing::Snapshots(
                        response.snapshots.unwrap_or_default(),
                    )),
                    branch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    summary: None,
                })
        }
    };
    response.map_err(CliError::from_model)
}
