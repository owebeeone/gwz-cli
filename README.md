# gwz

`gwz` is the primary command-line interface for coordinating multiple ordinary
Git repositories as one reproducible GWZ workspace. The root records the
workspace composition and exact state while every member remains a normal Git
repository.

Install this repository's CLI for normal terminal use. It is the primary and
most thoroughly tested GWZ command implementation; applications and services
can instead embed the message-driven [`gwz-core`](https://github.com/owebeeone/gwz-core).

## Install And Start

Install the latest release on macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Install the latest release on Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

Then follow the [Quick Start](https://owebeeone.github.io/gwz-cli/QuickStart/).
The [full user documentation](https://owebeeone.github.io/gwz-cli/) covers
workflows, member lifecycle, troubleshooting, and generated command reference.
Read [Why GWZ](https://github.com/owebeeone/gwz-core/blob/main/docs/WhyGwz.md)
for the product model and its relationship to Git and repository fan-out tools.

To build or change GWZ itself, clone the coordinated
[`gwz-dev`](https://github.com/owebeeone/gwz-dev) workspace.

## Command Help

```sh
gwz --help
gwz help status
```

The implemented surface is documented in the generated
[CLI reference](docs/CLI.md) and the [command pages](docs/commands/status.md).
The CLI parses arguments, sends typed requests to `gwz-core`, and renders human,
porcelain, JSON, or JSONL output.

See [Install](docs/Install.md) for pinned installs, source installs, checksums,
attestations, and smoke tests.

## Development

When this repository is checked out inside `gwz-dev`:

```sh
cargo fmt
cargo test
cargo fmt --check
python scripts/generate_cli_reference.py --check
cargo run -q -p gwz -- --version
```

## License

`gwz` is licensed under GPL-2.0-only, the same license family used by Git.
