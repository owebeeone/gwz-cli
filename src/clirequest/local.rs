use clap::{Args, Subcommand};

use crate::*;

use super::*;

/// The hazard vocabulary itself belongs to `gwz-core`
/// (`gwz_local_disposal::HazardWaiver`); this list is help text only, so a
/// name added there is not silently rejected here.
pub(crate) const HAZARD_NAMES: &str = "open-merge, dirty, unpreserved-history";

/// `--wait <secs>` help, identical on every family verb (GwzLaneCleanFixes
/// R21) so one wrapper may pass it to all of them.
const WAIT_HELP: &str = "Seconds to keep retrying a busy family lock";
const WAIT_LONG_HELP: &str = "Seconds to keep retrying a busy family lock before reporting it busy. The family lock is held for the whole of a create, a dispose or a disband, so two unattended invocations fired for one request would otherwise refuse each other outright. GWZ retries the try-lock at a short fixed interval until the deadline; there is no blocking acquisition, so the wait stays portable and is bounded by the number you give. Omit it and a busy lock refuses immediately, exactly as before. A wait that wins the lock rereads the index before acting, so a create that waited behind another create of the same name is answered by the family that create left, not by a stale view.";

/// `--owner <token>` (R20). The alphabet and the 128-byte limit are the
/// model's (`gwz_family_model::OwnerToken`); this parser is the earlier
/// answer, not the only one, so a token that would be refused after a
/// workspace discovery and a family read is refused at parse instead.
fn parse_owner_token(value: &str) -> Result<String, String> {
    const LIMIT: usize = 128;
    if value.is_empty() {
        return Err("--owner <token> must not be empty".to_owned());
    }
    if value.len() > LIMIT {
        return Err(format!(
            "--owner <token> is at most {LIMIT} bytes; this one is {}",
            value.len()
        ));
    }
    if let Some(character) = value.chars().find(|character| {
        !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | ':' | '-')
    }) {
        return Err(format!(
            "--owner <token> accepts only `[A-Za-z0-9._:-]`; `{character}` is not one of them"
        ));
    }
    Ok(value.to_owned())
}

/// `--wait <secs>`: a count of seconds, so a negative one is not a wait.
fn parse_wait_seconds(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "--wait <secs> requires an integer number of seconds".to_owned())?;
    if parsed < 0 {
        return Err("--wait <secs> must be zero or greater".to_owned());
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Args)]
pub(crate) struct LocalArgs {
    #[command(subcommand)]
    pub(crate) command: LocalCommandArgs,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum LocalCommandArgs {
    #[command(
        about = "Create a local clone of this workspace as a new family member",
        long_about = LOCAL_CLONE_LONG,
        after_long_help = LOCAL_CLONE_AFTER,
        override_usage = "gwz local clone <name> [dest] [--owner <token>] [--wait <secs>]"
    )]
    Clone(LocalCloneArgs),
    #[command(
        about = "List the local clone family recorded on the workspace root",
        long_about = LOCAL_LIST_LONG,
        after_long_help = LOCAL_LIST_AFTER
    )]
    List(LocalListArgs),
    #[command(
        about = "Dispose of a local family member, or detach it with --keep",
        long_about = LOCAL_DISPOSE_LONG,
        after_long_help = LOCAL_DISPOSE_AFTER,
        override_usage = "gwz local dispose <name> [--keep] [--wait <secs>]\n       gwz local dispose <name> --force <hazard,...> [--wait <secs>]"
    )]
    Dispose(LocalDisposeArgs),
    #[command(
        about = "Retire the family: remove pointers and the index; every tree stays",
        long_about = LOCAL_DISBAND_LONG,
        after_long_help = LOCAL_DISBAND_AFTER
    )]
    Disband(LocalDisbandArgs),
}

/// `gwz local list`. Observation-only: it takes no family lock, so it carries
/// no options at all (A2; Surface F6).
#[derive(Clone, Debug, Args)]
pub(crate) struct LocalListArgs {}

/// `gwz local disband`.
#[derive(Clone, Debug, Args)]
pub(crate) struct LocalDisbandArgs {
    #[arg(
        long,
        value_name = "secs",
        value_parser = parse_wait_seconds,
        help = WAIT_HELP,
        long_help = WAIT_LONG_HELP
    )]
    pub(crate) wait: Option<i64>,
}

/// `gwz local clone <name> [dest]` (design §4; operator ruling 2026-09-06,
/// §11 item 28). Creation lives under the family's own verb, with the member
/// name positional like `dispose <name>`'s. `gwz clone` is the URL form only.
#[derive(Clone, Debug, Args)]
pub(crate) struct LocalCloneArgs {
    #[arg(
        value_name = "name",
        help = "Family member name for the new clone",
        long_help = "Family member name for the new clone. `root`, `origin` and Git's reserved ref names are not accepted, and a name already recorded in the family is refused."
    )]
    pub(crate) name: String,

    #[arg(
        value_name = "dest",
        help = "Destination directory (default ../<root-dirname>-<name>)",
        long_help = "Destination directory of the new clone. Defaults to `../<root-dirname>-<name>` beside the workspace root. A nonempty directory, a directory that is already a workspace, and a path inside any family member are refused."
    )]
    pub(crate) dest: Option<String>,

    #[arg(
        long,
        help = "Copy the source tree as it sits (the default)",
        long_help = "Copy the source tree as it sits, including staged edits, unstaged edits, untracked files and build directories. This is the default mode. It is refused while the source has an open coordinated merge. Mutually exclusive with --clean and --bare."
    )]
    pub(crate) verbatim: bool,

    #[arg(
        long,
        help = "Reserved clean mode (unsupported in this build)",
        long_help = "Unsupported in this build; parsed but refused before copying. Check out the frozen source state in the destination: no worktree or index dirt is inherited, and no build directories are copied. Mutually exclusive with --verbatim."
    )]
    pub(crate) clean: bool,

    #[arg(
        long,
        help = "Reserved bare mode (unsupported in this build)",
        long_help = "Unsupported in this build; parsed but refused before copying. Create the destination as a share point: the same workspace layout, with every member repository bare. Implies --clean. Verbs that need a worktree refuse there; push, fetch, log, `gwz local list` and dispose work."
    )]
    pub(crate) bare: bool,

    #[arg(
        short = 'b',
        value_name = "branch",
        help = "Reserved branch option (unsupported in this build)",
        long_help = "Unsupported in this build; parsed but refused before copying. Create this branch in every destination repository at the frozen commit, before the clone is marked ready. Accepted only with --clean or --bare. If the branch already exists in any member, the whole create is refused."
    )]
    pub(crate) branch: Option<String>,

    #[arg(
        long,
        value_name = "name|path",
        help = "Copy from this family member or path instead of the current workspace",
        long_help = "Unsupported in this build; parsed but refused before copying. Copy from this family member or path instead of the current workspace. Accepts a family name recorded in the index or a filesystem path. Core resolves the token, and refuses one that names no readable source. The new clone is registered on the workspace root whichever member it was copied from."
    )]
    pub(crate) from: Option<String>,

    #[arg(
        long,
        value_name = "token",
        value_parser = parse_owner_token,
        help = "Record this opaque caller token on the new member row",
        long_help = "Record this opaque token on the new member's row, in the same index write that reserves the row. Up to 128 bytes of `[A-Za-z0-9._:-]`. It is the caller's own identity for the caller's own reuse decisions: GWZ stores it, reports it in `gwz local list` (a column, and an `owner` field under --json), and never interprets, matches or acts on it. A row created without --owner records none, and no later command ever sets, changes or clears a row's token. Recording one makes the family index format 2, which gwz 1.0.14 and later read; an older gwz refuses the whole index and says so."
    )]
    pub(crate) owner: Option<String>,

    #[arg(
        long,
        value_name = "secs",
        value_parser = parse_wait_seconds,
        help = WAIT_HELP,
        long_help = WAIT_LONG_HELP
    )]
    pub(crate) wait: Option<i64>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct LocalDisposeArgs {
    #[arg(value_name = "name", help = "Family member name recorded in the index")]
    pub(crate) name: String,

    // `--force` is the existing global switch (design §5.2 says the hazards are
    // "explicitly named by the existing force syntax"), so the names it
    // authorizes are operands of this command rather than a second `--force`
    // argument: a subcommand may never reuse a global Clap id (DR-5,
    // `tests::g01::no_subcommand_argument_reuses_a_global_argument_id`), and a
    // second `--force` long name in one command is rejected outright. The
    // spelling `gwz local dispose C --force open-merge,dirty` is unchanged.
    #[arg(
        value_name = "hazard,...",
        help = "Deletion hazards authorized by --force (comma-separated)",
        long_help = "Deletion hazards this disposal is authorized to waive, comma-separated, and accepted only together with --force. GWZ refuses to delete a member whose work or history is not verifiably preserved elsewhere; every hazard it reports must be named here before it will. Known names: open-merge, dirty, unpreserved-history. This is an operator loss waiver, not crash recovery, and it is mutually exclusive with --keep."
    )]
    pub(crate) hazards: Vec<String>,

    #[arg(
        long,
        help = "Detach only: remove the pointer and row; the tree stays on disk",
        long_help = "Detach only: remove the pointer and the index row. The member's entire tree, its open merge and its history stay on disk and remain usable as an ordinary workspace. Mutually exclusive with --force."
    )]
    pub(crate) keep: bool,

    #[arg(
        long,
        value_name = "secs",
        value_parser = parse_wait_seconds,
        help = WAIT_HELP,
        long_help = WAIT_LONG_HELP
    )]
    pub(crate) wait: Option<i64>,
}

impl LocalArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        force: bool,
    ) -> Result<CliRequest, CliError> {
        let request = match &self.command {
            LocalCommandArgs::Clone(args) => return args.request(meta),
            LocalCommandArgs::List(_) => gwz_core::LocalFamilyRequest {
                meta,
                op: gwz_core::LocalFamilyOp::List,
                name: None,
                keep: None,
                force_hazards: Vec::new(),
                // A listing takes no family lock, so there is nothing to wait
                // for and the field stays unset.
                wait_seconds: None,
            },
            LocalCommandArgs::Disband(args) => gwz_core::LocalFamilyRequest {
                meta,
                op: gwz_core::LocalFamilyOp::Disband,
                name: None,
                keep: None,
                force_hazards: Vec::new(),
                wait_seconds: args.wait,
            },
            LocalCommandArgs::Dispose(args) => {
                let force_hazards = args.force_hazards(force)?;
                if args.keep && !force_hazards.is_empty() {
                    return Err(CliError::invalid_request(
                        "--keep and --force <hazards> are mutually exclusive",
                    ));
                }
                gwz_core::LocalFamilyRequest {
                    meta,
                    op: gwz_core::LocalFamilyOp::Dispose,
                    name: Some(args.name.clone()),
                    keep: args.keep.then_some(true),
                    force_hazards,
                    wait_seconds: args.wait,
                }
            }
        };
        Ok(CliRequest::LocalFamily(request))
    }
}

impl LocalCloneArgs {
    /// `gwz local clone <name> [dest]` -> `CloneLocalWorkspaceRequest` (design
    /// §4, §7). Every refusal below happens before the request is built; core
    /// owns name validity, destination defaulting and every workspace check.
    /// The wire is the one `gwz clone --local` built before the 2026-09-06
    /// ruling: only the spelling moved.
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        // Operator ruling 4 (2026-09-06, design §7 and §11 item 20): an empty
        // name refuses here, before anything is encoded. Core's own shape
        // check (`validate_clone_local`) refuses the empty and the reserved
        // names as well, so this is the earlier answer rather than the only
        // one — it costs no workspace discovery and no family read to say the
        // same thing about a name that was never typed. A *missing* name is
        // Clap's own required-argument error, as it is for `dispose <name>`.
        if self.name.is_empty() {
            return Err(CliError::invalid_request(
                "local clone <name> must not be empty",
            ));
        }
        if self.verbatim && self.clean {
            return Err(CliError::invalid_request(
                "--verbatim and --clean are mutually exclusive",
            ));
        }
        if self.verbatim && self.bare {
            return Err(CliError::invalid_request(
                "--verbatim and --bare are mutually exclusive (--bare implies --clean)",
            ));
        }
        // Design §7 tag 6 is `copy_source` (operator ruling 2026-09-05). The
        // token itself is core's to resolve — a family name, a path, or
        // neither — so only the shape that would be *misread* on the wire is
        // refused here: an empty string is indistinguishable from "absent",
        // which core reads as "copy this workspace".
        if self.from.as_ref().is_some_and(|value| value.is_empty()) {
            return Err(CliError::invalid_request(
                "--from <name|path> must not be empty",
            ));
        }
        let mode = if self.bare {
            gwz_core::LocalCloneMode::Bare
        } else if self.clean {
            gwz_core::LocalCloneMode::Clean
        } else {
            gwz_core::LocalCloneMode::Verbatim
        };
        if self.branch.is_some() && mode == gwz_core::LocalCloneMode::Verbatim {
            return Err(CliError::invalid_request(
                "-b <branch> is accepted only with --clean or --bare",
            ));
        }
        Ok(CliRequest::CloneLocalWorkspace(
            gwz_core::CloneLocalWorkspaceRequest {
                meta,
                name: self.name.clone(),
                // Absent means core's `../<root-dirname>-<name>` default.
                dest: self.dest.clone(),
                mode,
                branch: self.branch.clone(),
                // Tag 6 `copy_source` (design §7, §11 item 11): `--from`, the
                // family name or path to copy *from*. Absent means the
                // workspace this command was run in.
                copy_source: self.from.clone(),
                // R20: the caller's opaque token, already shape-checked by
                // the value parser. Core checks the same shape again and
                // owns what it means -- which is nothing.
                owner: self.owner.clone(),
                // R21: how long a busy family lock is retried.
                wait_seconds: self.wait,
            },
        ))
    }
}

impl LocalDisposeArgs {
    /// The waived hazard names, exactly as typed. The vocabulary is core's, so
    /// an unrecognized name travels and core answers for it; only the shapes
    /// that would be *misread* on the wire are refused here — an empty list
    /// means "no force", so encoding one for a `--force` invocation would turn
    /// an authorization into its silent opposite (design §8.4).
    ///
    /// The split is on `,` and nothing else: no trimming, no case folding, no
    /// normalization. A deletion waiver is the operator's authorization token,
    /// and rewriting it before encoding would make the CLI answer for a
    /// vocabulary it does not own. `gwz-py` splits identically (lane CP), so
    /// the same argv encodes the same `force_hazards` in both drivers.
    fn force_hazards(&self, force: bool) -> Result<Vec<String>, CliError> {
        let hazards = self
            .hazards
            .iter()
            .flat_map(|value| value.split(','))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if !force {
            if hazards.is_empty() {
                return Ok(Vec::new());
            }
            return Err(CliError::invalid_request(
                "hazard names are accepted only with --force",
            ));
        }
        if hazards.is_empty() {
            return Err(CliError::invalid_request(format!(
                "--force requires one or more hazard names, comma-separated ({HAZARD_NAMES})"
            )));
        }
        if hazards.iter().any(String::is_empty) {
            return Err(CliError::invalid_request(format!(
                "--force <hazard,...> must not contain an empty hazard name ({HAZARD_NAMES})"
            )));
        }
        Ok(hazards)
    }
}
