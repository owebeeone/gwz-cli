# gwz

`gwz` is the command-line driver for GWZ multi-repository workspaces.

The CLI parses argv, builds GWZ requests, calls `gwz-core`, and renders
human-readable, porcelain, JSON, or JSONL responses. User-facing command docs
live in [docs/README.md](docs/README.md).

Hosted docs:
https://github.com/owebeeone/gwz-cli/tree/main/docs

## Install

Install the latest release on macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Install the latest release on Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

Install from source:

```sh
cargo install --git https://github.com/owebeeone/gwz-cli
```

See [docs/Install.md](docs/Install.md) for pinned installs, smoke tests,
checksums, attestations, and local development usage.

## Commands

The implemented command surface is documented in [docs/CLI.md](docs/CLI.md) and
the per-command pages under [docs/commands/](docs/commands/).

For terminal help:

```sh
gwz --help
gwz help status
```

When working from source:

```sh
cargo run -q -p gwz -- --help
cargo run -q -p gwz -- help status
```

## Development

```sh
cargo fmt
cargo test
cargo fmt --check
python scripts/generate_cli_reference.py --check
cargo run -q -p gwz -- --version
```

## License

`gwz` is licensed under GPL-2.0-only, the same license family used by Git.
