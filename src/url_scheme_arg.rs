use clap::ValueEnum;

use crate::*;

/// `--url-scheme` on `clone` and `materialize`: which URL form to clone from
/// on the known hosts (github.com, gitlab.com, bitbucket.org). `manifest`
/// is the built-in default and uses every URL as the manifest records it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum UrlSchemeArg {
    Manifest,
    Ssh,
    Https,
}

impl From<UrlSchemeArg> for gwz_core::UrlScheme {
    fn from(value: UrlSchemeArg) -> Self {
        match value {
            UrlSchemeArg::Manifest => gwz_core::UrlScheme::Manifest,
            UrlSchemeArg::Ssh => gwz_core::UrlScheme::Ssh,
            UrlSchemeArg::Https => gwz_core::UrlScheme::Https,
        }
    }
}

/// Environment fallback for `--url-scheme`; the flag wins when both are set.
pub(crate) const URL_SCHEME_ENV: &str = "GWZ_URL_SCHEME";

pub(crate) const URL_SCHEME_HELP: &str = "URL form for known-host repositories this run clones: manifest (as written, default), ssh, or https";

pub(crate) const URL_SCHEME_LONG_HELP: &str = "\
URL form used for every repository this run clones on github.com, gitlab.com \
or bitbucket.org. `manifest` (the default) uses each URL exactly as the \
manifest records it; `https` and `ssh` convert known-host URLs to that form \
before cloning. URLs on other hosts and local paths are used as written, and a \
member that is already checked out keeps its remotes. The environment \
variable GWZ_URL_SCHEME is an alternative to the flag; the flag wins. An \
`ssh` or `https` choice is remembered in <workspace>/.gwz/url-scheme.yml, so \
a later `gwz materialize` needs no flag; an explicit `manifest` clears it.";

/// The scheme this invocation asks core for: the flag, else `GWZ_URL_SCHEME`,
/// else nothing, in which case core applies the workspace's recorded
/// preference and then the manifest.
pub(crate) fn requested_url_scheme(
    flag: Option<UrlSchemeArg>,
) -> Result<Option<gwz_core::UrlScheme>, CliError> {
    if let Some(flag) = flag {
        return Ok(Some(flag.into()));
    }
    match std::env::var(URL_SCHEME_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            match UrlSchemeArg::from_str(trimmed, true) {
                Ok(scheme) => Ok(Some(scheme.into())),
                Err(_) => Err(CliError::new(format!(
                    "{URL_SCHEME_ENV} must be manifest, ssh or https, not {trimmed:?}"
                ))),
            }
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(CliError::new(format!(
            "{URL_SCHEME_ENV} is not valid UTF-8"
        ))),
    }
}

/// Records the requested scheme, if any, in the request's transport options.
pub(crate) fn apply_url_scheme(
    meta: &mut gwz_core::RequestMeta,
    flag: Option<UrlSchemeArg>,
) -> Result<(), CliError> {
    if let Some(scheme) = requested_url_scheme(flag)? {
        meta.transport
            .get_or_insert_with(Default::default)
            .url_scheme = Some(scheme);
    }
    Ok(())
}

/// The command-line spelling of a wire scheme.
pub(crate) fn url_scheme_name(scheme: gwz_core::UrlScheme) -> &'static str {
    match scheme {
        gwz_core::UrlScheme::Manifest => "manifest",
        gwz_core::UrlScheme::Ssh => "ssh",
        gwz_core::UrlScheme::Https => "https",
    }
}

/// The wire spelling of a resolution source.
pub(crate) fn url_scheme_source_name(source: gwz_core::UrlSchemeSource) -> &'static str {
    match source {
        gwz_core::UrlSchemeSource::Default => "default",
        gwz_core::UrlSchemeSource::Request => "request",
        gwz_core::UrlSchemeSource::Workspace => "workspace",
    }
}

/// Where a request-sourced scheme came from, for the human summary line. Core
/// only knows "the request"; the CLI knows it read the flag or the variable.
/// When the variable holds the applied scheme the variable is credited, which
/// is also right when the flag repeated the same value.
fn url_scheme_origin(resolution: &gwz_core::MemberUrlResolution) -> String {
    match resolution.source {
        gwz_core::UrlSchemeSource::Request => {
            let from_env = std::env::var(URL_SCHEME_ENV)
                .ok()
                .and_then(|value| UrlSchemeArg::from_str(value.trim(), true).ok())
                .is_some_and(|scheme| gwz_core::UrlScheme::from(scheme) == resolution.scheme);
            if from_env {
                format!("from {URL_SCHEME_ENV}")
            } else {
                "from --url-scheme".to_owned()
            }
        }
        gwz_core::UrlSchemeSource::Workspace => "from .gwz/url-scheme.yml".to_owned(),
        gwz_core::UrlSchemeSource::Default => "default".to_owned(),
    }
}

/// One summary line when the operation applied a scheme other than `manifest`.
pub(crate) fn url_scheme_summary(members: &[gwz_core::MemberResponse]) -> Option<String> {
    let resolution = members
        .iter()
        .filter_map(|member| member.url_resolution.as_ref())
        .find(|resolution| resolution.scheme != gwz_core::UrlScheme::Manifest)?;
    Some(format!(
        "url scheme: {} ({})",
        url_scheme_name(resolution.scheme),
        url_scheme_origin(resolution)
    ))
}

/// One line per member whose clone URL was rewritten, for `--verbose`.
pub(crate) fn url_resolution_human_lines(members: &[gwz_core::MemberResponse]) -> Vec<String> {
    members
        .iter()
        .filter_map(|member| {
            member
                .url_resolution
                .as_ref()
                .filter(|resolution| resolution.derived)
                .map(|resolution| {
                    format!(
                        "{}: {} -> {}",
                        member.member_path, resolution.manifest_url, resolution.effective_url
                    )
                })
        })
        .collect()
}

/// The JSON form of one member's resolution, for the response object.
pub(crate) fn url_resolution_json(resolution: &gwz_core::MemberUrlResolution) -> serde_json::Value {
    serde_json::json!({
        "manifest_url": resolution.manifest_url,
        "effective_url": resolution.effective_url,
        "scheme": url_scheme_name(resolution.scheme),
        "source": url_scheme_source_name(resolution.source),
        "derived": resolution.derived,
        "host_known": resolution.host_known,
    })
}
