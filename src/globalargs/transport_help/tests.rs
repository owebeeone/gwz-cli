//! The candidate build's help on Windows (Windows parity plan step 5.2): what `--verbose`,
//! `--ssh-timeout` and `--transport` say about Windows SSH, Pageant and the machine proxy, and
//! that the text appears only where it is true. Both defaults of `--ssh-timeout` and the HTTPS
//! stall are pinned in `g09` and `retry`; the rows here are the Windows ones.
use crate::Cli;
use clap::Parser;

/// The long help of `fetch`, whitespace-normalized.
fn long_help() -> String {
    let rendered = Cli::try_parse_from(["gwz", "fetch", "--help"])
        .expect_err("long help")
        .to_string();
    rendered.split_whitespace().collect::<Vec<_>>().join(" ")
}

const AGENT_SOURCES: &str = "On Windows, SSH authentication uses one agent, chosen when the operation starts and not changed if it fails: Pageant when a Pageant window is running (even when SSH_AUTH_SOCK is set), otherwise the local named pipe that SSH_AUTH_SOCK names (in the form \\\\.\\pipe\\<name>), otherwise the default pipe \\\\.\\pipe\\openssh-ssh-agent of the OpenSSH Authentication Agent service.";
const MACHINE_PROXY: &str = "On Windows, an HTTPS remote uses the machine proxy setting (netsh winhttp show proxy) and no other proxy: HTTP_PROXY, HTTPS_PROXY, ALL_PROXY, NO_PROXY and git's http.proxy are ignored.";
const PAGEANT_WAIT: &str = "On Windows, the wait for you to confirm a key in Pageant runs under this clock too: a prompt that stays open longer fails that attempt, and the message names this option. Answer the prompt and run the command again, raise this value (up to 30 seconds; 0 removes the limit), or use --transport native, which waits for Pageant as gwz 1.0 did.";
const HTTP_WARNING: &str = "On Windows, an http:// remote prints a warning before its first request, whether or not you give --verbose, because a Basic password is sent unencrypted and a Windows sign-in has no channel binding.";
const REDIRECT_DETAIL: &str = "A redirect that gwz follows while it discovers an HTTPS remote is printed as a line of detail only with --verbose.";

#[test]
fn every_platform_says_a_followed_redirect_is_printed_only_with_verbose() {
    assert!(long_help().contains(REDIRECT_DETAIL));
}

#[test]
fn windows_help_describes_ssh_agents_pageant_the_machine_proxy_and_the_http_warning() {
    let help = long_help();
    for text in [AGENT_SOURCES, MACHINE_PROXY, PAGEANT_WAIT, HTTP_WARNING] {
        assert_eq!(help.contains(text), cfg!(windows), "{text}");
    }
}

#[test]
fn the_windows_text_never_reaches_other_platforms_help() {
    if cfg!(windows) {
        return;
    }
    let help = long_help();
    for word in ["Pageant", "openssh-ssh-agent", "netsh winhttp", "WinHTTP"] {
        assert!(!help.contains(word), "{word}");
    }
}

#[test]
fn ssh_timeout_help_states_both_defaults_and_the_https_stall_on_every_platform() {
    let short = Cli::try_parse_from(["gwz", "fetch", "-h"])
        .expect_err("short help")
        .to_string();
    let short = short.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(short.contains("(0 = no timeout, default 9; 3 with --transport native)"));
    let help = long_help();
    assert!(help.contains("it applies to a stalled read during a fetch, push, or pull body on SSH and HTTPS."));
    assert!(help.contains("and it defaults to 3, as in gwz 1.0."));
}
