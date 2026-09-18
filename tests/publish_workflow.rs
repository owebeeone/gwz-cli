const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");
const PUBLISH_CRATE_WORKFLOW: &str = include_str!("../.github/workflows/publish-crate.yml");
const CI_WORKFLOW: &str = include_str!("../.github/workflows/ci.yml");
const DIST_WORKSPACE: &str = include_str!("../dist-workspace.toml");
const CARGO_MANIFEST: &str = include_str!("../Cargo.toml");
const RUST_TOOLCHAIN: &str = include_str!("../rust-toolchain.toml");

/// Whether some line of `text`, trimmed, is exactly `expected`.
///
/// Whole lines, never a literal `\n`: a Windows checkout rewrites the workflows
/// with CRLF endings, and `lines()` strips either ending.
fn has_line(text: &str, expected: &str) -> bool {
    text.lines().any(|line| line.trim() == expected)
}

/// How many lines of `text`, trimmed, are exactly `expected`.
fn count_lines(text: &str, expected: &str) -> usize {
    text.lines().filter(|line| line.trim() == expected).count()
}

/// The lines of the top-level job `name`: its header and everything up to the
/// next job header.
fn job_lines<'a>(workflow: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("  {name}:");
    let mut lines = workflow
        .lines()
        .skip_while(|line| line.trim_end() != header);
    let first = lines
        .next()
        .unwrap_or_else(|| panic!("the workflow has no `{name}` job"));
    let mut block = vec![first];
    block.extend(lines.take_while(|line| !is_job_header(line)));
    block
}

fn is_job_header(line: &str) -> bool {
    let line = line.trim_end();
    let key = line.trim_start();
    line.len() - key.len() == 2 && key.ends_with(':') && !key.starts_with('#')
}

/// The quoted value of the first `key = "value"` line of a TOML file.
fn quoted_value<'a>(toml: &'a str, key: &str) -> &'a str {
    toml.lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(key)?
                .trim_start()
                .strip_prefix('=')?
                .trim()
                .strip_prefix('"')?
                .strip_suffix('"')
        })
        .unwrap_or_else(|| panic!("no `{key} = \"...\"` line"))
}

#[test]
fn release_workflow_only_runs_for_explicit_releases() {
    assert!(RELEASE_WORKFLOW.contains("release:"));
    assert!(RELEASE_WORKFLOW.contains("types: [published]"));
    assert!(RELEASE_WORKFLOW.contains("workflow_dispatch"));
    assert!(!RELEASE_WORKFLOW.contains("pull_request:"));
    assert!(!RELEASE_WORKFLOW.contains("branches:"));
}

#[test]
fn release_workflow_builds_rust_split_platform_parity() {
    assert!(DIST_WORKSPACE.contains("aarch64-apple-darwin"));
    assert!(DIST_WORKSPACE.contains("x86_64-apple-darwin"));
    assert!(DIST_WORKSPACE.contains("aarch64-unknown-linux-gnu"));
    assert!(DIST_WORKSPACE.contains("x86_64-unknown-linux-gnu"));
    assert!(DIST_WORKSPACE.contains("x86_64-pc-windows-msvc"));
}

#[test]
fn release_workflow_uses_cargo_dist_installers() {
    assert!(RELEASE_WORKFLOW.contains("cargo-dist-installer.sh"));
    assert!(RELEASE_WORKFLOW.contains("dist host --steps=create"));
    assert!(RELEASE_WORKFLOW.contains("dist build"));
}

#[test]
fn release_workflow_uploads_and_attests_release_assets() {
    assert!(RELEASE_WORKFLOW.contains("actions/attest-build-provenance"));
    assert!(RELEASE_WORKFLOW.contains("artifacts/*.sha256"));
    assert!(RELEASE_WORKFLOW.contains("artifacts/sha256.sum"));
    assert!(RELEASE_WORKFLOW.contains("gh release edit"));
    assert!(RELEASE_WORKFLOW.contains("gh release upload"));
}

#[test]
fn release_workflow_calls_the_crate_publish_workflow_as_dist_generates_it() {
    // gwz-core dev-docs/GwzCratesIoPlan.md S3.2: the job dist 0.31.0 generates
    // from dist-workspace.toml, added by hand to this hand-constrained file.
    // A called workflow can only narrow the token permissions its caller job
    // grants, and a job-level block sets every scope it omits to none, so the
    // `github-custom-job-permissions` entry replaces dist's default publish-job
    // grant (id-token and packages) with the two scopes publish-crate.yml uses.
    let job = job_lines(RELEASE_WORKFLOW, "custom-publish-crate");
    let has = |expected: &str| job.iter().any(|line| line.trim() == expected);
    for expected in [
        "needs:",
        "- plan",
        "- host",
        "if: ${{ !fromJson(needs.plan.outputs.val).announcement_is_prerelease || fromJson(needs.plan.outputs.val).publish_prereleases }}",
        "uses: ./.github/workflows/publish-crate.yml",
        "plan: ${{ needs.plan.outputs.val }}",
        "secrets: inherit",
        "permissions:",
        "\"contents\": \"read\"",
        "\"id-token\": \"write\"",
    ] {
        assert!(has(expected), "custom-publish-crate lost `{expected}`");
    }
    assert!(!has("\"packages\": \"write\""));
    assert!(has_line(
        DIST_WORKSPACE,
        "publish-jobs = [\"./publish-crate\"]"
    ));
    assert!(has_line(
        DIST_WORKSPACE,
        "github-custom-job-permissions = { \"publish-crate\" = { contents = \"read\", id-token = \"write\" } }"
    ));
}

#[test]
fn crate_publish_workflow_publishes_gwz_by_trusted_publishing_alone() {
    // D5 and S3.2: release.yml is the only caller, so the OIDC `workflow_ref`
    // crates.io matches its trusted publisher against always names release.yml.
    for expected in [
        "workflow_call:",
        "plan:",
        "runs-on: ubuntu-24.04",
        "environment: crates-io",
        "contents: read",
        "id-token: write",
        "concurrency:",
        "group: crates-io-publish",
        "cancel-in-progress: false",
        "ref: ${{ fromJson(inputs.plan).announcement_tag }}",
        "persist-credentials: false",
        "uses: rust-lang/crates-io-auth-action@v1",
        "CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}",
        "run: cargo publish -p gwz --locked",
    ] {
        assert!(
            has_line(PUBLISH_CRATE_WORKFLOW, expected),
            "publish-crate.yml lost `{expected}`"
        );
    }
    assert!(!has_line(PUBLISH_CRATE_WORKFLOW, "workflow_dispatch:"));
    // No fallback credential: a failed token exchange fails the job.
    assert!(!PUBLISH_CRATE_WORKFLOW.contains("continue-on-error"));
    assert!(!PUBLISH_CRATE_WORKFLOW.contains("secrets.CARGO_REGISTRY_TOKEN"));
    assert!(!PUBLISH_CRATE_WORKFLOW.contains("${{ secrets"));
    // cargo's verification build stays on.
    assert!(!PUBLISH_CRATE_WORKFLOW.contains("--no-verify"));
}

#[test]
fn crate_publish_workflow_waits_for_gwz_core_and_skips_a_published_gwz() {
    // The verification build resolves the exact gwz-core pin from crates.io,
    // and a version crates.io already holds is skipped, so a re-run is safe.
    assert!(
        PUBLISH_CRATE_WORKFLOW.contains("https://crates.io/api/v1/crates/gwz-core/$CORE_VERSION")
    );
    assert!(PUBLISH_CRATE_WORKFLOW.contains("deadline=$((SECONDS + 900))"));
    assert!(PUBLISH_CRATE_WORKFLOW.contains("https://crates.io/api/v1/crates/gwz/$VERSION"));
    assert!(PUBLISH_CRATE_WORKFLOW.contains("--user-agent \"$USER_AGENT\""));
    assert_eq!(
        2,
        count_lines(
            PUBLISH_CRATE_WORKFLOW,
            "if: steps.present.outputs.publish == 'true'"
        ),
        "authentication and the upload must both wait on the skip decision"
    );
}

#[test]
fn crate_publish_workflow_builds_with_the_toolchain_gwz_declares() {
    let channel = quoted_value(RUST_TOOLCHAIN, "channel");
    let rust_version = quoted_value(CARGO_MANIFEST, "rust-version");
    assert!(
        channel == rust_version || channel.starts_with(&format!("{rust_version}.")),
        "rust-toolchain.toml channel {channel} is not rust-version {rust_version}"
    );
    assert!(has_line(
        PUBLISH_CRATE_WORKFLOW,
        &format!("RUST_TOOLCHAIN: \"{channel}\"")
    ));
}

#[test]
fn push_and_pull_request_ci_runs_the_release_script_unit_tests() {
    // S3.2 wires in S3.1's unit tests. This is gwz-cli's only push and
    // pull-request workflow (the Rust driver is tested on the workspace
    // tuple by gwz-dev's root push), and it names its unittest modules one
    // by one, so a module nobody adds here never runs in CI.
    assert!(has_line(CI_WORKFLOW, "pull_request:"));
    assert!(has_line(CI_WORKFLOW, "branches: [main]"));
    assert!(CI_WORKFLOW.contains("uses: actions/setup-python@v5"));
    assert!(has_line(
        CI_WORKFLOW,
        "run: python -m unittest scripts/test_release.py -v"
    ));
}
