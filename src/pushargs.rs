use clap::Args;

// `gwz push` options; everything else push takes is a global option. Push plan
// step 3.7 and D7 (`dev-docs/GwzUrlSchemePushPlan.md`).
#[derive(Clone, Debug, Args)]
pub(crate) struct PushArgs {
    #[arg(
        long = "check-remotes",
        help = CHECK_REMOTES_HELP,
        long_help = CHECK_REMOTES_LONG_HELP
    )]
    pub(crate) check_remotes: bool,
}

pub(crate) const CHECK_REMOTES_HELP: &str = "Read every selected remote and every root dependency instead of skipping repositories that are unchanged since the last fetch or push";

pub(crate) const CHECK_REMOTES_LONG_HELP: &str = "\
Read every selected remote and every root dependency instead of skipping \
repositories that are unchanged since the last fetch or push. Repositories \
whose remote lacks their branch's commit are pushed, and a selected root is \
proven against its committed lock even when it has nothing to push.";

impl PushArgs {
    /// `--check-remotes` sends `remote_check: always`. Without it the field
    /// stays unset, which core reads as `changed`. The global `--force` is not
    /// a forced push and plays no part (plan §3.5).
    pub(crate) fn remote_check(&self) -> Option<gwz_core::RemoteCheck> {
        self.check_remotes.then_some(gwz_core::RemoteCheck::Always)
    }
}

/// Push `Noop` reasons that end with this phrase were decided from the
/// last-known ref without reading the remote (plan §3.6): "up to date with
/// origin/<branch> as of the last fetch or push" and "behind origin/<branch> as
/// of the last fetch or push". "already on origin" was read in the operation.
const UNCHECKED_REASON_SUFFIX: &str = "as of the last fetch or push";

/// Whether a push `Noop` reason says the remote was not read. This is the one
/// place the CLI matches core's reason text.
pub(crate) fn push_reason_is_unchecked(reason: &str) -> bool {
    reason.ends_with(UNCHECKED_REASON_SUFFIX)
}

/// The human summary line for push `Noop` rows whose remote was not read, when
/// there is at least one.
pub(crate) fn unchecked_push_summary(members: &[gwz_core::MemberResponse]) -> Option<String> {
    let unchecked = members
        .iter()
        .filter(|member| member.status == gwz_core::MemberStatus::Noop)
        .filter_map(|member| member.planned.as_ref()?.message.as_deref())
        .filter(|reason| push_reason_is_unchecked(reason))
        .count();
    match unchecked {
        0 => None,
        1 => Some("1 repository unchanged since the last fetch or push was not checked for changes; --check-remotes to verify".to_owned()),
        count => Some(format!("{count} repositories unchanged since the last fetch or push were not checked for changes; --check-remotes to verify")),
    }
}
