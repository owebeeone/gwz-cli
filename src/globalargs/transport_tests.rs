use super::*;
use crate::{CliRequest, parse_args_with_request_id};

fn snapshot(entries: &[(&str, &str)]) -> gwz_core::session_host::EnvironmentSnapshot {
    gwz_core::session_host::EnvironmentSnapshot::from_os_pairs(
        entries
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into())),
    )
    .unwrap()
}

#[test]
fn flag_overrides_invalid_environment_and_native_fills_only_missing_policy() {
    let mut invocation = parse_args_with_request_id(
        vec!["--jobs".into(), "7".into(), "fetch".into()],
        "req_setting",
        std::path::Path::new("/"),
    )
    .unwrap();
    let report = prepare_transport(
        &mut invocation,
        Some(Transport::Native),
        &snapshot(&[("GWZ_TRANSPORT", "bad")]),
    )
    .unwrap()
    .unwrap();
    assert_eq!(report.setting.transport, Transport::Native);
    let CliRequest::Fetch(request) = invocation.request else {
        panic!("fetch")
    };
    let policy = request.meta.policy.unwrap();
    assert_eq!(policy.concurrency, Some(7));
    assert_eq!(policy.max_connections_per_host, Some(8));
}

#[test]
fn non_network_command_does_not_read_invalid_environment() {
    let mut invocation = parse_args_with_request_id(
        vec!["status".into()],
        "req_setting",
        std::path::Path::new("/"),
    )
    .unwrap();
    assert!(
        prepare_transport(
            &mut invocation,
            None,
            &snapshot(&[("GWZ_TRANSPORT", "bad")])
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn invalid_deciding_environment_refuses_before_dispatch() {
    let mut invocation = parse_args_with_request_id(
        vec!["fetch".into()],
        "req_setting",
        std::path::Path::new("/"),
    )
    .unwrap();
    let error = prepare_transport(
        &mut invocation,
        None,
        &snapshot(&[("GWZ_TRANSPORT", "bad")]),
    )
    .unwrap_err();
    assert!(
        error
            .message
            .contains("GWZ_TRANSPORT must be gwz or native")
    );
}

#[test]
fn help_states_native_defaults_and_all_selection_forms() {
    use clap::Parser;
    let help = crate::Cli::try_parse_from(["gwz", "fetch", "--help"])
        .unwrap_err()
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for text in [
        "Defaults to 100; 50 with --transport native.",
        "Defaults to 32; 8 with --transport native.",
        "it defaults to 3, as in gwz 1.0.",
        "GIT_CONFIG_GLOBAL",
        "Authentication rows (meta.transport in JSON) are reported on either transport.",
    ] {
        assert!(help.contains(text), "missing {text}");
    }
}

fn report(transport: Transport, source: Source) -> TransportReport {
    TransportReport {
        setting: Setting {
            transport,
            source,
            read: Vec::new(),
            skipped: Vec::new(),
        },
        ignored: Vec::new(),
    }
}

#[test]
fn native_defaults_and_explicit_timeouts_are_preserved() {
    let native = report(Transport::Native, Source::Environment);
    assert_eq!(native.timeout_seconds(None), 3);
    assert_eq!(native.timeout_seconds(Some(0)), 0);
    assert_eq!(native.timeout_seconds(Some(2)), 2);
    assert_eq!(
        report(Transport::Gwz, Source::Default).timeout_seconds(None),
        9
    );
    let mut invocation = parse_args_with_request_id(
        vec!["fetch".into()],
        "req_setting",
        std::path::Path::new("/"),
    )
    .unwrap();
    prepare_transport(&mut invocation, Some(Transport::Native), &snapshot(&[])).unwrap();
    let CliRequest::Fetch(request) = invocation.request else {
        panic!("fetch")
    };
    let policy = request.meta.policy.unwrap();
    assert_eq!(
        (policy.concurrency, policy.max_connections_per_host),
        (Some(50), Some(8))
    );
}

#[test]
fn json_presence_and_jsonl_keep_the_event_stream() {
    let rendered = r#"{"meta":{"transport":[{"kind":"authentication"}]},"ok":true}"#.to_owned();
    let default = report(Transport::Gwz, Source::Default);
    assert_eq!(
        default.render(rendered.clone(), crate::OutputMode::Json, false),
        rendered
    );
    let selected = report(Transport::Gwz, Source::Flag);
    let jsonl = selected.render(
        format!("{rendered}\n{{\"event\":\"unchanged\"}}"),
        crate::OutputMode::Jsonl,
        false,
    );
    let (first, rest) = jsonl.split_once('\n').unwrap();
    let parsed: serde_json::Value = serde_json::from_str(first).unwrap();
    assert_eq!(parsed["meta"]["transport_setting"]["transport"], "gwz");
    assert_eq!(parsed["meta"]["transport"][0]["kind"], "authentication");
    assert_eq!(rest, r#"{"event":"unchanged"}"#);
    assert!(selected.notes().is_empty());
    assert_eq!(
        default.verbose_line(),
        "transport: gwz (default; read none)"
    );
}

#[test]
fn native_notice_uses_the_deciding_form() {
    let native = report(Transport::Native, Source::Flag);
    assert_eq!(
        native.notes(),
        [
            "gwz: note: using libgit2's native transport (from --transport native), as gwz 1.0 did; omit --transport native to use gwz's transport"
        ]
    );
    assert_eq!(
        native.verbose_line(),
        "transport: native (from --transport)"
    );
}

#[test]
fn error_records_with_null_meta_are_untouched() {
    let rendered = r#"{"kind":"error","meta":null}"#.to_owned();
    assert_eq!(
        report(Transport::Native, Source::Flag).render(
            rendered.clone(),
            crate::OutputMode::Json,
            true
        ),
        rendered
    );
}
