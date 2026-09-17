//! D6's run-time copy-cost estimate (amendment A1).
//!
//! Walk the source once for apparent size and file count, probe the
//! destination's parent for block sharing, and read a pessimistic share off
//! the table. The guard protects the copy, not what a session builds
//! afterwards.

use std::path::Path;
use std::time::Duration;

use super::env::{HookEnv, ShareProbe, table_per_file_micros, table_share};
use super::{ESTIMATE_MARGIN_BYTES, HookFailure};

/// What one walk of the source found.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SourceWalk {
    pub(crate) apparent_bytes: u64,
    pub(crate) files: u64,
}

/// One walk of the source tree: the walk the completeness check needs
/// anyway. Symlinks are counted, never followed, so a link out of the tree
/// cannot make the walk unbounded.
pub(crate) fn walk_source(root: &Path) -> SourceWalk {
    let mut walk = SourceWalk::default();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push(entry.path());
            } else {
                walk.files = walk.files.saturating_add(1);
                if let Ok(metadata) = entry.metadata() {
                    walk.apparent_bytes = walk.apparent_bytes.saturating_add(metadata.len());
                }
            }
        }
    }
    walk
}

/// The estimate D6 states: apparent size x (1 - share) + a fixed margin,
/// with a copy time of file count x the table's per-file cost.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CopyEstimate {
    pub(crate) walk: SourceWalk,
    pub(crate) probe: ShareProbe,
    pub(crate) share: f64,
    pub(crate) cost_bytes: u64,
    pub(crate) copy_time: Duration,
}

pub(crate) fn estimate(
    env: &dyn HookEnv,
    source: &Path,
    destination_parent: &Path,
) -> CopyEstimate {
    let walk = walk_source(source);
    let probe = env.probe_share(source, destination_parent);
    estimate_from(walk, probe)
}

pub(crate) fn estimate_from(walk: SourceWalk, probe: ShareProbe) -> CopyEstimate {
    let share = table_share(&probe);
    let unshared = (walk.apparent_bytes as f64 * (1.0 - share)).max(0.0);
    let cost_bytes = (unshared as u64).saturating_add(ESTIMATE_MARGIN_BYTES);
    let copy_time = Duration::from_micros(
        walk.files
            .saturating_mul(table_per_file_micros(&probe))
            .max(1),
    );
    CopyEstimate {
        walk,
        probe,
        share,
        cost_bytes,
        copy_time,
    }
}

/// D6's first guard. The free space read is the one that holds the
/// destination's *parent* directory, not the workspace's volume, which can
/// differ under a symlinked or mounted root.
pub(crate) fn check_free_space(
    env: &dyn HookEnv,
    estimate: &CopyEstimate,
    destination_parent: &Path,
    min_free_gb: Option<f64>,
) -> Result<(), HookFailure> {
    let Some(free) = env.free_bytes(destination_parent) else {
        // The platform would not say. A guard that cannot measure does not
        // invent a refusal; the copy's own out-of-space failure is the
        // backstop, and S1.3 records it.
        return Ok(());
    };
    let floor = min_free_gb.map(|gb| (gb * 1e9) as u64);
    let required = floor.map_or(estimate.cost_bytes, |value| value.max(estimate.cost_bytes));
    if free >= required {
        return Ok(());
    }
    Err(HookFailure::refused(
        format!(
            "{} free at {} is below the {} this copy needs (apparent {}, share {:.0}%)",
            gigabytes(free),
            destination_parent.display(),
            gigabytes(required),
            gigabytes(estimate.walk.apparent_bytes),
            estimate.share * 100.0
        ),
        "retire a lane first: see `gwz local list`, then the retirement procedure in \
         docs/ClaudeCode.md",
    ))
}

/// D6's second guard: the family already holds `--max-lanes` rows in state
/// `ready`.
pub(crate) fn check_lane_ceiling(ready_rows: u32, max_lanes: u32) -> Result<(), HookFailure> {
    if ready_rows < max_lanes {
        return Ok(());
    }
    Err(HookFailure::refused(
        format!("the family already holds {ready_rows} ready lanes, the ceiling is {max_lanes}"),
        "retire a lane first: see `gwz local list`, then the retirement procedure in \
         docs/ClaudeCode.md",
    ))
}

fn gigabytes(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1e9)
}
