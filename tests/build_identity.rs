use std::process::Command;

#[test]
fn long_version_identifies_both_source_trees_and_build_system() {
    let output = Command::new(env!("CARGO_BIN_EXE_gwz"))
        .arg("--version")
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
