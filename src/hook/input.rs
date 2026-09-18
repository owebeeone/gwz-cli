//! Claude's hook JSON on stdin, and the two tokens the hooks take from it
//! (section 1, D2, D6).
//!
//! Only the fields the hooks use are read. A malformed document, or a field
//! that is missing or of the wrong type, refuses with a message naming the
//! field (S1.1).

use super::HookFailure;

/// `WorktreeCreate`: `name` is "a slug identifier for the new worktree",
/// `session_id` the common field D6 validates before use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateInput {
    pub(crate) name: String,
    pub(crate) session_id: String,
}

/// `WorktreeRemove`: the created path, as Claude recorded it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveInput {
    pub(crate) worktree_path: String,
    pub(crate) session_id: String,
}

/// Characters a lane name may carry (D2). GWZ itself refuses `/` and `:`
/// and the reserved names; this is the narrower slug class the hook accepts,
/// so a slug that would become an awkward directory name never reaches core.
const NAME_CHARS: fn(char) -> bool =
    |value| value.is_ascii_alphanumeric() || matches!(value, '.' | '_' | '-');
/// R20's owner-token grammar: the name characters plus `:`.
const TOKEN_CHARS: fn(char) -> bool = |value| NAME_CHARS(value) || value == ':';

/// Names GWZ refuses outright (`gwz-family-model`'s `RESERVED_NAMES` and its
/// directory shorthands, quoted here because gwz-cli does not depend on that
/// crate directly).
pub(crate) const RESERVED_NAMES: [&str; 6] = ["root", "origin", "HEAD", "FETCH_HEAD", ".", ".."];

fn document(body: &str) -> Result<serde_json::Value, HookFailure> {
    serde_json::from_str::<serde_json::Value>(body).map_err(|error| {
        HookFailure::refused(
            format!("the hook input is not JSON: {error}"),
            "check the hooks block written by `gwz claude-code setup`",
        )
    })
}

fn string_field(document: &serde_json::Value, field: &str) -> Result<String, HookFailure> {
    match document.get(field) {
        Some(serde_json::Value::String(value)) if !value.is_empty() => Ok(value.clone()),
        Some(serde_json::Value::String(_)) | None => Err(HookFailure::refused(
            format!("the hook input has no `{field}`"),
            "check the hooks block written by `gwz claude-code setup`",
        )),
        Some(_) => Err(HookFailure::refused(
            format!("the hook input's `{field}` is not a string"),
            "check the hooks block written by `gwz claude-code setup`",
        )),
    }
}

pub(crate) fn parse_create_input(body: &str) -> Result<CreateInput, HookFailure> {
    let document = document(body)?;
    Ok(CreateInput {
        name: string_field(&document, "name")?,
        session_id: string_field(&document, "session_id")?,
    })
}

pub(crate) fn parse_remove_input(body: &str) -> Result<RemoveInput, HookFailure> {
    let document = document(body)?;
    Ok(RemoveInput {
        worktree_path: string_field(&document, "worktree_path")?,
        // A removal that carries no session id is still a removal: the
        // remove hook classifies by path, never by owner (D8).
        session_id: string_field(&document, "session_id").unwrap_or_default(),
    })
}

/// D2: the lane name is Claude's slug, validated against GWZ's refused names
/// and characters outside `[A-Za-z0-9._-]`.
pub(crate) fn validate_lane_name(name: &str) -> Result<(), HookFailure> {
    if name.is_empty() {
        return Err(HookFailure::refused(
            "the hook input's `name` is empty",
            "name the worktree, for example `claude --worktree fix-123`",
        ));
    }
    if let Some(bad) = name.chars().find(|value| !NAME_CHARS(*value)) {
        return Err(HookFailure::refused(
            format!("lane name `{name}` carries `{bad}`, outside [A-Za-z0-9._-]"),
            "choose a slug of letters, digits, `.`, `_` or `-`",
        ));
    }
    if RESERVED_NAMES.contains(&name) {
        return Err(HookFailure::refused(
            format!("`{name}` is a name GWZ reserves"),
            "choose another worktree name",
        ));
    }
    Ok(())
}

/// D6: the hook validates `session_id` against R20's token grammar before
/// using it. The grammar assumed here is one line of `[A-Za-z0-9._-]`, at
/// most 128 characters, which every observed hook payload's UUID satisfies
/// (section 1; S1.3 confirms).
pub(crate) fn validate_session_token(token: &str) -> Result<(), HookFailure> {
    let ok = !token.is_empty() && token.len() <= 128 && token.chars().all(TOKEN_CHARS);
    if ok {
        Ok(())
    } else {
        Err(HookFailure::refused(
            "the hook input's `session_id` is not an owner token".to_owned(),
            "report the payload; a session id must be [A-Za-z0-9._-], at most 128 characters",
        ))
    }
}
