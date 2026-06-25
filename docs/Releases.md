# Releases

GWZ releases are distributed through GitHub Releases for the `gwz-cli`
repository:

https://github.com/owebeeone/gwz-cli/releases

The documentation entry point for released CLI docs is:

https://github.com/owebeeone/gwz-cli/tree/main/docs

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
https://github.com/owebeeone/gwz-cli/releases/download/v0.3.0/gwz-installer.sh
https://github.com/owebeeone/gwz-cli/releases/download/v0.3.0/gwz-installer.ps1
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
```

When changing docs, also verify command help for each documented command:

```sh
cargo run -q -p gwz -- help status
cargo run -q -p gwz -- help tag
```
