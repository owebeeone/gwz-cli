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

impl LocalDisposeArgs {
    /// The waived hazard names, exactly as typed. The vocabulary is core's, so
    /// an unrecognized name travels and core answers for it; only the shapes
    /// that would be *misread* on the wire are refused here — an empty list
    /// means "no force", so encoding one for a `--force` invocation would turn
    /// an authorization into its silent opposite (design §8.4).
    fn force_hazards(&self, force: bool) -> Result<Vec<String>, CliError> {
        let hazards = self
            .hazards
            .iter()
            .flat_map(|value| value.split(','))
            .map(|hazard| hazard.trim().to_owned())
            .collect::<Vec<_>>();
        if !force {
            if hazards.is_empty() {
                return Ok(Vec::new());
            }
            return Err(CliError::invalid_request(
                "hazard names are accepted only with --force",
            ));
        }
        if hazards.is_empty() || hazards.iter().all(String::is_empty) {
            return Err(CliError::invalid_request(format!(
                "--force requires one or more hazard names, comma-separated ({HAZARD_NAMES})"
            )));
        }
        if hazards.iter().any(String::is_empty) {
            return Err(CliError::invalid_request(
                "--force <hazard,...> must not contain an empty hazard name",
            ));
        }
        Ok(hazards)
    }
}
