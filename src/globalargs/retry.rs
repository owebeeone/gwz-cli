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
                help = "Extra setup attempts after the first failure (0 = do not retry, default 3)",
                long_help = "How many times to retry a connection that failed during setup, after the first attempt. Setup is before the session is reusable: for SSH, before it is authenticated; for HTTPS, before the first request byte. A stall or a reset during a fetch, push, or pull body is not retried. Defaults to 3, so four attempts. 0 reports the first failure and does not retry. Values above 3 are accepted. This is the only retry control. --ssh-timeout 0 does not turn retries off. A repository that succeeds on a later attempt counts as having answered. After each failed setup attempt the next one waits 1 second, then 2, then 4, doubling, and the wait does not grow past 30 seconds, plus up to 0.25 seconds of jitter. --ssh-timeout sets only the stall of an SSH setup attempt and of a body read. 0 on that flag clears the stall and the 30 second setup budget. It does not clear these waits."
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
                    parse_args_with_request_id(args, "req_retry", std::path::Path::new("/"))
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
                    "--max-retries <n> Extra setup attempts after the first failure (0 = do not retry, default 3)"
                ));
                let long = help(&["gwz", "fetch", "--help"]);
                assert!(long.contains("How many times to retry a connection that failed during setup, after the first attempt. Setup is before the session is reusable: for SSH, before it is authenticated; for HTTPS, before the first request byte. A stall or a reset during a fetch, push, or pull body is not retried. Defaults to 3, so four attempts. 0 reports the first failure and does not retry. Values above 3 are accepted. This is the only retry control. --ssh-timeout 0 does not turn retries off. A repository that succeeds on a later attempt counts as having answered. After each failed setup attempt the next one waits 1 second, then 2, then 4, doubling, and the wait does not grow past 30 seconds, plus up to 0.25 seconds of jitter. --ssh-timeout sets only the stall of an SSH setup attempt and of a body read. 0 on that flag clears the stall and the 30 second setup budget. It does not clear these waits."));
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
                // Absent, the transport's default of three retries applies.
                assert_eq!(policy(&["fetch"]).max_retries, None);
                for refused in ["-1", "x"] {
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
