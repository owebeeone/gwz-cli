use crate::*;

// Default coalescing for member_progress events: at most one per member per
// 100 ms. Set as a request option so a driver can tune or disable it (0).
pub(crate) const DEFAULT_PROGRESS_MIN_INTERVAL_MS: i64 = 100;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CliInvocation {
    pub(crate) request: CliRequest,
    pub(crate) output: OutputMode,
    pub(crate) start_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CliRequest {
    CreateWorkspace(gwz_core::CreateWorkspaceRequest),
    UpdateBootstrap {
        meta: gwz_core::RequestMeta,
    },
    CloneWorkspace {
        meta: gwz_core::RequestMeta,
        url: String,
        target: String,
    },
    InitFromSources(gwz_core::InitFromSourcesRequest),
    AddExistingRepo(gwz_core::AddExistingRepoRequest),
    CreateRepo(gwz_core::CreateRepoRequest),
    RepoSync(gwz_core::RepoSyncRequest),
    CloneRepoMember(gwz_core::CloneRepoMemberRequest),
    DetachRepoMember(gwz_core::DetachRepoMemberRequest),
    AttachRepoMember(gwz_core::AttachRepoMemberRequest),
    Materialize(gwz_core::MaterializeRequest),
    Status(gwz_core::StatusRequest),
    Ls {
        request: gwz_core::LsRequest,
        local: bool,
    },
    Forall {
        meta: gwz_core::RequestMeta,
        projects: Vec<String>,
        mode: gwz_core::ExecMode,
        command: Vec<String>,
        continue_on_fail: bool,
        no_banner: bool,
    },
    Snapshot(gwz_core::SnapshotRequest),
    Tag(gwz_core::TagRequest),
    Branch(gwz_core::BranchRequest),
    Merge(gwz_core::MergeRequest),
    Stash(gwz_core::StashRequest),
    PullHead(gwz_core::PullHeadRequest),
    PullSnapshot(gwz_core::PullSnapshotRequest),
    Push(gwz_core::PushRequest),
    Capture(gwz_core::CaptureRequest),
    Commit(gwz_core::CommitRequest),
    Stage(gwz_core::StageRequest),
    ListSnapshots(gwz_core::ListSnapshotsRequest),
    /// Diff is special: it streams patch bytes and owns its own exit code rather
    /// than returning a rendered response envelope. Boxed to keep the enum small.
    Diff(Box<DiffInvocation>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputMode {
    Human,
    Json,
    Jsonl,
    Porcelain,
}

/// A short verb for the progress line, derived from the request kind. Only the
/// I/O-bound operations emit member events, so only those labels are ever seen.
pub(crate) fn operation_label(request: &CliRequest) -> &'static str {
    match request {
        CliRequest::CloneWorkspace { .. } => "cloning",
        CliRequest::CloneRepoMember(_) => "cloning",
        CliRequest::Materialize(_) => "materializing",
        CliRequest::InitFromSources(_) => "initializing",
        CliRequest::UpdateBootstrap { .. } => "updating",
        CliRequest::PullSnapshot(_) => "pulling",
        _ => "working",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliError {
    pub(crate) message: String,
    pub(crate) code: Option<gwz_core::model::ErrorCode>,
    pub(crate) member_id: Option<String>,
    pub(crate) member_path: Option<String>,
    pub(crate) target_kind: Option<String>,
}

impl CliError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            member_id: None,
            member_path: None,
            target_kind: None,
        }
    }

    /// A rejected-request error carrying the `InvalidRequest` code, so `--json`/
    /// `--jsonl` render it structured (used by the diff argument parser to reject
    /// unsupported git options per D0).
    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(gwz_core::model::ErrorCode::InvalidRequest),
            member_id: None,
            member_path: None,
            target_kind: None,
        }
    }

    /// Preserve a gwz-core error's code (so `--json`/`--jsonl` can emit it
    /// structured) alongside its message.
    pub(crate) fn from_model(error: gwz_core::model::ModelError) -> Self {
        let has_member_context = error.member_id.is_some() || error.member_path.is_some();
        Self {
            message: error.message,
            code: Some(error.code),
            member_id: error.member_id,
            member_path: error.member_path,
            target_kind: has_member_context.then(|| "Member".to_owned()),
        }
    }

    /// Human rendering: prefix with the error code when present, matching
    /// gwz-core's `ModelError` Display.
    pub(crate) fn human_message(&self) -> String {
        match self.code {
            Some(code) => format!("{code:?}: {}", self.message),
            None => self.message.clone(),
        }
    }
}

/// The workspace-relative logical cwd (AD10): the physical cwd expressed relative
/// to the workspace root, so path operands resolve remote-safely. Returns `""`
/// when cwd is the root (or cannot be expressed under it — the safe default).
pub(crate) fn workspace_relative_cwd(
    workspace_root: &str,
    current_dir: &std::path::Path,
) -> String {
    let root = std::path::Path::new(workspace_root);
    // Canonicalize both sides where possible so `..`/symlinks compare correctly;
    // fall back to the raw paths if canonicalization fails (e.g. non-existent).
    let root_abs = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let cwd_abs = std::fs::canonicalize(current_dir).unwrap_or_else(|_| current_dir.to_path_buf());
    match cwd_abs.strip_prefix(&root_abs) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => String::new(),
    }
}
