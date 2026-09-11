use super::*;

fn envelope_with(member: gwz_core::MemberResponse) -> gwz_core::ResponseEnvelope {
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            transport: None,
            request_id: "req_url_scheme".to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            action: gwz_core::ActionKind::Materialize,
            aggregate_status: gwz_core::AggregateStatus::Ok,
            operation_id: Some("op_url_scheme".to_owned()),
            message: None,
            attribution: None,
        },
        members: vec![member],
        errors: Vec::new(),
    }
}

fn member(url_resolution: Option<gwz_core::MemberUrlResolution>) -> gwz_core::MemberResponse {
    gwz_core::MemberResponse {
        member_id: "mem_app".to_owned(),
        member_path: "repos/app".to_owned(),
        source_kind: gwz_core::SourceKind::Git,
        status: gwz_core::MemberStatus::Ok,
        error: None,
        planned: None,
        state: None,
        git_status: None,
        lock_match: None,
        target_kind: Some(gwz_core::TargetKind::Member),
        lock_difference_reasons: None,
        url_resolution,
    }
}

fn resolution(
    scheme: gwz_core::UrlScheme,
    source: gwz_core::UrlSchemeSource,
    derived: bool,
) -> gwz_core::MemberUrlResolution {
    gwz_core::MemberUrlResolution {
        manifest_url: "git@github.com:o/r.git".to_owned(),
        effective_url: if derived {
            "https://github.com/o/r.git".to_owned()
        } else {
            "git@github.com:o/r.git".to_owned()
        },
        scheme,
        source,
        derived,
        host_known: true,
    }
}

#[test]
pub(crate) fn human_output_summarises_a_requested_scheme_and_verbose_lists_rewrites() {
    let response = CliResponse::envelope(envelope_with(member(Some(resolution(
        gwz_core::UrlScheme::Https,
        gwz_core::UrlSchemeSource::Request,
        true,
    )))));
    let rendered = render_response(&response, OutputMode::Human);
    assert!(
        rendered.contains("url scheme: https (from --url-scheme)")
            || rendered.contains("url scheme: https (from GWZ_URL_SCHEME)"),
        "{rendered}"
    );
    assert!(!rendered.contains(" -> "), "{rendered}");
    let verbose = render_response_with_transport(&response, OutputMode::Human, true);
    assert!(
        verbose.contains("repos/app: git@github.com:o/r.git -> https://github.com/o/r.git"),
        "{verbose}"
    );
}

#[test]
pub(crate) fn workspace_preference_and_manifest_default_render_as_specified() {
    let remembered = CliResponse::envelope(envelope_with(member(Some(resolution(
        gwz_core::UrlScheme::Ssh,
        gwz_core::UrlSchemeSource::Workspace,
        false,
    )))));
    let rendered = render_response(&remembered, OutputMode::Human);
    assert!(
        rendered.contains("url scheme: ssh (from .gwz/url-scheme.yml)"),
        "{rendered}"
    );
    let verbose = render_response_with_transport(&remembered, OutputMode::Human, true);
    assert!(!verbose.contains(" -> "), "an unchanged URL lists no rewrite: {verbose}");

    let plain = CliResponse::envelope(envelope_with(member(Some(resolution(
        gwz_core::UrlScheme::Manifest,
        gwz_core::UrlSchemeSource::Default,
        false,
    )))));
    let rendered = render_response(&plain, OutputMode::Human);
    assert!(!rendered.contains("url scheme:"), "{rendered}");
    let none = CliResponse::envelope(envelope_with(member(None)));
    assert!(!render_response(&none, OutputMode::Human).contains("url scheme:"));
}

#[test]
pub(crate) fn json_member_entries_carry_url_resolution() {
    let value = member_json(&member(Some(resolution(
        gwz_core::UrlScheme::Https,
        gwz_core::UrlSchemeSource::Workspace,
        true,
    ))));
    assert_eq!(value["url_resolution"]["manifest_url"], "git@github.com:o/r.git");
    assert_eq!(value["url_resolution"]["effective_url"], "https://github.com/o/r.git");
    assert_eq!(value["url_resolution"]["scheme"], "https");
    assert_eq!(value["url_resolution"]["source"], "workspace");
    assert_eq!(value["url_resolution"]["derived"], true);
    assert_eq!(value["url_resolution"]["host_known"], true);
    assert!(member_json(&member(None))["url_resolution"].is_null());
}
