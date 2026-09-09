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
pub(crate) fn root_help_is_a_navigation_map_with_all_top_level_commands() {
    let usage = usage_text();
    assert!(usage.lines().count() < 40, "{usage}");
    for command in <Cli as clap::CommandFactory>::command().get_subcommands() {
        assert!(
            usage
                .split_whitespace()
                .any(|word| word == command.get_name()),
            "missing command {}",
            command.get_name()
        );
    }
    assert!(usage.contains("gwz help COMMAND"));
    assert!(usage.contains("--target TARGET"));
    assert!(
        usage
            .trim_end()
            .ends_with("https://owebeeone.github.io/gwz-cli/")
    );
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

#[test]
fn structured_help_uses_parser_arguments_without_opening_a_workspace() {
    let error = crate::help::parse_from(["gwz", "--json", "help", "repo", "detach"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    let rendered = error.render().to_string();
    let rendered = rendered.strip_prefix("error: ").unwrap_or(&rendered);
    let document: serde_json::Value = serde_json::from_str(rendered).unwrap();
    assert_eq!(document["schema_version"], 1);
    assert_eq!(document["command"], "gwz repo detach");
    assert!(
        document["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg["required"] == true)
    );
    assert!(
        document["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg["long"] == "root")
    );
}

#[test]
fn structured_help_describes_value_ranges_and_repetition() {
    let error = crate::help::parse_from(["gwz", "--json", "help", "add"]).unwrap_err();
    let rendered = error.render().to_string();
    let rendered = rendered.strip_prefix("error: ").unwrap_or(&rendered);
    let document: serde_json::Value = serde_json::from_str(rendered).unwrap();
    let pathspec = document["arguments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|arg| arg["name"] == "pathspecs")
        .unwrap();
    assert_eq!(pathspec["takes_values"], true);
    assert_eq!(pathspec["repeatable"], true);
    assert_eq!(pathspec["action"], "append");
    assert_eq!(pathspec["value_range"]["min"], 1);
    assert_eq!(pathspec["value_range"]["max"], serde_json::Value::Null);
    let target = document["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|arg| arg["long"] == "remote-identity")
        .unwrap();
    assert_eq!(target["repeatable"], true);
    assert_eq!(target["action"], "append");
}

#[test]
fn help_routing_preserves_normal_parsing_and_rejects_unknown_topics() {
    assert!(crate::help::parse_from(["gwz", "--root", "help", "status"]).is_ok());
    assert!(crate::help::parse_from(["gwz", "help", "status", "--json"]).is_err());
    assert!(crate::help::parse_from(["gwz", "add", "help"]).is_ok());
    let error = crate::help::parse_from(["gwz", "--json", "help", "imaginary"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    assert!(
        crate::help::parse_from(["gwz", "help", "local", "clone"])
            .unwrap_err()
            .to_string()
            .contains("--clean")
    );
}
