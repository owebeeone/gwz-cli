fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.as_slice() == ["--version"] {
        println!("gwz {}", gwz_core::version());
        return;
    }

    match parse_args(args) {
        Ok(invocation) => match execute_invocation(&invocation) {
            Ok(response) => {
                println!("{}", render_response(&response, invocation.output));
                std::process::exit(exit_code_for_response(&response));
            }
            Err(error) => {
                eprintln!("gwz: {}", error.message);
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("gwz: {}", error.message);
            std::process::exit(2);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CliInvocation {
    request: CliRequest,
    output: OutputMode,
    start_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
enum CliRequest {
    CreateWorkspace(gwz_core::CreateWorkspaceRequest),
    InitFromSources(gwz_core::InitFromSourcesRequest),
    AddExistingRepo(gwz_core::AddExistingRepoRequest),
    CreateRepo(gwz_core::CreateRepoRequest),
    Materialize(gwz_core::MaterializeRequest),
    Status(gwz_core::StatusRequest),
    Snapshot(gwz_core::SnapshotRequest),
    Tag(gwz_core::TagRequest),
    PullHead(gwz_core::PullHeadRequest),
    PullSnapshot(gwz_core::PullSnapshotRequest),
    Push(gwz_core::PushRequest),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Human,
    Json,
    Jsonl,
    Porcelain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CliError {
    message: String,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<CliInvocation, CliError> {
    let cwd = std::env::current_dir().map_err(|error| CliError::new(error.to_string()))?;
    parse_args_with_request_id(args, &new_request_id(), &cwd)
}

fn parse_args_with_request_id(
    args: Vec<String>,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    let parsed = ParsedArgs::parse(args)?;
    let output = parsed.output_mode()?;
    let meta = parsed.request_meta(request_id);
    let workspace_root = parsed
        .root
        .clone()
        .unwrap_or_else(|| current_dir.to_string_lossy().into_owned());
    let request = parsed.command_request(meta, workspace_root)?;
    Ok(CliInvocation {
        request,
        output,
        start_dir: current_dir.to_path_buf(),
    })
}

fn execute_invocation(invocation: &CliInvocation) -> Result<gwz_core::ResponseEnvelope, CliError> {
    let backend = gwz_core::git::Git2Backend::new();
    let operation_id = new_operation_id();
    let start = invocation.start_dir.as_path();
    let response = match &invocation.request {
        CliRequest::CreateWorkspace(request) => {
            gwz_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::InitFromSources(request) => gwz_core::workspace_ops::handle_init_from_sources(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::AddExistingRepo(request) => gwz_core::workspace_ops::handle_add_existing_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::CreateRepo(request) => gwz_core::workspace_ops::handle_create_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Materialize(request) => gwz_core::workspace_ops::handle_materialize(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Status(request) => {
            gwz_core::status::handle_status(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Snapshot(request) => {
            gwz_core::workspace_ops::handle_snapshot(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Tag(request) => {
            gwz_core::workspace_ops::handle_tag(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::PullHead(request) => gwz_core::workspace_ops::handle_pull_head(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::PullSnapshot(request) => gwz_core::workspace_ops::handle_pull_snapshot(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Push(request) => {
            gwz_core::workspace_ops::handle_push(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
    };
    response.map_err(|error| CliError::new(error.to_string()))
}

fn render_response(response: &gwz_core::ResponseEnvelope, output: OutputMode) -> String {
    match output {
        OutputMode::Human => render_human_response(response),
        OutputMode::Json => response_json(response).to_string(),
        OutputMode::Jsonl => render_jsonl_stream(response, &[], None),
        OutputMode::Porcelain => render_porcelain_response(response),
    }
}

fn render_human_response(response: &gwz_core::ResponseEnvelope) -> String {
    let mut lines = vec![format!("status: {:?}", response.meta.aggregate_status)];
    for member in &response.members {
        let mut line = format!(
            "{} {} {:?}",
            member.member_id, member.member_path, member.status
        );
        if let Some(error) = &member.error {
            line.push_str(&format!(" {:?}: {}", error.code, error.message));
        }
        lines.push(line);
    }
    for error in &response.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

fn render_porcelain_response(response: &gwz_core::ResponseEnvelope) -> String {
    response
        .members
        .iter()
        .filter(|member| member.status != gwz_core::MemberStatus::Ok)
        .map(|member| format!("!! {}", member.member_path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_jsonl_stream(
    response: &gwz_core::ResponseEnvelope,
    events: &[gwz_core::OperationEvent],
    result: Option<&gwz_core::OperationResult>,
) -> String {
    let mut lines = Vec::with_capacity(1 + events.len() + usize::from(result.is_some()));
    lines.push(response_json(response).to_string());
    lines.extend(events.iter().map(|event| event_json(event).to_string()));
    if let Some(result) = result {
        lines.push(result_json(result).to_string());
    }
    lines.join("\n")
}

fn exit_code_for_response(response: &gwz_core::ResponseEnvelope) -> i32 {
    match response.meta.aggregate_status {
        gwz_core::AggregateStatus::Accepted
        | gwz_core::AggregateStatus::Ok
        | gwz_core::AggregateStatus::Noop => 0,
        gwz_core::AggregateStatus::Rejected => 2,
        gwz_core::AggregateStatus::Partial | gwz_core::AggregateStatus::Failed => 1,
    }
}

fn response_json(response: &gwz_core::ResponseEnvelope) -> serde_json::Value {
    serde_json::json!({
        "kind": "response",
        "meta": response_meta_json(&response.meta),
        "members": response.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": response.errors.iter().map(error_json).collect::<Vec<_>>(),
    })
}

fn result_json(result: &gwz_core::OperationResult) -> serde_json::Value {
    serde_json::json!({
        "kind": "result",
        "operation_id": result.operation_id,
        "request_id": result.request_id,
        "action": format!("{:?}", result.action),
        "aggregate_status": format!("{:?}", result.aggregate_status),
        "started_at_ms": result.started_at_ms,
        "finished_at_ms": result.finished_at_ms,
        "members": result.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": result.errors.iter().map(error_json).collect::<Vec<_>>(),
    })
}

fn event_json(event: &gwz_core::OperationEvent) -> serde_json::Value {
    serde_json::json!({
        "kind": "event",
        "operation_id": event.operation_id,
        "request_id": event.request_id,
        "sequence": event.sequence,
        "timestamp_ms": event.timestamp_ms,
        "event_kind": format!("{:?}", event.kind),
        "severity": format!("{:?}", event.severity),
        "member_id": event.member_id,
        "member_path": event.member_path,
        "message": event.message,
        "member": event.member.as_ref().map(member_json),
        "error": event.error.as_ref().map(error_json),
    })
}

fn response_meta_json(meta: &gwz_core::ResponseMeta) -> serde_json::Value {
    serde_json::json!({
        "request_id": meta.request_id,
        "schema_version": meta.schema_version,
        "action": format!("{:?}", meta.action),
        "aggregate_status": format!("{:?}", meta.aggregate_status),
        "operation_id": meta.operation_id,
        "message": meta.message,
    })
}

fn member_json(member: &gwz_core::MemberResponse) -> serde_json::Value {
    serde_json::json!({
        "member_id": member.member_id,
        "member_path": member.member_path,
        "source_kind": format!("{:?}", member.source_kind),
        "status": format!("{:?}", member.status),
        "error": member.error.as_ref().map(error_json),
        "planned": member.planned.as_ref().map(planned_json),
        "lock_match": member.lock_match.map(|lock_match| format!("{:?}", lock_match)),
    })
}

fn planned_json(planned: &gwz_core::PlannedChange) -> serde_json::Value {
    serde_json::json!({
        "action": format!("{:?}", planned.action),
        "from_ref": planned.from_ref,
        "to_ref": planned.to_ref,
        "message": planned.message,
    })
}

fn error_json(error: &gwz_core::GwzError) -> serde_json::Value {
    serde_json::json!({
        "code": format!("{:?}", error.code),
        "message": error.message,
        "member_id": error.member_id,
        "member_path": error.member_path,
        "detail": error.detail,
    })
}

#[derive(Clone, Debug, Default)]
struct ParsedArgs {
    root: Option<String>,
    members: Vec<String>,
    paths: Vec<String>,
    all: bool,
    dry_run: bool,
    partial: bool,
    force: bool,
    sync: Option<gwz_core::SyncBehavior>,
    remote: Option<String>,
    jobs: Option<i64>,
    json: bool,
    jsonl: bool,
    porcelain: bool,
    combined_status: bool,
    no_combined: bool,
    no_files: bool,
    no_branches: bool,
    target: Option<ParsedTarget>,
    positionals: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum ParsedTarget {
    Lock,
    Head,
    Snapshot(String),
    Tag(String),
}

impl ParsedArgs {
    fn parse(args: Vec<String>) -> Result<Self, CliError> {
        let mut parsed = Self::default();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            match arg.as_str() {
                "--root" => {
                    parsed.root = Some(take_value(&args, &mut index, "--root")?);
                }
                "--member" => parsed
                    .members
                    .push(take_value(&args, &mut index, "--member")?),
                "--path" => parsed.paths.push(take_value(&args, &mut index, "--path")?),
                "--all" => parsed.all = true,
                "--dry-run" => parsed.dry_run = true,
                "--partial" => parsed.partial = true,
                "--force" => parsed.force = true,
                "--sync" => {
                    parsed.sync = Some(parse_sync(&take_value(&args, &mut index, "--sync")?)?);
                }
                "--remote" => {
                    parsed.remote = Some(take_value(&args, &mut index, "--remote")?);
                }
                "--jobs" => {
                    let value = take_value(&args, &mut index, "--jobs")?;
                    parsed.jobs = Some(
                        value
                            .parse::<i64>()
                            .map_err(|_| CliError::new("--jobs requires an integer"))?,
                    );
                }
                "--json" => parsed.json = true,
                "--jsonl" => parsed.jsonl = true,
                "--porcelain" => parsed.porcelain = true,
                "--combined" => parsed.combined_status = true,
                "--no-combined" => parsed.no_combined = true,
                "--no-files" => parsed.no_files = true,
                "--no-branches" => parsed.no_branches = true,
                "--lock" => parsed.set_target(ParsedTarget::Lock)?,
                "--head" => parsed.set_target(ParsedTarget::Head)?,
                "--snapshot" => {
                    let name = take_value(&args, &mut index, "--snapshot")?;
                    parsed.set_target(ParsedTarget::Snapshot(name))?;
                }
                "--tag" => {
                    let name = take_value(&args, &mut index, "--tag")?;
                    parsed.set_target(ParsedTarget::Tag(name))?;
                }
                value if value.starts_with("--") => {
                    return Err(CliError::new(format!("unknown flag {value}")));
                }
                value => parsed.positionals.push(value.to_owned()),
            }
            index += 1;
        }
        parsed.validate()?;
        Ok(parsed)
    }

    fn set_target(&mut self, target: ParsedTarget) -> Result<(), CliError> {
        if self.target.is_some() {
            return Err(CliError::new("only one target flag may be supplied"));
        }
        self.target = Some(target);
        Ok(())
    }

    fn validate(&self) -> Result<(), CliError> {
        if self.json && self.jsonl {
            return Err(CliError::new("--json and --jsonl are mutually exclusive"));
        }
        if self.porcelain && (self.json || self.jsonl) {
            return Err(CliError::new(
                "--porcelain cannot be combined with --json or --jsonl",
            ));
        }
        if self.all && (!self.members.is_empty() || !self.paths.is_empty()) {
            return Err(CliError::new(
                "--all cannot be combined with --member or --path",
            ));
        }
        if self.no_files && self.no_branches {
            return Err(CliError::new(
                "--no-files and --no-branches cannot both be supplied",
            ));
        }
        if self.combined_status && self.no_combined {
            return Err(CliError::new(
                "--combined and --no-combined cannot both be supplied",
            ));
        }
        if self.porcelain && self.no_combined {
            return Err(CliError::new(
                "--porcelain cannot be combined with --no-combined",
            ));
        }
        if self.no_combined && (self.no_files || self.no_branches) {
            return Err(CliError::new(
                "--no-files and --no-branches can only be used with combined status",
            ));
        }
        if self.jobs.is_some_and(|jobs| jobs < 1) {
            return Err(CliError::new("--jobs must be greater than zero"));
        }
        Ok(())
    }

    fn output_mode(&self) -> Result<OutputMode, CliError> {
        if self.porcelain {
            Ok(OutputMode::Porcelain)
        } else if self.json {
            Ok(OutputMode::Json)
        } else if self.jsonl {
            Ok(OutputMode::Jsonl)
        } else {
            Ok(OutputMode::Human)
        }
    }

    fn request_meta(&self, request_id: &str) -> gwz_core::RequestMeta {
        gwz_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            workspace: self.root.as_ref().map(|root| gwz_core::WorkspaceRef {
                root: Some(root.clone()),
                workspace_id: None,
            }),
            selection: self.selection(),
            policy: self.policy(),
            dry_run: self.dry_run.then_some(true),
            ..Default::default()
        }
    }

    fn selection(&self) -> Option<gwz_core::Selection> {
        if self.all || !self.members.is_empty() || !self.paths.is_empty() {
            Some(gwz_core::Selection {
                all: self.all.then_some(true),
                member_ids: self.members.clone(),
                paths: self.paths.clone(),
            })
        } else {
            None
        }
    }

    fn policy(&self) -> Option<gwz_core::OperationPolicy> {
        if self.partial
            || self.force
            || self.sync.is_some()
            || self.remote.is_some()
            || self.jobs.is_some()
        {
            Some(gwz_core::OperationPolicy {
                partial: self.partial.then_some(gwz_core::PartialBehavior::Partial),
                destructive: self.force.then_some(gwz_core::DestructiveBehavior::Allow),
                sync: self.sync,
                remote: self.remote.clone(),
                concurrency: self.jobs,
                ..Default::default()
            })
        } else {
            None
        }
    }

    fn command_request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        let Some(command) = self.positionals.first().map(String::as_str) else {
            return Err(CliError::new("missing command"));
        };
        if command != "status" && self.has_status_specific_flags() {
            return Err(CliError::new(
                "status-specific flags can only be used with status",
            ));
        }
        match command {
            "init" => self.init_request(meta, workspace_root),
            "add" => self.add_request(meta),
            "repo" => self.repo_request(meta),
            "materialize" => self.materialize_request(meta),
            "pull" => self.pull_request(meta),
            "snapshot" => self.snapshot_request(meta),
            "tag" => self.tag_request(meta),
            "push" => self.no_arg_request("push").map(|_| {
                CliRequest::Push(gwz_core::PushRequest {
                    remote: self.remote.clone(),
                    refspec: None,
                    meta,
                })
            }),
            "status" => self.status_request(meta),
            _ => Err(CliError::new(format!("unknown command {command}"))),
        }
    }

    fn has_status_specific_flags(&self) -> bool {
        self.combined_status
            || self.no_combined
            || self.porcelain
            || self.no_files
            || self.no_branches
    }

    fn status_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.no_arg_request("status")?;
        let combined = !self.no_combined;
        Ok(CliRequest::Status(gwz_core::StatusRequest {
            meta,
            mode: Some(if combined {
                gwz_core::StatusMode::Combined
            } else {
                gwz_core::StatusMode::Summary
            }),
            include_file_changes: if combined { Some(!self.no_files) } else { None },
            include_branch_summary: if combined {
                Some(!self.no_branches)
            } else {
                None
            },
            path_style: combined.then_some(gwz_core::StatusPathStyle::WorkspaceRelative),
        }))
    }

    fn init_request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        let urls = self.positionals.iter().skip(1).cloned().collect::<Vec<_>>();
        if urls.is_empty() {
            Ok(CliRequest::CreateWorkspace(
                gwz_core::CreateWorkspaceRequest {
                    meta,
                    workspace_root,
                    workspace_id: None,
                },
            ))
        } else {
            Ok(CliRequest::InitFromSources(
                gwz_core::InitFromSourcesRequest {
                    meta,
                    workspace_root,
                    sources: urls
                        .into_iter()
                        .map(|url| gwz_core::SourceUrl {
                            url,
                            path: None,
                            remote_name: None,
                            branch: None,
                        })
                        .collect(),
                    target: Some(gwz_core::MaterializeTarget {
                        kind: gwz_core::MaterializeTargetKind::Head,
                        name: None,
                        commit: None,
                    }),
                    workspace_id: None,
                },
            ))
        }
    }

    fn add_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("add", 1)?;
        Ok(CliRequest::AddExistingRepo(
            gwz_core::AddExistingRepoRequest {
                meta,
                repository_path: self.positionals[1].clone(),
                member_path: None,
                member_id: None,
                source_id: None,
            },
        ))
    }

    fn repo_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        if self.positionals.get(1).map(String::as_str) != Some("create") {
            return Err(CliError::new("expected 'repo create <member-path>'"));
        }
        if self.positionals.len() != 3 {
            return Err(CliError::new("repo create requires a member path"));
        }
        Ok(CliRequest::CreateRepo(gwz_core::CreateRepoRequest {
            meta,
            member_path: self.positionals[2].clone(),
            initial_branch: None,
            member_id: None,
            source_id: None,
        }))
    }

    fn materialize_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.no_arg_request("materialize")?;
        Ok(CliRequest::Materialize(gwz_core::MaterializeRequest {
            meta,
            target: self.materialize_target()?,
        }))
    }

    fn pull_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.no_arg_request("pull")?;
        match self.target.as_ref().unwrap_or(&ParsedTarget::Head) {
            ParsedTarget::Head => Ok(CliRequest::PullHead(gwz_core::PullHeadRequest { meta })),
            ParsedTarget::Snapshot(name) => {
                Ok(CliRequest::PullSnapshot(gwz_core::PullSnapshotRequest {
                    meta,
                    snapshot_id: name.clone(),
                }))
            }
            ParsedTarget::Lock | ParsedTarget::Tag(_) => {
                Err(CliError::new("pull supports --head or --snapshot <name>"))
            }
        }
    }

    fn snapshot_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("snapshot", 1)?;
        Ok(CliRequest::Snapshot(gwz_core::SnapshotRequest {
            meta,
            snapshot_id: self.positionals[1].clone(),
        }))
    }

    fn tag_request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("tag", 1)?;
        Ok(CliRequest::Tag(gwz_core::TagRequest {
            meta,
            tag_name: self.positionals[1].clone(),
        }))
    }

    fn materialize_target(&self) -> Result<gwz_core::MaterializeTarget, CliError> {
        let target = self.target.as_ref().unwrap_or(&ParsedTarget::Lock);
        match target {
            ParsedTarget::Lock => Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Lock,
                name: None,
                commit: None,
            }),
            ParsedTarget::Head => Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Head,
                name: None,
                commit: None,
            }),
            ParsedTarget::Snapshot(name) => Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Snapshot,
                name: Some(name.clone()),
                commit: None,
            }),
            ParsedTarget::Tag(name) => Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Tag,
                name: Some(name.clone()),
                commit: None,
            }),
        }
    }

    fn no_arg_request(&self, command: &str) -> Result<(), CliError> {
        self.require_arg_count(command, 0)
    }

    fn require_arg_count(&self, command: &str, count: usize) -> Result<(), CliError> {
        if self.positionals.len() == count + 1 {
            Ok(())
        } else {
            Err(CliError::new(format!(
                "{command} expects {count} argument(s)"
            )))
        }
    }
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, CliError> {
    *index += 1;
    let value = args
        .get(*index)
        .ok_or_else(|| CliError::new(format!("{flag} requires a value")))?;
    if value.starts_with("--") {
        return Err(CliError::new(format!("{flag} requires a value")));
    }
    Ok(value.clone())
}

fn parse_sync(value: &str) -> Result<gwz_core::SyncBehavior, CliError> {
    match value {
        "fetch-only" => Ok(gwz_core::SyncBehavior::FetchOnly),
        "ff-only" => Ok(gwz_core::SyncBehavior::FfOnly),
        "merge" => Ok(gwz_core::SyncBehavior::Merge),
        "rebase" => Ok(gwz_core::SyncBehavior::Rebase),
        "reset" => Ok(gwz_core::SyncBehavior::Reset),
        "driver-selected" => Ok(gwz_core::SyncBehavior::DriverSelected),
        _ => Err(CliError::new("unknown --sync value")),
    }
}

fn new_request_id() -> String {
    format!("req_{}", unique_suffix())
}

fn new_operation_id() -> String {
    format!("op_{}", unique_suffix())
}

fn unique_suffix() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}_{}", std::process::id(), millis)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn parses_init_workspace_with_root() {
        let invocation = parse_args_with_request_id(
            strings(["--root", "/tmp/gwz-test", "init"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Human);
        let CliRequest::CreateWorkspace(request) = invocation.request else {
            panic!("expected create workspace");
        };
        assert_eq!(request.workspace_root, "/tmp/gwz-test");
        assert_eq!(request.meta.request_id, "req_test");
    }

    #[test]
    fn parses_init_sources_from_plain_urls() {
        let invocation = parse_args_with_request_id(
            strings([
                "init",
                "git@github.com:org/repo-a.git",
                "https://github.com/org/repo-b",
            ]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        let CliRequest::InitFromSources(request) = invocation.request else {
            panic!("expected init from sources");
        };
        assert_eq!(request.workspace_root, "/cwd");
        assert_eq!(request.sources[0].url, "git@github.com:org/repo-a.git");
        assert_eq!(request.sources[0].path, None);
        assert_eq!(request.sources[1].url, "https://github.com/org/repo-b");
    }

    #[test]
    fn parses_global_selection_policy_and_output_flags() {
        let invocation = parse_args_with_request_id(
            strings([
                "--root",
                "/ws",
                "--member",
                "mem_app",
                "--path",
                "repos/lib",
                "--dry-run",
                "--partial",
                "--force",
                "--sync",
                "reset",
                "--remote",
                "origin",
                "--jobs",
                "4",
                "--json",
                "status",
            ]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Json);
        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        let workspace = request.meta.workspace.unwrap();
        assert_eq!(workspace.root, Some("/ws".to_owned()));
        let selection = request.meta.selection.unwrap();
        assert_eq!(selection.member_ids, vec!["mem_app"]);
        assert_eq!(selection.paths, vec!["repos/lib"]);
        let policy = request.meta.policy.unwrap();
        assert_eq!(policy.partial, Some(gwz_core::PartialBehavior::Partial));
        assert_eq!(
            policy.destructive,
            Some(gwz_core::DestructiveBehavior::Allow)
        );
        assert_eq!(policy.sync, Some(gwz_core::SyncBehavior::Reset));
        assert_eq!(policy.remote, Some("origin".to_owned()));
        assert_eq!(policy.concurrency, Some(4));
        assert_eq!(request.meta.dry_run, Some(true));
    }

    #[test]
    fn parses_combined_status_flags() {
        let invocation = parse_args_with_request_id(
            strings(["status", "--porcelain", "--no-branches"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Porcelain);
        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
        assert_eq!(request.include_file_changes, Some(true));
        assert_eq!(request.include_branch_summary, Some(false));
        assert_eq!(
            request.path_style,
            Some(gwz_core::StatusPathStyle::WorkspaceRelative)
        );
    }

    #[test]
    fn parses_status_as_combined_by_default() {
        let invocation =
            parse_args_with_request_id(strings(["status"]), "req_test", Path::new("/cwd")).unwrap();

        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
        assert_eq!(request.include_file_changes, Some(true));
        assert_eq!(request.include_branch_summary, Some(true));
        assert_eq!(
            request.path_style,
            Some(gwz_core::StatusPathStyle::WorkspaceRelative)
        );
    }

    #[test]
    fn parses_no_combined_status_as_summary_mode() {
        let invocation = parse_args_with_request_id(
            strings(["status", "--no-combined"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Summary));
        assert_eq!(request.include_file_changes, None);
        assert_eq!(request.include_branch_summary, None);
        assert_eq!(request.path_style, None);
    }

    #[test]
    fn parses_command_matrix() {
        assert!(matches!(
            parse(strings(["add", "repos/app"])).request,
            CliRequest::AddExistingRepo(_)
        ));
        assert!(matches!(
            parse(strings(["repo", "create", "repos/app"])).request,
            CliRequest::CreateRepo(_)
        ));
        assert!(matches!(
            parse(strings(["materialize", "--lock"])).request,
            CliRequest::Materialize(_)
        ));
        assert!(matches!(
            parse(strings(["materialize", "--snapshot", "snap_one"])).request,
            CliRequest::Materialize(_)
        ));
        assert!(matches!(
            parse(strings(["pull", "--head"])).request,
            CliRequest::PullHead(_)
        ));
        assert!(matches!(
            parse(strings(["pull", "--snapshot", "snap_one"])).request,
            CliRequest::PullSnapshot(_)
        ));
        assert!(matches!(
            parse(strings(["snapshot", "snap_one"])).request,
            CliRequest::Snapshot(_)
        ));
        assert!(matches!(
            parse(strings(["tag", "release_one"])).request,
            CliRequest::Tag(_)
        ));
        assert!(matches!(
            parse(strings(["push"])).request,
            CliRequest::Push(_)
        ));
    }

    #[test]
    fn rejects_invalid_command_combinations_before_core_execution() {
        assert!(parse_result(strings(["--json", "--jsonl", "status"])).is_err());
        assert!(parse_result(strings(["--all", "--member", "mem_app", "status"])).is_err());
        assert!(parse_result(strings(["status", "--no-files", "--no-branches"])).is_err());
        assert!(parse_result(strings(["status", "--combined", "--no-combined"])).is_err());
        assert!(parse_result(strings(["status", "--porcelain", "--no-combined"])).is_err());
        assert!(parse_result(strings(["status", "--no-combined", "--no-files"])).is_err());
        assert!(parse_result(strings(["push", "--combined"])).is_err());
        assert!(parse_result(strings(["push", "--no-combined"])).is_err());
        assert!(parse_result(strings(["materialize", "--snapshot"])).is_err());
        assert!(parse_result(strings(["pull", "--lock"])).is_err());
        assert!(parse_result(strings(["unknown"])).is_err());
    }

    #[test]
    fn can_call_core_status_in_process() {
        let temp = TempDir::new("cli-status");
        gwz_core::workspace_ops::handle_create_workspace(
            gwz_core::CreateWorkspaceRequest {
                meta: request_meta("req_setup"),
                workspace_root: temp.path().to_string_lossy().into_owned(),
                workspace_id: Some("ws_cli".to_owned()),
            },
            "op_setup",
        )
        .unwrap();
        let invocation = parse_args_with_request_id(
            strings([
                "--root",
                temp.path().to_str().unwrap(),
                "status",
                "--no-combined",
            ]),
            "req_status",
            temp.path(),
        )
        .unwrap();

        let response = execute_invocation(&invocation).unwrap();

        assert_eq!(
            response.meta.aggregate_status,
            gwz_core::AggregateStatus::Ok
        );
        assert!(response.members.is_empty());
    }

    #[test]
    fn json_renderer_outputs_structured_response() {
        let response = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);

        let json: serde_json::Value =
            serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();

        assert_eq!(json["kind"], "response");
        assert_eq!(json["meta"]["aggregate_status"], "Ok");
        assert_eq!(json["members"][0]["member_id"], "mem_app");
        assert_eq!(json["members"][0]["status"], "Ok");
    }

    #[test]
    fn jsonl_renderer_emits_response_event_and_result_in_order() {
        let response = sample_response(
            gwz_core::AggregateStatus::Accepted,
            gwz_core::MemberStatus::Planned,
        );
        let event = sample_event();
        let result = sample_result();

        let lines = render_jsonl_stream(&response, &[event], Some(&result))
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["kind"], "response");
        assert_eq!(lines[1]["kind"], "event");
        assert_eq!(lines[2]["kind"], "result");
    }

    #[test]
    fn human_renderer_smoke_covers_success_rejection_and_member_failure() {
        let success = render_response(
            &sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok),
            OutputMode::Human,
        );
        assert!(success.contains("status: Ok"));

        let rejected = render_response(
            &sample_response(
                gwz_core::AggregateStatus::Rejected,
                gwz_core::MemberStatus::Rejected,
            ),
            OutputMode::Human,
        );
        assert!(rejected.contains("status: Rejected"));

        let failed = render_response(
            &sample_response(
                gwz_core::AggregateStatus::Failed,
                gwz_core::MemberStatus::Failed,
            ),
            OutputMode::Human,
        );
        assert!(failed.contains("RemoteRejected"));
    }

    #[test]
    fn exit_code_mapping_distinguishes_success_rejected_and_failed() {
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Ok,
                gwz_core::MemberStatus::Ok,
            )),
            0
        );
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Rejected,
                gwz_core::MemberStatus::Rejected,
            )),
            2
        );
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Failed,
                gwz_core::MemberStatus::Failed,
            )),
            1
        );
    }

    fn parse(args: Vec<String>) -> CliInvocation {
        parse_result(args).unwrap()
    }

    fn parse_result(args: Vec<String>) -> Result<CliInvocation, CliError> {
        parse_args_with_request_id(args, "req_test", Path::new("/cwd"))
    }

    fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn request_meta(request_id: &str) -> gwz_core::RequestMeta {
        gwz_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            ..Default::default()
        }
    }

    fn sample_response(
        aggregate_status: gwz_core::AggregateStatus,
        member_status: gwz_core::MemberStatus,
    ) -> gwz_core::ResponseEnvelope {
        let error = (member_status == gwz_core::MemberStatus::Failed
            || member_status == gwz_core::MemberStatus::Rejected)
            .then(|| gwz_core::GwzError {
                code: gwz_core::GwzErrorCode::RemoteRejected,
                message: "remote rejected".to_owned(),
                member_id: Some("mem_app".to_owned()),
                member_path: Some("repos/app".to_owned()),
                detail: None,
            });
        gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                request_id: "req_render".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Status,
                aggregate_status,
                operation_id: Some("op_render".to_owned()),
                message: None,
                attribution: None,
            },
            members: vec![gwz_core::MemberResponse {
                member_id: "mem_app".to_owned(),
                member_path: "repos/app".to_owned(),
                source_kind: gwz_core::SourceKind::Git,
                status: member_status,
                error,
                planned: None,
                state: None,
                git_status: None,
                lock_match: None,
            }],
            errors: Vec::new(),
        }
    }

    fn sample_event() -> gwz_core::OperationEvent {
        gwz_core::OperationEvent {
            operation_id: "op_render".to_owned(),
            request_id: "req_render".to_owned(),
            sequence: 0,
            timestamp_ms: 1,
            kind: gwz_core::EventKind::OperationStarted,
            severity: gwz_core::Severity::Info,
            member_id: None,
            member_path: None,
            message: Some("started".to_owned()),
            member: None,
            error: None,
            attribution: None,
        }
    }

    fn sample_result() -> gwz_core::OperationResult {
        gwz_core::OperationResult {
            operation_id: "op_render".to_owned(),
            request_id: "req_render".to_owned(),
            action: gwz_core::ActionKind::Status,
            aggregate_status: gwz_core::AggregateStatus::Ok,
            started_at_ms: 1,
            finished_at_ms: 2,
            members: Vec::new(),
            errors: Vec::new(),
            attribution: None,
        }
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("gwz-cli-{prefix}-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
