use clap::{Args, Subcommand};

use crate::*;

use super::*;

/// The hazard vocabulary itself belongs to `gwz-core`
/// (`gwz_local_disposal::HazardWaiver`); this list is help text only, so a
/// name added there is not silently rejected here.
pub(crate) const HAZARD_NAMES: &str = "open-merge, dirty, unpreserved-history";

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
        override_usage = "gwz local clone <name> [dest] [--clean | --bare] [-b <branch>] [--from <name|path>]"
    )]
    Clone(LocalCloneArgs),
    #[command(
        about = "List the local clone family recorded on the workspace root",
        long_about = LOCAL_LIST_LONG,
        after_long_help = LOCAL_LIST_AFTER
    )]
    List,
    #[command(
        about = "Dispose of a local family member, or detach it with --keep",
        long_about = LOCAL_DISPOSE_LONG,
        after_long_help = LOCAL_DISPOSE_AFTER,
        override_usage = "gwz local dispose <name> [--keep]\n       gwz local dispose <name> --force <hazard,...>"
    )]
    Dispose(LocalDisposeArgs),
    #[command(
        about = "Retire the family: remove pointers and the index; every tree stays",
        long_about = LOCAL_DISBAND_LONG,
        after_long_help = LOCAL_DISBAND_AFTER
    )]
    Disband,
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
        help = "Check out the frozen source state without worktree dirt",
        long_help = "Check out the frozen source state in the destination: no worktree or index dirt is inherited, and no build directories are copied. Mutually exclusive with --verbatim."
    )]
    pub(crate) clean: bool,

    #[arg(
        long,
        help = "Create bare member repositories (implies --clean)",
        long_help = "Create the destination as a share point: the same workspace layout, with every member repository bare. Implies --clean. Verbs that need a worktree refuse there; push, fetch, log, `gwz local list` and dispose work."
    )]
    pub(crate) bare: bool,

    #[arg(
        short = 'b',
        value_name = "branch",
        help = "Create this branch in every destination repository (--clean/--bare only)",
        long_help = "Create this branch in every destination repository at the frozen commit, before the clone is marked ready. Accepted only with --clean or --bare. If the branch already exists in any member, the whole create is refused."
    )]
    pub(crate) branch: Option<String>,

    #[arg(
        long,
        value_name = "name|path",
        help = "Copy from this family member or path instead of the current workspace",
        long_help = "Copy from this family member or path instead of the current workspace. Accepts a family name recorded in the index or a filesystem path. Core resolves the token, and refuses one that names no readable source. The new clone is registered on the workspace root whichever member it was copied from."
    )]
    pub(crate) from: Option<String>,
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
}

impl LocalArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        force: bool,
    ) -> Result<CliRequest, CliError> {
        let request = match &self.command {
            LocalCommandArgs::Clone(args) => return args.request(meta),
            LocalCommandArgs::List => gwz_core::LocalFamilyRequest {
                meta,
                op: gwz_core::LocalFamilyOp::List,
                name: None,
                keep: None,
                force_hazards: Vec::new(),
            },
            LocalCommandArgs::Disband => gwz_core::LocalFamilyRequest {
                meta,
                op: gwz_core::LocalFamilyOp::Disband,
                name: None,
                keep: None,
                force_hazards: Vec::new(),
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
