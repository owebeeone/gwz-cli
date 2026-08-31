//! No-pager lifecycle driver for the finite core commit-log output spool.
//!
//! Human and machine modes drain and render the bounded core record spool
//! directly. Every path preserves caller-owned release and clean EPIPE
//! termination; porcelain retains the S3.1 plumbing response.

use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::{
    CliError, CliResponse, LogInvocation, OutputMode, exit_code_for_response, log_color_enabled,
    render_log_degradation, render_log_entry, render_response, write_log_machine_output,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LogExit {
    pub(crate) code: i32,
}

pub(crate) struct LogIo<'a, W: Write, E: Write> {
    pub(crate) stdout: &'a mut W,
    pub(crate) stderr: &'a mut E,
    pub(crate) stdout_is_tty: bool,
}

pub(crate) fn run_log(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
) -> Result<LogExit, CliError> {
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let stdout = io::stdout();
    let stdout_is_tty = stdout.is_terminal();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    run_log_with_registry_io(
        invocation,
        output,
        start,
        operation_id,
        &registry,
        LogIo {
            stdout: &mut stdout,
            stderr: &mut stderr,
            stdout_is_tty,
        },
    )
}

#[cfg(test)]
pub(crate) fn run_log_with_registry<W: Write>(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    stdout: &mut W,
) -> Result<LogExit, CliError> {
    let mut stderr = io::sink();
    run_log_with_registry_io(
        invocation,
        output,
        start,
        operation_id,
        registry,
        LogIo {
            stdout,
            stderr: &mut stderr,
            stdout_is_tty: false,
        },
    )
}

pub(crate) fn run_log_with_registry_io<W: Write, E: Write>(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    io: LogIo<'_, W, E>,
) -> Result<LogExit, CliError> {
    run_log_with_registry_io_using_machine(
        invocation,
        output,
        start,
        operation_id,
        registry,
        io,
        |registry, log_id, output, stdout| {
            write_log_machine_output(registry, log_id, output, stdout)
        },
    )
}

fn run_log_with_registry_io_using_machine<W, E, M>(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    io: LogIo<'_, W, E>,
    machine_output: M,
) -> Result<LogExit, CliError>
where
    W: Write,
    E: Write,
    M: FnOnce(
        &gwz_core::operation::CommitLogOutputRegistry,
        &str,
        OutputMode,
        &mut W,
    ) -> Result<(), crate::LogMachineOutputError>,
{
    let response =
        gwz_core::operation::handle_log(start, invocation.request.clone(), operation_id, registry)
            .map_err(CliError::from_model)?;
    let log_id = response.output.log_id.clone();
    let code = exit_code_for_response(&response.response);
    let result = match output {
        OutputMode::Human => render_human_log(
            invocation,
            registry,
            &log_id,
            io.stdout,
            io.stderr,
            io.stdout_is_tty,
            code,
        ),
        OutputMode::Json | OutputMode::Jsonl => {
            match machine_output(registry, &log_id, output, io.stdout) {
                Ok(()) => Ok(LogExit { code }),
                Err(crate::LogMachineOutputError::Write(error)) => {
                    log_exit_after_write(code, Err(error))
                }
                Err(crate::LogMachineOutputError::Read(error)) => Err(CliError::from_model(error)),
                Err(crate::LogMachineOutputError::InvalidRecord(message)) => {
                    Err(CliError::from_model(gwz_core::model::ModelError::new(
                        gwz_core::model::ErrorCode::InternalError,
                        message,
                    )))
                }
            }
        }
        OutputMode::Porcelain => {
            let rendered = render_response(&CliResponse::envelope(response.response), output);
            let write_result = if rendered.is_empty() {
                Ok(())
            } else {
                let mut bytes = rendered.into_bytes();
                bytes.push(b'\n');
                io.stdout.write_all(&bytes)
            };
            log_exit_after_write(code, write_result)
        }
    };

    // The caller owns the finite spool. Release it after every successful or
    // failed read/render/write disposition, including a closed consumer.
    registry.release(&log_id);
    result
}

fn render_human_log<W: Write, E: Write>(
    invocation: &LogInvocation,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    log_id: &str,
    stdout: &mut W,
    stderr: &mut E,
    stdout_is_tty: bool,
    code: i32,
) -> Result<LogExit, CliError> {
    let color = log_color_enabled(invocation.color, stdout_is_tty);
    let mut cursor = None;
    loop {
        let batch = registry
            .read(
                log_id,
                &gwz_core::operation::CommitLogReadRequest {
                    cursor,
                    max_records: Some(128),
                },
            )
            .map_err(CliError::from_model)?;
        for record in batch.records {
            match (record.kind, record.entry, record.degradation) {
                (gwz_core::LogOutputRecordKind::Entry, Some(entry), None) => {
                    let mut rendered =
                        render_log_entry(&entry, invocation.full, color).into_bytes();
                    rendered.push(b'\n');
                    if invocation.full {
                        rendered.push(b'\n');
                    }
                    match stdout.write_all(&rendered) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                            return Ok(LogExit { code: 0 });
                        }
                        Err(error) => return Err(log_output_error("stdout", error)),
                    }
                }
                (gwz_core::LogOutputRecordKind::Degradation, None, Some(degradation)) => {
                    let mut rendered = render_log_degradation(&degradation, color).into_bytes();
                    rendered.push(b'\n');
                    stderr
                        .write_all(&rendered)
                        .map_err(|error| log_output_error("stderr", error))?;
                }
                _ => return Err(invalid_log_output_record()),
            }
        }
        if batch.state == gwz_core::operation::CommitLogReadState::Eof {
            return Ok(LogExit { code });
        }
        cursor = Some(batch.next_cursor);
    }
}

fn log_output_error(channel: &str, error: io::Error) -> CliError {
    CliError::from_model(gwz_core::model::ModelError::new(
        gwz_core::model::ErrorCode::IoError,
        format!("cannot write log {channel}: {error}"),
    ))
}

fn invalid_log_output_record() -> CliError {
    CliError::from_model(gwz_core::model::ModelError::new(
        gwz_core::model::ErrorCode::InternalError,
        "commit-log output record kind does not match its payload",
    ))
}

#[cfg(test)]
pub(crate) fn run_log_with_registry_and_machine_for_test<W, M>(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    stdout: &mut W,
    machine_output: M,
) -> Result<LogExit, CliError>
where
    W: Write,
    M: FnOnce(
        &gwz_core::operation::CommitLogOutputRegistry,
        &str,
        OutputMode,
        &mut W,
    ) -> Result<(), crate::LogMachineOutputError>,
{
    let mut stderr = io::sink();
    run_log_with_registry_io_using_machine(
        invocation,
        output,
        start,
        operation_id,
        registry,
        LogIo {
            stdout,
            stderr: &mut stderr,
            stdout_is_tty: false,
        },
        machine_output,
    )
}

pub(crate) fn log_exit_after_write(
    code: i32,
    write_result: io::Result<()>,
) -> Result<LogExit, CliError> {
    match write_result {
        Ok(()) => Ok(LogExit { code }),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(LogExit { code: 0 }),
        Err(error) => Err(CliError::from_model(gwz_core::model::ModelError::new(
            gwz_core::model::ErrorCode::IoError,
            format!("cannot write log output: {error}"),
        ))),
    }
}

pub(crate) fn write_log_machine_error<W: Write>(
    error: &CliError,
    stdout: &mut W,
) -> Result<LogExit, CliError> {
    let mut bytes = crate::render_error_json(error).into_bytes();
    bytes.push(b'\n');
    log_exit_after_write(exit_code_for_log_error(error), stdout.write_all(&bytes))
}

pub(crate) fn exit_code_for_log_error(error: &CliError) -> i32 {
    match error.code {
        Some(
            gwz_core::model::ErrorCode::IoError
            | gwz_core::model::ErrorCode::InternalError
            | gwz_core::model::ErrorCode::GitCommandFailed
            | gwz_core::model::ErrorCode::ExternalToolMissing
            | gwz_core::model::ErrorCode::RemoteRejected,
        )
        | None => 1,
        Some(_) => 2,
    }
}
