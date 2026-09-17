//! Everything the hook learns from the machine rather than from its input:
//! free space, the reflink probe, the busy-build check, `HOME`, the clock
//! and the stderr channel (D6, D10, D11).
//!
//! The platform-bound helpers live in the two enclosing platform modules at
//! the bottom of this file, per the standing rule on conditional
//! compilation: no `#[cfg(...)]` sits on an import or any other unbraced
//! declaration, and the unconditional imports stay outside them.
//!
//! Tests do not force these through environment variables; they implement
//! this trait, which is the same forcing with none of a process-global
//! variable's cross-test bleed (D1's note that such variables are test
//! scaffolding, not user configuration).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// What the destination's parent filesystem does with a block-sharing copy
/// (D6). `fs_type` is lowercase and best effort; an unknown filesystem is
/// read pessimistically.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ShareProbe {
    /// The 64 MB probe file cloned, and the `df` delta said the clone shared
    /// its blocks.
    pub(crate) cloned: bool,
    /// The destination's parent and the source sit on one filesystem.
    /// Sharing never crosses a filesystem boundary (section 1).
    pub(crate) same_filesystem: bool,
    pub(crate) fs_type: Option<String>,
}

pub(crate) trait HookEnv {
    fn now(&self) -> SystemTime;
    fn sleep(&self, duration: Duration);
    /// Free bytes on the filesystem that holds `dir`. `None` when the
    /// platform could not say.
    fn free_bytes(&self, dir: &Path) -> Option<u64>;
    fn probe_share(&self, source: &Path, destination_parent: &Path) -> ShareProbe;
    /// D10: a `cargo` process holding the source's target directory open.
    fn build_busy(&self, root: &Path) -> bool;
    fn home_dir(&self) -> Option<PathBuf>;
    /// One line of hook stderr. Never stdout: stdout carries the path alone.
    fn warn(&self, line: &str);
}

/// The production environment.
pub(crate) struct SystemEnv;

impl HookEnv for SystemEnv {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn free_bytes(&self, dir: &Path) -> Option<u64> {
        platform::free_bytes(dir)
    }

    fn probe_share(&self, source: &Path, destination_parent: &Path) -> ShareProbe {
        platform::probe_share(source, destination_parent)
    }

    fn build_busy(&self, root: &Path) -> bool {
        build_busy(root)
    }

    fn home_dir(&self) -> Option<PathBuf> {
        platform::home_dir()
    }

    fn warn(&self, line: &str) {
        eprintln!("{line}");
    }
}

/// D10's busy-build check, and the one machine fact that needs no platform
/// branch: Cargo holds an exclusive advisory lock on
/// `<target>/<profile>/.cargo-lock` for the whole of a build, and
/// `File::try_lock` reports that on every supported host.
fn build_busy(root: &Path) -> bool {
    for profile in ["debug", "release"] {
        let lock = root.join("target").join(profile).join(".cargo-lock");
        let Ok(file) = std::fs::File::open(&lock) else {
            continue;
        };
        match file.try_lock() {
            Ok(()) => {
                let _ = file.unlock();
            }
            // Held by a build, or the platform would not say: both are
            // reported as busy, because the warning is advisory (D10).
            Err(_) => {
                return true;
            }
        }
    }
    false
}

/// The bytes the reflink probe writes before cloning them (D6).
const PROBE_BYTES: u64 = 64 << 20;

/// The clone is judged to have shared blocks when the free-space delta it
/// costs is below this. A byte-for-byte copy costs `PROBE_BYTES`.
const PROBE_SHARE_CEILING: u64 = 8 << 20;

/// A filesystem name the share table knows (D6). Every other cloning
/// filesystem is read at the lower share, and every filesystem that fails to
/// clone at zero.
pub(crate) fn table_share(probe: &ShareProbe) -> f64 {
    if !probe.cloned || !probe.same_filesystem {
        return 0.0;
    }
    match probe.fs_type.as_deref() {
        // Never credited with sharing, whatever a probe seems to say.
        Some("ext4") | Some("ext2/ext3") | Some("ntfs") | Some("ntfs3") => 0.0,
        Some("apfs") | Some("xfs") | Some("btrfs") => 0.94,
        Some("refs") => 0.70,
        _ => 0.70,
    }
}

/// The per-file cost the copy-time estimate uses, in microseconds (D6). A
/// clone's wall time is bound by file count, not bytes, on every sharing
/// filesystem (section 1). Placeholders until S3.2 measures them.
pub(crate) fn table_per_file_micros(probe: &ShareProbe) -> u64 {
    if probe.cloned && probe.same_filesystem {
        400
    } else {
        4_000
    }
}

// The platform-bound helpers. Each is a whole module with an explicit
// boundary, so a condition can never silently transfer to a neighbouring
// declaration.

#[cfg(unix)]
mod platform {
    use super::{PROBE_BYTES, PROBE_SHARE_CEILING, ShareProbe};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub(super) fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    /// `df -Pk` is POSIX: a header, then one line whose fourth field is the
    /// available 1024-byte blocks and whose first is the device.
    fn df(dir: &Path) -> Option<(String, u64)> {
        let output = Command::new("df").arg("-Pk").arg(dir).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        let line = text.lines().nth(1)?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            return None;
        }
        let available: u64 = fields[3].parse().ok()?;
        Some((fields[0].to_owned(), available.saturating_mul(1024)))
    }

    pub(super) fn free_bytes(dir: &Path) -> Option<u64> {
        df(dir).map(|(_, free)| free)
    }

    fn filesystem_type(dir: &Path) -> Option<String> {
        // GNU coreutils answers directly; the BSD `stat` does not, so macOS
        // reads the mount table instead.
        if let Ok(output) = Command::new("stat")
            .arg("-f")
            .arg("-c")
            .arg("%T")
            .arg(dir)
            .output()
            && output.status.success()
        {
            let value = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            if !value.is_empty() {
                return Some(value);
            }
        }
        let device = df(dir)?.0;
        let output = Command::new("mount").output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        for line in text.lines() {
            if line.starts_with(&device)
                && let Some(start) = line.find('(')
            {
                let rest = &line[start + 1..];
                let name = rest.split(',').next()?.trim().to_lowercase();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    fn reflink(source: &Path, destination: &Path) -> bool {
        let mut command = Command::new("cp");
        if cfg!(target_vendor = "apple") {
            command.arg("-c");
        } else {
            command.arg("--reflink=always");
        }
        command
            .arg(source)
            .arg(destination)
            .status()
            .is_ok_and(|status| status.success())
    }

    /// D6's probe: write one 64 MB file at the destination's parent, clone
    /// it with the platform's reflink call, and read the `df` delta.
    pub(super) fn probe_share(source: &Path, destination_parent: &Path) -> ShareProbe {
        let same_filesystem = match (df(source), df(destination_parent)) {
            (Some((left, _)), Some((right, _))) => left == right,
            _ => false,
        };
        let fs_type = filesystem_type(destination_parent);
        let mut probe = ShareProbe {
            cloned: false,
            same_filesystem,
            fs_type,
        };
        if !same_filesystem {
            return probe;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        let seed =
            destination_parent.join(format!(".gwz-clone-probe-{}-{stamp}", std::process::id()));
        let clone = destination_parent.join(format!(
            ".gwz-clone-probe-{}-{stamp}.clone",
            std::process::id()
        ));
        let written = (|| -> std::io::Result<()> {
            let mut file = std::fs::File::create(&seed)?;
            let block = vec![0_u8; 1 << 20];
            for _ in 0..(PROBE_BYTES >> 20) {
                file.write_all(&block)?;
            }
            file.sync_all()
        })();
        if written.is_ok()
            && let Some((_, before)) = df(destination_parent)
            && reflink(&seed, &clone)
            && let Some((_, after)) = df(destination_parent)
        {
            probe.cloned = before.saturating_sub(after) < PROBE_SHARE_CEILING;
        }
        let _ = std::fs::remove_file(&clone);
        let _ = std::fs::remove_file(&seed);
        probe
    }
}

#[cfg(not(unix))]
mod platform {
    use super::ShareProbe;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub(super) fn home_dir() -> Option<PathBuf> {
        std::env::var_os("USERPROFILE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    /// `fsutil volume diskfree` prints the free bytes of the volume holding
    /// the path. S5.1 verifies this on the Windows host (D11).
    pub(super) fn free_bytes(dir: &Path) -> Option<u64> {
        let output = Command::new("fsutil")
            .arg("volume")
            .arg("diskfree")
            .arg(dir)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        for line in text.lines() {
            if let Some((_, value)) = line.split_once(':') {
                let digits: String = value.chars().filter(char::is_ascii_digit).collect();
                if let Ok(parsed) = digits.parse::<u64>() {
                    return Some(parsed);
                }
            }
        }
        None
    }

    /// ReFS block cloning is reached through `FSCTL_DUPLICATE_EXTENTS_TO_FILE`,
    /// which this driver cannot call without a raw binding; until S5.1 the
    /// Windows probe reports no sharing, which is the pessimistic side of
    /// D6's table.
    pub(super) fn probe_share(_source: &Path, _destination_parent: &Path) -> ShareProbe {
        ShareProbe::default()
    }
}
