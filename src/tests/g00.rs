use super::*;

#[test]
pub(crate) fn usage_text_covers_standard_help_and_commands() {
    let usage = usage_text();

    assert!(usage.contains("Usage: gwz"));
    assert!(usage.contains("-h, --help"));
    assert!(usage.contains("init"));
    assert!(usage.contains("status"));
}

#[test]
pub(crate) fn root_help_advertises_hosted_docs_near_top_and_at_end() {
    let usage = usage_text();
    let docs_url = "https://owebeeone.github.io/gwz-cli/";

    let positions = usage
        .match_indices(docs_url)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();

    assert_eq!(positions.len(), 2, "{usage}");
    assert!(positions[0] < usage.find("Usage: gwz").unwrap(), "{usage}");
    assert!(usage.trim_end().ends_with(docs_url), "{usage}");
}

#[test]
pub(crate) fn cli_reference_doc_matches_generated_clap_help() {
    let checked_in = include_str!("../../docs/CLI.md");

    assert_eq!(
        checked_in,
        cli_reference_markdown(),
        "gwz-cli/docs/CLI.md is stale; run `python scripts/generate_cli_reference.py --write`"
    );
}
