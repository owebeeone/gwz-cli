// A CLI built without packaging metadata provisions its own SSPI worker (operator decision D23).
//
// The CLI's worker is its own image (`std::env::current_exe`), so the host's expected build bytes and the
// worker's Hello can both come from one compile-time value, and "same build" only has to mean "the binary
// that was compiled with this value". A fresh random value per build gives that, and it is also why a worker
// image from any other build is refused: it was compiled with a different value.
//
// The value is an identity, not a secret: it is readable in the binary, exactly as a packaged artifact-set
// identifier is readable in a release. It is judged only by the exact-match Hello check in gwz-sspi.
//
// Two builds of the same source differ in this one string, so a Cargo-built `gwz` is not bit-for-bit
// reproducible. A build that needs reproducibility supplies `GWZ_SSPI_PACKAGING_TARGETS`, whose identifiers are
// a deterministic function of the build's inputs.
use sha2::{Digest, Sha256};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// A new 64-digit lowercase hexadecimal fingerprint, different on every call. `context` (the target, the
/// output directory) is mixed in as well, but the uniqueness does not depend on it.
///
/// The randomness is std's `RandomState`, which the standard library seeds from the operating system, hashed
/// with the clock, the process id and the context; no crate is added to the build.
pub fn fresh(context: &[&str]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"gwz-cli self-provisioned sspi worker build, v1");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    for round in 0u8..4 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u8(round);
        hasher.write_u32(std::process::id());
        hasher.write_u128(nanos);
        hash.update(hasher.finish().to_be_bytes());
    }
    hash.update(nanos.to_be_bytes());
    hash.update(std::process::id().to_be_bytes());
    for part in context {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    format!("{:x}", hash.finalize())
}
