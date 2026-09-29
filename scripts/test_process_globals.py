#!/usr/bin/env python3
"""gwz-cli's process-global state check (GwzCoreSessionDesign O9, §5.7).

gwz-core's `scripts/checks/check_process_globals.py` scans this crate's
production code from `src/lib.rs` and `src/main.rs`, across every platform
branch. A new static, thread-local, environment read or child process that
`scripts/process_globals_allowlist.json` does not list fails here, and so does
a listed entry that stops matching, so the list only shrinks. The same run
fails on production code under a crate's `tests/` directory.

gwz-cli builds against the gwz-core checkout beside it (the `path` dependency
on `main`), so the checker is always there; CI checks gwz-core out beside
gwz-cli for this test. gwz-py runs the same checker the same way
(`gwz-py/src/tests/test_process_globals.py`). Python 3.10 or later:
`python -m unittest scripts/test_process_globals.py`.

Accepted coupling (S-2 of the 2026-09-29 cleanup's Safety review): CI runs the
checker from gwz-core's `main`, unpinned, as gwz-py's CI does. gwz-core must
therefore reach GitHub before gwz-cli, or with it, and a checker change on
gwz-core's `main` changes this gate without any change here.
"""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT.parent / "gwz-core" / "scripts" / "checks" / "check_process_globals.py"
ALLOWLIST = ROOT / "scripts" / "process_globals_allowlist.json"


class ProcessGlobals(unittest.TestCase):
    def test_the_cli_adds_no_process_global_state(self) -> None:
        self.assertTrue(CHECKER.is_file(), f"gwz-core checker not found at {CHECKER}")
        result = subprocess.run(
            [sys.executable, str(CHECKER), "--repo", str(ROOT), "--allowlist", str(ALLOWLIST)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
