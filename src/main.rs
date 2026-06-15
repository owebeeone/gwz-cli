fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.as_slice() == ["--version"] {
        println!("gws {}", gws_core::version());
        return;
    }

    match parse_args(args) {
        Ok(invocation) => match execute_invocation(&invocation) {
            Ok(response) => println!("{:?}", response.meta.aggregate_status),
            Err(error) => {
                eprintln!("gws: {}", error.message);
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("gws: {}", error.message);
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
    CreateWorkspace(gws_core::CreateWorkspaceRequest),
    InitFromSources(gws_core::InitFromSourcesRequest),
    AddExistingRepo(gws_core::AddExistingRepoRequest),
    CreateRepo(gws_core::CreateRepoRequest),
    Materialize(gws_core::MaterializeRequest),
    Status(gws_core::StatusRequest),
    Snapshot(gws_core::SnapshotRequest),
    Tag(gws_core::TagRequest),
    PullHead(gws_core::PullHeadRequest),
    PullSnapshot(gws_core::PullSnapshotRequest),
    Push(gws_core::PushRequest),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Human,
    Json,
    Jsonl,
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

fn execute_invocation(invocation: &CliInvocation) -> Result<gws_core::ResponseEnvelope, CliError> {
    let backend = gws_core::git::Git2Backend::new();
    let operation_id = new_operation_id();
    let start = invocation.start_dir.as_path();
    let response = match &invocation.request {
        CliRequest::CreateWorkspace(request) => {
            gws_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::InitFromSources(request) => gws_core::workspace_ops::handle_init_from_sources(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::AddExistingRepo(request) => gws_core::workspace_ops::handle_add_existing_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::CreateRepo(request) => gws_core::workspace_ops::handle_create_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Materialize(request) => gws_core::workspace_ops::handle_materialize(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Status(request) => {
            gws_core::status::handle_status(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Snapshot(request) => {
            gws_core::workspace_ops::handle_snapshot(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Tag(request) => {
            gws_core::workspace_ops::handle_tag(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::PullHead(request) => gws_core::workspace_ops::handle_pull_head(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::PullSnapshot(request) => gws_core::workspace_ops::handle_pull_snapshot(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Push(request) => {
            gws_core::workspace_ops::handle_push(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
    };
    response.map_err(|error| CliError::new(error.to_string()))
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
    sync: Option<gws_core::SyncBehavior>,
    remote: Option<String>,
    jobs: Option<i64>,
    json: bool,
    jsonl: bool,
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
        if self.all && (!self.members.is_empty() || !self.paths.is_empty()) {
            return Err(CliError::new(
                "--all cannot be combined with --member or --path",
            ));
        }
        if self.jobs.is_some_and(|jobs| jobs < 1) {
            return Err(CliError::new("--jobs must be greater than zero"));
        }
        Ok(())
    }

    fn output_mode(&self) -> Result<OutputMode, CliError> {
        if self.json {
            Ok(OutputMode::Json)
        } else if self.jsonl {
            Ok(OutputMode::Jsonl)
        } else {
            Ok(OutputMode::Human)
        }
    }

    fn request_meta(&self, request_id: &str) -> gws_core::RequestMeta {
        gws_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gws.protocol/v0".to_owned(),
            workspace: self.root.as_ref().map(|root| gws_core::WorkspaceRef {
                root: Some(root.clone()),
                workspace_id: None,
            }),
            selection: self.selection(),
            policy: self.policy(),
            dry_run: self.dry_run.then_some(true),
            ..Default::default()
        }
    }

    fn selection(&self) -> Option<gws_core::Selection> {
        if self.all || !self.members.is_empty() || !self.paths.is_empty() {
            Some(gws_core::Selection {
                all: self.all.then_some(true),
                member_ids: self.members.clone(),
                paths: self.paths.clone(),
            })
        } else {
            None
        }
    }

    fn policy(&self) -> Option<gws_core::OperationPolicy> {
        if self.partial
            || self.force
            || self.sync.is_some()
            || self.remote.is_some()
            || self.jobs.is_some()
        {
            Some(gws_core::OperationPolicy {
                partial: self.partial.then_some(gws_core::PartialBehavior::Partial),
                destructive: self.force.then_some(gws_core::DestructiveBehavior::Allow),
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
        meta: gws_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        let Some(command) = self.positionals.first().map(String::as_str) else {
            return Err(CliError::new("missing command"));
        };
        match command {
            "init" => self.init_request(meta, workspace_root),
            "add" => self.add_request(meta),
            "repo" => self.repo_request(meta),
            "materialize" => self.materialize_request(meta),
            "pull" => self.pull_request(meta),
            "snapshot" => self.snapshot_request(meta),
            "tag" => self.tag_request(meta),
            "push" => self.no_arg_request("push").map(|_| {
                CliRequest::Push(gws_core::PushRequest {
                    remote: self.remote.clone(),
                    refspec: None,
                    meta,
                })
            }),
            "status" => self
                .no_arg_request("status")
                .map(|_| CliRequest::Status(gws_core::StatusRequest { meta })),
            _ => Err(CliError::new(format!("unknown command {command}"))),
        }
    }

    fn init_request(
        &self,
        meta: gws_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        let urls = self.positionals.iter().skip(1).cloned().collect::<Vec<_>>();
        if urls.is_empty() {
            Ok(CliRequest::CreateWorkspace(
                gws_core::CreateWorkspaceRequest {
                    meta,
                    workspace_root,
                    workspace_id: None,
                },
            ))
        } else {
            Ok(CliRequest::InitFromSources(
                gws_core::InitFromSourcesRequest {
                    meta,
                    workspace_root,
                    sources: urls
                        .into_iter()
                        .map(|url| gws_core::SourceUrl {
                            url,
                            path: None,
                            remote_name: None,
                            branch: None,
                        })
                        .collect(),
                    target: Some(gws_core::MaterializeTarget {
                        kind: gws_core::MaterializeTargetKind::Head,
                        name: None,
                        commit: None,
                    }),
                    workspace_id: None,
                },
            ))
        }
    }

    fn add_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("add", 1)?;
        Ok(CliRequest::AddExistingRepo(
            gws_core::AddExistingRepoRequest {
                meta,
                repository_path: self.positionals[1].clone(),
                member_path: None,
                member_id: None,
                source_id: None,
            },
        ))
    }

    fn repo_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        if self.positionals.get(1).map(String::as_str) != Some("create") {
            return Err(CliError::new("expected 'repo create <member-path>'"));
        }
        if self.positionals.len() != 3 {
            return Err(CliError::new("repo create requires a member path"));
        }
        Ok(CliRequest::CreateRepo(gws_core::CreateRepoRequest {
            meta,
            member_path: self.positionals[2].clone(),
            initial_branch: None,
            member_id: None,
            source_id: None,
        }))
    }

    fn materialize_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.no_arg_request("materialize")?;
        Ok(CliRequest::Materialize(gws_core::MaterializeRequest {
            meta,
            target: self.materialize_target()?,
        }))
    }

    fn pull_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.no_arg_request("pull")?;
        match self.target.as_ref().unwrap_or(&ParsedTarget::Head) {
            ParsedTarget::Head => Ok(CliRequest::PullHead(gws_core::PullHeadRequest { meta })),
            ParsedTarget::Snapshot(name) => {
                Ok(CliRequest::PullSnapshot(gws_core::PullSnapshotRequest {
                    meta,
                    snapshot_id: name.clone(),
                }))
            }
            ParsedTarget::Lock | ParsedTarget::Tag(_) => {
                Err(CliError::new("pull supports --head or --snapshot <name>"))
            }
        }
    }

    fn snapshot_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("snapshot", 1)?;
        Ok(CliRequest::Snapshot(gws_core::SnapshotRequest {
            meta,
            snapshot_id: self.positionals[1].clone(),
        }))
    }

    fn tag_request(&self, meta: gws_core::RequestMeta) -> Result<CliRequest, CliError> {
        self.require_arg_count("tag", 1)?;
        Ok(CliRequest::Tag(gws_core::TagRequest {
            meta,
            tag_name: self.positionals[1].clone(),
        }))
    }

    fn materialize_target(&self) -> Result<gws_core::MaterializeTarget, CliError> {
        let target = self.target.as_ref().unwrap_or(&ParsedTarget::Lock);
        match target {
            ParsedTarget::Lock => Ok(gws_core::MaterializeTarget {
                kind: gws_core::MaterializeTargetKind::Lock,
                name: None,
                commit: None,
            }),
            ParsedTarget::Head => Ok(gws_core::MaterializeTarget {
                kind: gws_core::MaterializeTargetKind::Head,
                name: None,
                commit: None,
            }),
            ParsedTarget::Snapshot(name) => Ok(gws_core::MaterializeTarget {
                kind: gws_core::MaterializeTargetKind::Snapshot,
                name: Some(name.clone()),
                commit: None,
            }),
            ParsedTarget::Tag(name) => Ok(gws_core::MaterializeTarget {
                kind: gws_core::MaterializeTargetKind::Tag,
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

fn parse_sync(value: &str) -> Result<gws_core::SyncBehavior, CliError> {
    match value {
        "fetch-only" => Ok(gws_core::SyncBehavior::FetchOnly),
        "ff-only" => Ok(gws_core::SyncBehavior::FfOnly),
        "merge" => Ok(gws_core::SyncBehavior::Merge),
        "rebase" => Ok(gws_core::SyncBehavior::Rebase),
        "reset" => Ok(gws_core::SyncBehavior::Reset),
        "driver-selected" => Ok(gws_core::SyncBehavior::DriverSelected),
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
            strings(["--root", "/tmp/gws-test", "init"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Human);
        let CliRequest::CreateWorkspace(request) = invocation.request else {
            panic!("expected create workspace");
        };
        assert_eq!(request.workspace_root, "/tmp/gws-test");
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
        assert_eq!(policy.partial, Some(gws_core::PartialBehavior::Partial));
        assert_eq!(
            policy.destructive,
            Some(gws_core::DestructiveBehavior::Allow)
        );
        assert_eq!(policy.sync, Some(gws_core::SyncBehavior::Reset));
        assert_eq!(policy.remote, Some("origin".to_owned()));
        assert_eq!(policy.concurrency, Some(4));
        assert_eq!(request.meta.dry_run, Some(true));
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
        assert!(parse_result(strings(["materialize", "--snapshot"])).is_err());
        assert!(parse_result(strings(["pull", "--lock"])).is_err());
        assert!(parse_result(strings(["unknown"])).is_err());
    }

    #[test]
    fn can_call_core_status_in_process() {
        let temp = TempDir::new("cli-status");
        gws_core::workspace_ops::handle_create_workspace(
            gws_core::CreateWorkspaceRequest {
                meta: request_meta("req_setup"),
                workspace_root: temp.path().to_string_lossy().into_owned(),
                workspace_id: Some("ws_cli".to_owned()),
            },
            "op_setup",
        )
        .unwrap();
        let invocation = parse_args_with_request_id(
            strings(["--root", temp.path().to_str().unwrap(), "status"]),
            "req_status",
            temp.path(),
        )
        .unwrap();

        let response = execute_invocation(&invocation).unwrap();

        assert_eq!(
            response.meta.aggregate_status,
            gws_core::AggregateStatus::Ok
        );
        assert!(response.members.is_empty());
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

    fn request_meta(request_id: &str) -> gws_core::RequestMeta {
        gws_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gws.protocol/v0".to_owned(),
            ..Default::default()
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
                .join(format!("gws-cli-{prefix}-{}-{unique}", std::process::id()));
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
