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
    let path = std::env::current_exe().map_err(|_| ErrorKind::WorkerUnavailable)?;
    descriptor_at(path, option_env!("GWZ_SSPI_BUILD_FINGERPRINT"))
}
/// The descriptor of the worker at `path`, the loader's name for this executable, for the build whose compiled
/// fingerprint is `compiled`. The fingerprint is judged first, so an unprovisioned build is `WorkerUnavailable`
/// and a malformed one `WorkerMismatch`, whatever `path` is.
fn descriptor_at(
    path: std::path::PathBuf,
    compiled: Option<&str>,
) -> Result<WorkerExecutable, ErrorKind> {
    let build = build_fingerprint(compiled).map_err(|error| error.kind())?;
    WorkerExecutable::new(path, build).map_err(|error| error.kind())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_installed_descriptor_is_the_executable_wherever_it_is_installed() {
        // WH3 installed paths (step 4.10): the worker of the CLI is its own image, named by the loader, so an
        // install under a directory with spaces or non-ASCII characters is the same descriptor as any other.
        let build = "ab".repeat(32);
        let root = if cfg!(windows) { "C:\\" } else { "/" };
        for directory in [
            "Program Files\\gwz\\bin",
            "\u{fc}ml\u{e4}ut \u{65e5}\u{672c}\u{8a9e}\\bin",
            "with spaces\\and \u{43f}\u{430}\u{43f}\u{43a}\u{430}",
        ] {
            let path: PathBuf = [root, directory, "gwz"].iter().collect();
            let descriptor = descriptor_at(path.clone(), Some(&build)).unwrap();
            assert_eq!(descriptor.path(), path.as_path());
        }
    }

    #[test]
    fn an_unprovisioned_or_mismatched_build_has_no_descriptor_and_no_fallback() {
        let path = std::env::current_exe().unwrap();
        // No compiled fingerprint: a plain `cargo build`, not a provisioned release.
        assert_eq!(
            descriptor_at(path.clone(), None).err(),
            Some(ErrorKind::WorkerUnavailable)
        );
        // A malformed one is a mismatch, and is never repaired by looking elsewhere.
        for bad in ["", "42", &"zz".repeat(32)] {
            assert_eq!(
                descriptor_at(path.clone(), Some(bad)).err(),
                Some(ErrorKind::WorkerMismatch),
                "{bad:?}"
            );
        }
        // A relative image path is no path at all: nothing is resolved against the working directory.
        assert_eq!(
            descriptor_at(PathBuf::from("gwz"), Some(&"ab".repeat(32))).err(),
            Some(ErrorKind::InvalidRequest)
        );
    }
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
