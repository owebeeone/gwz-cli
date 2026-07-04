//! Client-side rendering for `gwz diff` (AD3/AD6 — presentation lives here).
//!
//! Two rendering surfaces:
//!
//! - **Manifest-only text** (`--name-only`, `--name-status`, `--stat`,
//!   `--numstat`, `--shortstat`, `--summary`): built from the wire
//!   [`DiffManifestResponse`] (which already carries workspace-relative paths,
//!   status, and per-file stats), so no byte log is read (D3).
//! - **Machine output** (`--json` / `--jsonl`): the manifest + summary as JSON,
//!   and — for `--jsonl` — the byte output records with `data` base64-expanded
//!   (patch bytes are `BYTES`; JSON has no byte type, so D5 base64-encodes them).
//!
//! No patch-hunk rendering happens here: patch/raw bytes come pre-rendered from
//! the core `diff.output` log and are written through verbatim by `diff_exec`.

use gwz_core::{
    DiffFileEntry, DiffManifestResponse, DiffOutputFormat, DiffOutputRecord, DiffOutputRecordKind,
    DiffStatus,
};
use serde_json::{Value, json};

/// Render a manifest-only human format to bytes. `patch`/`raw` formats never
/// reach here (they read the byte log); this covers name/status/stat/summary.
/// `null_terminated` (`-z`) switches the record separator to NUL for the
/// name/status formats (git `-z` semantics); stat/summary ignore it.
pub(crate) fn render_manifest_text(
    response: &DiffManifestResponse,
    format: DiffOutputFormat,
    null_terminated: bool,
) -> Vec<u8> {
    let sep: u8 = if null_terminated { 0 } else { b'\n' };
    match format {
        DiffOutputFormat::NameOnly => name_only(&response.files, sep),
        DiffOutputFormat::NameStatus => name_status(&response.files, sep),
        DiffOutputFormat::Numstat => numstat(&response.files, sep),
        DiffOutputFormat::Stat => stat(&response.files),
        DiffOutputFormat::Shortstat => shortstat(&response.files),
        DiffOutputFormat::Summary => summary(&response.files),
        // Any other format arriving without a byte log is empty (e.g. no diffs).
        _ => Vec::new(),
    }
}

/// Workspace-relative primary (new, else old) path for an entry.
fn primary_path(entry: &DiffFileEntry) -> &str {
    entry
        .new_path
        .as_deref()
        .or(entry.old_path.as_deref())
        .unwrap_or("")
}

/// The single-letter git status code for an entry.
fn status_letter(status: DiffStatus) -> char {
    match status {
        DiffStatus::Added => 'A',
        DiffStatus::Modified => 'M',
        DiffStatus::Deleted => 'D',
        DiffStatus::Renamed => 'R',
        DiffStatus::Copied => 'C',
        DiffStatus::TypeChanged => 'T',
        DiffStatus::Unmerged => 'U',
    }
}

fn name_only(files: &[DiffFileEntry], sep: u8) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for entry in files {
        out.extend_from_slice(primary_path(entry).as_bytes());
        out.push(sep);
    }
    out
}

fn name_status(files: &[DiffFileEntry], sep: u8) -> Vec<u8> {
    // With `-z`, git separates every field (status, oldpath, newpath) with NUL;
    // otherwise it uses a tab between fields and a newline between records.
    let field: u8 = if sep == 0 { 0 } else { b'\t' };
    let mut out: Vec<u8> = Vec::new();
    for entry in files {
        let letter = status_letter(entry.status);
        match entry.status {
            DiffStatus::Renamed | DiffStatus::Copied => {
                let sim = entry.similarity.unwrap_or(0);
                out.extend_from_slice(format!("{letter}{sim:03}").as_bytes());
                out.push(field);
                out.extend_from_slice(entry.old_path.as_deref().unwrap_or("").as_bytes());
                out.push(field);
                out.extend_from_slice(entry.new_path.as_deref().unwrap_or("").as_bytes());
                out.push(sep);
            }
            _ => {
                out.push(letter as u8);
                out.push(field);
                out.extend_from_slice(primary_path(entry).as_bytes());
                out.push(sep);
            }
        }
    }
    out
}

fn numstat(files: &[DiffFileEntry], sep: u8) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for entry in files {
        let (added, deleted) = if entry.is_binary.unwrap_or(false) {
            ("-".to_owned(), "-".to_owned())
        } else {
            (
                entry.insertions.unwrap_or(0).to_string(),
                entry.deletions.unwrap_or(0).to_string(),
            )
        };
        out.extend_from_slice(format!("{added}\t{deleted}\t").as_bytes());
        out.extend_from_slice(primary_path(entry).as_bytes());
        out.push(sep);
    }
    out
}

fn stat(files: &[DiffFileEntry]) -> Vec<u8> {
    let mut out = String::new();
    let mut total_ins = 0i64;
    let mut total_del = 0i64;
    for entry in files {
        let ins = entry.insertions.unwrap_or(0);
        let del = entry.deletions.unwrap_or(0);
        total_ins += ins;
        total_del += del;
        let changes = ins + del;
        let bar: String = "+".repeat(ins.max(0) as usize) + &"-".repeat(del.max(0) as usize);
        out.push_str(&format!(" {} | {changes} {bar}\n", primary_path(entry)));
    }
    out.push_str(&shortstat_line(files.len(), total_ins, total_del));
    out.into_bytes()
}

fn shortstat(files: &[DiffFileEntry]) -> Vec<u8> {
    let total_ins: i64 = files.iter().filter_map(|e| e.insertions).sum();
    let total_del: i64 = files.iter().filter_map(|e| e.deletions).sum();
    shortstat_line(files.len(), total_ins, total_del).into_bytes()
}

fn shortstat_line(files: usize, insertions: i64, deletions: i64) -> String {
    if files == 0 {
        return String::new();
    }
    let plural = |n: i64, s: &str| {
        if n == 1 {
            s.to_owned()
        } else {
            format!("{s}s")
        }
    };
    let mut parts = vec![format!(
        " {files} {}",
        plural(files as i64, "file") + " changed"
    )];
    if insertions > 0 {
        parts.push(format!(
            "{insertions} {}(+)",
            plural(insertions, "insertion")
        ));
    }
    if deletions > 0 {
        parts.push(format!("{deletions} {}(-)", plural(deletions, "deletion")));
    }
    format!("{}\n", parts.join(", "))
}

fn summary(files: &[DiffFileEntry]) -> Vec<u8> {
    let mut out = String::new();
    for entry in files {
        match entry.status {
            DiffStatus::Added => out.push_str(&format!(
                " create mode {:06o} {}\n",
                entry.new_mode.unwrap_or(0),
                primary_path(entry)
            )),
            DiffStatus::Deleted => out.push_str(&format!(
                " delete mode {:06o} {}\n",
                entry.old_mode.unwrap_or(0),
                primary_path(entry)
            )),
            DiffStatus::Renamed => out.push_str(&format!(
                " rename {} => {} ({}%)\n",
                entry.old_path.as_deref().unwrap_or(""),
                entry.new_path.as_deref().unwrap_or(""),
                entry.similarity.unwrap_or(0),
            )),
            _ => {}
        }
    }
    out.into_bytes()
}

// ── machine output ───────────────────────────────────────────────────────────

/// The full `--json` document: metadata + manifest + summary, no patch bytes
/// (AD5). One JSON object printed once.
pub(crate) fn manifest_json(response: &DiffManifestResponse) -> String {
    let files: Vec<Value> = response.files.iter().map(file_entry_json).collect();
    let summary = response.summary.as_ref().map(|summary| {
        json!({
            "has_differences": summary.has_differences,
            "repos_examined": summary.repos_examined,
            "repos_with_differences": summary.repos_with_differences,
            "files_changed": summary.files_changed,
            "insertions": summary.insertions,
            "deletions": summary.deletions,
        })
    });
    let excluded: Vec<Value> = response
        .excluded_targets
        .iter()
        .map(|target| {
            json!({
                "reason": format!("{:?}", target.reason),
                "snapshot_id": target.snapshot_id,
                "member_id": target.scope.member_id,
                "member_path": target.scope.member_path,
                "root": target.scope.root,
                "message": target.message,
            })
        })
        .collect();
    json!({
        "kind": "diff",
        "files": files,
        "summary": summary,
        "excluded_targets": excluded,
    })
    .to_string()
}

/// One `--jsonl` line per manifest entry (printed before any output records).
pub(crate) fn print_manifest_jsonl(response: &DiffManifestResponse) {
    if let Some(summary) = &response.summary {
        println!(
            "{}",
            json!({
                "kind": "diff_summary",
                "has_differences": summary.has_differences,
                "files_changed": summary.files_changed,
                "insertions": summary.insertions,
                "deletions": summary.deletions,
            })
        );
    }
    for entry in &response.files {
        println!(
            "{}",
            json!({ "kind": "diff_file", "entry": file_entry_json(entry) })
        );
    }
}

/// A `--jsonl` line for one `DiffOutputRecord`. Patch `data` is base64-expanded
/// (BYTES has no JSON representation, so D5 base64-encodes it); `stale_file` is
/// surfaced with its `stale` marker.
pub(crate) fn output_record_json(record: &DiffOutputRecord) -> String {
    let kind = match record.kind {
        DiffOutputRecordKind::PatchBytes => "patch_bytes",
        DiffOutputRecordKind::FileStarted => "file_started",
        DiffOutputRecordKind::FileFinished => "file_finished",
        DiffOutputRecordKind::StaleFile => "stale_file",
        DiffOutputRecordKind::Diagnostic => "diagnostic",
    };
    json!({
        "kind": "diff_output",
        "record_kind": kind,
        "file_id": record.file_id,
        "data_base64": record.data.as_ref().map(|bytes| base64_encode(bytes)),
        "stale": record.stale,
        "diagnostic": record.diagnostic,
    })
    .to_string()
}

fn file_entry_json(entry: &DiffFileEntry) -> Value {
    json!({
        "file_id": entry.file_id,
        "status": format!("{:?}", entry.status),
        "old_path": entry.old_path,
        "new_path": entry.new_path,
        "old_mode": entry.old_mode,
        "new_mode": entry.new_mode,
        "similarity": entry.similarity,
        "insertions": entry.insertions,
        "deletions": entry.deletions,
        "is_binary": entry.is_binary,
        "scope": {
            "root": entry.scope.root,
            "member_id": entry.scope.member_id,
            "member_path": entry.scope.member_path,
        },
    })
}

/// Minimal standard-alphabet base64 (no external dep). Used to expand patch
/// `BYTES` into a JSON-safe string for `--jsonl`.
fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
