mod dispatch;
mod invocation;
mod open_merge_gate;
mod parser;
mod render_exit;

pub(crate) use dispatch::execute_invocation;
#[cfg(test)]
pub(crate) use invocation::parse_args_with_request_id;
pub(crate) use invocation::{invocation_from_cli, new_request_id};
pub(crate) use parser::*;
pub(crate) use render_exit::{exit_code_for_response, render_response};
