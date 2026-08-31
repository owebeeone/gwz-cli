//! Streaming machine rendering for the finite core commit-log output spool.

use std::io::{self, Write};

use serde_json::{Value, json};

use crate::OutputMode;

const LOG_SCHEMA: &str = "gwz.log/v0";
const READ_BATCH_RECORDS: u32 = 128;

#[derive(Debug)]
pub(crate) enum LogMachineOutputError {
    Read(gwz_core::model::ModelError),
    Write(io::Error),
    InvalidRecord(String),
}

pub(crate) fn write_log_machine_output<W: Write>(
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    log_id: &str,
    output: OutputMode,
    writer: &mut W,
) -> Result<(), LogMachineOutputError> {
    write_log_machine_output_with(output, |request| registry.read(log_id, &request), writer)
}

pub(crate) fn write_log_machine_output_with<W, R>(
    output: OutputMode,
    mut read: R,
    writer: &mut W,
) -> Result<(), LogMachineOutputError>
where
    W: Write,
    R: FnMut(
        gwz_core::operation::CommitLogReadRequest,
    ) -> gwz_core::model::ModelResult<gwz_core::operation::CommitLogReadResponse>,
{
    match output {
        OutputMode::Json => write_and_flush(writer, b"{\"records\":[")?,
        OutputMode::Jsonl => write_and_flush(
            writer,
            b"{\"record\":\"header\",\"schema\":\"gwz.log/v0\"}\n",
        )?,
        OutputMode::Human | OutputMode::Porcelain => {
            return Err(LogMachineOutputError::InvalidRecord(
                "commit-log machine renderer requires --json or --jsonl".to_owned(),
            ));
        }
    }

    let mut cursor = None;
    let mut first_json_record = true;
    loop {
        let response = read(gwz_core::operation::CommitLogReadRequest {
            cursor,
            max_records: Some(READ_BATCH_RECORDS),
        })
        .map_err(LogMachineOutputError::Read)?;

        match response.state {
            gwz_core::operation::CommitLogReadState::Eof => {
                if !response.records.is_empty() {
                    return Err(LogMachineOutputError::InvalidRecord(
                        "commit-log EOF response contains records".to_owned(),
                    ));
                }
                break;
            }
            gwz_core::operation::CommitLogReadState::Data => {
                if response.records.is_empty() {
                    return Err(LogMachineOutputError::InvalidRecord(
                        "commit-log data response contains no records".to_owned(),
                    ));
                }
                if cursor == Some(response.next_cursor) {
                    return Err(LogMachineOutputError::InvalidRecord(
                        "commit-log data response did not advance its cursor".to_owned(),
                    ));
                }
            }
        }

        for record in response.records {
            let serialized = log_record_json(record)?.to_string();
            let chunk = match output {
                OutputMode::Json if first_json_record => serialized,
                OutputMode::Json => format!(",{serialized}"),
                OutputMode::Jsonl => format!("{serialized}\n"),
                OutputMode::Human | OutputMode::Porcelain => unreachable!(),
            };
            write_and_flush(writer, chunk.as_bytes())?;
            first_json_record = false;
        }
        cursor = Some(response.next_cursor);
    }

    if output == OutputMode::Json {
        write_and_flush(
            writer,
            format!("],\"schema\":\"{LOG_SCHEMA}\"}}\n").as_bytes(),
        )?;
    }
    Ok(())
}

fn write_and_flush<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<(), LogMachineOutputError> {
    writer
        .write_all(bytes)
        .and_then(|()| writer.flush())
        .map_err(LogMachineOutputError::Write)
}

fn log_record_json(record: gwz_core::LogOutputRecord) -> Result<Value, LogMachineOutputError> {
    match (record.kind, record.entry, record.degradation) {
        (gwz_core::LogOutputRecordKind::Entry, Some(entry), None) => entry_json(entry),
        (gwz_core::LogOutputRecordKind::Degradation, None, Some(degradation)) => {
            Ok(degradation_json(degradation))
        }
        (gwz_core::LogOutputRecordKind::Entry, _, _) => Err(LogMachineOutputError::InvalidRecord(
            "commit-log entry record has inconsistent payload arms".to_owned(),
        )),
        (gwz_core::LogOutputRecordKind::Degradation, _, _) => {
            Err(LogMachineOutputError::InvalidRecord(
                "commit-log degradation record has inconsistent payload arms".to_owned(),
            ))
        }
    }
}

fn entry_json(entry: gwz_core::LogEntry) -> Result<Value, LogMachineOutputError> {
    let members = entry
        .members
        .into_iter()
        .map(|member| {
            json!({
                "hash": member.commit,
                "member_id": member.member_id,
                "member_path": member.member_path,
                "parents": member.parents,
            })
        })
        .collect::<Vec<_>>();
    let provenance = provenance_token(entry.provenance)?;
    let author = identity_json(entry.author, entry.author_timestamp_seconds, "author")?;
    let committer = identity_json(
        entry.committer,
        entry.committer_timestamp_seconds,
        "committer",
    )?;
    let mut object = json!({
        "author": author,
        "committer": committer,
        "members": members,
        "provenance": provenance,
        "record": "entry",
        "subject": entry.subject,
    })
    .as_object()
    .expect("object literal")
    .clone();
    if let Some(body) = entry.body {
        object.insert("body".to_owned(), Value::String(body));
    }
    if entry.lossy == Some(true) {
        object.insert("lossy".to_owned(), Value::Bool(true));
    }
    Ok(Value::Object(object))
}

fn identity_json(
    identity: gwz_core::GitObjectIdentity,
    seconds: i64,
    label: &str,
) -> Result<Value, LogMachineOutputError> {
    let offset_min = identity.timezone_offset_minutes.ok_or_else(|| {
        LogMachineOutputError::InvalidRecord(format!(
            "commit-log {label} identity has no recorded timezone offset"
        ))
    })?;
    Ok(json!({
        "email": identity.email,
        "name": identity.name,
        "time": {
            "offset_min": offset_min,
            "time": seconds,
        },
    }))
}

fn provenance_token(
    provenance: gwz_core::LogMergeProvenance,
) -> Result<String, LogMachineOutputError> {
    match (provenance.kind, provenance.gwz_commit_id.as_deref()) {
        (gwz_core::LogMergeKind::None, Some("marker-invalid")) => Ok("marker-invalid".to_owned()),
        (gwz_core::LogMergeKind::None, None) => Ok("none".to_owned()),
        (gwz_core::LogMergeKind::Heuristic, None) => Ok("heuristic".to_owned()),
        (gwz_core::LogMergeKind::Marker, Some(marker)) => Ok(format!("marker:{marker}")),
        _ => Err(LogMachineOutputError::InvalidRecord(
            "commit-log entry has inconsistent merge provenance".to_owned(),
        )),
    }
}

fn degradation_json(degradation: gwz_core::LogDegradation) -> Value {
    let reason = match degradation.reason {
        gwz_core::LogDegradationReason::RepositoryUnreadable => "repository_unreadable",
        gwz_core::LogDegradationReason::RepositoryMissing => "repository_missing",
        gwz_core::LogDegradationReason::Unborn => "unborn",
        gwz_core::LogDegradationReason::RevisionUnresolved => "revision_unresolved",
        gwz_core::LogDegradationReason::SnapshotEntryMissing => "snapshot_entry_missing",
        gwz_core::LogDegradationReason::LockEntryMissing => "lock_entry_missing",
        gwz_core::LogDegradationReason::UnsupportedSourceKind => "unsupported_source_kind",
    };
    json!({
        "member_id": degradation.member_id,
        "member_path": degradation.member_path,
        "message": degradation.message,
        "operand": degradation.operand,
        "reason": reason,
        "record": "degradation",
    })
}
