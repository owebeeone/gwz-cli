# Releasing gwz-cli

gwz-cli is the CLI binary. It depends on **gwz-core**, and that dependency is the only
thing that differs between branches:

- **`main` (dev):** `gwz-core = { path = "../gwz-core" }` — builds against the local
  sibling checkout, so you must have `../gwz-core` checked out next to this repo. The
  crate `version` carries a `-dev` suffix. **Do not cut release tags here.**
- **`release`:** `gwz-core = "=X.Y.Z"` — an exact pin on the gwz-core release published on
  crates.io, so the branch is standalone-buildable and reproducible, and the `gwz` package
  can itself be published (crates.io accepts neither git nor path-only dependencies).
  **Release tags are cut off `release`.** Releases up to 1.0.11 pinned
  `gwz-core = { git = "https://github.com/owebeeone/gwz-core", tag = "vX.Y.Z" }` instead;
  `scripts/release.py` moves that line to the registry pin the first time it meets it.

## Process

1. **Release the matching gwz-core first, including its crates.io publish job.** Tag it off
   its `main` (see [gwz-core/RELEASE.md](../gwz-core/RELEASE.md)) and note the new tag
   `vX.Y.Z`. gwz-core's release workflow publishes gwz-core and its internal crates to
   crates.io; that run must finish before this release, because the `release` branch
   resolves gwz-core `X.Y.Z` from crates.io, not from the tag.
2. `git switch release && git merge main`.
3. Reconcile the one intentional branch difference: the `gwz-core` dependency must stay the
   **exact registry pin** and name the gwz-core release that contains the code this release
   relies on: `gwz-core = "=X.Y.Z"` (NOT the `path` form that lives on `main`).
4. Set the real release `version` (drop the `-dev` suffix), run `cargo generate-lockfile` to
   refresh `Cargo.lock` (gwz-core and its internal `gwz-*` crates must come from
   `registry+https://github.com/rust-lang/crates.io-index`), then `cargo test` and
   `cargo package --locked`, which builds the packaged `gwz` against crates.io alone.
5. Commit and tag **off `release`**: `git tag vX.Y.Z`, the same tag as gwz-core's unless only
   the CLI advances (`--core-tag` below). Push `release` and the tag, then publish the GitHub
   release for the tag, which builds the binaries and publishes the crate (see below).
6. **The release is not done until PyPI moves too.** Release gwz-py at the same
   `vX.Y.Z` (see [gwz-py/RELEASE.md](../gwz-py/RELEASE.md) — same
   `scripts/release.py` interface), then publish its GitHub release for the tag,
   which triggers the PyPI trusted publish. The v0.11.1 cut initially missed this
   channel precisely because no forward pointer existed here (2026-08-29).

`python scripts/release.py vX.Y.Z` does steps 2 to 5 in a temporary worktree (pushing only
with `--push`; add `--core-tag` when only the CLI advances) and advances `release` only when
every check passes. Before it creates anything it checks that the gwz-core tag exists at
`--core-url` (default `https://github.com/owebeeone/gwz-core`), because the parity tests read
gwz-core's fixtures from a clone at that tag, and then waits for gwz-core `X.Y.Z` on
crates.io, polling for up to `--registry-timeout` seconds (default 900). A run started while
gwz-core's crates.io publish job is still going therefore carries on once the version lands,
and one started before the core is published at all fails without touching `release`. The
commit it writes reads `chore(release): gwz-cli X.Y.Z (pins gwz-core X.Y.Z from crates.io)`.
`python scripts/test_release.py` runs the unit tests of its helpers.

## Publishing the gwz crate

Publishing the GitHub release for `vX.Y.Z` runs the dist-generated `.github/workflows/release.yml`.
After its `host` job has put the binaries on the GitHub release, dist's `custom-publish-crate` job
runs `.github/workflows/publish-crate.yml` (listed as `publish-jobs = ["./publish-crate"]` in
`dist-workspace.toml`). That job waits for gwz-core `X.Y.Z` on crates.io, skips a `gwz` version
crates.io already holds, and otherwise runs `cargo publish -p gwz --locked`, authenticated only by
Trusted Publishing. Prereleases skip the crate publish. If the job fails, re-run that job in the
Release run; a version already on crates.io is skipped.

One-time requirement: crates.io needs a trusted publisher for `gwz` with owner `owebeeone`,
repository `gwz-cli`, workflow `release.yml` and environment `crates-io`. The workflow is
`release.yml` rather than `publish-crate.yml` because crates.io takes it from the calling workflow
named in the GitHub OIDC token.

## The merge gotcha

`main` always carries the `path` dependency; `release` always carries the exact registry pin
`gwz-core = "=X.Y.Z"`. As long as `main` never edits that dependency line, merging `main` →
`release` resolves it cleanly to release's form. The release-time job is to make sure the pin
names the gwz-core version that actually contains the gwz-core code this gwz-cli release uses
(per step 1), and that crates.io already serves it — bump it every release. If `main` does
edit the line, the merge brings `main`'s `path` form (or a conflict) into `release`; the
release script refuses that rather than rewriting it, since a path dependency builds only
inside `gwz-dev` and cannot be published.

## Slow architecture tests

The source-mutation/compiler suites live in gwz-core and are manual-only:
`python ../gwz-core/scripts/run_compiler_tests.py`. They are not part of CLI
release checks or automatic CI. See gwz-core's release documentation.
