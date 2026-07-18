# Releases

GWZ releases are distributed through GitHub Releases for the `gwz-cli`
repository:

https://github.com/owebeeone/gwz-cli/releases

The [hosted documentation](https://owebeeone.github.io/gwz-cli/) is built from
the tag of the most recently published release, so its command model matches
the released CLI rather than unreleased work on `main`.

## Unreleased Compatibility Notes

- First-class merge JSON and JSONL include the complete current merge-response
  key set. Reserved lifecycle fields are empty or null until the corresponding
  features are available. Structured errors include `target_kind` and retain
  member id/path context even for whole-operation preflight failures. Because
  GWZ is pre-1.0, strict consumers must tolerate additive keys.
- Merge and `pull --sync merge` reject source and target commits with unrelated
  histories, matching Git porcelain. GWZ does not implicitly allow unrelated
  histories.
- The current merge implementation advances the workspace lock for verified
  clean participants even if a later unexpected failure halts the batch.
  Coordinated continue and abort are not yet available.
- Merge commits currently use the unquoted message
  `Merge <source> into <target-branch>` without GWZ operation trailers.

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
