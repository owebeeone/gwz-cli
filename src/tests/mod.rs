pub(crate) use super::*;
pub(crate) use clap::Parser;

mod g00;
mod g01;
mod g02;
mod g03;
mod g04;
mod g05;
mod g06;
mod g07;
mod g08;
mod g09;
mod g10;
mod g11;
mod g12;
mod g13;
mod g14;
mod g15;
mod m2c;
pub(crate) mod temp_dir;
mod temp_dir_tests;
mod transport_scope;

/// The working directory the parsing tests pass: absolute on every platform and
/// never touched. A literal `/cwd` has no drive on Windows, where it is not
/// absolute and the invocation refuses it.
pub(crate) fn test_cwd() -> std::path::PathBuf {
    std::env::temp_dir().join("cwd")
}
