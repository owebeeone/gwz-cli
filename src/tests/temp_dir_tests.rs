//! The temp-dir helper's own promise: every directory it returns is one that
//! call created, never one another caller already holds.
//!
//! The fixtures here named their directories from the process id and a clock
//! that macOS reads to the microsecond, and `create_dir_all` accepted a
//! directory that already existed, so two parallel tests could share one, as
//! two of gwz-core's diff tests did on 2026-10-02.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Barrier;

use super::temp_dir::{TempDir, create_fresh};

const THREADS: usize = 8;
const PER_THREAD: usize = 200;

/// Parallel tests with one label, modelled as threads that start together and
/// each create directories in a tight loop. Every directory stays alive until
/// the end, as a running test's would, so a shared one cannot hide.
#[test]
fn temp_dirs_with_one_label_are_distinct_and_created_fresh() {
    let start = Barrier::new(THREADS);
    let held: Vec<(Vec<TempDir>, Vec<PathBuf>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|thread| {
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    let mut dirs = Vec::with_capacity(PER_THREAD);
                    let mut not_fresh = Vec::new();
                    for index in 0..PER_THREAD {
                        let dir = TempDir::new("temp-dir-unique");
                        // Fresh: nothing another TempDir wrote is visible here.
                        if fs::read_dir(dir.path()).unwrap().next().is_some() {
                            not_fresh.push(dir.path().to_path_buf());
                        }
                        fs::write(dir.path().join(format!("owner-{thread}-{index}")), "").unwrap();
                        dirs.push(dir);
                    }
                    (dirs, not_fresh)
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect()
    });

    let not_fresh: Vec<&PathBuf> = held.iter().flat_map(|(_, paths)| paths).collect();
    assert!(
        not_fresh.is_empty(),
        "{} directories already held another TempDir's files, first {:?}",
        not_fresh.len(),
        not_fresh.first()
    );
    let mut seen = HashSet::new();
    let shared: Vec<&Path> = held
        .iter()
        .flat_map(|(dirs, _)| dirs)
        .map(TempDir::path)
        .filter(|path| !seen.insert(*path))
        .collect();
    assert!(
        shared.is_empty(),
        "{} of {} TempDirs reused a path, first {:?}",
        shared.len(),
        THREADS * PER_THREAD,
        shared.first()
    );
}

/// The collision above without the race: two calls that draw one name, as two
/// tests in one clock tick do. The second gets a directory of its own and
/// leaves the first one's as it was.
#[test]
fn a_name_already_taken_is_skipped_and_left_alone() {
    let parent = TempDir::new("taken-name");
    let first = create_fresh(parent.path(), "same-tick").unwrap();
    fs::write(first.join("owner"), "first").unwrap();

    let second = create_fresh(parent.path(), "same-tick").unwrap();

    assert_ne!(first, second);
    assert_eq!(fs::read_dir(&second).unwrap().count(), 0);
    assert_eq!(fs::read_to_string(first.join("owner")).unwrap(), "first");
}
