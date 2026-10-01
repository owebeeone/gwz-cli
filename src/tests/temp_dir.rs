//! Temporary directories that two tests never share.
//!
//! A name drawn from the process id and the clock is not unique: macOS reads
//! the clock to the microsecond, so tests that start together draw one name,
//! and a create that accepts an existing directory then hands each of them
//! the same one. Here `create_dir` refuses a name that exists, so a directory
//! returned is one this call created, and a taken name is retried with a
//! suffix. The clock only spreads the names out; no counter or other state is
//! kept between calls. A name that is free gets no suffix, so it is no longer
//! than the names fixtures drew before, which matters under Windows' MAX_PATH.
//!
//! gwz-core's tests take theirs from its `src/test_support/temp_dir.rs`, the
//! same helper; a test-only module of one crate cannot serve another.
//!
//! This file names nothing outside std and holds no tests, so the integration
//! tests under `tests/` include it by `#[path]`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Names one call may try before giving up. A taken name needs another
/// caller with the same stem in the same clock tick, so a few are plenty.
const NAME_ATTEMPTS: u32 = 1000;

/// A directory under `parent` that this call created, named
/// `{stem}-{pid}-{nanos}`, or that with `-{n}` added when the name is taken.
/// Removing it is the caller's job.
fn unique_dir(parent: &Path, stem: &str) -> io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    create_fresh(parent, &format!("{stem}-{}-{nanos}", std::process::id()))
}

/// The first of `{base}`, `{base}-1`, `{base}-2`, ... under `parent` that this
/// call creates; a name that already exists is skipped and left as it is.
pub(super) fn create_fresh(parent: &Path, base: &str) -> io::Result<PathBuf> {
    for attempt in 0..NAME_ATTEMPTS {
        let path = if attempt == 0 {
            parent.join(base)
        } else {
            parent.join(format!("{base}-{attempt}"))
        };
        match fs::create_dir(&path) {
            Ok(()) => {
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "{base} and {base}-1 to {base}-{} under {} are all taken",
            NAME_ATTEMPTS - 1,
            parent.display()
        ),
    ))
}

/// A directory under the system temp dir that this test created, removed
/// on drop.
pub(crate) struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// `label` names the directory `gwz-cli-{label}-...`; tests with the
    /// same label still get directories of their own.
    pub(crate) fn new(label: &str) -> Self {
        let path = unique_dir(&std::env::temp_dir(), &format!("gwz-cli-{label}"))
            .unwrap_or_else(|error| panic!("create a temp dir for {label}: {error}"));
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
