use crate::*;

use super::*;

impl MergeArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let lifecycle_ops = usize::from(self.resume)
            + usize::from(self.abort)
            + usize::from(self.status.is_some())
            + usize::from(self.gc.is_some());
        if lifecycle_ops > 1 {
            return Err(CliError::invalid_request(
                "merge accepts only one lifecycle operation",
            ));
        }
        if self.ff_only && self.no_ff {
            return Err(CliError::invalid_request(
                "--ff-only and --no-ff are mutually exclusive",
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
