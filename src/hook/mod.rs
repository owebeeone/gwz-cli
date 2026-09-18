//! The Claude Code integration: `gwz hook claude-code worktree-create`,
//! `gwz hook claude-code worktree-remove` and `gwz hook claude-code setup`
//! (gwz-cli `dev-docs/GwzClaudeIntegrationPlan.md`, D1 to D11, S1.1 and S1.2).
//!
//! The contract, stated once in D1 and enforced here: **the create hook
//! prints only a path it created or verified in this invocation, and the
//! remove hook deletes only the lane or worktree that `worktree_path`
//! canonically names.** Nothing else ever reaches stdout, and every failure
//! is one stderr line `gwz: <cause>; <remedy>` with a non-zero exit and an
//! empty stdout.
//!
//! Layering: this module owns the hook's own decisions (the reuse table, the
//! guards, the classification, the log). Workspace semantics stay in
//! gwz-core and are reached through the same in-process handlers every other
//! command uses. The one exception is the D8 fallback, which runs in a plain
//! Git repository that is not a GWZ workspace at all: there is no workspace
//! semantic to delegate, so [`fallback`] drives `git worktree` itself.

pub(crate) mod create;
pub(crate) mod env;
pub(crate) mod estimate;
pub(crate) mod fallback;
pub(crate) mod family;
pub(crate) mod ignore;
pub(crate) mod input;
pub(crate) mod logging;
pub(crate) mod remove;
pub(crate) mod setup;

use std::path::PathBuf;

pub(crate) use create::run_worktree_create;
pub(crate) use env::SystemEnv;
pub(crate) use input::{CreateInput, RemoveInput, parse_create_input, parse_remove_input};
pub(crate) use logging::{LogRecord, resolve_log_location, write_log};
pub(crate) use remove::run_worktree_remove;
pub(crate) use setup::{SetupPlacement, run_setup};

/// The compiled-in ceiling on `ready` family rows (D6). A handler `setup`
/// writes may carry `--max-lanes` instead; the bare handler is guarded by
/// this.
pub(crate) const DEFAULT_MAX_LANES: u32 = 8;

/// The compiled-in create deadline in seconds (D6). `setup` bakes the
/// estimated copy time in its place when it runs inside a workspace (S1.2).
pub(crate) const DEFAULT_WAIT_SECS: u64 = 300;

/// The fixed margin the copy-cost estimate adds on top of the unshared bytes
/// (D6). Placeholder until S3.2 measures it.
pub(crate) const ESTIMATE_MARGIN_BYTES: u64 = 1 << 30;

/// Hook options, every one of them a command-line option on the handler
/// itself, because the desktop app passes no environment variable of ours
/// (D1, section 1).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HookOptions {
    pub(crate) min_free_gb: Option<f64>,
    pub(crate) max_lanes: u32,
    pub(crate) wait_secs: u64,
    pub(crate) base_ref: Option<String>,
    pub(crate) log: Option<String>,
}

impl Default for HookOptions {
    fn default() -> Self {
        Self {
            min_free_gb: None,
            max_lanes: DEFAULT_MAX_LANES,
            wait_secs: DEFAULT_WAIT_SECS,
            base_ref: None,
            log: None,
        }
    }
}

/// What the hook decided this invocation was (D10's classification
/// vocabulary, including D8's three refusal classes).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Classification {
    Lane,
    MemberInLane,
    FallbackWorktree,
    /// A removal whose path, or whose family row, was already gone: an exit
    /// of zero and no refusal at all, so it is not one of the three refusal
    /// classes (F4).
    AlreadyAbsent,
    RefusedByHook,
    RefusedByHazard,
    FamilyBusy,
}

impl Classification {
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Lane => "lane",
            Self::MemberInLane => "member-in-lane",
            Self::FallbackWorktree => "fallback-worktree",
            Self::AlreadyAbsent => "already-absent",
            Self::RefusedByHook => "refused-by-hook",
            Self::RefusedByHazard => "refused-by-hazard",
            Self::FamilyBusy => "family-busy",
        }
    }
}

/// A hook refusal: one cause, one remedy, one class. The stderr line and the
/// log line are both built from it, so they never drift (D10).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HookFailure {
    pub(crate) class: Classification,
    pub(crate) cause: String,
    pub(crate) remedy: String,
    /// The lane this refusal was about, when the hook learned it from the
    /// family rather than from the payload (F4). The remove payload carries
    /// no `name`.
    pub(crate) name: Option<String>,
    /// The path the refusal was about, when it was known (F4).
    pub(crate) path: Option<PathBuf>,
}

impl HookFailure {
    pub(crate) fn refused(cause: impl Into<String>, remedy: impl Into<String>) -> Self {
        Self {
            class: Classification::RefusedByHook,
            cause: cause.into(),
            remedy: remedy.into(),
            name: None,
            path: None,
        }
    }

    pub(crate) fn hazard(cause: impl Into<String>, remedy: impl Into<String>) -> Self {
        Self {
            class: Classification::RefusedByHazard,
            cause: cause.into(),
            remedy: remedy.into(),
            name: None,
            path: None,
        }
    }

    pub(crate) fn busy(cause: impl Into<String>) -> Self {
        Self {
            class: Classification::FamilyBusy,
            cause: cause.into(),
            remedy: "family busy; retry".to_owned(),
            name: None,
            path: None,
        }
    }

    /// Record the path this refusal was about, so the log line carries it
    /// instead of `path=-` (F4).
    pub(crate) fn at(mut self, path: &std::path::Path) -> Self {
        self.path = Some(path.to_path_buf());
        self
    }

    /// Record the lane name the family gave, so the log line carries it
    /// instead of the destination's basename (F4).
    pub(crate) fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// D10's one stderr line: `gwz: <cause>; <the one command that resolves
    /// it>`.
    pub(crate) fn line(&self) -> String {
        format!("gwz: {}; {}", self.cause, self.remedy)
    }
}

/// What a create decided, and what the remove hook did.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HookSuccess {
    /// The path the create hook prints. The remove hook prints nothing, so
    /// it records the path it acted on here instead and the driver keeps it
    /// off stdout.
    pub(crate) path: PathBuf,
    pub(crate) class: Classification,
    pub(crate) outcome: &'static str,
    /// The lane name the family gave, when the hook learned it there (F4).
    pub(crate) name: Option<String>,
}

/// The invocation context: what the hook resolved before it read the family.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HookContext {
    /// `CLAUDE_PROJECT_DIR`, or the process working directory when the
    /// variable is absent so the command can be run by hand (S1.1).
    pub(crate) project_dir: PathBuf,
    pub(crate) options: HookOptions,
}

/// The pieces of `gwz-core` the hook drives, named once so the two R20/R21
/// integration points are the only places the pending API shape is assumed.
pub(crate) mod integration {
    use std::time::Duration;

    /// Who owns the wait on a busy family lock.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum WaitOwner {
        /// The request carries no wait and the hook polls to its own
        /// deadline (used when the wait rounds to zero).
        Hook,
        /// The request carries `--wait <secs>` and gwz-core polls (R21).
        Core,
    }

    /// What the owner integration point is being asked to do.
    pub(crate) enum OwnerOp<'a> {
        /// Put this invocation's `session_id` on the clone request, so the
        /// row records it in the same index write that reserves it (R20).
        Attach(&'a mut gwz_core::CloneLocalWorkspaceRequest, &'a str),
        /// Read the owner token back off a listed row (R20).
        Read(&'a gwz_core::LocalFamilyMemberEntry),
    }

    /// R20 integration point: the owner token, in and out.
    ///
    /// The clone request's `owner` is recorded on the family row in the same
    /// index write that reserves it, and read back from the listed row. A
    /// row with no owner (made by hand, or before the workspace's first
    /// owned lane) reads `None`, which is the fail-closed side of D6's
    /// table: it is refused rather than reused.
    pub(crate) fn owner(op: OwnerOp<'_>) -> Option<String> {
        match op {
            OwnerOp::Attach(request, session_id) => {
                request.owner = Some(session_id.to_owned());
                request.owner.clone()
            }
            OwnerOp::Read(entry) => entry.owner.clone(),
        }
    }

    /// R21 integration point: the dispose wait.
    ///
    /// The request carries `--wait <secs>` and gwz-core polls `try_lock` to
    /// the deadline, rereading before it acts. A wait under one second
    /// rounds to none, and then the remove hook polls itself.
    pub(crate) fn dispose_wait(
        request: &mut gwz_core::LocalFamilyRequest,
        wait: Duration,
    ) -> WaitOwner {
        let secs = wait.as_secs();
        if secs == 0 {
            request.wait_seconds = None;
            WaitOwner::Hook
        } else {
            request.wait_seconds = Some(i64::try_from(secs).unwrap_or(i64::MAX));
            WaitOwner::Core
        }
    }
}
