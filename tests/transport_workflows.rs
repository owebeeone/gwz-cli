//! Process-entry transport settings: isolated environment and real output modes.
#[path = "../src/tests/temp_dir.rs"]
mod temp_dir;
use std::process::{Command, Output};
use temp_dir::TempDir;

fn run(temp: &TempDir, args: &[&str], environment: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gwz"))
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.path().join("xdg"))
        .env("GWZ_TRANSPORT", environment)
        .env_remove("GIT_CONFIG_GLOBAL")
        .args(args)
        .output()
        .unwrap()
}

cfg_if::cfg_if! {
    if #[cfg(all(unix, gwz_transport_candidate))] {
        fn workspace() -> TempDir {
            let temp = TempDir::new("transport-setting");
            let init = run(&temp, &["init"], "bad");
            assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
            temp
        }

        #[test]
        fn invalid_deciding_environment_refuses_but_status_ignores_it() {
            let temp = workspace();
            let fetch = run(&temp, &["fetch", "--json"], "bad");
            assert_eq!(fetch.status.code(), Some(2));
            let value: serde_json::Value = serde_json::from_slice(&fetch.stdout).unwrap();
            assert!(value.to_string().contains("GWZ_TRANSPORT must be gwz or native"));
            assert!(!value.to_string().contains("transport_setting"));
            let status = run(&temp, &["status", "--json"], "bad");
            assert!(status.status.success());
            assert!(!String::from_utf8_lossy(&status.stdout).contains("transport_setting"));
            assert!(status.stderr.is_empty());
        }

        #[test]
        fn each_form_and_flag_precedence_reach_machine_output_without_notices() {
            let temp = workspace();
            std::fs::write(temp.path().join(".gitconfig"), "[gwz]\ntransport = native\n").unwrap();
            for (args, environment, source, transport) in [
                (vec!["fetch", "--dry-run", "--json"], "", "global_configuration", "native"),
                (vec!["fetch", "--dry-run", "--jsonl"], "native", "environment", "native"),
                (vec!["fetch", "--dry-run", "--json", "--transport", "gwz"], "native", "flag", "gwz"),
                (vec!["fetch", "--dry-run", "--json", "--transport", "native"], "bad", "flag", "native"),
            ] {
                let output = run(&temp, &args, environment);
                assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
                assert!(output.stderr.is_empty());
                let stdout = String::from_utf8(output.stdout).unwrap();
                let response = stdout.lines().filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                    .find(|value| value["kind"] == "response").unwrap();
                assert_eq!(response["meta"]["transport_setting"]["source"], source);
                assert_eq!(response["meta"]["transport_setting"]["transport"], transport);
            }
        }

        #[test]
        fn human_dry_run_prints_native_notice_and_ignored_repository_value_once() {
            let temp = workspace();
            let repo = git2::Repository::open(temp.path()).unwrap();
            repo.config().unwrap().set_str("gwz.transport", "bad").unwrap();
            let output = run(&temp, &["fetch", "--dry-run", "--verbose"], "native");
            assert!(output.status.success());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert_eq!(stderr.matches("using libgit2's native transport").count(), 1);
            assert_eq!(stderr.matches("ignoring gwz.transport").count(), 1);
            assert!(stderr.contains("(root)"));
            assert!(String::from_utf8_lossy(&output.stdout).contains("transport: native (from GWZ_TRANSPORT)"));
        }
    } else {
        #[test]
        fn ordinary_build_refuses_transport_flag_and_ignores_variable() {
            let temp = TempDir::new("ordinary-transport");
            assert_eq!(run(&temp, &["fetch", "--transport", "native"], "bad").status.code(), Some(2));
            assert!(run(&temp, &["init"], "bad").status.success());
            let status = run(&temp, &["status", "--json"], "bad");
            assert!(status.status.success());
            assert!(!String::from_utf8_lossy(&status.stdout).contains("transport_setting"));
        }
    }
}
