const RELEASE_SCRIPT: &str = include_str!("../scripts/release.py");
const RELEASE_SCRIPT_TESTS: &str = include_str!("../scripts/test_release.py");
const RELEASE_GUIDE: &str = include_str!("../RELEASE.md");

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
    assert!(RELEASE_SCRIPT.contains("checkout_gwz_core(core_url, core_tag, core_checkout)"));
    assert!(RELEASE_SCRIPT.contains("\"--branch\", tag"));
}

#[test]
fn release_script_pins_gwz_core_exactly_from_crates_io() {
    assert!(RELEASE_SCRIPT.contains("lines[index] = f'gwz-core = \"={core_version}\"'"));
    assert!(
        RELEASE_SCRIPT.contains(
            "CRATES_IO_SOURCE = \"registry+https://github.com/rust-lang/crates.io-index\""
        )
    );
    assert!(RELEASE_SCRIPT.contains("verify_locked_registry_pin(worktree, core_version)"));
    assert!(!RELEASE_SCRIPT.contains("verify_locked_git_pin"));
    assert!(RELEASE_SCRIPT.contains(
        "f\"chore(release): gwz-cli {version} (pins gwz-core {core_version} from crates.io)\""
    ));
}

#[test]
fn release_script_waits_for_gwz_core_on_crates_io_before_creating_the_worktree() {
    assert!(
        RELEASE_SCRIPT
            .contains("REGISTRY_API = \"https://crates.io/api/v1/crates/gwz-core/{version}\"")
    );
    assert!(RELEASE_SCRIPT.contains("headers={\"User-Agent\": USER_AGENT}"));
    assert!(RELEASE_SCRIPT.contains("DEFAULT_REGISTRY_TIMEOUT = 900"));
    assert!(RELEASE_SCRIPT.contains("\"--registry-timeout\""));
    let wait = RELEASE_SCRIPT
        .find("wait_for_registry_core(core_version, timeout=args.registry_timeout)")
        .expect("main waits for gwz-core on crates.io");
    let worktree = RELEASE_SCRIPT
        .find("git([\"worktree\", \"add\", worktree, args.release])")
        .expect("main creates the release worktree");
    assert!(wait < worktree);
}

#[test]
fn release_script_takes_the_core_url_from_an_option_for_the_tag_check() {
    assert!(
        RELEASE_SCRIPT.contains("DEFAULT_CORE_URL = \"https://github.com/owebeeone/gwz-core\"")
    );
    assert!(RELEASE_SCRIPT.contains("\"--core-url\", default=DEFAULT_CORE_URL"));
    assert!(RELEASE_SCRIPT.contains("verify_core_tag(core_url, core_tag)"));
    assert!(!RELEASE_SCRIPT.contains("def gwz_core_url"));
}

#[test]
fn release_script_packages_the_cli_before_the_release_commit() {
    assert!(
        RELEASE_SCRIPT
            .contains("[\"cargo\", \"package\", \"--locked\", \"--allow-dirty\", \"-p\", \"gwz\"]")
    );
    assert!(!RELEASE_SCRIPT.contains("--no-verify"));
    let gate = RELEASE_SCRIPT
        .find("verify_cli_package(worktree, version)")
        .expect("the packaging gate runs");
    let commit = RELEASE_SCRIPT
        .find("git_wt(worktree, [\"commit\", \"-m\", message])")
        .expect("the release commit");
    assert!(gate < commit);
}

#[test]
fn release_script_helpers_keep_their_unit_tests() {
    for case in [
        "test_the_git_tag_pin_is_migrated_to_the_exact_registry_pin",
        "test_a_registry_pin_moves_to_the_new_core_version",
        "test_any_other_dependency_shape_is_refused_and_nothing_is_written",
        "test_a_lock_taking_the_core_and_its_internals_from_crates_io_passes",
        "test_a_git_sourced_core_is_refused",
        "test_a_different_core_version_is_refused",
        "test_a_core_entry_without_a_checksum_is_refused",
        "test_an_absent_core_is_polled_until_it_appears",
        "test_an_answer_waiting_cannot_change_fails_without_sleeping",
        "test_a_core_that_never_appears_times_out",
    ] {
        assert!(
            RELEASE_SCRIPT_TESTS.contains(&format!("def {case}(")),
            "scripts/test_release.py lost {case}"
        );
    }
}

#[test]
fn release_guide_describes_the_registry_pin_and_the_wait() {
    assert!(RELEASE_GUIDE.contains("`gwz-core = \"=X.Y.Z\"`"));
    assert!(RELEASE_GUIDE.contains("crates.io publish job"));
    assert!(RELEASE_GUIDE.contains("--registry-timeout"));
}
