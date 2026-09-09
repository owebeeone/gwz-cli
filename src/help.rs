//! CLI metadata only: help never opens a workspace.
use crate::Cli;
use clap::{Arg, Command, CommandFactory, FromArgMatches};
use serde_json::{Value, json};
use std::ffi::OsString;

pub(crate) const ROOT_HELP: &str = "\
GWZ — manage a workspace of Git repositories

Usage: gwz [OPTIONS] <COMMAND>

Inspect:    status  ls  diff  log
Change:     add  commit  branch  tag  stash  merge  pull  push
Workspace:  init  clone  snapshot  capture  materialize
Members:    repo add|create|clone|detach|attach|sync
Lanes:      local clone|list|dispose|disband
Other:      auth  forall

Selection (default: root and every member):
  --root PATH       Workspace to operate in (default: current directory)
  --target TARGET   Repositories to include: @root, @all, member ID or path
  --no-target TARGET  Repositories to exclude
  --remote NAME     Git remote for network operations; local lane for merge

Common options:
  --json            Structured output; try gwz --json help [COMMAND...]
  --verbose         Authentication diagnostics
  --dry-run         Preview where supported; some commands refuse it
  -h, --help        Show help
  -V, --version     Show version; --build-info adds source identity

Details and all options: gwz help COMMAND [SUBCOMMAND]
Example: gwz --root ROOT --target @all merge --remote LANE
  Receive LANE's root and member histories into ROOT.

Documentation: https://owebeeone.github.io/gwz-cli/
";

/// Parse values and boundaries with Clap: a path named "help" is not a request.
pub(crate) fn parse_from<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut command = Cli::command().disable_help_subcommand(true).subcommand(
        Command::new("help")
            .about("Show human or JSON command help")
            .arg(Arg::new("topic").num_args(0..)),
    );
    let matches = command.clone().try_get_matches_from(args)?;
    if let Some(("help", help)) = matches.subcommand() {
        let topics: Vec<String> = help
            .get_many::<String>("topic")
            .map(|values| values.cloned().collect())
            .unwrap_or_default();
        command.build();
        let mut selected = &mut command;
        for topic in &topics {
            selected = selected.find_subcommand_mut(topic).ok_or_else(|| {
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidSubcommand,
                    format!("unknown help topic '{topic}'\n"),
                )
            })?;
        }
        let output = if matches.get_flag("json") || matches.get_flag("jsonl") {
            let path = std::iter::once("gwz")
                .chain(topics.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" ");
            serde_json::to_string(&document(selected, &path)).expect("help contains JSON values")
                + "\n"
        } else {
            selected.render_long_help().to_string()
        };
        return Err(clap::Error::raw(
            clap::error::ErrorKind::DisplayHelp,
            output,
        ));
    }
    Cli::from_arg_matches(&matches)
}

fn argument(arg: &Arg) -> Value {
    let range = arg.get_num_args();
    let takes_values = arg.get_action().takes_values();
    let value_range = if takes_values {
        let range = range.unwrap_or_else(|| clap::builder::ValueRange::new(1));
        Some(json!({
            "min": range.min_values(),
            "max": if range.max_values() == usize::MAX {
                Value::Null
            } else {
                json!(range.max_values())
            },
        }))
    } else {
        None
    };
    let action = match arg.get_action() {
        clap::ArgAction::Set => "set",
        clap::ArgAction::Append => "append",
        clap::ArgAction::SetTrue => "set_true",
        clap::ArgAction::SetFalse => "set_false",
        clap::ArgAction::Count => "count",
        clap::ArgAction::Help => "help",
        clap::ArgAction::HelpShort => "help_short",
        clap::ArgAction::HelpLong => "help_long",
        clap::ArgAction::Version => "version",
        _ => "other",
    };
    let repeatable = matches!(
        arg.get_action(),
        clap::ArgAction::Append | clap::ArgAction::Count
    );
    json!({
        "name": arg.get_id().as_str(),
        "short": arg.get_short().map(|value| value.to_string()),
        "long": arg.get_long(),
        "description": arg.get_help().map(ToString::to_string),
        "required": arg.is_required_set(),
        "global": arg.is_global_set(),
        "takes_values": takes_values,
        "action": action,
        "value_range": value_range,
        "value_delimiter": arg.get_value_delimiter().map(|value| value.to_string()),
        "repeatable": repeatable,
        "value_names": arg.get_value_names().map(|names| names.iter().map(|name| name.as_str()).collect::<Vec<_>>()),
        "defaults": arg.get_default_values().iter().map(|value| value.to_string_lossy()).collect::<Vec<_>>(),
        "choices": arg.get_value_parser().possible_values().map(|values| {
            values.filter(|value| !value.is_hide_set()).map(|value| value.get_name().to_owned()).collect::<Vec<_>>()
        }),
    })
}

fn document(command: &mut Command, path: &str) -> Value {
    let arguments: Vec<_> = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .collect();
    let options: Vec<_> = arguments
        .iter()
        .filter(|arg| !arg.is_positional())
        .map(|arg| argument(arg))
        .collect();
    let positional: Vec<_> = arguments
        .iter()
        .filter(|arg| arg.is_positional())
        .map(|arg| argument(arg))
        .collect();
    let children: Vec<_> = command.get_subcommands().filter(|child| !child.is_hide_set()).map(|child| {
        json!({"name": child.get_name(), "description": child.get_about().map(ToString::to_string)})
    }).collect();
    json!({
        "schema_version": 1,
        "kind": "help",
        "command": path,
        "description": command.get_about().map(ToString::to_string),
        "usage": command.render_usage().to_string(),
        "arguments": positional,
        "options": options,
        "commands": children,
        "details": format!("gwz help {}", path.strip_prefix("gwz").unwrap_or(path).trim()),
    })
}
