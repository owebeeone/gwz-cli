use crate::*;

use super::merge::merge_start_request;
use super::*;

impl BranchArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let operations = usize::from(self.list)
            + usize::from(self.create.is_some())
            + usize::from(self.delete.is_some())
            + usize::from(self.merge.is_some());
        if operations > 1 {
            return Err(CliError::new(
                "branch accepts only one of --list, --create, --delete, or --merge",
            ));
        }
        if self.switch && self.create.is_none() {
            return Err(CliError::new("--switch requires --create"));
        }
        if self.from.is_some() && self.create.is_none() {
            return Err(CliError::new("--from requires --create"));
        }

        if let Some(source_ref) = &self.merge {
            return Ok(CliRequest::Merge(merge_start_request(
                meta,
                source_ref.clone(),
            )));
        }

        let op = if self.create.is_some() {
            gwz_core::BranchOp::Create
        } else if self.delete.is_some() {
            gwz_core::BranchOp::Delete
        } else {
            gwz_core::BranchOp::List
        };

        Ok(CliRequest::Branch(gwz_core::BranchRequest {
            meta,
            op,
            name: self.create.clone().or_else(|| self.delete.clone()),
            start_ref: self
                .create
                .as_ref()
                .map(|_| self.from.clone().unwrap_or_else(|| "HEAD".to_owned())),
            switch_after_create: self.switch.then_some(true),
        }))
    }
}

impl StashArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match &self.command {
            StashCommandArgs::Push(args) => args.request(meta),
            StashCommandArgs::List(args) => Ok(CliRequest::Stash(gwz_core::StashRequest {
                meta,
                op: gwz_core::StashOp::List,
                expanded: args.expanded.then_some(true),
                ..Default::default()
            })),
            StashCommandArgs::Apply(args) => Ok(CliRequest::Stash(gwz_core::StashRequest {
                meta,
                op: gwz_core::StashOp::Apply,
                stash_id: args.stash_id.clone(),
                ..Default::default()
            })),
            StashCommandArgs::Pop(args) => Ok(CliRequest::Stash(gwz_core::StashRequest {
                meta,
                op: gwz_core::StashOp::Pop,
                stash_id: args.stash_id.clone(),
                ..Default::default()
            })),
            StashCommandArgs::Drop(args) => Ok(CliRequest::Stash(gwz_core::StashRequest {
                meta,
                op: gwz_core::StashOp::Drop,
                stash_id: Some(args.stash_id.clone()),
                ..Default::default()
            })),
        }
    }
}

impl StashPushArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        if self.include_untracked && self.include_ignored {
            return Err(CliError::new("-u and -a are mutually exclusive"));
        }
        Ok(CliRequest::Stash(gwz_core::StashRequest {
            meta,
            op: gwz_core::StashOp::Push,
            stash_id: None,
            message: self.message.clone(),
            include_untracked: self.include_untracked.then_some(true),
            include_ignored: self.include_ignored.then_some(true),
            expanded: None,
            preserve_index: None,
        }))
    }
}
