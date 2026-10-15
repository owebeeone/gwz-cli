//! Internal early bootstrap and trusted self-executable descriptor; no HTTP use.
use gwz_sspi::{Error, ErrorKind, WorkerBootstrap, WorkerExecutable, packaging::build_fingerprint};
use std::ffi::OsString;

/// The artifact-set identifier compiled into this executable. `build.rs` always supplies it for a Cargo build:
/// the packaging table's entry for the target when `GWZ_SSPI_PACKAGING_TARGETS` is given, else a fresh
/// per-build value (D23, `build_support/self_provision.rs`). It is absent only in a build `build.rs` never ran
/// for (the Bazel library), which stays unprovisioned and refuses. The host (`executable`) and the worker
/// (`early`) both read it here, and both are this one executable.
const COMPILED_BUILD: Option<&str> = option_env!("GWZ_SSPI_BUILD_FINGERPRINT");

/// The 32 bytes the worker's Hello must carry for the host to accept it: what `compiled` decodes to. The host's
/// descriptor and the worker's `worker_entry` are both given exactly these bytes.
fn expected_build(compiled: Option<&str>) -> Result<[u8; 32], Error> {
    build_fingerprint(compiled)
}

fn dispatch(
    args: impl Iterator<Item = OsString>,
    compiled: Option<&str>,
    entry: impl FnOnce(WorkerBootstrap, [u8; 32]) -> Result<(), Error>,
) -> Option<i32> {
    let bootstrap = match WorkerBootstrap::from_args(args) {
        Ok(None) => return None,
        Ok(Some(value)) => value,
        Err(_) => return Some(2),
    };
    let result = expected_build(compiled).and_then(|build| entry(bootstrap, build));
    Some(if result.is_ok() { 0 } else { 2 })
}
/// The worker-host dispatch that runs before anything else. `argv` is the
/// process's arguments, program name first, as `run()` reads them.
pub(crate) fn early(argv: impl Iterator<Item = OsString>) -> Option<i32> {
    dispatch(argv.skip(1), COMPILED_BUILD, gwz_sspi::worker_entry)
}
/// Capture this installed executable and its trusted compiled artifact-set
/// identifier. This creates no process or supervisor and is not yet used by
/// HTTP. A build `build.rs` did not run for has no identifier and refuses
/// without runtime/PATH fallback.
pub fn executable() -> Result<WorkerExecutable, ErrorKind> {
    let path = std::env::current_exe().map_err(|_| ErrorKind::WorkerUnavailable)?;
    descriptor_at(path, COMPILED_BUILD)
}
/// The descriptor of the worker at `path`, the loader's name for this executable, for the build whose compiled
/// fingerprint is `compiled`. The fingerprint is judged first, so an unprovisioned build is `WorkerUnavailable`
/// and a malformed one `WorkerMismatch`, whatever `path` is.
fn descriptor_at(
    path: std::path::PathBuf,
    compiled: Option<&str>,
) -> Result<WorkerExecutable, ErrorKind> {
    let build = expected_build(compiled).map_err(|error| error.kind())?;
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
        // No compiled fingerprint: a build `build.rs` never ran for (Bazel); a Cargo build provisions itself (D23).
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
    fn a_build_without_packaging_metadata_provisions_its_own_worker() {
        // D23: a plain `cargo build` or `cargo install gwz` has no GWZ_SSPI_PACKAGING_TARGETS, and its worker is
        // its own image, so `build.rs` compiles in a per-build fingerprint and the descriptor is no longer refused.
        // The unprovisioned refusal stays for a build that no `build.rs` ran for (the Bazel library), and is tested
        // above through `descriptor_at`.
        let descriptor = executable().expect("a Cargo build provisions its own worker");
        assert_eq!(descriptor.path(), std::env::current_exe().unwrap());
    }

    /// What a worker image compiled with `compiled` puts in its Hello: the bytes `worker_entry` is handed.
    fn hello_of(compiled: Option<&str>) -> Option<Option<[u8; 32]>> {
        let mut said = None;
        let code = dispatch(
            ["--gwz-sspi-worker", "3", "4"]
                .map(OsString::from)
                .into_iter(),
            compiled,
            |_, build| {
                said = Some(build);
                Ok(())
            },
        );
        match code {
            Some(0) => Some(said),
            _ => None,
        }
    }

    #[test]
    fn the_worker_of_a_cargo_build_says_the_build_its_host_expects() {
        // The host expects `expected_build(compiled)` of the worker's Hello (`descriptor_at` hands exactly
        // that to `WorkerExecutable`); the worker of the same build says the bytes it was handed.
        let compiled = option_env!("GWZ_SSPI_BUILD_FINGERPRINT");
        assert_eq!(
            hello_of(compiled),
            Some(Some(expected_build(compiled).unwrap()))
        );
    }

    #[test]
    fn a_worker_image_of_another_build_does_not_say_the_build_the_host_expects() {
        // A foreign image (a different Cargo build's gwz, a release's) was compiled with another value, so
        // its Hello differs from the expectation and gwz-sspi refuses it before Begin. Absence of metadata
        // in the image refuses outright, with no Hello at all.
        let own = "ab".repeat(32);
        let foreign = "cd".repeat(32);
        let expected = expected_build(Some(&own)).unwrap();
        let said = hello_of(Some(&foreign)).unwrap().unwrap();
        assert_ne!(said, expected);
        assert_eq!(
            hello_of(Some(&own)).unwrap().unwrap(),
            expected,
            "the same value is accepted"
        );
        assert_eq!(hello_of(None), None);
    }

    #[test]
    fn early_worker_shape_never_falls_through_to_ordinary_parser() {
        for compiled in [None, Some("malformed")] {
            assert_eq!(
                dispatch(
                    ["--gwz-sspi-worker", "1", "2"]
                        .map(OsString::from)
                        .into_iter(),
                    compiled,
                    |_, _| unreachable!(
                        "a malformed or absent fingerprint never reaches worker entry"
                    )
                ),
                Some(2)
            );
        }
        for args in [
            vec!["--gwz-sspi-worker", "--build-info"],
            vec!["--gwz-sspi-worker", "sentinel"],
        ] {
            assert_eq!(
                dispatch(
                    args.into_iter().map(OsString::from),
                    Some(&"42".repeat(32)),
                    |_, _| unreachable!(
                        "a malformed internal invocation never reaches worker entry"
                    )
                ),
                Some(2)
            );
        }
        assert_eq!(
            dispatch(
                ["--build-info"].map(OsString::from).into_iter(),
                None,
                |_, _| unreachable!("an ordinary invocation never reaches worker entry")
            ),
            None
        );
    }
}
