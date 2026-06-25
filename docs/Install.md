# Install

Install the latest GitHub Release on macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Install the latest GitHub Release on Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

The `latest` release URL resolves to the newest non-prerelease GitHub Release.
For pinned installs, replace `latest` with a concrete tag:

```text
https://github.com/owebeeone/gwz-cli/releases/download/v0.3.0/gwz-installer.sh
https://github.com/owebeeone/gwz-cli/releases/download/v0.3.0/gwz-installer.ps1
```

## Install From Source

Users with Rust can install from the repository:

```sh
cargo install --git https://github.com/owebeeone/gwz-cli
```

For local development inside the source workspace, run the package directly:

```sh
cargo run -q -p gwz -- --version
cargo run -q -p gwz -- --help
```

Use the same form for command help:

```sh
cargo run -q -p gwz -- help status
```

## Verify A Release Asset

Release assets are checksummed and have GitHub artifact attestations. The
installer scripts are convenience entry points; users who need stronger supply
chain verification should download the binary archive, verify the attestation,
compare the SHA-256 checksum, and then install the binary.

Typical verification flow:

1. Download the release archive, checksum file, and attestation from the GitHub
   Release.
2. Compare the archive hash with the published checksum.
3. Verify the artifact attestation with GitHub tooling for the release asset.
4. Run `gwz --version` and `gwz --help`.

## Smoke Test Installers

Test the Unix installer without modifying `PATH`:

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

Test the Windows installer without modifying `PATH`:

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

## Documentation

The hosted documentation entry point is:
https://github.com/owebeeone/gwz-cli/tree/main/docs
