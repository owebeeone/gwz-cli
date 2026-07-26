const RELEASE_SCRIPT: &str = include_str!("../scripts/release.py");

#[test]
fn release_script_checks_generated_cli_reference_by_default() {
    assert!(RELEASE_SCRIPT.contains("verify_cli_reference_docs(worktree)"));
    assert!(RELEASE_SCRIPT.contains("generate_cli_reference.py"));
    assert!(RELEASE_SCRIPT.contains("python scripts/generate_cli_reference.py --write"));
    assert!(RELEASE_SCRIPT.contains("--no-doc-check"));
    assert!(RELEASE_SCRIPT.contains("generated CLI reference is out of date"));
}

#[test]
fn release_script_checks_out_the_exact_core_tag_for_parity_fixtures() {
    assert!(RELEASE_SCRIPT.contains("def checkout_gwz_core"));
    assert!(RELEASE_SCRIPT.contains("core_checkout = temp_root / \"gwz-core\""));
    assert!(RELEASE_SCRIPT.contains("checkout_gwz_core(core_url, tag, core_checkout)"));
    assert!(RELEASE_SCRIPT.contains("\"--branch\", tag"));
}
