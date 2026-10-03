#[path = "build_support/provenance.rs"]
mod provenance;
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
    }
}
