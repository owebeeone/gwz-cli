use crate::*;

use super::*;

impl MergeArgs {
    /// `local_source_name` is the global `--remote` token when merge is the
    /// verb (design §6): family-only, resolved by core at operation time and
    /// never persisted as a Git remote.
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        local_source_name: Option<String>,
    ) -> Result<CliRequest, CliError> {
        let lifecycle_ops = usize::from(self.resume)
            + usize::from(self.abort)
            + usize::from(self.status.is_some())
            + usize::from(self.gc.is_some());
        if lifecycle_ops > 1 {
            return Err(CliError::invalid_request(
                "merge accepts only one lifecycle operation",
            ));
        }
        // §7: the selector is start-only. Core repeats the rule; the driver
        // fails fast so no lifecycle request is built carrying it.
        if local_source_name.is_some() && lifecycle_ops > 0 {
            return Err(CliError::invalid_request(
                "--remote <name> is accepted only when starting a merge",
            ));
        }
        if self.ff_only && self.no_ff {
            return Err(CliError::invalid_request(
                "--ff-only and --no-ff are mutually exclusive",
            ));
        }
        // The crash-recovery decision is made once, at the start that opens the
        // attempt; a later lifecycle op never consults the flag. Mirrors core's
        // own rule so the driver fails fast — core stays the authority.
        if self.filesystem_strict && lifecycle_ops > 0 {
            return Err(CliError::invalid_request(
                "--filesystem-strict is accepted only when starting a merge",
            ));
        }
        let op = if self.resume {
            gwz_core::MergeOp::Resume
        } else if self.abort {
            gwz_core::MergeOp::Abort
        } else if self.status.is_some() {
            gwz_core::MergeOp::Status
        } else if self.gc.is_some() {
            gwz_core::MergeOp::Gc
        } else {
            gwz_core::MergeOp::Start
        };
        Ok(CliRequest::Merge(gwz_core::MergeRequest {
            meta,
            op,
            source_ref: self.source.clone(),
            merge_id: self
                .status
                .clone()
                .flatten()
                .or_else(|| self.gc.clone().flatten()),
            mode: if self.ff_only {
                Some(gwz_core::MergeMode::FfOnly)
            } else if self.no_ff {
                Some(gwz_core::MergeMode::NoFf)
            } else {
                None
            },
            message: self.message.clone(),
            preserve: self.preserve.then_some(true),
            filesystem_strict: self.filesystem_strict.then_some(true),
            local_source_name,
        }))
    }
}

pub(super) fn merge_start_request(
    meta: gwz_core::RequestMeta,
    source_ref: String,
) -> gwz_core::MergeRequest {
    gwz_core::MergeRequest {
        meta,
        op: gwz_core::MergeOp::Start,
        source_ref: Some(source_ref),
        ..gwz_core::MergeRequest::default()
    }
}
