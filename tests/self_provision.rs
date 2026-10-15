//! D23: the fingerprint `build.rs` compiles into a CLI built without packaging metadata.
//!
//! The generator is `build_support/self_provision.rs`, included here exactly as `build.rs` includes it, so these
//! rows run the code the build runs.
#[path = "../build_support/self_provision.rs"]
mod self_provision;

use std::collections::HashSet;

#[test]
fn a_self_provisioned_fingerprint_is_the_form_the_worker_entry_accepts() {
    let value = self_provision::fresh(&["x86_64-pc-windows-msvc", "target/out"]);
    assert_eq!(value.len(), 64, "{value}");
    assert!(
        value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "{value}"
    );
    assert!(gwz_sspi::packaging::build_fingerprint(Some(&value)).is_ok());
}

#[test]
fn every_build_gets_its_own_fingerprint_so_another_builds_worker_is_foreign() {
    // The same context, one after another and from several threads: no two values meet, so a worker image from
    // another Cargo build (compiled with its own value) never says the build this one expects.
    let context = ["x86_64-pc-windows-msvc", "target/out"];
    let mut seen: HashSet<String> = (0..1000).map(|_| self_provision::fresh(&context)).collect();
    assert_eq!(seen.len(), 1000);
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(move || {
                (0..200)
                    .map(|_| self_provision::fresh(&context))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for thread in threads {
        for value in thread.join().unwrap() {
            assert!(seen.insert(value), "a fingerprint was generated twice");
        }
    }
    assert_eq!(seen.len(), 1000 + 8 * 200);
}
