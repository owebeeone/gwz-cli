//! The "HTTPS Credential Failure" section of docs/Troubleshooting.md is the target of the credential messages' last
//! sentence ("See Troubleshooting: HTTPS Credential Failure."). It names the cases those messages report, by the
//! words each message starts with, and no longer says Windows parity is under qualification.
use std::fs;
use std::path::PathBuf;

/// The "HTTPS Credential Failure" section: from its heading to the next second-level heading.
fn section() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/Troubleshooting.md");
    let text = fs::read_to_string(path).expect("docs/Troubleshooting.md");
    let start = text
        .find("## HTTPS Credential Failure")
        .expect("the section exists");
    let rest = &text[start + 2..];
    let end = rest.find("\n## ").map_or(rest.len(), |at| at + 2);
    text[start..start + end].to_owned()
}

/// The section with its line breaks folded, so that a pin does not depend on where the prose wraps.
fn folded() -> String {
    section().split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_section_no_longer_says_windows_parity_is_under_qualification() {
    let text = folded();
    assert!(!text.contains("under qualification"), "{text}");
    assert!(!text.contains("remains under"), "{text}");
}

#[test]
fn the_section_names_the_declined_case_by_its_message() {
    let text = folded();
    assert!(
        text.contains("The credential helper in your git configuration gave no password for"),
        "{text}"
    );
    // What it may mean, and what to do: the helper's own sign-in, run where it can ask.
    assert!(text.contains("cancelled"), "{text}");
    assert!(text.contains("git ls-remote"), "{text}");
}

#[test]
fn the_section_names_the_configuration_refused_case_by_its_message() {
    let text = folded();
    assert!(
        text.contains(
            "gwz did not use your git configuration to find a credential helper, because it refused to read it"
        ),
        "{text}"
    );
    // The kinds of refusal, and how to see what git reads.
    for kind in ["too large", "include.path", "git config --list --show-origin"] {
        assert!(text.contains(kind), "{kind} in {text}");
    }
}

#[test]
fn the_section_names_the_windows_no_helper_case() {
    let text = folded();
    assert!(
        text.contains("no git credential helper is configured"),
        "{text}"
    );
    assert!(text.contains("credential.helper manager"), "{text}");
}
