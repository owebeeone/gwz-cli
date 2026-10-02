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

        #[test]
        fn actual_execution_refusals_keep_null_meta_and_print_human_selection_once() {
            let temp = TempDir::new("missing-workspace-setting");
            for mode in ["--json", "--jsonl"] {
                let output = run(&temp, &["fetch", mode, "--transport", "native"], "bad");
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stderr.is_empty());
                let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                assert!(value["meta"].is_null());
                assert!(!value.to_string().contains("transport_setting"));
                let status = run(&temp, &["status", mode], "bad");
                assert_eq!(status.status.code(), Some(1));
                assert!(!String::from_utf8_lossy(&status.stdout).contains("transport_setting"));
            }
            let human = run(&temp, &["fetch", "--verbose", "--transport", "native"], "bad");
            assert_eq!(human.status.code(), Some(1));
            let stderr = String::from_utf8(human.stderr).unwrap();
            assert_eq!(stderr.matches("transport: native (from --transport)").count(), 1);
            assert_eq!(stderr.matches("using libgit2's native transport").count(), 1);
            assert!(human.stdout.is_empty());
        }

        fn tagged_workspace() -> TempDir {
            let temp = workspace();
            let origin_path = temp.path().join("origin.git");
            let origin = git2::Repository::init_bare(&origin_path).unwrap();
            let tree_id = origin.treebuilder(None).unwrap().write().unwrap();
            let tree = origin.find_tree(tree_id).unwrap();
            let sig = git2::Signature::now("Test", "test@example.invalid").unwrap();
            let commit = origin.commit(Some("HEAD"), &sig, &sig, "fixture", &tree, &[]).unwrap();
            origin.tag_lightweight("v1", &origin.find_object(commit, None).unwrap(), false).unwrap();
            git2::Repository::clone(&format!("file://{}", origin_path.display()), temp.path().join("app")).unwrap();
            let add = run(&temp, &["repo", "add", "app", "--member-id", "mem_app"], "bad");
            assert!(add.status.success(), "{}", String::from_utf8_lossy(&add.stderr));
            temp
        }

        fn listing(output: Output) -> serde_json::Value {
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            assert!(output.stderr.is_empty());
            let values: Vec<serde_json::Value> = String::from_utf8(output.stdout).unwrap().lines()
                .map(|line| serde_json::from_str(line).unwrap()).collect();
            assert!(!values.iter().any(|value| value["kind"] == "Diagnostic" || value["event_kind"] == "Diagnostic"));
            values.into_iter().find(|value| value["kind"] == "tags").unwrap()
        }

        #[test]
        fn remote_tag_listing_reports_settings_without_changing_entries_or_local_output() {
            use std::os::unix::fs::PermissionsExt;
            let temp = tagged_workspace();
            for mode in ["--json", "--jsonl"] {
                let baseline = listing(run(&temp, &["tag", "--list", "--remote", "origin", mode], ""));
                assert!(baseline.get("meta").is_none());
                assert!(!baseline["entries"].as_array().unwrap().is_empty());
                for transport in ["native", "gwz"] {
                    let selected = listing(run(&temp, &["tag", "--list", "--remote", "origin", mode, "--transport", transport], ""));
                    assert_eq!(selected["entries"], baseline["entries"]);
                    assert_eq!(selected["meta"]["transport_setting"]["transport"], transport);
                }
                let repo = git2::Repository::open(temp.path().join("app")).unwrap();
                repo.config().unwrap().set_str("gwz.transport", "bad").unwrap();
                let ignored = listing(run(&temp, &["tag", "--list", "--remote", "origin", mode], ""));
                assert_eq!(ignored["entries"], baseline["entries"]);
                assert_eq!(ignored["meta"]["transport_setting"]["ignored"][0]["value"], "bad");
                repo.config().unwrap().remove("gwz.transport").unwrap();
                let file = temp.path().join(".gitconfig");
                std::fs::write(&file, "[gwz]\ntransport = native\n").unwrap();
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
                let skipped = listing(run(&temp, &["tag", "--list", "--remote", "origin", mode], ""));
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
                std::fs::remove_file(&file).unwrap();
                assert_eq!(skipped["meta"]["transport_setting"]["skipped"][0]["file"], file.to_str().unwrap());
                let local = listing(run(&temp, &["tag", "--list", mode], "bad"));
                assert!(local.get("meta").is_none());
            }
        }

        #[test]
        fn supported_file_recipe_sets_and_removes_under_git_config_global_override() {
            let temp = workspace();
            let alternate = temp.path().join("alternate-config");
            std::fs::write(&alternate, "[gwz]\ntransport = gwz\n").unwrap();
            let set = "git config --file \"$HOME/.gitconfig\" gwz.transport native";
            let remove = "git config --file \"$HOME/.gitconfig\" --unset-all gwz.transport";
            let help = run(&temp, &["fetch", "--help"], "");
            let help = String::from_utf8(help.stdout).unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
            let docs = include_str!("../docs/commands/auth.md");
            for command in [set, remove] {
                assert!(help.contains(command));
                assert!(docs.contains(command));
                let executed = Command::new("sh").arg("-c").arg(command).env("HOME", temp.path())
                    .env("GIT_CONFIG_GLOBAL", &alternate).output().unwrap();
                assert!(executed.status.success());
                let fetch = Command::new(env!("CARGO_BIN_EXE_gwz")).current_dir(temp.path()).env("HOME", temp.path())
                    .env("XDG_CONFIG_HOME", temp.path().join("xdg")).env("GIT_CONFIG_GLOBAL", &alternate)
                    .env_remove("GWZ_TRANSPORT").args(["fetch", "--dry-run", "--json"]).output().unwrap();
                assert!(fetch.status.success());
                let value: serde_json::Value = serde_json::from_slice(&fetch.stdout).unwrap();
                if command == set { assert_eq!(value["meta"]["transport_setting"]["transport"], "native"); }
                else { assert!(value["meta"].get("transport_setting").is_none()); }
            }
            assert_eq!(std::fs::read_to_string(alternate).unwrap(), "[gwz]\ntransport = gwz\n");
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
