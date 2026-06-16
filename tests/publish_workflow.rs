const PUBLISH_WORKFLOW: &str = include_str!("../.github/workflows/publish.yml");

#[test]
fn publish_workflow_tests_linux_and_windows() {
    assert!(PUBLISH_WORKFLOW.contains("ubuntu-latest"));
    assert!(PUBLISH_WORKFLOW.contains("windows-latest"));
    assert!(PUBLISH_WORKFLOW.contains("cargo test"));
    assert!(PUBLISH_WORKFLOW.contains("cargo clippy --all-targets -- -D warnings"));
}

#[test]
fn publish_workflow_checks_out_core_as_sibling_dependency() {
    assert!(PUBLISH_WORKFLOW.contains("repository: owebeeone/gwz-core"));
    assert!(PUBLISH_WORKFLOW.contains("path: gwz-core"));
    assert!(PUBLISH_WORKFLOW.contains("path: gwz-cli"));
}

#[test]
fn publish_workflow_installs_release_taut_proto_for_core_protocol_tests() {
    assert!(PUBLISH_WORKFLOW.contains("actions/setup-python"));
    assert!(PUBLISH_WORKFLOW.contains("TAUT_PYTHON: python"));
    assert!(PUBLISH_WORKFLOW.contains("python -m pip install --upgrade pip taut-proto"));
}

#[test]
fn publish_workflow_builds_installable_release_assets() {
    assert!(PUBLISH_WORKFLOW.contains("x86_64-unknown-linux-gnu"));
    assert!(PUBLISH_WORKFLOW.contains("x86_64-pc-windows-msvc"));
    assert!(PUBLISH_WORKFLOW.contains("cargo build --release --locked"));
    assert!(PUBLISH_WORKFLOW.contains("gwz-${{ steps.version.outputs.version }}"));
    assert!(PUBLISH_WORKFLOW.contains("gh release create"));
    assert!(PUBLISH_WORKFLOW.contains("SHA256SUMS"));
}
