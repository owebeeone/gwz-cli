//! Human rendering for the finite core commit-log record stream.

use gwz_core::{LogDegradation, LogDegradationReason, LogEntry};

use crate::LogColor;

pub(crate) fn log_color_enabled(color: LogColor, stdout_is_tty: bool) -> bool {
    match color {
        LogColor::Always => true,
        LogColor::Never => false,
        LogColor::Auto => stdout_is_tty,
    }
}

pub(crate) fn render_log_entry(
    entry: &LogEntry,
    full: bool,
    color: bool,
) -> Result<String, &'static str> {
    let representative = entry
        .members
        .first()
        .ok_or("commit-log entry has no members")?;
    Ok(if full {
        render_full_entry(entry, representative, color)
    } else {
        render_compact_entry(entry, representative, color)
    })
}

pub(crate) fn render_log_degradation(record: &LogDegradation, color: bool) -> String {
    let path = sanitize_inline(if record.member_path.is_empty() {
        &record.member_id
    } else {
        &record.member_path
    });
    let mut reason = match record.reason {
        LogDegradationReason::RepositoryUnreadable => "repository unreadable".to_owned(),
        LogDegradationReason::RepositoryMissing => "repository missing".to_owned(),
        LogDegradationReason::Unborn => "unborn history".to_owned(),
        LogDegradationReason::RevisionUnresolved => "revision unresolved".to_owned(),
        LogDegradationReason::SnapshotEntryMissing => "snapshot entry missing".to_owned(),
        LogDegradationReason::LockEntryMissing => "lock entry missing".to_owned(),
        LogDegradationReason::UnsupportedSourceKind => "unsupported source kind".to_owned(),
    };
    if let Some(operand) = &record.operand {
        reason.push_str(" for '");
        reason.push_str(&sanitize_inline(operand));
        reason.push('\'');
    }
    if let Some(message) = record
        .message
        .as_deref()
        .filter(|message| !message.is_empty())
    {
        reason.push_str(" — ");
        reason.push_str(&sanitize_inline(message));
    }
    format!(
        "{} {path}: {reason}",
        colorize("gwz log: degraded", "33", color)
    )
}

fn render_compact_entry(
    entry: &LogEntry,
    representative: &gwz_core::LogEntryMember,
    color: bool,
) -> String {
    let date = format_date(
        entry.committer_timestamp_seconds,
        entry.committer.timezone_offset_minutes,
    );
    let members = compact_member_set(entry);
    let hash = representative.commit.chars().take(12).collect::<String>();
    let subject = sanitize_inline(&entry.subject);
    format!(
        "{} {} {} {subject}",
        colorize(&date, "2", color),
        colorize(&members, "36", color),
        colorize(&sanitize_inline(&hash), "33", color),
    )
}

fn compact_member_set(entry: &LogEntry) -> String {
    if entry.members.len() == 1 {
        return sanitize_inline(&entry.members[0].member_path);
    }
    if entry.members.len() <= 3 {
        return format!(
            "[{}]",
            entry
                .members
                .iter()
                .map(|member| sanitize_inline(&member.member_path))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let non_root = entry
        .members
        .iter()
        .filter(|member| member.member_id != "@root")
        .count();
    if non_root < entry.members.len() {
        format!("[root+{non_root}]")
    } else {
        format!("[{} members]", entry.members.len())
    }
}

fn render_full_entry(
    entry: &LogEntry,
    representative: &gwz_core::LogEntryMember,
    color: bool,
) -> String {
    let representative = sanitize_inline(&representative.commit);
    let rows = entry
        .members
        .iter()
        .map(|member| {
            (
                sanitize_inline(&member.member_id),
                sanitize_inline(&member.member_path),
                sanitize_inline(&member.commit),
            )
        })
        .collect::<Vec<_>>();
    let id_width = rows
        .iter()
        .map(|(id, _, _)| id.chars().count())
        .max()
        .unwrap_or(0)
        .max(2);
    let path_width = rows
        .iter()
        .map(|(_, path, _)| path.chars().count())
        .max()
        .unwrap_or(0)
        .max(4);

    let mut output = String::new();
    output.push_str(&colorize(&format!("commit {representative}"), "33", color));
    output.push('\n');
    output.push_str(&colorize("Members:", "36", color));
    output.push('\n');
    output.push_str(&format!(
        "    {:<id_width$}  {:<path_width$}  COMMIT\n",
        "ID", "PATH"
    ));
    for (id, path, commit) in rows {
        output.push_str(&format!(
            "    {id:<id_width$}  {path:<path_width$}  {commit}\n"
        ));
    }
    output.push_str(&format!(
        "Author: {} <{}>\n",
        sanitize_inline(&entry.author.name),
        sanitize_inline(&entry.author.email)
    ));
    output.push_str(&format!(
        "Date:   {}\n\n",
        format_date(
            entry.author_timestamp_seconds,
            entry.author.timezone_offset_minutes
        )
    ));

    let mut message = sanitize_inline(&entry.subject);
    if let Some(body) = &entry.body {
        message.push('\n');
        message.push_str(&sanitize_multiline(body));
    }
    for (index, line) in message.split('\n').enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str("    ");
        output.push_str(line);
    }
    output
}

fn format_date(seconds: i64, offset_minutes: Option<i64>) -> String {
    let offset_minutes = i128::from(offset_minutes.unwrap_or(0));
    let local_seconds = i128::from(seconds) + offset_minutes * 60;
    let days = local_seconds.div_euclid(86_400);
    let seconds_in_day = local_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_in_day / 3_600;
    let minute = seconds_in_day % 3_600 / 60;
    let second = seconds_in_day % 60;
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let absolute_offset = offset_minutes.abs();
    let offset_hour = absolute_offset / 60;
    let offset_minute = absolute_offset % 60;
    format!(
        "{}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} {sign}{offset_hour:02}{offset_minute:02}",
        format_year(year)
    )
}

fn civil_from_days(days_since_epoch: i128) -> (i128, i128, i128) {
    let shifted = days_since_epoch + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i128::from(month <= 2);
    (year, month, day)
}

fn format_year(year: i128) -> String {
    if year < 0 {
        format!("-{:04}", -year)
    } else {
        format!("{year:04}")
    }
}

fn sanitize_inline(value: &str) -> String {
    sanitize(value, false)
}

fn sanitize_multiline(value: &str) -> String {
    sanitize(value, true)
}

fn sanitize(value: &str, preserve_newlines: bool) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' if preserve_newlines => '\n',
            '\t' => ' ',
            '\u{0}'..='\u{1f}' => '\u{fffd}',
            _ => character,
        })
        .collect()
}

fn colorize(value: &str, code: &str, color: bool) -> String {
    if color {
        format!("\u{1b}[{code}m{value}\u{1b}[0m")
    } else {
        value.to_owned()
    }
}
