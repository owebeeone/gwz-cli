mod dispatch;
mod invocation;
mod open_merge_gate;
mod parser;
mod render_exit;
mod retry;
mod transport;
mod transport_help;

cfg_if::cfg_if! { if #[cfg(any(test, not(all(unix, gwz_transport_candidate))))] {
    pub(crate) use dispatch::execute_invocation;
} }
#[cfg(test)]
pub(crate) use invocation::parse_args_with_request_id;
pub(crate) use invocation::{invocation_from_cli, new_request_id};
pub(crate) use parser::*;
pub(crate) use render_exit::{
    exit_code_for_response, render_response, render_response_with_transport,
};
pub(crate) use retry::{RetryArgs, retry_sentence};
pub(crate) use transport::TransportArgs;
cfg_if::cfg_if! { if #[cfg(all(unix, gwz_transport_candidate))] {
    pub(crate) use dispatch::execute_invocation_selected;
    pub(crate) use transport::{prepare_transport, render_execution_error};
} }
