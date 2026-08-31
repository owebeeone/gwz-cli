# Releases

GWZ releases are distributed through GitHub Releases for the `gwz-cli`
repository:

https://github.com/owebeeone/gwz-cli/releases

The [hosted documentation](https://owebeeone.github.io/gwz-cli/) is built from
the tag of the most recently published release, so its command model matches
the released CLI rather than unreleased work on `main`.

## Unreleased Compatibility Notes

- `gwz log` adds one newest-first history across the workspace root and selected
  member repositories, with coordinated-marker and conservative heuristic
  coalescing, revision/snapshot/lock ranges, six filters, compact and full human
  rendering, and the dedicated `gwz.log/v0` JSON/JSONL record schema. The
  Python API and `gwz-py log` expose the same records and rendering contract.
- Structural workspace commands refuse uncommitted hand edits to `gwz.conf/`
  (the machine-managed manifest and lock). The refusal names the sanctioned
  `gwz repo` verbs and the acceptance path (`gwz init --update --force`);
  states produced by git itself — clone, pull, branch switch — reconcile
  silently, and read-only/list commands never gate. Workspace bootstrap also
  emits a machine-managed banner on `gwz.yml`, records digests in
  `gwz.conf/markers/conf-integrity.yml`, and writes or merges a
  `.claude/settings.json` deny rule (`Edit(/gwz.conf/**)`) for agent sessions
  started at the workspace root.
- A checked-artifact private anchor directory holding a foreign,
  non-canonical retired-anchor rendering (for example
  `.ca1-anchor-retired-007`) now refuses operations on that family until the
  foreign file is removed; such names were previously adopted silently. The
  canonical rendering is unpadded (`.ca1-anchor-retired-7`).
- First-class merge JSON and JSONL include the complete current merge-response
  key set, including finalization progress. Structured errors include
  `target_kind` and retain member id/path context even for whole-operation
  preflight failures. Durable record compatibility errors also include typed
  `record_context` rather than encoding merge id, schema/version, required
  wave, or legacy mode only in prose. Because GWZ is pre-1.0, strict consumers
  must tolerate additive keys.
- Merge status rows expose durable pending-action reconciliation as
  `NotStarted`, `ExpectedConflict`, `CompletedExactly`, or `Ambiguous`.
  Ambiguity is also reported as dedicated structured drift and remains
  mutation-blocking.
- Merge and `pull --sync merge` reject source and target commits with unrelated
  histories, matching Git porcelain. GWZ does not implicitly allow unrelated
  histories.
- While a coordinated merge is open, the accepted workspace lock remains the
  exact pre-merge baseline. Clean, conflicted, failed, and unattempted outcomes
  are retained in the local durable operation record rather than published as
  a partial composition.
- Merge commits use the quoted default message
  `Merge '<source>' into '<target-branch>'`, or the body supplied with `-m`,
  with mandatory `GWZ-Merge-ID` and `GWZ-Operation-ID` identity lines. The
  exact final message is frozen before mutation and survives restart and
  conflict resolution. A request that creates a commit supplies its
  author/committer identity when present; otherwise the target repository
  identity is used.
- Coordinated merge start, dry-run, status, continue, and safe abort are
  available together. Successful changed merges publish a checked root
  composition commit; interrupted finalization resumes idempotently.
  Recovery must not substitute raw `git merge --abort` for the coordinated
  operation.

## Install Latest

macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

## Install A Pinned Version

Replace `latest` with a concrete tag:

```text
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.sh
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.ps1
```

## Verify Assets

Release assets are checksummed and have GitHub artifact attestations. For
stronger verification:

1. Download the release archive and checksum file.
2. Compare the archive SHA-256 with the checksum.
3. Verify the GitHub artifact attestation.
4. Run `gwz --version`.
5. Run `gwz --help`.

## Smoke Test A Unix Installer

```sh
tmp="$(mktemp -d)"

curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh \
  -o "${tmp}/gwz-installer.sh"

GWZ_UNMANAGED_INSTALL="${tmp}/bin" \
GWZ_NO_MODIFY_PATH=1 \
sh "${tmp}/gwz-installer.sh"

"${tmp}/bin/gwz" --version
"${tmp}/bin/gwz" --help
```

## Smoke Test A Windows Installer

```powershell
$ErrorActionPreference = "Stop"

$tmp = Join-Path $env:TEMP "gwz-test-$([guid]::NewGuid())"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

$installer = Join-Path $tmp "gwz-installer.ps1"
Invoke-WebRequest `
  "https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1" `
  -OutFile $installer

$env:GWZ_UNMANAGED_INSTALL = Join-Path $tmp "bin"
$env:GWZ_NO_MODIFY_PATH = "1"

Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& $installer

$exe = Join-Path $env:GWZ_UNMANAGED_INSTALL "gwz.exe"
& $exe --version
& $exe --help
```

## Local Release Checks

From the development workspace:

```sh
cargo fmt --check
cargo test
cargo run -q -p gwz -- --help
python scripts/generate_cli_reference.py --check
```

When changing command docs, inspect the generated reference and spot-check
command help for the affected commands:

```sh
cargo run -q -p gwz -- help status
cargo run -q -p gwz -- help tag
```

`scripts/release.py` runs the generated CLI reference check by default before it
commits the release worktree. If the release must proceed while docs are being
reconciled separately, pass `--no-doc-check`; otherwise update the reference
with:

```sh
python scripts/generate_cli_reference.py --write
```
