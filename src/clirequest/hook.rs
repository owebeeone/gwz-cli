//! `gwz hook claude-code worktree-create|worktree-remove|setup`: the argument
//! surface (D1).
//!
//! Every knob the hooks have is a command-line option on the hook itself,
//! because the desktop app passes no environment variable of ours (section
//! 1). `setup` takes the same options and bakes them into the handler text.
//! Each leaf carries only the options it obeys: creation's guards are absent
//! from `worktree-remove`.

use clap::{Args, Subcommand};

use crate::*;

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HookEvent {
    WorktreeCreate,
    WorktreeRemove,
}

impl HookEvent {
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::WorktreeCreate => "worktree-create",
            Self::WorktreeRemove => "worktree-remove",
        }
    }
}

/// One hook invocation: the event, and the options the handler carried.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HookInvocation {
    pub(crate) event: HookEvent,
    pub(crate) options: crate::hook::HookOptions,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct HookArgs {
    #[command(subcommand)]
    pub(crate) command: HookCommandArgs,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum HookCommandArgs {
    #[command(
        name = "claude-code",
        about = "Serve Claude Code's worktree hooks for the workspace a session starts in",
        long_about = HOOK_CLAUDE_CODE_LONG,
        after_long_help = HOOK_CLAUDE_CODE_AFTER
    )]
    ClaudeCode(HookClaudeCodeArgs),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct HookClaudeCodeArgs {
    #[command(subcommand)]
    pub(crate) command: HookClaudeCodeCommandArgs,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum HookClaudeCodeCommandArgs {
    #[command(
        name = "worktree-create",
        about = "Create or verify the working copy Claude Code asked for, and print its path",
        long_about = HOOK_CREATE_LONG,
        after_long_help = HOOK_CREATE_AFTER
    )]
    WorktreeCreate(HookCreateOptionArgs),
    #[command(
        name = "worktree-remove",
        about = "Retire the lane or worktree that `worktree_path` names",
        long_about = HOOK_REMOVE_LONG,
        after_long_help = HOOK_REMOVE_AFTER
    )]
    WorktreeRemove(HookRemoveOptionArgs),
    #[command(
        about = "Print the Claude Code hooks block, and with --write or --remove edit a settings file",
        long_about = CLAUDE_CODE_SETUP_LONG,
        after_long_help = CLAUDE_CODE_SETUP_AFTER,
        override_usage = "gwz hook claude-code setup <--project [--local] | --user> [--write | --remove] [OPTIONS]"
    )]
    Setup(ClaudeCodeSetupArgs),
}

/// The creation leaf's options: the two guards, the deadline, the fallback
/// base and the log.
#[derive(Clone, Debug, Default, Args)]
pub(crate) struct HookCreateOptionArgs {
    #[arg(
        long = "min-free-gb",
        value_name = "gb",
        help = "Refuse a creation that would leave less than this free (default: no floor; only the estimate applies)",
        long_help = "Refuse a creation when free space on the filesystem holding the destination's parent is below this many gigabytes. It is a floor applied on top of the copy-cost estimate, never a replacement for it. Default: no floor, so only the estimate applies."
    )]
    pub(crate) min_free_gb: Option<f64>,

    #[arg(
        long = "max-lanes",
        value_name = "n",
        help = "Refuse a creation once the family holds this many ready lanes (default 8)"
    )]
    pub(crate) max_lanes: Option<u32>,

    #[arg(
        long = "wait-secs",
        value_name = "secs",
        help = "Deadline for the attempt loop (default 300)"
    )]
    pub(crate) wait_secs: Option<u64>,

    #[arg(
        long = "base-ref",
        value_name = "ref",
        help = "Base for the fallback git worktree (default origin/<default-branch>, else HEAD)"
    )]
    pub(crate) base_ref: Option<String>,

    #[arg(
        long = "log",
        value_name = "path",
        help = "Write the hook log here instead of the fixed location"
    )]
    pub(crate) log: Option<String>,
}

/// The removal leaf's options. Removal consumes nothing, so none of
/// creation's guards apply to it.
#[derive(Clone, Debug, Default, Args)]
pub(crate) struct HookRemoveOptionArgs {
    #[arg(
        long = "wait-secs",
        value_name = "secs",
        help = "Deadline for the family lock (default 300)"
    )]
    pub(crate) wait_secs: Option<u64>,

    #[arg(
        long = "log",
        value_name = "path",
        help = "Write the hook log here instead of the fixed location"
    )]
    pub(crate) log: Option<String>,
}

impl HookCreateOptionArgs {
    pub(crate) fn options(&self) -> crate::hook::HookOptions {
        crate::hook::HookOptions {
            min_free_gb: self.min_free_gb,
            max_lanes: self.max_lanes.unwrap_or(crate::hook::DEFAULT_MAX_LANES),
            wait_secs: self.wait_secs.unwrap_or(crate::hook::DEFAULT_WAIT_SECS),
            base_ref: self.base_ref.clone(),
            log: self.log.clone(),
        }
    }
}

impl HookRemoveOptionArgs {
    pub(crate) fn options(&self) -> crate::hook::HookOptions {
        crate::hook::HookOptions {
            wait_secs: self.wait_secs.unwrap_or(crate::hook::DEFAULT_WAIT_SECS),
            log: self.log.clone(),
            ..crate::hook::HookOptions::default()
        }
    }
}

#[derive(Clone, Debug, Args)]
pub(crate) struct ClaudeCodeSetupArgs {
    #[arg(
        long,
        help = "Use the workspace root's .claude/settings.json",
        conflicts_with = "user"
    )]
    pub(crate) project: bool,

    #[arg(long, help = "Use ~/.claude/settings.json")]
    pub(crate) user: bool,

    #[arg(
        long,
        requires = "project",
        help = "With --project, use settings.local.json instead"
    )]
    pub(crate) local: bool,

    #[arg(
        long,
        conflicts_with = "remove",
        help = "Merge the block into the file; without it the block is only printed"
    )]
    pub(crate) write: bool,

    #[arg(
        long,
        help = "Take the block back out of the file; the file itself is never deleted"
    )]
    pub(crate) remove: bool,

    #[arg(
        long,
        value_name = "PATH",
        help = "Pin an absolute gwz binary in the handler instead of the bare `gwz`"
    )]
    pub(crate) command: Option<String>,

    #[command(flatten)]
    pub(crate) options: HookCreateOptionArgs,
}

impl HookArgs {
    pub(crate) fn request(&self) -> Result<CliRequest, CliError> {
        let HookCommandArgs::ClaudeCode(claude) = &self.command;
        let (event, options) = match &claude.command {
            HookClaudeCodeCommandArgs::WorktreeCreate(options) => {
                (HookEvent::WorktreeCreate, options.options())
            }
            HookClaudeCodeCommandArgs::WorktreeRemove(options) => {
                (HookEvent::WorktreeRemove, options.options())
            }
            HookClaudeCodeCommandArgs::Setup(args) => {
                return args.request();
            }
        };
        Ok(CliRequest::Hook(HookInvocation { event, options }))
    }
}

impl ClaudeCodeSetupArgs {
    pub(crate) fn request(&self) -> Result<CliRequest, CliError> {
        if !self.project && !self.user {
            return Err(CliError::invalid_request(
                "hook claude-code setup needs --project or --user",
            ));
        }
        let placement = if self.project {
            crate::hook::SetupPlacement::Project { local: self.local }
        } else {
            crate::hook::SetupPlacement::User
        };
        Ok(CliRequest::ClaudeCodeSetup(
            crate::hook::setup::SetupRequest {
                placement,
                write: self.write,
                remove: self.remove,
                command: self.command.clone(),
                options: self.options.options(),
            },
        ))
    }
}
