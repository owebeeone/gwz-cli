use crate::*;

pub(super) fn open_merge_gate_request(
    request: &CliRequest,
) -> Option<(
    Option<&gwz_core::WorkspaceRef>,
    gwz_core::operation::OpenMergeCommand,
)> {
    use gwz_core::operation::OpenMergeCommand as Command;

    let (meta, command) = match request {
        CliRequest::CreateWorkspace(_) | CliRequest::CloneWorkspace { .. } => return None,
        CliRequest::UpdateBootstrap { meta } => (meta, Command::InitUpdate),
        CliRequest::InitFromSources(request) => (&request.meta, Command::InitExistingPlan),
        CliRequest::AddExistingRepo(request) => (&request.meta, Command::RepoMutate),
        CliRequest::CreateRepo(request) => (&request.meta, Command::RepoMutate),
        CliRequest::RepoSync(request) => (&request.meta, Command::RepoMutate),
        CliRequest::CloneRepoMember(request) => (&request.meta, Command::RepoMutate),
        CliRequest::DetachRepoMember(request) => (&request.meta, Command::RepoMutate),
        CliRequest::AttachRepoMember(request) => (&request.meta, Command::RepoMutate),
        CliRequest::Materialize(request) => (&request.meta, Command::Materialize),
        CliRequest::Status(request) => (&request.meta, Command::Status),
        CliRequest::Ls { request, .. } => (&request.meta, Command::Ls),
        CliRequest::Forall { meta, .. } => (meta, Command::Forall),
        CliRequest::Snapshot(request) => (&request.meta, Command::Snapshot),
        CliRequest::ListSnapshots(request) => (&request.meta, Command::SnapshotList),
        CliRequest::Tag(request) => (
            &request.meta,
            if request.op == gwz_core::TagOp::List {
                Command::TagList
            } else {
                Command::TagMutate
            },
        ),
        CliRequest::Branch(request) => (
            &request.meta,
            if request.op == gwz_core::BranchOp::List {
                Command::BranchList
            } else {
                Command::BranchMutate
            },
        ),
        // First-class merge owns one core lifecycle envelope around context
        // conversion, gating, validation, and dispatch. A driver pre-gate
        // would bypass its required start/finish events on rejection.
        CliRequest::Merge(_) => return None,
        CliRequest::Stash(request) => (
            &request.meta,
            if request.op == gwz_core::StashOp::List {
                Command::StashList
            } else {
                Command::StashMutate
            },
        ),
        CliRequest::PullHead(request) => (&request.meta, Command::Pull),
        CliRequest::PullSnapshot(request) => (&request.meta, Command::Pull),
        CliRequest::Push(request) => (&request.meta, Command::Push),
        CliRequest::Capture(request) => (&request.meta, Command::Capture),
        CliRequest::Commit(request) => (&request.meta, Command::Commit),
        CliRequest::Stage(request) => (&request.meta, Command::StageConflictResolution),
        CliRequest::Diff(request) => (&request.request.meta, Command::Diff),
    };
    Some((meta.workspace.as_ref(), command))
}
