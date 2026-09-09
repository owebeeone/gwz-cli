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
    let document: serde_json::Value = serde_json::from_str(&error.to_string()).unwrap();
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
fn help_routing_preserves_normal_parsing_and_rejects_unknown_topics() {
    assert!(crate::help::parse_from(["gwz", "--root", "help", "status"]).is_ok());
    let error = crate::help::parse_from(["gwz", "--json", "help", "imaginary"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    assert!(
        crate::help::parse_from(["gwz", "help", "local", "clone"])
            .unwrap_err()
            .to_string()
            .contains("--clean")
    );
}
