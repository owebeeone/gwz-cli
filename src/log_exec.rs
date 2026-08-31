//! No-pager lifecycle driver for the finite core commit-log output spool.
//!
//! S3.1 deliberately emits only the existing generic response status. The
//! caller-owned registry and EPIPE-aware writer are the stable seams that the
//! S3.2/S3.3 record renderers will consume.

use std::io::{self, Write};
use std::path::Path;

use crate::{
    CliError, CliResponse, LogInvocation, OutputMode, exit_code_for_response, render_response,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LogExit {
    pub(crate) code: i32,
}

pub(crate) fn run_log(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
) -> Result<LogExit, CliError> {
    let registry = gwz_core::operation::CommitLogOutputRegistry::new();
    let mut stdout = io::stdout().lock();
    run_log_with_registry(
        invocation,
        output,
        start,
        operation_id,
        &registry,
        &mut stdout,
    )
}

pub(crate) fn run_log_with_registry<W: Write>(
    invocation: &LogInvocation,
    output: OutputMode,
    start: &Path,
    operation_id: String,
    registry: &gwz_core::operation::CommitLogOutputRegistry,
    stdout: &mut W,
) -> Result<LogExit, CliError> {
    let response =
        gwz_core::operation::handle_log(start, invocation.request.clone(), operation_id, registry)
            .map_err(CliError::from_model)?;
    let log_id = response.output.log_id.clone();
    let code = exit_code_for_response(&response.response);
    let rendered = render_response(&CliResponse::envelope(response.response), output);
    let write_result = if rendered.is_empty() {
        Ok(())
    } else {
        let mut bytes = rendered.into_bytes();
        bytes.push(b'\n');
        stdout.write_all(&bytes)
    };

    // S3.1 has no record renderer, so the finite spool is intentionally unread;
    // release it on every output disposition, including a closed consumer.
    registry.release(&log_id);
    log_exit_after_write(code, write_result)
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
