//! Candidate CLI transport selection; core owns resolution and target scans.
use clap::Args;

cfg_if::cfg_if! {
    if #[cfg(any(all(unix, gwz_transport_candidate), all(windows, gwz_transport_candidate, gwz_windows_https_qualification)))] {
        use crate::{CliError, CliInvocation, CliRequest};
        use gwz_core::transport_scope::Operation;
        use gwz_core::transport_setting::{self, Driver, IgnoredValue, Setting, Source, Transport};

        #[derive(Clone, Debug, Default, Args)]
        pub(crate) struct TransportArgs {
            #[arg(long, global = true, value_enum, value_name = "gwz|native",
                help = "Transport for network operations: gwz (the default) or native, libgit2's own, as gwz 1.0 used",
                long_help = "Which transport carries network operations: gwz's own SSH and HTTPS transport (gwz, the default), or libgit2's native transport, as gwz 1.0 used (native). native is an escape hatch you choose before a command runs, not a fallback: gwz never moves an operation from one transport to the other. With native, --jobs defaults to 50, --max-per-host to 8 and --ssh-timeout to 3, as in gwz 1.0, unless you give them; --ssh-timeout is libgit2's connect and read timeout; and there is no connection pooling, no setup retry and no 30 second setup budget, so --max-retries has no effect. git://, http:// and file:// remotes always use the native transport. This flag overrides the environment variable GWZ_TRANSPORT (gwz or native; unsetting it removes it), which overrides gwz.transport in your global git configuration (git config --file \"$HOME/.gitconfig\" gwz.transport native sets it; git config --file \"$HOME/.gitconfig\" --unset-all gwz.transport removes it). gwz reads ~/.gitconfig and $XDG_CONFIG_HOME/git/config (by default ~/.config/git/config), not a file named by GIT_CONFIG_GLOBAL (the paired --file commands above bypass that variable), and not system git configuration, and it applies no conditional include (includeIf). gwz.transport in the workspace root's or a member's own git configuration (.git/config or config.worktree) is ignored, with a note. While native is selected, each command that uses the network says so once.")]
            pub(crate) transport: Option<TransportArg>,
        }

        #[derive(Clone, Copy, Debug, clap::ValueEnum)]
        pub(crate) enum TransportArg {
            Gwz,
            Native,
        }

        impl TransportArgs {
            pub(crate) fn selected(&self) -> Option<Transport> {
                self.transport.map(|value| match value {
                    TransportArg::Gwz => Transport::Gwz,
                    TransportArg::Native => Transport::Native,
                })
            }
        }

        #[derive(Clone, Debug)]
        pub(crate) struct TransportReport {
            pub(crate) setting: Setting,
            pub(crate) ignored: Vec<IgnoredValue>,
        }

        impl TransportReport {
            pub(crate) fn is_native(&self) -> bool {
                self.setting.transport == Transport::Native
            }

            pub(crate) fn timeout_seconds(&self, explicit: Option<i64>) -> i64 {
                explicit.unwrap_or(if self.is_native() { 3 } else { 9 })
            }

            pub(crate) fn notes(&self) -> Vec<String> {
                let mut notes = Vec::new();
                if self.is_native() {
                    let origin = match &self.setting.source {
                        Source::Flag => "from --transport native".to_owned(),
                        Source::Environment => "from GWZ_TRANSPORT=native".to_owned(),
                        Source::GlobalConfiguration(location) => {
                            format!("from gwz.transport in {}", location.where_text())
                        }
                        Source::Default => "default".to_owned(),
                    };
                    let remedy = match &self.setting.source {
                        Source::Flag => "omit --transport native to use gwz's transport".to_owned(),
                        Source::Environment => "unset GWZ_TRANSPORT, or pass --transport gwz for one command, to use gwz's transport".to_owned(),
                        Source::GlobalConfiguration(location) => format!("{}, or pass --transport gwz for one command, to use gwz's transport", location.remove_text()),
                        Source::Default => "pass --transport gwz to use gwz's transport".to_owned(),
                    };
                    notes.push(format!("gwz: note: using libgit2's native transport ({origin}), as gwz 1.0 did; {remedy}"));
                }
                for ignored in &self.ignored {
                    notes.push(format!(
                        "gwz: note: ignoring {} in {} ({}): only --transport, GWZ_TRANSPORT and your global git configuration select the transport; {}",
                        ignored.entry_text(), ignored.location.where_text(), ignored.scope.text(), ignored.location.remove_text()
                    ));
                }
                notes
            }

            pub(crate) fn verbose_line(&self) -> String {
                let origin = match &self.setting.source {
                    Source::Flag => "from --transport".to_owned(),
                    Source::Environment => "from GWZ_TRANSPORT".to_owned(),
                    Source::GlobalConfiguration(location) => format!("from gwz.transport in {}", location.where_text()),
                    Source::Default => "default".to_owned(),
                };
                let mut line = format!("transport: {} ({origin}", self.setting.transport.name());
                if matches!(self.setting.source, Source::Default | Source::GlobalConfiguration(_)) {
                    line.push_str("; read ");
                    if self.setting.read.is_empty() {
                        line.push_str("none");
                    } else {
                        line.push_str(&self.setting.read.iter().map(|path| transport_setting::path_text(path)).collect::<Vec<_>>().join(", "));
                    }
                }
                for path in &self.setting.skipped {
                    line.push_str("; skipped unreadable ");
                    line.push_str(&transport_setting::path_text(path));
                }
                line.push(')');
                line
            }

            pub(crate) fn json(&self) -> Option<serde_json::Value> {
                if self.setting.transport == Transport::Gwz
                    && matches!(self.setting.source, Source::Default)
                    && self.ignored.is_empty()
                    && self.setting.skipped.is_empty()
                {
                    return None;
                }
                let (file, included) = match &self.setting.source {
                    Source::GlobalConfiguration(location) => (Some(location.file.to_string_lossy().into_owned()), location.included),
                    _ => (None, false),
                };
                Some(serde_json::json!({
                    "transport": self.setting.transport.name(),
                    "source": self.setting.source.name(),
                    "file": file,
                    "included": included,
                    "ignored": self.ignored.iter().map(|value| serde_json::json!({
                        "scope": value.scope.name(),
                        "member_id": value.scope.member_id(),
                        "file": value.location.file.to_string_lossy(),
                        "included": value.location.included,
                        "value": value.value,
                    })).collect::<Vec<_>>(),
                    "skipped": self.setting.skipped.iter().map(|file| serde_json::json!({
                        "file": file.to_string_lossy(),
                    })).collect::<Vec<_>>(),
                }))
            }

            pub(crate) fn render(&self, rendered: String, output: crate::OutputMode, verbose: bool) -> String {
                match output {
                    crate::OutputMode::Human if verbose => {
                        if rendered.is_empty() { self.verbose_line() } else { format!("{}\n{rendered}", self.verbose_line()) }
                    }
                    crate::OutputMode::Json | crate::OutputMode::Jsonl => {
                        let Some(setting) = self.json() else { return rendered };
                        let (first, rest) = rendered.split_once('\n').unwrap_or((&rendered, ""));
                        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(first) else { return rendered };
                        // Remote tag listings historically carry kind/entries without meta.
                        // Only a required setting enriches that shape; null-meta errors stay null.
                        if value.get("meta").is_none() && value["kind"] == "tags" {
                            value["meta"] = serde_json::json!({});
                        }
                        if !value.get("meta").is_some_and(serde_json::Value::is_object) {
                            return rendered;
                        }
                        value["meta"]["transport_setting"] = setting;
                        if rest.is_empty() { value.to_string() } else { format!("{}\n{rest}", value) }
                    }
                    _ => rendered,
                }
            }
        }

        pub(crate) fn render_execution_error(
            error: &CliError, output: crate::OutputMode, verbose: bool,
            report: Option<&TransportReport>,
        ) -> String {
            let rendered = match output {
                crate::OutputMode::Json | crate::OutputMode::Jsonl => crate::render_error_json(error),
                _ => format!("gwz: {}", error.human_message_with_transport(verbose)),
            };
            match report {
                Some(report) => report.render(rendered, output, verbose),
                None => rendered,
            }
        }

        pub(crate) fn prepare_transport(
            invocation: &mut CliInvocation,
            flag: Option<Transport>,
            environment: &gwz_core::session_host::EnvironmentSnapshot,
        ) -> Result<Option<TransportReport>, CliError> {
            let Some((operation, meta)) = transport_request(&mut invocation.request) else {
                return Ok(None);
            };
            let setting = transport_setting::resolve(flag, environment)
                .map_err(|error| CliError::invalid_request(error.message(Driver::Cli)))?;
            let ignored = transport_setting::ignored_values(operation, &invocation.start_dir, meta);
            if setting.transport == Transport::Native {
                let policy = meta.policy.get_or_insert_with(Default::default);
                policy.concurrency.get_or_insert(50);
                policy.max_connections_per_host.get_or_insert(8);
            }
            Ok(Some(TransportReport { setting, ignored }))
        }

        fn transport_request(
            request: &mut CliRequest,
        ) -> Option<(Operation, &mut gwz_core::RequestMeta)> {
            let (operation, meta, remote_tag) = match request {
                CliRequest::CloneWorkspace { meta, .. } => (Operation::CloneWorkspace, meta, true),
                CliRequest::InitFromSources(r) => (Operation::InitFromSources, &mut r.meta, true),
                CliRequest::CloneRepoMember(r) => (Operation::CloneRepoMember, &mut r.meta, true),
                CliRequest::Materialize(r) => (Operation::Materialize, &mut r.meta, true),
                CliRequest::Fetch(r) => (Operation::Fetch, &mut r.meta, true),
                CliRequest::PullHead(r) => (Operation::PullHead, &mut r.meta, true),
                CliRequest::PullSnapshot(r) => (Operation::PullSnapshot, &mut r.meta, true),
                CliRequest::Push(r) => (Operation::Push, &mut r.meta, true),
                CliRequest::Tag(r) => {
                    let remote = gwz_core::transport_scope::in_scope(Operation::Tag, Some(r));
                    (Operation::Tag, &mut r.meta, remote)
                }
                _ => return None,
            };
            remote_tag.then_some((operation, meta))
        }

        cfg_if::cfg_if! { if #[cfg(test)] {
            #[path = "transport_tests.rs"]
            mod tests;
        } }
    } else {
        #[derive(Clone, Debug, Default, Args)]
        pub(crate) struct TransportArgs {}
    }
}
