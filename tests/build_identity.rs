use std::process::Command;

#[test]
fn build_info_identifies_both_source_trees_and_build_system() {
    let output = Command::new(env!("CARGO_BIN_EXE_gwz"))
        .arg("--build-info")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains(concat!("gwz ", env!("CARGO_PKG_VERSION"))));
    assert!(text.contains("cli: revision="), "{text}");
    assert!(
        text.contains(&format!("core {}: revision=", gwz_core::VERSION)),
        "{text}"
    );
    assert_eq!(text.matches("source-sha256=").count(), 2, "{text}");
    assert_eq!(text.matches("build=cargo").count(), 2, "{text}");
}

#[test]
fn version_is_only_the_release_version() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_gwz"))
            .arg(flag)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            concat!("gwz ", env!("CARGO_PKG_VERSION"), "\n")
        );
    }
}
