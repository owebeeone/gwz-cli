//! The "Windows Sign-In" section of docs/Troubleshooting.md is the target of the Windows sign-in messages' last
//! sentence ("See Troubleshooting: Windows Sign-In."), which gwz-core pins as built. It covers a 403 after the
//! sign-in, a rejected logon identity and the worker texts, by the words each message starts with, and qualifies the
//! `git ls-remote` advice for a server that offers NTLM only. "HTTPS Credential Failure" points at it. A text that
//! changes in one place and not the other fails a test on its side.
use std::fs;
use std::path::PathBuf;

const HEADING: &str = "## Windows Sign-In";

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
fn the_section_covers_a_403_after_the_sign_in_and_qualifies_the_ls_remote_advice() {
    let text = folded_section(&troubleshooting(), HEADING);
    assert!(
        text.contains(
            "The server refused this account (HTTP 403) after gwz sent your Windows sign-in."
        ),
        "{text}"
    );
    // A refusal of the request, not a rejection of the sign-in, which is a 401.
    assert!(text.contains("not a rejection of the sign-in"), "{text}");
    // What git does for itself is not what gwz sent: on an NTLM-only server it may prompt.
    for fact in [
        "git ls-remote",
        "offers NTLM only",
        "may ask you for a password",
        "cancel it",
    ] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

#[test]
fn the_section_covers_a_rejected_logon_identity() {
    let text = folded_section(&troubleshooting(), HEADING);
    for fact in [
        "rejected your Windows account (Negotiate)",
        "(or `(NTLM)`)",
        "GWZ tries nothing else",
        "no credential helper is asked",
        "use an SSH remote",
    ] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

#[test]
fn the_section_covers_the_worker_texts_and_the_ways_out() {
    let text = folded_section(&troubleshooting(), HEADING);
    for fact in [
        "GWZ could not use its Windows sign-in worker",
        "it is missing, or it is from a different build of gwz",
        "GWZ's Windows sign-in worker did not answer as the one installed with gwz does",
        "No credential was sent.",
        "Reinstall gwz",
        "--transport native",
        "GWZ_TRANSPORT=native",
    ] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

#[test]
fn the_section_covers_the_redirect_during_a_sign_in() {
    let text = folded_section(&troubleshooting(), HEADING);
    for fact in [
        "answered GWZ's Windows sign-in with a redirect",
        "does not follow a redirect once a Windows sign-in has started",
    ] {
        assert!(text.contains(fact), "{fact} in {text}");
    }
}

#[test]
fn the_credential_section_points_at_it() {
    let text = troubleshooting();
    let credential = folded_section(&text, "## HTTPS Credential Failure");
    assert!(
        credential.contains("[Windows Sign-In](#windows-sign-in)"),
        "{credential}"
    );
}
