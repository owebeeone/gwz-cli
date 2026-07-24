use crate::*;

use super::*;

impl Cli {
    pub(crate) fn validate(&self) -> Result<(), CliError> {
        if self.global.json && self.global.jsonl {
            return Err(CliError::new("--json and --jsonl are mutually exclusive"));
        }
        if let CommandArgs::Status(status) = &self.command {
            status.validate(&self.global)?;
        }
        if matches!(&self.command, CommandArgs::Clone(_)) && self.global.dry_run {
            return Err(CliError::new("--dry-run is not supported for clone"));
        }
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> OutputMode {
        if matches!(&self.command, CommandArgs::Status(status) if status.porcelain) {
            OutputMode::Porcelain
        } else if self.global.json {
            OutputMode::Json
        } else if self.global.jsonl {
            OutputMode::Jsonl
        } else {
            OutputMode::Human
        }
    }

    pub(crate) fn request_meta(&self, request_id: &str) -> gwz_core::RequestMeta {
        gwz_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            workspace: self
                .global
                .root
                .as_ref()
                .map(|root| gwz_core::WorkspaceRef {
                    root: Some(root.clone()),
                    workspace_id: None,
                }),
            selection: self.selection(),
            policy: self.policy(),
            dry_run: self.global.dry_run.then_some(true),
            ..Default::default()
        }
    }

    pub(crate) fn selection(&self) -> Option<gwz_core::Selection> {
        let mut targets = Vec::new();
        if self.global.all {
            targets.push("@all".to_owned());
        }
        targets.extend(self.global.targets.clone());
        targets.extend(self.global.members.clone());
        targets.extend(self.global.paths.clone());

        let mut exclude_targets = self.global.exclude_targets.clone();
        exclude_targets.extend(self.global.exclude_members.clone());
        exclude_targets.extend(self.global.exclude_paths.clone());

        if !targets.is_empty() || !exclude_targets.is_empty() {
            Some(gwz_core::Selection {
                targets,
                exclude_targets,
                ..Default::default()
            })
        } else {
            None
        }
    }

    pub(crate) fn policy(&self) -> Option<gwz_core::OperationPolicy> {
        Some(gwz_core::OperationPolicy {
            partial: self
                .global
                .partial
                .then_some(gwz_core::PartialBehavior::Partial),
            destructive: self
                .global
                .force
                .then_some(gwz_core::DestructiveBehavior::Allow),
            sync: self.global.sync.map(Into::into),
            remote: self.global.remote.clone(),
            concurrency: self.global.jobs,
            max_connections_per_host: self.global.max_per_host,
            progress_min_interval_ms: Some(
                self.global
                    .progress_interval
                    .unwrap_or(DEFAULT_PROGRESS_MIN_INTERVAL_MS),
            ),
            ..Default::default()
        })
    }

    pub(crate) fn command_request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
        current_dir: &std::path::Path,
    ) -> Result<CliRequest, CliError> {
        match &self.command {
            CommandArgs::Init(args) => args.request(meta, workspace_root),
            CommandArgs::Clone(args) => args.request(meta),
            CommandArgs::Add(args) => args.request(meta, current_dir),
            CommandArgs::Repo(args) => args.request(meta),
            CommandArgs::Status(args) => args.request(meta),
            CommandArgs::Ls(args) => args.request(meta),
            CommandArgs::Forall(args) => {
                if self.global.json || self.global.jsonl {
                    return Err(CliError::new("forall does not support --json/--jsonl"));
                }
                let (mode, command) = match (&args.command_string, args.command.is_empty()) {
                    (Some(script), true) => (gwz_core::ExecMode::Shell, vec![script.clone()]),
                    (None, false) => (gwz_core::ExecMode::Argv, args.command.clone()),
                    (Some(_), false) => {
                        return Err(CliError::new(
                            "use either `-c <string>` or `-- <cmd>`, not both",
                        ));
                    }
                    (None, true) => {
                        return Err(CliError::new(
                            "no command (use `-- <cmd>` or `-c <string>`)",
                        ));
                    }
                };
                Ok(CliRequest::Forall {
                    meta,
                    projects: args.projects.clone(),
                    mode,
                    command,
                    continue_on_fail: self.global.partial,
                    no_banner: args.no_banner,
                })
            }
            CommandArgs::Snapshot(args) => match args.name.clone() {
                Some(name) if !args.list => Ok(CliRequest::Snapshot(gwz_core::SnapshotRequest {
                    meta,
                    snapshot_id: name,
                    source: args.source(),
                })),
                _ if args.branch.is_some() => Err(CliError::new(
                    "--branch requires a snapshot name and cannot be combined with --list",
                )),
                _ => Ok(CliRequest::ListSnapshots(gwz_core::ListSnapshotsRequest {
                    meta,
                })),
            },
            CommandArgs::Tag(args) => {
                let op = if args.push {
                    gwz_core::TagOp::Push
                } else if args.fetch {
                    gwz_core::TagOp::Fetch
                } else if args.delete {
                    gwz_core::TagOp::Delete
                } else if args.list || args.name.is_none() {
                    gwz_core::TagOp::List
                } else {
                    gwz_core::TagOp::Create
                };
                Ok(CliRequest::Tag(gwz_core::TagRequest {
                    meta,
                    op,
                    name: args.name.clone(),
                    message: args.message.clone(),
                    signed: args.signed.then_some(true),
                    remote: self.global.remote.clone(),
                    all: None,
                }))
            }
            CommandArgs::Branch(args) => args.request(if args.merge.is_some() {
                self.merge_meta(meta)
            } else {
                meta
            }),
            CommandArgs::Merge(args) => args.request(self.merge_meta(meta)),
            CommandArgs::Stash(args) => args.request(meta),
            CommandArgs::Materialize(args) => args.request(meta),
            CommandArgs::Pull(args) => args.request(meta),
            CommandArgs::Push => Ok(CliRequest::Push(gwz_core::PushRequest {
                remote: self.global.remote.clone(),
                refspec: None,
                meta,
            })),
            CommandArgs::Capture => Ok(CliRequest::Capture(gwz_core::CaptureRequest { meta })),
            CommandArgs::Commit(args) => args.request(meta),
            CommandArgs::Diff(args) => {
                let workspace_cwd = workspace_relative_cwd(&workspace_root, current_dir);
                args.request(meta, workspace_cwd)
            }
        }
    }

    fn merge_meta(&self, mut meta: gwz_core::RequestMeta) -> gwz_core::RequestMeta {
        if self.global.progress_interval.is_none()
            && let Some(policy) = &mut meta.policy
        {
            policy.progress_min_interval_ms = None;
        }
        meta
    }
}
