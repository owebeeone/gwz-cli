use super::open_merge_gate::open_merge_gate_request;
use crate::*;

cfg_if::cfg_if! { if #[cfg(any(test, not(all(gwz_transport_candidate, any(unix, windows)))))] {
pub(crate) fn execute_invocation(invocation: &CliInvocation) -> Result<CliResponse, CliError> {
    execute_invocation_selected(invocation, false)
}

} }

cfg_if::cfg_if! { if #[cfg(all(gwz_transport_candidate, any(unix, windows)))] {
/// Who is told what the transport's limit machines say (the adaptive
/// concurrency design's §9): a person reading human output, as `note: ` lines
/// above the live progress block. Every machine-readable mode gets none, so
/// `--json` and `--jsonl` carry no notes, and neither does `--porcelain`.
///
/// The detail lines, such as a followed discovery redirect, are printed only with `--verbose`.
fn note_sink(
    output: OutputMode,
    verbose: bool,
    progress: &std::sync::Arc<StderrProgressSink>,
) -> Option<gwz_core::transport_host::NoteSink> {
    let progress = progress.clone();
    (output == OutputMode::Human).then(|| {
        gwz_core::transport_host::NoteSink::new(move |line| progress.println_above(line))
            .with_detail(verbose)
    })
}

cfg_if::cfg_if! { if #[cfg(test)] { mod notes_tests; } }
} }

/// Who is told why a commit or tag ran the Git CLI instead of the native
/// route: a person reading human output who asked for `--verbose`. Every
/// machine-readable mode gets none.
fn route_notes(
    output: OutputMode,
    verbose: bool,
    progress: &std::sync::Arc<StderrProgressSink>,
) -> Option<gwz_core::git::RouteNotes> {
    let progress = progress.clone();
    (output == OutputMode::Human && verbose)
        .then(|| gwz_core::git::RouteNotes::new(move |line| progress.println_above(line)))
}

cfg_if::cfg_if! { if #[cfg(test)] { mod route_notes_tests; } }

pub(crate) fn execute_invocation_selected(
    invocation: &CliInvocation,
    _native: bool,
) -> Result<CliResponse, CliError> {
    let operation_id = new_operation_id();
    // The live progress line and the notes share one terminal writer.
    let progress_sink = std::sync::Arc::new(StderrProgressSink::new(operation_label(
        &invocation.request,
    )));
    cfg_if::cfg_if! { if #[cfg(all(gwz_transport_candidate, any(unix, windows)))] {
        if let Some(meta) = transport_meta(&invocation.request).filter(|_| !_native) {
            let supervisor = crate::sspi_worker_executable().and_then(|descriptor|
                gwz_sspi::Supervisor::new(descriptor, gwz_sspi::Options::default()).map(std::sync::Arc::new).map_err(|error| error.kind()));
            let caller = gwz_core::transport_host::NativeCaller::capture(&supervisor);
            let run = |backend: &gwz_core::git::Git2Backend| {
                execute_with_backend(invocation, backend, operation_id.clone(), &progress_sink)
            };
            let (result, cleanup) = match note_sink(invocation.output, invocation.verbose, &progress_sink) {
                Some(notes) => gwz_core::transport_host::with_local_transport_native_noted(
                    meta.clone(),
                    operation_id.clone(),
                    caller,
                    notes,
                    run,
                ),
                None => gwz_core::transport_host::with_local_transport_native(
                    meta.clone(),
                    operation_id.clone(),
                    caller,
                    run,
                ),
            }
            .map_err(CliError::from_model)?;
            if cleanup.pending_local_work != 0 {
                eprintln!(
                    "gwz-alpha: transport cleanup has {} pending local jobs",
                    cleanup.pending_local_work
                );
            }
            return result;
        }
    } }
    execute_with_backend(
        invocation,
        &gwz_core::git::Git2Backend::new(),
        operation_id,
        &progress_sink,
    )
}

fn execute_with_backend(
    invocation: &CliInvocation,
    backend: &gwz_core::git::Git2Backend,
    operation_id: String,
    progress_sink: &std::sync::Arc<StderrProgressSink>,
) -> Result<CliResponse, CliError> {
    let noted;
    let backend = match route_notes(invocation.output, invocation.verbose, progress_sink) {
        Some(notes) => {
            noted = backend.with_route_notes(notes);
            &noted
        }
        None => backend,
    };
    let services = backend.operation_services();
    let start = invocation.start_dir.as_path();
    // --jsonl streams machine records to stdout; Human renders a live progress
    // line to stderr (TTY-gated); Json/Porcelain stay quiet.
    let jsonl_sink = JsonlSink;
    let null_sink = gwz_core::operation::NullSink;
    let events: &dyn gwz_core::operation::EventSink = match invocation.output {
        OutputMode::Jsonl => &jsonl_sink,
        OutputMode::Human => progress_sink.as_ref(),
        OutputMode::Json | OutputMode::Porcelain => &null_sink,
    };
    // Most mutations are guarded authoritatively by their public core handler.
    // `forall` executes arbitrary commands in this driver, so it retains the
    // core workspace guard itself across the complete dispatch.
    //
    // DR-3: the guard is taken with the request's dry-run answer. A planned run
    // holds the mutator lock exactly like a real one (both arms lock, so the plan
    // observes the workspace a real run would have mutated) but receives no write
    // authority — and `execute_forall` refuses to spawn.
    let _forall_guard = if let CliRequest::Forall { meta, .. } = &invocation.request {
        Some(
            gwz_core::workspace_ops::acquire_workspace_mutation_guard_for_request_with_services(
                &services,
                start,
                meta,
                gwz_core::operation::OpenMergeCommand::Forall,
                meta.dry_run.unwrap_or(false),
            )
            .map_err(CliError::from_model)?,
        )
    } else {
        if let Some((meta, command)) = open_merge_gate_request(&invocation.request) {
            gwz_core::workspace_ops::enforce_workspace_open_merge_gate_for_request(
                start, meta, command,
            )
            .map_err(CliError::from_model)?;
        }
        None
    };
    let response = match &invocation.request {
        CliRequest::RemoteIdentity(request) => gwz_core::workspace_ops::handle_remote_identity(
            backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| {
            CliResponse::listing(
                response.response,
                ArtifactListing::Identities(response.identities),
            )
        }),
        CliRequest::CloneWorkspace { meta, url, target } => {
            gwz_core::workspace_ops::handle_clone_workspace_at(
                backend,
                start,
                meta.clone(),
                url,
                target,
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::CloneLocalWorkspace(request) => {
            gwz_core::workspace_ops::handle_clone_local_workspace(
                backend,
                start,
                request.clone(),
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::LocalFamily(request) => gwz_core::workspace_ops::handle_local_family(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::local_family(request.op, response)),
        CliRequest::CreateWorkspace(request) => {
            gwz_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::UpdateBootstrap { meta, commit } => {
            gwz_core::workspace_ops::handle_update_workspace_bootstrap_with_commit(
                backend,
                start,
                meta.clone(),
                *commit,
                operation_id,
            )
            .map(CliResponse::envelope)
        }
        CliRequest::InitFromSources(request) => gwz_core::workspace_ops::handle_init_from_sources(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::AddExistingRepo(request) => gwz_core::workspace_ops::handle_add_existing_repo(
            backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::CreateRepo(request) => gwz_core::workspace_ops::handle_create_repo(
            backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::RepoSync(request) => {
            gwz_core::workspace_ops::handle_repo_sync(backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::CloneRepoMember(request) => gwz_core::workspace_ops::handle_clone_repo_member(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::DetachRepoMember(request) => {
            gwz_core::workspace_ops::handle_detach_repo_member(
                backend,
                start,
                request.clone(),
                operation_id,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::AttachRepoMember(request) => {
            gwz_core::workspace_ops::handle_attach_repo_member(
                backend,
                start,
                request.clone(),
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Materialize(request) => gwz_core::workspace_ops::handle_materialize(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Status(request) => {
            gwz_core::status::handle_status(backend, start, request.clone(), operation_id).map(
                |response| CliResponse {
                    envelope: response.response,
                    workspace_git_status: response.workspace_git_status,
                    status_mode: request.mode,
                    listing: None,
                    branch_repos: None,
                    fetch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    local_family: None,
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
                    fetch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    local_family: None,
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
            gwz_core::workspace_ops::handle_snapshot(backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Tag(request) => gwz_core::workspace_ops::handle_tag_with_services(
            &services,
            backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| match response.tags {
            Some(tags) => CliResponse::listing(response.response, ArtifactListing::Tags(tags)),
            None => CliResponse::envelope(response.response),
        }),
        CliRequest::Branch(request) => {
            gwz_core::workspace_ops::handle_branch(backend, start, request.clone(), operation_id)
                .map(CliResponse::branch)
        }
        // The one merge entry point: a request carrying `local_source_name`
        // takes core's family wrapper, and any other request reaches the
        // unchanged engine entry through it (design §6; LCM1.0c §2.4).
        CliRequest::Merge(request) => gwz_core::workspace_ops::handle_merge_with_local_family(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(CliResponse::merge),
        CliRequest::Stash(request) => {
            gwz_core::workspace_ops::handle_stash(backend, start, request.clone(), operation_id)
                .map(CliResponse::stash)
        }
        CliRequest::PullHead(request) => gwz_core::workspace_ops::handle_pull_head_with_events(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::PullSnapshot(request) => gwz_core::workspace_ops::handle_pull_snapshot(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Push(request) => gwz_core::workspace_ops::handle_push_with_events(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Fetch(request) => gwz_core::workspace_ops::handle_fetch_with_events(
            backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(CliResponse::fetch),
        CliRequest::Capture(request) => {
            gwz_core::workspace_ops::handle_capture(backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Commit(request) => {
            gwz_core::workspace_ops::handle_commit(backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Stage(request) => {
            gwz_core::workspace_ops::handle_stage(backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Diff(_) => {
            // Diff is dispatched in `run()` before this function (it streams patch
            // bytes and owns its exit code); it never reaches the envelope path.
            unreachable!("diff is handled by diff_exec::run_diff, not execute_invocation")
        }
        CliRequest::Hook(_) | CliRequest::ClaudeCodeSetup(_) => {
            // The Claude Code commands are dispatched in `run()` before this
            // function: they own their own stdin, stdout contract and exit
            // code (D1).
            unreachable!(
                "the claude-code commands are handled by hook_exec, not execute_invocation"
            )
        }
        CliRequest::Log(_) => {
            // Log is dispatched in `run()` because it owns the output registry,
            // stdout lifecycle, and aggregate exit code.
            unreachable!("log is handled by log_exec::run_log, not execute_invocation")
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
                    fetch_repos: None,
                    merge_response: None,
                    stash_bundles: None,
                    local_family: None,
                    summary: None,
                })
        }
    };
    response.map_err(CliError::from_model)
}

cfg_if::cfg_if! { if #[cfg(all(gwz_transport_candidate, any(unix, windows)))] {
    /// The request's metadata when it runs inside a transport runtime: gwz-core's
    /// transport scope decides, the predicate gwz-py's extension shares
    /// (gwz-py dev-docs/GwzPyPerOperationTransportDesign.md §2.1).
    fn transport_meta(request: &CliRequest) -> Option<&gwz_core::RequestMeta> {
        use gwz_core::transport_scope::{Operation, in_scope};
        let (operation, meta, tag) = match request {
            CliRequest::CloneWorkspace { meta, .. } => (Operation::CloneWorkspace, meta, None),
            CliRequest::InitFromSources(r) => (Operation::InitFromSources, &r.meta, None),
            CliRequest::CloneRepoMember(r) => (Operation::CloneRepoMember, &r.meta, None),
            CliRequest::Materialize(r) => (Operation::Materialize, &r.meta, None),
            CliRequest::Fetch(r) => (Operation::Fetch, &r.meta, None),
            CliRequest::PullHead(r) => (Operation::PullHead, &r.meta, None),
            CliRequest::PullSnapshot(r) => (Operation::PullSnapshot, &r.meta, None),
            CliRequest::Push(r) => (Operation::Push, &r.meta, None),
            CliRequest::Tag(r) => (Operation::Tag, &r.meta, Some(r)),
            _ => return None,
        };
        in_scope(operation, tag).then_some(meta)
    }
} }
