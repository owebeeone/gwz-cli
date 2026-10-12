//! `--max-retries`, and the sentence that the long help of fetch, push and
//! pull gains for it (gwz-core dev-docs/GwzRemoteTransportRetryPlan.md §8).
//! Only the transport build retries a setup, so only it has either: rule (a)
//! of gwz-core dev-docs/GwzTransportReleasePlanAmendment-2.md §3.13.
use clap::Args;

cfg_if::cfg_if! {
    if #[cfg(gwz_transport_candidate)] {
        #[derive(Clone, Debug, Default, Args)]
        pub(crate) struct RetryArgs {
            #[arg(
                long = "max-retries",
                global = true,
                value_name = "n",
                value_parser = parse_max_retries,
                help = "Extra attempts after a failed setup or a refusal (0 = no retries, and --max-per-host is never lowered; default 3)",
                long_help = "How many times to retry a connection that failed during setup, after the first attempt. Setup is before the session is reusable: for SSH, before it is authenticated; for HTTPS, before the first request byte. A stall or a reset during a fetch, push, or pull body is not retried. Defaults to 3, so four attempts. 0 reports the first failure and does not retry. Values above 3 are accepted. This is the only retry control. --ssh-timeout 0 does not turn retries off. A repository that succeeds on a later attempt counts as having answered. After each failed setup attempt the next one waits 1 second, then 2, then 4, doubling, and the wait does not grow past 30 seconds, plus up to 0.25 seconds of jitter. --ssh-timeout sets only the stall of an SSH setup attempt and of a body read. 0 on that flag clears the stall and the 30 second setup budget. It does not clear these waits. A connection that the host refused because it holds too many connections (an HTTP 429, or a 503 with Retry-After) is retried within the same count, so a host that keeps refusing makes a repository give up after this many attempts in all. 0 also turns off the adaptive per-host limit: gwz never lowers the number of connections it uses because a host refused one, a refusal fails that repository at once, and a Retry-After wait still applies. See --max-per-host."
            )]
            pub(crate) max_retries: Option<i64>,
        }
        impl RetryArgs {
            /// The operation's policy carries the flag; absent, the
            /// transport's default of three retries applies.
            pub(crate) fn apply(&self, policy: &mut gwz_core::OperationPolicy) {
                policy.max_retries = self.max_retries;
            }
        }
        fn parse_max_retries(value: &str) -> Result<i64, String> {
            let parsed = value
                .parse::<i64>()
                .map_err(|_| "--max-retries requires an integer".to_owned())?;
            if parsed < 0 {
                return Err("--max-retries must be zero or greater".to_owned());
            }
            // The transport counts attempts in a u32 and refuses a larger budget.
            if parsed > i64::from(u32::MAX) {
                return Err(format!("--max-retries must be at most {}", u32::MAX));
            }
            Ok(parsed)
        }
        /// The sentence each of fetch, push and pull ends its long help with.
        macro_rules! retry_sentence {
            () => {
                "\n\nA connection that fails during setup is retried before that repository counts as failed. See --max-retries and --ssh-timeout. A repository that succeeds on a later attempt counts as having answered."
            };
        }
    } else {
        #[derive(Clone, Debug, Default, Args)]
        pub(crate) struct RetryArgs {}
        impl RetryArgs {
            pub(crate) fn apply(&self, _policy: &mut gwz_core::OperationPolicy) {}
        }
        macro_rules! retry_sentence {
            () => {
                ""
            };
        }
    }
}
pub(crate) use retry_sentence;

cfg_if::cfg_if! {
    if #[cfg(all(test, gwz_transport_candidate))] {
        mod pins {
            use crate::{Cli, CliRequest, parse_args_with_request_id};
            use clap::Parser;

            /// The help that `args` render, whitespace-normalized.
            fn help(args: &[&str]) -> String {
                let rendered = Cli::try_parse_from(args).expect_err("help").to_string();
                rendered.split_whitespace().collect::<Vec<_>>().join(" ")
            }
            fn policy(args: &[&str]) -> gwz_core::OperationPolicy {
                let args = args.iter().map(ToString::to_string).collect();
                let invocation =
                    parse_args_with_request_id(args, "req_retry", &crate::tests::test_cwd())
                        .unwrap();
                let CliRequest::Fetch(request) = invocation.request else {
                    panic!("expected fetch");
                };
                request.meta.policy.expect("a fetch carries its policy")
            }

            #[test]
            fn max_retries_renders_the_retry_plans_help() {
                let short = help(&["gwz", "fetch", "-h"]);
                assert!(short.contains(
                    "--max-retries <n> Extra attempts after a failed setup or a refusal (0 = no retries, and --max-per-host is never lowered; default 3)"
                ));
                let long = help(&["gwz", "fetch", "--help"]);
                assert!(long.contains("How many times to retry a connection that failed during setup, after the first attempt. Setup is before the session is reusable: for SSH, before it is authenticated; for HTTPS, before the first request byte. A stall or a reset during a fetch, push, or pull body is not retried. Defaults to 3, so four attempts. 0 reports the first failure and does not retry. Values above 3 are accepted. This is the only retry control. --ssh-timeout 0 does not turn retries off. A repository that succeeds on a later attempt counts as having answered. After each failed setup attempt the next one waits 1 second, then 2, then 4, doubling, and the wait does not grow past 30 seconds, plus up to 0.25 seconds of jitter. --ssh-timeout sets only the stall of an SSH setup attempt and of a body read. 0 on that flag clears the stall and the 30 second setup budget. It does not clear these waits."));
            }

            /// The adaptive limit's help (adaptive concurrency design section 9): each
            /// flag states its default and what the other does to it.
            #[test]
            fn max_retries_help_states_its_default_and_what_zero_does_to_the_host_limit() {
                let long = help(&["gwz", "fetch", "--help"]);
                assert!(long.contains("Defaults to 3, so four attempts."));
                assert!(long.contains("A connection that the host refused because it holds too many connections (an HTTP 429, or a 503 with Retry-After) is retried within the same count, so a host that keeps refusing makes a repository give up after this many attempts in all."));
                assert!(long.contains("0 also turns off the adaptive per-host limit: gwz never lowers the number of connections it uses because a host refused one, a refusal fails that repository at once, and a Retry-After wait still applies. See --max-per-host."));
            }

            #[test]
            fn max_per_host_help_states_the_ceiling_and_that_gwz_may_use_fewer() {
                let short = help(&["gwz", "fetch", "-h"]);
                assert!(short.contains(
                    "Maximum (ceiling) concurrent member operations to any one hostname (default 32; 8 with --transport native)"
                ));
                let long = help(&["gwz", "fetch", "--help"]);
                assert!(long.contains("Maximum (ceiling) concurrent member operations against one remote hostname."));
                assert!(long.contains("Defaults to 32; 8 with --transport native."));
                assert!(long.contains("gwz may use fewer connections than this when the host refuses more, and raises the number again when the host recovers; a note: line on stderr says so."));
                assert!(long.contains("It keeps the number it uses for each scheme and port of a host (SSH and HTTPS to one host are separate), and never goes above this ceiling."));
                assert!(long.contains("--max-retries 0 turns the lowering off, and --transport native never lowers it."));
                assert!(long.contains("Troubleshooting: Fewer connections than --max-per-host, or a Capacity error."));
            }

            #[test]
            fn ssh_timeout_help_says_a_refusal_waits_for_the_host_not_for_this_clock() {
                let long = help(&["gwz", "fetch", "--help"]);
                assert!(long.contains("Retries are controlled by --max-retries (default 3 extra attempts, so 4 in all; the same count bounds retries after a host refuses connections)."));
                assert!(long.contains("A host that refuses a connection (an HTTP 429, or a 503 with Retry-After) is not a stalled setup: the repository waits for the host's Retry-After, at most 30 seconds at a time, or about half a second when it gave none, and the refusal counts as one of its attempts."));
            }

            #[test]
            fn fetch_push_and_pull_long_help_say_a_failed_setup_is_retried() {
                for command in ["fetch", "push", "pull"] {
                    let long = help(&["gwz", command, "--help"]);
                    assert!(
                        long.contains("A connection that fails during setup is retried before that repository counts as failed. See --max-retries and --ssh-timeout. A repository that succeeds on a later attempt counts as having answered."),
                        "{command}"
                    );
                }
            }

            #[test]
            fn max_retries_reaches_the_operation_policy() {
                assert_eq!(policy(&["--max-retries", "0", "fetch"]).max_retries, Some(0));
                assert_eq!(policy(&["--max-retries", "7", "fetch"]).max_retries, Some(7));
                // The largest count the transport holds is kept as given.
                assert_eq!(
                    policy(&["--max-retries", "4294967295", "fetch"]).max_retries,
                    Some(4_294_967_295)
                );
                // Absent, the transport's default of three retries applies.
                assert_eq!(policy(&["fetch"]).max_retries, None);
                for refused in ["-1", "x", "4294967296"] {
                    assert!(
                        Cli::try_parse_from(["gwz", "--max-retries", refused, "fetch"]).is_err(),
                        "{refused}"
                    );
                }
            }
        }
    } else if #[cfg(test)] {
        mod pins {
            use crate::Cli;
            use clap::Parser;

            #[test]
            fn the_ordinary_build_has_no_max_retries_and_no_retry_sentence() {
                assert!(Cli::try_parse_from(["gwz", "--max-retries", "0", "fetch"]).is_err());
                for command in ["fetch", "push", "pull"] {
                    let long = Cli::try_parse_from(["gwz", command, "--help"])
                        .expect_err("help")
                        .to_string();
                    let long = long.split_whitespace().collect::<Vec<_>>().join(" ");
                    assert!(!long.contains("--max-retries <n>"), "{command}");
                    assert!(
                        !long.contains("A connection that fails during setup is retried"),
                        "{command}"
                    );
                }
            }
        }
    }
}
