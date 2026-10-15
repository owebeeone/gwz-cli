#[path = "build_support/provenance.rs"]
mod provenance;
#[path = "build_support/self_provision.rs"]
mod self_provision;
fn main() {
    println!("cargo:rustc-check-cfg=cfg(gwz_transport_candidate)");
    println!("cargo:rustc-check-cfg=cfg(gwz_session_candidate)");
    provenance::emit();
    println!("cargo:rerun-if-env-changed=GWZ_SSPI_PACKAGING_TARGETS");
    if let Ok(table) = std::env::var("GWZ_SSPI_PACKAGING_TARGETS") {
        let target = std::env::var("TARGET").unwrap();
        let value = table
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(key, _)| *key == target)
            .expect("packaging metadata missing actual target")
            .1;
        assert!(
            value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()),
            "malformed packaging metadata"
        );
        println!("cargo:rustc-env=GWZ_SSPI_BUILD_FINGERPRINT={value}");
    } else {
        // No packaging metadata (a plain `cargo build` or `cargo install gwz`): this CLI provisions its own
        // worker with a fresh per-build fingerprint (D23; build_support/self_provision.rs).
        let target = std::env::var("TARGET").unwrap_or_default();
        let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
        let value = self_provision::fresh(&[&target, &out_dir]);
        println!("cargo:rustc-env=GWZ_SSPI_BUILD_FINGERPRINT={value}");
    }
}
