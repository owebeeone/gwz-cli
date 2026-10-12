//! Candidate transport help preserves the ordinary CLI surface.
cfg_if::cfg_if! {
    if #[cfg(all(gwz_transport_candidate, any(unix, windows)))] {
        // What the help adds on Windows (Windows parity plan step 5.2; the design's section 14 item 13 and the
        // user guide): the SSH agent sources, the machine proxy, the http:// warning and the Pageant wait. Other
        // platforms add nothing, because none of it is true there.
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                macro_rules! windows_verbose_help {
                    () => {
                        concat!(
                            " On Windows, SSH authentication uses one agent, chosen when the operation starts and not changed if it fails: Pageant when a Pageant window is running (even when SSH_AUTH_SOCK is set), otherwise the local named pipe that SSH_AUTH_SOCK names (in the form \\\\.\\pipe\\<name>), otherwise the default pipe \\\\.\\pipe\\openssh-ssh-agent of the OpenSSH Authentication Agent service.",
                            " On Windows, an HTTPS remote uses the machine proxy setting (netsh winhttp show proxy) and no other proxy: HTTP_PROXY, HTTPS_PROXY, ALL_PROXY, NO_PROXY and git's http.proxy are ignored.",
                            " On Windows, an http:// remote prints a warning before its first request, whether or not you give --verbose, because a Basic password is sent unencrypted and a Windows sign-in has no channel binding.",
                        )
                    };
                }
                macro_rules! windows_ssh_timeout_help {
                    () => {
                        " On Windows, the wait for you to confirm a key in Pageant runs under this clock too: a prompt that stays open longer fails that attempt, and the message names this option. Answer the prompt and run the command again, raise this value (0 removes the limit), or use --transport native, which waits for Pageant as gwz 1.0 did."
                    };
                }
            } else {
                macro_rules! windows_verbose_help {
                    () => {
                        ""
                    };
                }
                macro_rules! windows_ssh_timeout_help {
                    () => {
                        ""
                    };
                }
            }
        }
        pub(super) const JOBS_HELP: &str = "Global ceiling on concurrent member operations (default 100; 50 with --transport native)";
        pub(super) const JOBS_LONG_HELP: &str = "Global ceiling on the total number of member repositories processed concurrently across all hosts. These are concurrent operations in this process, not extra processes. Defaults to 100; 50 with --transport native. The smallest value is 1. 0 is rejected. Per-host concurrency is bounded separately by --max-per-host. Values above 100 are accepted.";
        pub(super) const MAX_PER_HOST_HELP: &str = "Maximum (ceiling) concurrent member operations to any one hostname (default 32; 8 with --transport native)";
        pub(super) const MAX_PER_HOST_LONG_HELP: &str = "Maximum (ceiling) concurrent member operations against one remote hostname. The hostname is the host in the remote URL, lowercased, before SSH config is applied. Two URLs share this limit when they contain the same host. A URL whose host cannot be parsed is bounded only by --jobs. Defaults to 32; 8 with --transport native. The smallest value is 1. 0 is rejected. Values above 32 are accepted. Each of these operations uses one connection. gwz may use fewer connections than this when the host refuses more, and raises the number again when the host recovers; a note: line on stderr says so. It keeps the number it uses for each scheme and port of a host (SSH and HTTPS to one host are separate), and never goes above this ceiling. --max-retries 0 turns the lowering off, and --transport native never lowers it. If a repository still fails because the host kept refusing, see Troubleshooting: Fewer connections than --max-per-host, or a Capacity error.";
        pub(super) const VERBOSE_HELP: &str = "Show transport authentication diagnostics";
        pub(super) const VERBOSE_LONG_HELP: &str = concat!("Show one transport diagnostic for every remote authentication attempt. These diagnostics are omitted from normal human output and remain available in --json and --jsonl output. It also names the transport selected for network operations, gwz or native, and what selected it. Authentication rows (meta.transport in JSON) are reported on either transport. A redirect that gwz follows while it discovers an HTTPS remote is printed as a line of detail only with --verbose.", windows_verbose_help!());
        pub(super) const SSH_TIMEOUT_HELP: &str = "Per-attempt stall limit for setup and for a body read; stalled setup is retried (0 = no timeout, default 9; 3 with --transport native)";
        pub(super) const SSH_TIMEOUT_LONG_HELP: &str = concat!("Seconds without progress before that read fails. The clock applies to SSH setup, and it applies to a stalled read during a fetch, push, or pull body on SSH and HTTPS. A body stall aborts that repository and is not retried. A setup stall fails one attempt, and that attempt is retried. HTTPS setup has no stall clock, only the 30 second setup budget that SSH setup also has. libssh2 has no timeout by default, so a missing ssh-agent identity or an unreachable host would otherwise hang forever. 0 disables this stall clock and the 30 second setup budget on both SSH and HTTPS. It does not disable retries. Defaults to 9. Retries are controlled by --max-retries (default 3 extra attempts, so 4 in all; the same count bounds retries after a host refuses connections). The wait after a failed setup attempt starts at 1 second and doubles, and does not grow past 30 seconds, plus up to 0.25 seconds of jitter. That wait is not this flag. For the network clocks alone at these defaults, an SSH setup that makes no progress uses at most 4 times 9 seconds plus 1, 2, and 4 seconds of waits and under 1 second of jitter, about 44 seconds, and an HTTPS setup that makes no progress after 4 times 30 seconds plus the same waits, about 128 seconds. A setup that keeps making progress but never finishes can use the 30 second budget on each attempt, about 128 seconds. These are network-clock examples, not total elapsed-time bounds. Local credential-helper admission waits separately (up to the retained allocation budget, normally 30 seconds); every admitted helper has a separate interaction allowance of up to 120 seconds. SSH setup network clocks pause during those helper phases. --ssh-timeout does not shorten those helper allowances. Helper timeout is CredentialHelperTimeout (code 75); see docs/Troubleshooting.md#https-credential-failure for recovery. A host-key prompt can add up to 120 seconds on each attempt. A host that refuses a connection (an HTTP 429, or a 503 with Retry-After) is not a stalled setup: the repository waits for the host's Retry-After, at most 30 seconds at a time, or about half a second when it gave none, and the refusal counts as one of its attempts. With --transport native it is instead libgit2's connect and read timeout, with no setup retry and no 30 second setup budget, and it defaults to 3, as in gwz 1.0.", windows_ssh_timeout_help!());
        cfg_if::cfg_if! { if #[cfg(test)] { mod tests; } }
    } else {
        pub(super) const JOBS_HELP: &str = "Global ceiling on concurrent member operations (default 100)";
        pub(super) const JOBS_LONG_HELP: &str = "Global ceiling on the total number of member repositories processed concurrently across all hosts. These are concurrent operations in this process, not extra processes. Defaults to 100. The smallest value is 1. 0 is rejected. Per-host concurrency is bounded separately by --max-per-host. Values above 100 are accepted.";
        pub(super) const MAX_PER_HOST_HELP: &str = "Max concurrent member operations to any one hostname (default 32)";
        pub(super) const MAX_PER_HOST_LONG_HELP: &str = "Maximum concurrent member operations against one remote hostname. The hostname is the host in the remote URL, lowercased, before SSH config is applied. Two URLs share this limit when they contain the same host. A URL whose host cannot be parsed is bounded only by --jobs. Defaults to 32. The smallest value is 1. 0 is rejected. Values above 32 are accepted. Each of these operations uses one connection.";
        pub(super) const VERBOSE_HELP: &str = "Show transport authentication diagnostics";
        pub(super) const VERBOSE_LONG_HELP: &str = "Show one transport diagnostic for every remote authentication attempt. These diagnostics are omitted from normal human output and remain available in --json and --jsonl output.";
        pub(super) const SSH_TIMEOUT_HELP: &str = "Per-attempt stall limit for SSH setup and for a body read; stalled setup is retried (0 = no timeout, default 9)";
        pub(super) const SSH_TIMEOUT_LONG_HELP: &str = "Seconds without progress before that read fails. The clock applies to SSH setup, and it applies to a stalled read during a fetch, push, or pull body on SSH and HTTPS. A body stall aborts that repository and is not retried. A setup stall fails one attempt, and that attempt is retried. HTTPS setup has no stall clock, only the 30 second setup budget that SSH setup also has. libssh2 has no timeout by default, so a missing ssh-agent identity or an unreachable host would otherwise hang forever. 0 disables this stall clock and the 30 second setup budget on both SSH and HTTPS. It does not disable retries. Defaults to 9. Retries are controlled by --max-retries (default 3 extra attempts). The wait after a failed setup attempt starts at 1 second and doubles, and does not grow past 30 seconds, plus up to 0.25 seconds of jitter. That wait is not this flag. At these defaults, an SSH setup that makes no progress is reported after at most 4 times 9 seconds plus 1, 2, and 4 seconds of waits and under 1 second of jitter, about 44 seconds, and an HTTPS setup that makes no progress after 4 times 30 seconds plus the same waits, about 128 seconds. A setup that keeps making progress but never finishes can use the 30 second budget on each attempt, about 128 seconds. A host-key prompt can add up to 120 seconds on each attempt.";
    }
}
