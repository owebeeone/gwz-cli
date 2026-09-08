#[path = "build_support/provenance.rs"]
mod provenance;
fn main() {
    provenance::emit();
}
