#!/usr/bin/env python3
"""gwz-cli's candidate switch inventory (TR2.12; rule (a) of gwz-core dev-docs/GwzTransportReleasePlanAmendment-2.md §3.13).

gwz-core's `scripts/checks/check_candidate_switches.py` finds every site in this
crate where `gwz_transport_candidate` or `gwz_session_candidate` appears in a
cfg, and fails unless the sites equal `scripts/candidate_switch_inventory.txt`
and `build.rs` declares both switches. CI checks gwz-core's main out beside
gwz-cli for this test, as for `test_process_globals.py`, with the same accepted
coupling: a checker change on gwz-core's main changes this gate. Python 3.10 or
later: `python -m unittest scripts/test_candidate_switches.py`.
"""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT.parent / "gwz-core" / "scripts" / "checks" / "check_candidate_switches.py"


class CandidateSwitches(unittest.TestCase):
    def test_the_switch_sites_equal_the_inventory(self) -> None:
        self.assertTrue(CHECKER.is_file(), f"gwz-core checker not found at {CHECKER}")
        result = subprocess.run(
            [sys.executable, str(CHECKER), "--repo", str(ROOT)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_ci_runs_this_test(self) -> None:
        # ci.yml names each test module it runs; a module it does not name never runs in CI.
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("python -m unittest scripts/test_candidate_switches.py", workflow)


if __name__ == "__main__":
    unittest.main()
