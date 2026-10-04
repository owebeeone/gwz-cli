//! Internal early bootstrap and trusted self-executable descriptor; no HTTP use.
use gwz_sspi::{ErrorKind, WorkerBootstrap, WorkerExecutable, packaging::build_fingerprint};
use std::ffi::OsString;
fn dispatch(args: impl Iterator<Item = OsString>, compiled: Option<&str>) -> Option<i32> {
    let bootstrap = match WorkerBootstrap::from_args(args) {
        Ok(None) => return None,
        Ok(Some(value)) => value,
        Err(_) => return Some(2),
    };
    let result =
        build_fingerprint(compiled).and_then(|build| gwz_sspi::worker_entry(bootstrap, build));
    Some(if result.is_ok() { 0 } else { 2 })
}
/// The worker-host dispatch that runs before anything else. `argv` is the
/// process's arguments, program name first, as `run()` reads them.
pub(crate) fn early(argv: impl Iterator<Item = OsString>) -> Option<i32> {
    dispatch(argv.skip(1), option_env!("GWZ_SSPI_BUILD_FINGERPRINT"))
}
/// Capture this installed executable and its trusted compiled artifact-set
/// identifier. This creates no process or supervisor and is not yet used by
/// HTTP; absent packaging metadata refuses without runtime/PATH fallback.
pub fn executable() -> Result<WorkerExecutable, ErrorKind> {
    let build = build_fingerprint(option_env!("GWZ_SSPI_BUILD_FINGERPRINT"))
        .map_err(|error| error.kind())?;
    let path = std::env::current_exe().map_err(|_| ErrorKind::WorkerUnavailable)?;
    WorkerExecutable::new(path, build).map_err(|error| error.kind())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn early_worker_shape_never_falls_through_to_ordinary_parser() {
        for compiled in [None, Some("malformed")] {
            assert_eq!(
                dispatch(
                    ["--gwz-sspi-worker", "1", "2"]
                        .map(OsString::from)
                        .into_iter(),
                    compiled
                ),
                Some(2)
            );
        }
        for args in [
            vec!["--gwz-sspi-worker", "--build-info"],
            vec!["--gwz-sspi-worker", "sentinel"],
        ] {
            assert_eq!(
                dispatch(args.into_iter().map(OsString::from), Some(&"42".repeat(32))),
                Some(2)
            );
        }
        assert_eq!(
            dispatch(["--build-info"].map(OsString::from).into_iter(), None),
            None
        );
    }
}
