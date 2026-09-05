//! `gwz local list` presentation (design §8.1, §7, §11 item 12).
//!
//! The payload is `LocalFamilyResponse.members`: one `LocalFamilyMemberEntry`
//! per family row, the root first and then members in name order, exactly as
//! `gwz-core` projects the pure model's `ListState` table. Nothing is sorted,
//! filtered, repaired or inferred here — the listing is observation-only on
//! both sides of the wire.
//!
//! Every enum match below is exhaustive with no wildcard, mirroring core's own
//! projection: a variant the protocol gains later fails to compile here rather
//! than being folded into a neighbouring word.

/// The design §8.1 spelling of a member kind.
pub(crate) fn member_kind_word(kind: gwz_core::LocalMemberKind) -> &'static str {
    match kind {
        gwz_core::LocalMemberKind::Checkout => "checkout",
        gwz_core::LocalMemberKind::Bare => "bare",
    }
}

/// The recorded lifecycle state of the index row (design §3.1).
pub(crate) fn recorded_state_word(state: gwz_core::LocalMemberState) -> &'static str {
    match state {
        gwz_core::LocalMemberState::Creating => "creating",
        gwz_core::LocalMemberState::Ready => "ready",
        gwz_core::LocalMemberState::Disposing => "disposing",
    }
}

/// The state core observed on disk, mirroring the pure model's projection.
pub(crate) fn observed_state_word(state: gwz_core::LocalObservedState) -> &'static str {
    match state {
        gwz_core::LocalObservedState::Ready => "ready",
        gwz_core::LocalObservedState::Incomplete => "incomplete",
        gwz_core::LocalObservedState::InterruptedDisposal => "interrupted_disposal",
        gwz_core::LocalObservedState::Missing => "missing",
        gwz_core::LocalObservedState::PointerRemoved => "pointer_removed",
        gwz_core::LocalObservedState::Mismatched => "mismatched",
        gwz_core::LocalObservedState::Malformed => "malformed",
        gwz_core::LocalObservedState::Unobserved => "unobserved",
    }
}

/// The `state` column. One word while the row and the disk agree — the whole
/// of design §8.1's sample listing — and `recorded/observed` the moment they
/// do not, so an interrupted create (`creating/incomplete`) or an interrupted
/// disposal (`disposing/interrupted_disposal`) is visible without a second
/// command.
pub(crate) fn state_column(entry: &gwz_core::LocalFamilyMemberEntry) -> String {
    let recorded = recorded_state_word(entry.recorded_state);
    let observed = observed_state_word(entry.observed_state);
    if recorded == observed {
        recorded.to_owned()
    } else {
        format!("{recorded}/{observed}")
    }
}

/// The human listing: `name  kind  state  path`, column-aligned, plus a
/// trailing `last_error` column that exists only when some row carries one
/// (design §7 makes it optional, and a family with nothing wrong must render
/// exactly the four columns of §8.1).
pub(crate) fn render_local_family_members(members: &[gwz_core::LocalFamilyMemberEntry]) -> String {
    if members.is_empty() {
        return "no local clone family members".to_owned();
    }
    let states: Vec<String> = members.iter().map(state_column).collect();
    let width = |values: &mut dyn Iterator<Item = usize>| values.max().unwrap_or(0);
    let name_width = width(&mut members.iter().map(|entry| entry.name.chars().count()));
    let kind_width = width(
        &mut members
            .iter()
            .map(|entry| member_kind_word(entry.kind).chars().count()),
    );
    let state_width = width(&mut states.iter().map(|state| state.chars().count()));
    let any_error = members.iter().any(|entry| entry.last_error.is_some());
    let path_width = if any_error {
        width(&mut members.iter().map(|entry| entry.path.chars().count()))
    } else {
        0
    };

    members
        .iter()
        .zip(states.iter())
        .map(|(entry, state)| {
            let mut line = format!(
                "{:name_width$}  {:kind_width$}  {:state_width$}  ",
                entry.name,
                member_kind_word(entry.kind),
                state,
            );
            match &entry.last_error {
                // A diagnostic is one line: a recorded newline would otherwise
                // forge a row of its own in a listing consumers read by line.
                Some(error) => {
                    line.push_str(&format!("{:path_width$}  ", entry.path));
                    line.push_str(&error.replace(['\n', '\r'], " "));
                }
                None => line.push_str(&entry.path),
            }
            line.trim_end().to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One member entry as JSON, field for field. Enum values follow this driver's
/// existing machine convention (the generated variant name, as
/// `branch_repos` and `stash_bundles` already render theirs).
pub(crate) fn local_family_member_json(
    entry: &gwz_core::LocalFamilyMemberEntry,
) -> serde_json::Value {
    serde_json::json!({
        "name": entry.name,
        "kind": format!("{:?}", entry.kind),
        "recorded_state": format!("{:?}", entry.recorded_state),
        "observed_state": format!("{:?}", entry.observed_state),
        "path": entry.path,
        "last_error": entry.last_error,
    })
}
