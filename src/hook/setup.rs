//! `gwz hook claude-code setup` (D1, D5; S1.2).
//!
//! Prints the hooks block, and with `--write` merges it into the chosen
//! settings file, or with `--remove` takes it back out. Either edit touches
//! another program's configuration, so it parses the existing file first and
//! refuses one that does not parse, is a symbolic link or is not a regular
//! file; writes a temporary file beside the target, fsyncs, re-parses it, and
//! renames over the original; and changes no byte outside the inserted or
//! removed entries.

use std::io::Write;
use std::path::{Path, PathBuf};

use super::env::HookEnv;
use super::estimate::estimate;
use super::{DEFAULT_MAX_LANES, DEFAULT_WAIT_SECS, HookFailure, HookOptions};

pub(crate) const CREATE_EVENT: &str = "WorktreeCreate";
pub(crate) const REMOVE_EVENT: &str = "WorktreeRemove";

/// Where the block goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SetupPlacement {
    Project { local: bool },
    User,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SetupRequest {
    pub(crate) placement: SetupPlacement,
    pub(crate) write: bool,
    pub(crate) remove: bool,
    pub(crate) command: Option<String>,
    pub(crate) options: HookOptions,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SetupOutput {
    /// The block, as it is printed and as `--write` inserts it.
    pub(crate) block: String,
    pub(crate) target: PathBuf,
    /// What changed, or that nothing did.
    pub(crate) note: String,
}

/// The two handlers, with the command text and the timeout each carries.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Handlers {
    pub(crate) create_command: String,
    pub(crate) remove_command: String,
    pub(crate) create_timeout: u64,
    pub(crate) remove_timeout: u64,
}

/// The two roots `setup` works from, which are not the same question (F5).
///
/// `settings` is where the block goes: a workspace root, or a plain
/// repository's root for the D8 fallback case. `workspace` is the GWZ
/// workspace whose copy cost may size the handlers, and is `None` for
/// anything that is not one -- a plain repository has no lane to estimate,
/// and sizing a 1 s wait from a three-file repository is a hair trigger.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SetupRoots {
    pub(crate) settings: Option<PathBuf>,
    pub(crate) workspace: Option<PathBuf>,
}

impl SetupRoots {
    /// Neither root: `--user` from nowhere in particular.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    /// A settings root that is not a GWZ workspace (D8's fallback case).
    pub(crate) fn settings_only(settings: &Path) -> Self {
        Self {
            settings: Some(settings.to_path_buf()),
            workspace: None,
        }
    }

    /// What `setup` resolves from the directory it was run in: the workspace
    /// if there is one, and otherwise the enclosing repository for the
    /// settings file alone.
    pub(crate) fn resolve(start_dir: &Path) -> Self {
        if let Ok(workspace) = gwz_core::workspace::discover_workspace_root(start_dir) {
            return Self {
                settings: Some(workspace.clone()),
                workspace: Some(workspace),
            };
        }
        match super::ignore::enclosing_repository(start_dir) {
            Some(repository) => Self::settings_only(&repository),
            None => Self::none(),
        }
    }
}

/// S1.2, as F5 corrects it: `--wait-secs` is the compiled-in default with
/// the estimated copy time as a floor raised only inside a workspace -- the
/// estimate models the copy and not the queueing behind the family lock,
/// which is what the wait exists for, and on gwz-dev the estimate was 132 s
/// against a copy that took 207 s. The create timeout is one wait plus one
/// estimated copy plus 60 s, and the remove timeout one wait plus 60 s.
/// Outside a workspace both are the compiled-in defaults.
pub(crate) fn handlers(
    env: &dyn HookEnv,
    workspace_root: Option<&Path>,
    request: &SetupRequest,
) -> Handlers {
    let binary = request.command.clone().unwrap_or_else(|| "gwz".to_owned());
    let mut options = request.options.clone();
    let copy_seconds = workspace_root.and_then(|root| {
        let parent = root.parent()?;
        Some(estimate(env, root, parent).copy_time.as_secs().max(1))
    });
    options.wait_secs = baked_wait(options.wait_secs, copy_seconds);
    let wait = options.wait_secs;
    let copy = copy_seconds.unwrap_or(0);
    let create_suffix = suffix(&handler_words(&options, Leaf::Create));
    let remove_suffix = suffix(&handler_words(&options, Leaf::Remove));
    Handlers {
        create_command: format!("{binary} hook claude-code worktree-create{create_suffix}"),
        remove_command: format!("{binary} hook claude-code worktree-remove{remove_suffix}"),
        create_timeout: if copy_seconds.is_some() {
            wait + copy + 60
        } else {
            600
        },
        remove_timeout: if copy_seconds.is_some() {
            wait + 60
        } else {
            600
        },
    }
}

/// The wait a handler carries (F5): inside a workspace, the estimated copy
/// time with the compiled-in default as a *floor*, so the estimate can only
/// raise it; outside one, whatever the request configured, which is the
/// compiled-in default unless the operator said otherwise.
///
/// The floor exists because the estimate models the copy and not the wait
/// for the family lock, which is what `--wait-secs` is for: on gwz-dev the
/// baked 132 s met a copy that took 207 s, all but 30 s of it queueing
/// behind two other lanes (the probe of 2026-09-18, §3).
pub(crate) fn baked_wait(configured: u64, copy_seconds: Option<u64>) -> u64 {
    match copy_seconds {
        Some(copy) => copy.max(DEFAULT_WAIT_SECS),
        None => configured,
    }
}

/// Which hook leaf a handler drives. Each leaf carries only the options it
/// obeys, so the remove handler never advertises a creation guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Leaf {
    Create,
    Remove,
}

fn suffix(words: &[String]) -> String {
    if words.is_empty() {
        String::new()
    } else {
        format!(" {}", words.join(" "))
    }
}

/// The options as they appear inside a handler's command text, in a stable
/// order, so two `setup` runs with the same options produce the same handler
/// string (D5's dedupe keys on identical text).
fn handler_words(options: &HookOptions, leaf: Leaf) -> Vec<String> {
    let mut words = Vec::new();
    if leaf == Leaf::Create {
        if let Some(gb) = options.min_free_gb {
            words.push("--min-free-gb".to_owned());
            words.push(format_gb(gb));
        }
        if options.max_lanes != DEFAULT_MAX_LANES {
            words.push("--max-lanes".to_owned());
            words.push(options.max_lanes.to_string());
        }
    }
    if options.wait_secs != DEFAULT_WAIT_SECS {
        words.push("--wait-secs".to_owned());
        words.push(options.wait_secs.to_string());
    }
    if leaf == Leaf::Create
        && let Some(base) = &options.base_ref
    {
        words.push("--base-ref".to_owned());
        words.push(base.clone());
    }
    if let Some(log) = &options.log {
        words.push("--log".to_owned());
        words.push(log.clone());
    }
    words
}

fn format_gb(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value}")
    }
}

fn handler_value(command: &str, timeout: u64) -> serde_json::Value {
    serde_json::json!({
        "hooks": [
            { "type": "command", "command": command, "timeout": timeout }
        ]
    })
}

fn block_value(handlers: &Handlers) -> serde_json::Value {
    serde_json::json!({
        CREATE_EVENT: [handler_value(&handlers.create_command, handlers.create_timeout)],
        REMOVE_EVENT: [handler_value(&handlers.remove_command, handlers.remove_timeout)],
    })
}

/// The printed block: what `--write` would insert, as a settings fragment.
pub(crate) fn render_block(handlers: &Handlers) -> String {
    let document = serde_json::json!({ "hooks": block_value(handlers) });
    serde_json::to_string_pretty(&document).unwrap_or_default()
}

pub(crate) fn settings_path(
    env: &dyn HookEnv,
    placement: SetupPlacement,
    workspace_root: Option<&Path>,
) -> Result<PathBuf, HookFailure> {
    match placement {
        SetupPlacement::Project { local } => {
            let root = workspace_root.ok_or_else(|| {
                HookFailure::refused(
                    "--project needs a workspace or repository root",
                    "run it from the root, or use --user",
                )
            })?;
            let name = if local {
                "settings.local.json"
            } else {
                "settings.json"
            };
            Ok(root.join(".claude").join(name))
        }
        SetupPlacement::User => {
            let home = env.home_dir().ok_or_else(|| {
                HookFailure::refused(
                    "the home directory is not set",
                    "set HOME, or use --project",
                )
            })?;
            Ok(home.join(".claude/settings.json"))
        }
    }
}

pub(crate) fn run_setup(
    env: &dyn HookEnv,
    roots: &SetupRoots,
    request: &SetupRequest,
) -> Result<SetupOutput, HookFailure> {
    let workspace_root = roots.settings.as_deref();
    let handlers = handlers(env, roots.workspace.as_deref(), request);
    let block = render_block(&handlers);
    let target = settings_path(env, request.placement, workspace_root)?;
    if request.remove {
        let note = remove_from(&target)?;
        return Ok(SetupOutput {
            block,
            target,
            note,
        });
    }
    warn_about_other_placements(env, workspace_root, &target, &handlers);
    if !request.write {
        return Ok(SetupOutput {
            block,
            target,
            note: "not written; pass --write to merge it".to_owned(),
        });
    }
    let note = merge_into(&target, &handlers)?;
    Ok(SetupOutput {
        block,
        target,
        note,
    })
}

/// D5: warn, naming both files, when another placement on this machine
/// carries a differing handler.
fn warn_about_other_placements(
    env: &dyn HookEnv,
    workspace_root: Option<&Path>,
    target: &Path,
    handlers: &Handlers,
) {
    let mut candidates = Vec::new();
    if let Some(root) = workspace_root {
        candidates.push(root.join(".claude/settings.json"));
        candidates.push(root.join(".claude/settings.local.json"));
    }
    if let Some(home) = env.home_dir() {
        candidates.push(home.join(".claude/settings.json"));
    }
    for candidate in candidates {
        if candidate == target {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        let Ok(document) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        for command in existing_commands(&document, CREATE_EVENT) {
            if command != handlers.create_command {
                env.warn(&format!(
                    "gwz: {} carries a differing {CREATE_EVENT} handler (`{command}`); both run, \
                     and a refusal by either orphans the other's lane (see docs/ClaudeCode.md)",
                    candidate.display()
                ));
            }
        }
    }
}

fn existing_commands(document: &serde_json::Value, event: &str) -> Vec<String> {
    document
        .get("hooks")
        .and_then(|hooks| hooks.get(event))
        .and_then(serde_json::Value::as_array)
        .map(|groups| {
            groups
                .iter()
                .filter_map(|group| group.get("hooks").and_then(serde_json::Value::as_array))
                .flatten()
                .filter_map(|handler| handler.get("command").and_then(serde_json::Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Merge the block into `target`, changing no byte outside the insertion.
fn merge_into(target: &Path, handlers: &Handlers) -> Result<String, HookFailure> {
    let existing = match std::fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(HookFailure::refused(
                format!("{} is a symbolic link", target.display()),
                "write the block into the file the link names, or remove the link",
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(HookFailure::refused(
                format!("{} is not a regular file", target.display()),
                "move it aside and run the command again",
            ));
        }
        Ok(_) => Some(std::fs::read_to_string(target).map_err(|error| {
            HookFailure::refused(
                format!("{} could not be read: {error}", target.display()),
                "check the permissions on that file",
            )
        })?),
        Err(_) => None,
    };
    let Some(text) = existing else {
        let document = serde_json::json!({ "hooks": block_value(handlers) });
        let body = format!(
            "{}\n",
            serde_json::to_string_pretty(&document).unwrap_or_default()
        );
        write_atomically(target, &body)?;
        return Ok(format!("wrote {}", target.display()));
    };
    let document = serde_json::from_str::<serde_json::Value>(&text).map_err(|error| {
        HookFailure::refused(
            format!("{} is not JSON: {error}", target.display()),
            "fix that file, then run the command again",
        )
    })?;
    if !document.is_object() {
        return Err(HookFailure::refused(
            format!("{} is not a JSON object", target.display()),
            "fix that file, then run the command again",
        ));
    }
    let mut updated = text.clone();
    let mut inserted = Vec::new();
    // Inserted last to first (F1): each insertion goes immediately after the
    // brace it is inserted under, so the last one written is the first one
    // read, and the file ends up in the printed block's event order.
    for (event, command, timeout) in [
        (
            REMOVE_EVENT,
            handlers.remove_command.clone(),
            handlers.remove_timeout,
        ),
        (
            CREATE_EVENT,
            handlers.create_command.clone(),
            handlers.create_timeout,
        ),
    ] {
        let present = serde_json::from_str::<serde_json::Value>(&updated)
            .ok()
            .map(|document| existing_commands(&document, event).contains(&command))
            .unwrap_or(false);
        if present {
            continue;
        }
        updated = insert_handler(&updated, event, &command, timeout)?;
        inserted.push(event);
    }
    // The note names them the way the block prints them.
    inserted.reverse();
    if inserted.is_empty() {
        return Ok(format!("{} already carries the block", target.display()));
    }
    write_atomically(target, &updated)?;
    Ok(format!(
        "merged {} into {}",
        inserted.join(" and "),
        target.display()
    ))
}

/// The file's own indentation: the leading whitespace of its first indented
/// line, and two spaces when it has none to copy (F1).
fn indent_unit(text: &str) -> String {
    for line in text.lines() {
        let indent: String = line
            .chars()
            .take_while(|value| *value == ' ' || *value == '\t')
            .collect();
        if !indent.is_empty() && indent.len() < line.len() {
            return indent;
        }
    }
    "  ".to_owned()
}

/// The leading whitespace of the line the byte at `at` sits on.
fn line_indent(text: &str, at: usize) -> String {
    let start = text[..at].rfind('\n').map(|index| index + 1).unwrap_or(0);
    text[start..at]
        .chars()
        .take_while(|value| *value == ' ' || *value == '\t')
        .collect()
}

/// A value rendered the way `setup` prints it -- pretty, one member to a
/// line -- but in the file's indentation and at the file's depth (F1).
///
/// `serde_json` pretty-prints with two spaces; every line's leading pair is
/// exchanged for `unit`, and every line but the first is placed at `indent`.
/// The values here are this tool's own handler objects, whose strings carry
/// no raw newline, so counting leading spaces is exact.
fn rendered(value: &serde_json::Value, indent: &str, unit: &str) -> String {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_default();
    let mut out = String::with_capacity(pretty.len() * 2);
    for (index, line) in pretty.lines().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push_str(indent);
        }
        let depth = line.chars().take_while(|value| *value == ' ').count() / 2;
        for _ in 0..depth {
            out.push_str(unit);
        }
        out.push_str(line.trim_start_matches(' '));
    }
    out
}

/// Insert one handler group textually, so every byte outside the insertion
/// is the file's own, and the insertion itself reads like the block `setup`
/// prints (F1).
fn insert_handler(
    text: &str,
    event: &str,
    command: &str,
    timeout: u64,
) -> Result<String, HookFailure> {
    let unit = indent_unit(text);
    let handler = handler_value(command, timeout);
    if let Some(hooks) = value_start(text, "hooks", 1) {
        if let Some(array) = value_start(text, event, 2).filter(|position| *position > hooks) {
            // The event exists: add this handler to its array.
            let at = array + 1;
            let indent = format!("{}{unit}", line_indent(text, array));
            let separator = if next_meaningful(text, at) == Some(']') {
                String::new()
            } else {
                ",".to_owned()
            };
            let group = rendered(&handler, &indent, &unit);
            return Ok(splice(text, at, &indent, &format!("{group}{separator}")));
        }
        let at = hooks + 1;
        let indent = format!("{}{unit}", line_indent(text, hooks));
        let separator = if next_meaningful(text, at) == Some('}') {
            String::new()
        } else {
            ",".to_owned()
        };
        let array = rendered(&serde_json::json!([handler]), &indent, &unit);
        return Ok(splice(
            text,
            at,
            &indent,
            &format!("\"{event}\": {array}{separator}"),
        ));
    }
    let open = text.find('{').ok_or_else(|| {
        HookFailure::refused(
            "the settings file has no top-level object",
            "fix that file, then run the command again",
        )
    })?;
    let at = open + 1;
    let indent = format!("{}{unit}", line_indent(text, open));
    let separator = if next_meaningful(text, at) == Some('}') {
        String::new()
    } else {
        ",".to_owned()
    };
    let object = rendered(&serde_json::json!({ event: [handler] }), &indent, &unit);
    Ok(splice(
        text,
        at,
        &indent,
        &format!("\"hooks\": {object}{separator}"),
    ))
}

fn splice(text: &str, at: usize, indent: &str, insertion: &str) -> String {
    let mut updated = String::with_capacity(text.len() + insertion.len() + indent.len() + 2);
    updated.push_str(&text[..at]);
    updated.push('\n');
    updated.push_str(indent);
    updated.push_str(insertion);
    updated.push_str(&text[at..]);
    updated
}

fn next_meaningful(text: &str, from: usize) -> Option<char> {
    text[from..].chars().find(|value| !value.is_whitespace())
}

/// The byte offset of the `{` or `[` that opens the value of `key`, when the
/// key sits at `depth` in the document.
fn value_start(text: &str, key: &str, depth: usize) -> Option<usize> {
    find_member(text, key, depth).map(|(_, value)| value)
}

/// The byte offset of `key`'s opening quote and of the `{` or `[` that opens
/// its value, when the key sits at `depth` in the document. A scan, not a
/// parser: it tracks strings, escapes and container depth, which is all that
/// is needed to find a key that a `serde_json` parse has already proved is
/// there.
fn find_member(text: &str, key: &str, depth: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut index = 0_usize;
    let mut current = 0_usize;
    while index < bytes.len() {
        match bytes[index] {
            b'{' | b'[' => {
                current += 1;
                index += 1;
            }
            b'}' | b']' => {
                current = current.saturating_sub(1);
                index += 1;
            }
            b'"' => {
                let key_start = index;
                let start = index + 1;
                let mut end = start;
                while end < bytes.len() {
                    if bytes[end] == b'\\' {
                        end += 2;
                        continue;
                    }
                    if bytes[end] == b'"' {
                        break;
                    }
                    end += 1;
                }
                let end = end.min(bytes.len());
                let token = &text[start..end];
                index = end + 1;
                if current == depth && token == key {
                    let mut after = index;
                    while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                        after += 1;
                    }
                    if bytes.get(after) == Some(&b':') {
                        after += 1;
                        while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                            after += 1;
                        }
                        if matches!(bytes.get(after), Some(b'{') | Some(b'[')) {
                            return Some((key_start, after));
                        }
                    }
                }
            }
            _ => {
                index += 1;
            }
        }
    }
    None
}

/// The byte offset just past the `}` or `]` that closes the container opened
/// at `start`.
fn balanced_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0_usize;
    let mut index = start;
    let mut in_string = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            match byte {
                b'\\' => {
                    index += 2;
                    continue;
                }
                b'"' => {
                    in_string = false;
                }
                _ => {}
            }
            index += 1;
            continue;
        }
        match byte {
            b'"' => {
                in_string = true;
            }
            b'{' | b'[' => {
                depth += 1;
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// The byte spans of the elements of the array opened at `open`.
fn array_elements(text: &str, open: usize) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let Some(close) = balanced_end(text, open) else {
        return Vec::new();
    };
    let mut spans = Vec::new();
    let mut index = open + 1;
    while index < close - 1 {
        while index < close - 1 && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
            index += 1;
        }
        if index >= close - 1 {
            break;
        }
        let end = if matches!(bytes[index], b'{' | b'[') {
            match balanced_end(text, index) {
                Some(end) => end,
                None => break,
            }
        } else {
            let mut scan = index;
            let mut in_string = false;
            while scan < close - 1 {
                let byte = bytes[scan];
                if in_string {
                    if byte == b'\\' {
                        scan += 2;
                        continue;
                    }
                    if byte == b'"' {
                        in_string = false;
                    }
                    scan += 1;
                    continue;
                }
                if byte == b'"' {
                    in_string = true;
                } else if byte == b',' {
                    break;
                }
                scan += 1;
            }
            while scan > index && bytes[scan - 1].is_ascii_whitespace() {
                scan -= 1;
            }
            scan
        };
        spans.push((index, end));
        index = end;
    }
    spans
}

/// Delete `[start, end)` together with the separator and the whitespace the
/// insertion introduced, so removing an inserted span restores the original
/// bytes exactly.
fn remove_span(text: &str, start: usize, end: usize) -> String {
    let bytes = text.as_bytes();
    let mut from = start;
    while from > 0 && bytes[from - 1].is_ascii_whitespace() {
        from -= 1;
    }
    let mut after = end;
    while after < bytes.len() && bytes[after].is_ascii_whitespace() {
        after += 1;
    }
    let mut to = end;
    if bytes.get(after) == Some(&b',') {
        to = after + 1;
    } else if from > 0 && bytes[from - 1] == b',' {
        from -= 1;
    }
    let mut updated = String::with_capacity(text.len());
    updated.push_str(&text[..from]);
    updated.push_str(&text[to..]);
    updated
}

/// True when this handler command is one this tool wrote for `event`.
fn ours(command: &str, event: &str) -> bool {
    let leaf = if event == CREATE_EVENT {
        "hook claude-code worktree-create"
    } else {
        "hook claude-code worktree-remove"
    };
    command.contains(leaf)
}

/// The span of the first handler group under `event` that this tool wrote.
fn our_entry_span(text: &str, event: &str) -> Option<(usize, usize)> {
    let hooks = value_start(text, "hooks", 1)?;
    let array = value_start(text, event, 2).filter(|position| *position > hooks)?;
    for (start, end) in array_elements(text, array) {
        let Ok(group) = serde_json::from_str::<serde_json::Value>(&text[start..end]) else {
            continue;
        };
        let carries = group
            .get("hooks")
            .and_then(serde_json::Value::as_array)
            .map(|handlers| {
                handlers.iter().any(|handler| {
                    handler
                        .get("command")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|command| ours(command, event))
                })
            })
            .unwrap_or(false);
        if carries {
            return Some((start, end));
        }
    }
    None
}

/// The span of the member `"key": <value>` at `depth`, from its opening quote
/// to the end of its value.
fn member_span(text: &str, key: &str, depth: usize) -> Option<(usize, usize)> {
    let (key_start, value) = find_member(text, key, depth)?;
    let end = balanced_end(text, value)?;
    Some((key_start, end))
}

fn container_is_empty(text: &str, open: usize) -> bool {
    let Some(end) = balanced_end(text, open) else {
        return false;
    };
    text[open + 1..end - 1].trim().is_empty()
}

/// `--remove`: take the two entries this tool wrote out of `target`, with the
/// writer's discipline. The file itself is never deleted.
fn remove_from(target: &Path) -> Result<String, HookFailure> {
    let text = match std::fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(HookFailure::refused(
                format!("{} is a symbolic link", target.display()),
                "remove the block from the file the link names, or remove the link",
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(HookFailure::refused(
                format!("{} is not a regular file", target.display()),
                "move it aside and run the command again",
            ));
        }
        Ok(_) => std::fs::read_to_string(target).map_err(|error| {
            HookFailure::refused(
                format!("{} could not be read: {error}", target.display()),
                "check the permissions on that file",
            )
        })?,
        Err(_) => {
            return Ok(format!(
                "{} does not exist; nothing to remove",
                target.display()
            ));
        }
    };
    let document = serde_json::from_str::<serde_json::Value>(&text).map_err(|error| {
        HookFailure::refused(
            format!("{} is not JSON: {error}", target.display()),
            "fix that file, then run the command again",
        )
    })?;
    if !document.is_object() {
        return Err(HookFailure::refused(
            format!("{} is not a JSON object", target.display()),
            "fix that file, then run the command again",
        ));
    }
    let mut updated = text.clone();
    let mut removed = Vec::new();
    for event in [CREATE_EVENT, REMOVE_EVENT] {
        while let Some((start, end)) = our_entry_span(&updated, event) {
            updated = remove_span(&updated, start, end);
            if !removed.contains(&event) {
                removed.push(event);
            }
        }
    }
    if removed.is_empty() {
        return Ok(format!(
            "{} does not carry the block; nothing was changed",
            target.display()
        ));
    }
    // An array or a `hooks` object left empty behind the entries goes too.
    for event in [CREATE_EVENT, REMOVE_EVENT] {
        let hooks = value_start(&updated, "hooks", 1);
        let array = value_start(&updated, event, 2)
            .filter(|position| hooks.is_some_and(|hooks| *position > hooks));
        if let Some(array) = array
            && container_is_empty(&updated, array)
            && let Some((start, end)) = member_span(&updated, event, 2)
        {
            updated = remove_span(&updated, start, end);
        }
    }
    if let Some(hooks) = value_start(&updated, "hooks", 1)
        && container_is_empty(&updated, hooks)
        && let Some((start, end)) = member_span(&updated, "hooks", 1)
    {
        updated = remove_span(&updated, start, end);
    }
    write_atomically(target, &updated)?;
    Ok(format!(
        "removed {} from {}",
        removed.join(" and "),
        target.display()
    ))
}

/// Temporary file beside the target, fsync, re-parse, rename (D1).
fn write_atomically(target: &Path, body: &str) -> Result<(), HookFailure> {
    if serde_json::from_str::<serde_json::Value>(body).is_err() {
        return Err(HookFailure::refused(
            "the merged settings would not parse; nothing was written",
            "report this: the insertion is a gwz defect",
        ));
    }
    let parent = target.parent().ok_or_else(|| {
        HookFailure::refused(
            format!("{} has no parent directory", target.display()),
            "choose another settings location",
        )
    })?;
    std::fs::create_dir_all(parent).map_err(|error| {
        HookFailure::refused(
            format!("{} could not be created: {error}", parent.display()),
            "check the permissions on that directory",
        )
    })?;
    let temporary = parent.join(format!(
        ".{}.gwz-{}",
        target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "settings.json".to_owned()),
        std::process::id()
    ));
    let written = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(body.as_bytes())?;
        file.sync_all()
    })();
    if let Err(error) = written {
        let _ = std::fs::remove_file(&temporary);
        return Err(HookFailure::refused(
            format!("{} could not be written: {error}", temporary.display()),
            "check the permissions on that directory",
        ));
    }
    let reread = std::fs::read_to_string(&temporary).ok();
    let parses = reread
        .as_deref()
        .is_some_and(|text| serde_json::from_str::<serde_json::Value>(text).is_ok());
    if !parses {
        let _ = std::fs::remove_file(&temporary);
        return Err(HookFailure::refused(
            "the temporary settings file did not read back as JSON; nothing was written",
            "retry, and check the filesystem if it happens again",
        ));
    }
    std::fs::rename(&temporary, target).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        HookFailure::refused(
            format!("{} could not be replaced: {error}", target.display()),
            "check the permissions on that file",
        )
    })
}
