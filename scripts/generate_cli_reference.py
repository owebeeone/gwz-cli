#!/usr/bin/env python3
"""Generate or check docs/CLI.md from the gwz Clap command definitions.

This script is the supported cross-platform entrypoint for maintaining the
generated CLI reference. It delegates the actual Clap rendering to the Rust
example so the output is produced from the same parser used by the binary.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


REPO = Path(__file__).resolve().parent.parent


def fail(message: str) -> None:
    print(f"generate_cli_reference: error: {message}", file=sys.stderr)
    raise SystemExit(1)


def run_generator(mode: str | None) -> int:
    cargo = shutil.which("cargo")
    if cargo is None:
        fail("`cargo` not found on PATH")

    cmd = [cargo, "run", "-q", "--example", "generate_cli_docs", "--"]
    if mode is not None:
        cmd.append(mode)
    result = subprocess.run(cmd, cwd=REPO)
    return result.returncode


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate or check gwz-cli/docs/CLI.md from Clap help."
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--write",
        action="store_true",
        help="rewrite docs/CLI.md from current Clap definitions",
    )
    group.add_argument(
        "--check",
        action="store_true",
        help="fail if docs/CLI.md is not current",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.write:
        return run_generator("--write")
    if args.check:
        return run_generator("--check")
    return run_generator(None)


if __name__ == "__main__":
    raise SystemExit(main())
