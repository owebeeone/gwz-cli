//! The "An SSH Key File Is Protected By A Passphrase" section of docs/Troubleshooting.md quotes the start of the
//! message gwz-core builds for a key file with a passphrase (`ssh_limits`, D17), names the three forms gwz recognizes,
//! and gives the two ways out, with the Windows difference. "SSH Or Credential Failure" points at it. The quoted words
//! are the ones gwz-core's tests pin as built; a text that changes in one place and not the other fails a test on its
//! side.
use std::fs;
use std::path::PathBuf;

const HEADING: &str = "## An SSH Key File Is Protected By A Passphrase";

fn troubleshooting() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/Troubleshooting.md");
    fs::read_to_string(path).expect("docs/Troubleshooting.md")
}

/// `heading`'s section: from its heading to the next second-level heading, with its line breaks folded so that a pin
/// does not depend on where the prose wraps.
fn folded_section(text: &str, heading: &str) -> String {
    let start = text.find(heading).expect("the section exists");
    let rest = &text[start + 2..];
    let end = rest.find("\n## ").map_or(rest.len(), |at| at + 2);
    text[start..start + end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_section_quotes_the_message_it_is_the_answer_to() {
    let text = folded_section(&troubleshooting(), HEADING);
    assert!(
        text.contains(
            "The SSH key file <path> is protected by a passphrase, and gwz cannot ask for a passphrase, so it cannot use the file."
        ),
        "{text}"
    );
    // It is a refusal of the file on every platform, before any connection, that nothing retries.
    for fact in [
        "UnsupportedOperation",
        "on any platform",
        "before gwz connects",
        "nothing is retried",
    ] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

#[test]
fn the_section_names_the_three_forms_gwz_recognizes_as_protected() {
    let text = folded_section(&troubleshooting(), HEADING);
    for form in [
        "Proc-Type: 4,ENCRYPTED",
        "DEK-Info:",
        "ENCRYPTED PRIVATE KEY",
        "whose cipher is not `none`",
    ] {
        assert!(text.contains(form), "{form} in {text}");
    }
}

#[test]
fn the_section_gives_both_ways_out_and_the_windows_difference() {
    let text = folded_section(&troubleshooting(), HEADING);
    // Way one: an agent, with no key file named (a named one is the only key offered).
    for step in [
        "ssh-add <path>",
        "OpenSSH Authentication Agent service",
        "Pageant",
        "drop `--identity` and `--remote-identity`",
        "gwz auth identity <remote> --unset",
    ] {
        assert!(text.contains(step), "{step} in {text}");
    }
    // Way two: a copy without the passphrase, and `-m PEM` for an RSA key on Windows only.
    for step in [
        "ssh-keygen -p -f <path>-nopass",
        "press Enter for the new one",
        "On Windows gwz reads a key file only if it is an RSA key in PEM form",
        "ssh-keygen -p -m PEM -f <path>-nopass",
        "Ed25519 key can only come from an agent there",
    ] {
        assert!(text.contains(step), "{step} in {text}");
    }
    let before_windows = text.split("On Windows gwz reads").next().unwrap();
    assert!(!before_windows.contains("-m PEM"), "{before_windows}");
}

#[test]
fn the_general_ssh_section_points_at_it() {
    let text = troubleshooting();
    let general = folded_section(&text, "## SSH Or Credential Failure");
    assert!(
        general.contains(
            "If the message says an SSH key file \"is protected by a passphrase\", see [An SSH Key File Is Protected By A Passphrase](#an-ssh-key-file-is-protected-by-a-passphrase)."
        ),
        "{general}"
    );
}
