#[path = "build_support/provenance.rs"]
mod provenance;
fn main() {
    println!("cargo:rustc-check-cfg=cfg(gwz_transport_candidate)");
    println!("cargo:rustc-check-cfg=cfg(gwz_session_candidate)");
    provenance::emit();
}
