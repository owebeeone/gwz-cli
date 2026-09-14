#!/usr/bin/env python3
"""Unit tests for the gwz-cli release script on the registry core (gwz-core
dev-docs/GwzCratesIoPlan.md D6, step S3.1).

`scripts/release.py` pins gwz-core on the `release` branch as `gwz-core = "=X.Y.Z"` from
crates.io instead of the git tag it used through 1.0.11. What decides whether a release may go
ahead is checked here without a network, a git repository or a cargo:

- `reconcile_cargo_toml`, over copies of this repository's own Cargo.toml and BUILD.bazel with
  the gwz-core line set to each shape a release branch can hold: the old git + tag pin
  (migrated once), the registry pin (moved to the new version, or left alone when current), and
  the shapes the script must refuse.
- `verify_locked_registry_pin`, over synthetic Cargo.lock files: a passing lock, a git-sourced
  core, a wrong version, a missing checksum, the workspace path core, two cores, and internal
  crates from somewhere other than crates.io.
- `wait_for_registry_core`, with the crates.io lookup, the sleep and the clock injected: found
  at once, found after polling, transient answers, an answer polling cannot change, and the
  timeout; and `registry_status` with `urlopen` replaced, to prove the User-Agent crates.io
  insists on is sent and that errors become statuses.
- `verify_cli_package`, with its cargo call replaced: only an archive this run produced passes.

Style follows gwz-core's `scripts/test_release_bump.py`: load the script by path, drive its
helpers directly, assert on what an operator would read. Python 3.10 or later:
`python scripts/test_release.py`.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import re
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
RELEASE_PATH = ROOT / "scripts" / "release.py"
SPEC = importlib.util.spec_from_file_location("gwz_cli_release", RELEASE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load release script")
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


MAIN_DEPENDENCY = 'gwz-core = { path = "../gwz-core" }'
GIT_TAG_PIN = 'gwz-core = { git = "https://github.com/owebeeone/gwz-core", tag = "v1.0.11" }'
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
GIT_SOURCE = "git+https://github.com/owebeeone/gwz-core?tag=v1.0.12#" + "a" * 40
CORE_URL = "https://crates.io/api/v1/crates/gwz-core/1.0.12"


def quietly():
    """Swallow the script's progress lines."""
    return contextlib.redirect_stdout(io.StringIO())


def make_worktree(parent: Path, dependency: str, *, version: str = "1.0.11") -> Path:
    """This repository's Cargo.toml and BUILD.bazel as a release branch at `version` holds them."""
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if manifest.count(MAIN_DEPENDENCY) != 1:
        raise AssertionError(f"Cargo.toml on main no longer carries `{MAIN_DEPENDENCY}` once")
    manifest = manifest.replace(MAIN_DEPENDENCY, dependency)
    manifest, count = re.subn(
        r'^version = "[^"]*"', f'version = "{version}"', manifest, count=1, flags=re.M
    )
    if count != 1:
        raise AssertionError("Cargo.toml has no package version line")
    bazel, count = re.subn(
        r'^(\s*version\s*=\s*)"[^"\n]*"',
        rf'\g<1>"{version}"',
        (ROOT / "BUILD.bazel").read_text(encoding="utf-8"),
        flags=re.M,
    )
    if count != 2:
        raise AssertionError("BUILD.bazel no longer carries two artifact versions")
    worktree = Path(tempfile.mkdtemp(dir=parent)) / "gwz-cli"
    worktree.mkdir()
    (worktree / "Cargo.toml").write_text(manifest, encoding="utf-8", newline="\n")
    (worktree / "BUILD.bazel").write_text(bazel, encoding="utf-8", newline="\n")
    return worktree


def snapshot(worktree: Path) -> dict[str, bytes]:
    return {name: (worktree / name).read_bytes() for name in ("Cargo.toml", "BUILD.bazel")}


def package_version(manifest: str) -> str:
    match = re.search(r'^version = "([^"]*)"', manifest, re.M)
    if match is None:
        raise AssertionError("Cargo.toml has no package version line")
    return match.group(1)


def core_lines(manifest: str) -> list[str]:
    return [line for line in manifest.splitlines() if line.startswith("gwz-core")]


def bazel_versions(worktree: Path) -> list[str]:
    bazel = (worktree / "BUILD.bazel").read_text(encoding="utf-8")
    return re.findall(r'^\s*version\s*=\s*"([^"]*)"', bazel, re.M)


def lock_entry(name, version, *, source=CRATES_IO, checksum="5" * 64, dependencies=()) -> str:
    lines = ["[[package]]", f'name = "{name}"', f'version = "{version}"']
    if source is not None:
        lines.append(f'source = "{source}"')
    if checksum is not None:
        lines.append(f'checksum = "{checksum}"')
    if dependencies:
        lines.append("dependencies = [")
        lines.extend(f' "{dependency}",' for dependency in dependencies)
        lines.append("]")
    return "\n".join(lines)


CLAP = lock_entry("clap", "4.5.48", dependencies=("clap_builder",))
CLI = lock_entry("gwz", "1.0.12", source=None, checksum=None, dependencies=("clap", "gwz-core"))
CORE = lock_entry("gwz-core", "1.0.12", dependencies=("gwz-family-model", "gwz-repo-contract"))
INTERNALS = (lock_entry("gwz-family-model", "0.0.3"), lock_entry("gwz-repo-contract", "0.0.3"))


class ReleaseTestCase(unittest.TestCase):
    """A scratch directory per test, and a way to read what a refusal said."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.parent = Path(self.temporary.name)

    def refusal(self, call) -> str:
        stderr = io.StringIO()
        with quietly(), contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit):
            call()
        return stderr.getvalue()


class ReconcileTests(ReleaseTestCase):
    """The release branch's manifest, reconciled onto the registry pin."""

    def reconcile(self, worktree: Path, core_version: str, version: str) -> bool:
        with quietly():
            return release.reconcile_cargo_toml(worktree, core_version, version)

    def test_the_git_tag_pin_is_migrated_to_the_exact_registry_pin(self) -> None:
        reversed_keys = (
            'gwz-core = { tag = "v1.0.11", git = "https://github.com/owebeeone/gwz-core" }'
        )
        for pin in (GIT_TAG_PIN, reversed_keys):
            with self.subTest(pin=pin):
                worktree = make_worktree(self.parent, pin)
                before = (worktree / "Cargo.toml").read_text(encoding="utf-8")
                self.assertTrue(self.reconcile(worktree, "1.0.12", "1.0.12"))
                after = (worktree / "Cargo.toml").read_text(encoding="utf-8")
                self.assertEqual(['gwz-core = "=1.0.12"'], core_lines(after))
                self.assertEqual("1.0.12", package_version(after))
                self.assertEqual(["1.0.12", "1.0.12"], bazel_versions(worktree))
                # Only the two reconciled lines move: the metadata, the include list, the
                # profiles and every comment stay byte-identical.
                self.assertEqual(len(before.splitlines()), len(after.splitlines()))
                moved = [
                    (old, new)
                    for old, new in zip(before.splitlines(), after.splitlines())
                    if old != new
                ]
                self.assertEqual(
                    [('version = "1.0.11"', 'version = "1.0.12"'), (pin, 'gwz-core = "=1.0.12"')],
                    moved,
                )

    def test_a_registry_pin_moves_to_the_new_core_version(self) -> None:
        worktree = make_worktree(self.parent, 'gwz-core = "=1.0.12-rc.1"', version="1.0.12-rc.1")
        self.assertTrue(self.reconcile(worktree, "1.0.12", "1.0.12"))
        after = (worktree / "Cargo.toml").read_text(encoding="utf-8")
        self.assertEqual(['gwz-core = "=1.0.12"'], core_lines(after))
        self.assertEqual("1.0.12", package_version(after))
        self.assertEqual(["1.0.12", "1.0.12"], bazel_versions(worktree))

    def test_a_reconciled_release_branch_is_left_byte_for_byte_alone(self) -> None:
        worktree = make_worktree(self.parent, 'gwz-core = "=1.0.12"', version="1.0.12")
        before = snapshot(worktree)
        self.assertFalse(self.reconcile(worktree, "1.0.12", "1.0.12"))
        self.assertEqual(before, snapshot(worktree))

    def test_the_cli_may_advance_alone_or_pin_a_release_candidate_core(self) -> None:
        for core_version, version in (("1.0.12", "1.0.13"), ("1.0.12-rc.1", "1.0.12-rc.1")):
            with self.subTest(core=core_version, cli=version):
                worktree = make_worktree(self.parent, GIT_TAG_PIN)
                self.assertTrue(self.reconcile(worktree, core_version, version))
                after = (worktree / "Cargo.toml").read_text(encoding="utf-8")
                self.assertEqual([f'gwz-core = "={core_version}"'], core_lines(after))
                self.assertEqual(version, package_version(after))

    def test_any_other_dependency_shape_is_refused_and_nothing_is_written(self) -> None:
        shapes = (
            MAIN_DEPENDENCY,
            'gwz-core = "1.0.12"',
            'gwz-core = { version = "=1.0.12" }',
            'gwz-core = { path = "../gwz-core", version = "=1.0.12" }',
            'gwz-core = { git = "https://github.com/owebeeone/gwz-core", branch = "main" }',
            'gwz-core = { git = "https://github.com/owebeeone/gwz-core", rev = "a11dce4" }',
            'gwz-core = { git = "https://github.com/owebeeone/gwz-core", tag = "v1.0.11", '
            'features = ["x"] }',
        )
        for shape in shapes:
            with self.subTest(shape=shape):
                worktree = make_worktree(self.parent, shape)
                before = snapshot(worktree)
                message = self.refusal(
                    lambda: release.reconcile_cargo_toml(worktree, "1.0.12", "1.0.12")
                )
                self.assertIn("unexpected gwz-core dependency on the release branch", message)
                self.assertIn(shape, message)
                self.assertEqual(before, snapshot(worktree))

    def test_a_missing_duplicated_or_misplaced_dependency_is_refused(self) -> None:
        pin = 'gwz-core = "=1.0.11"'
        edits = {
            "missing": lambda text: text.replace(pin + "\n", ""),
            "duplicated": lambda text: text.replace(pin, pin + "\n" + pin),
            "under dev-dependencies": lambda text: text.replace(pin + "\n", "").replace(
                "[dev-dependencies]\n", "[dev-dependencies]\n" + pin + "\n"
            ),
            "as its own table": lambda text: text.replace(pin + "\n", "")
            + '\n[dependencies.gwz-core]\nversion = "=1.0.11"\n',
        }
        for case, edit in edits.items():
            with self.subTest(case=case):
                worktree = make_worktree(self.parent, pin)
                manifest = worktree / "Cargo.toml"
                manifest.write_text(
                    edit(manifest.read_text(encoding="utf-8")), encoding="utf-8", newline="\n"
                )
                before = snapshot(worktree)
                message = self.refusal(
                    lambda: release.reconcile_cargo_toml(worktree, "1.0.12", "1.0.12")
                )
                self.assertIn("exactly one line of [dependencies]", message)
                self.assertEqual(before, snapshot(worktree))


class LockPinTests(ReleaseTestCase):
    """The regenerated lock must take the core and its internal crates from crates.io."""

    def write_lock(self, *entries: str) -> Path:
        worktree = Path(tempfile.mkdtemp(dir=self.parent))
        (worktree / "Cargo.lock").write_text(
            "# This file is automatically @generated by Cargo.\n"
            "# It is not intended for manual editing.\n"
            "version = 4\n\n" + "\n\n".join(entries) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        return worktree

    def refused_lock(self, *entries: str) -> str:
        worktree = self.write_lock(*entries)
        return self.refusal(lambda: release.verify_locked_registry_pin(worktree, "1.0.12"))

    def test_a_lock_taking_the_core_and_its_internals_from_crates_io_passes(self) -> None:
        worktree = self.write_lock(CLAP, CLI, CORE, *INTERNALS)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            release.verify_locked_registry_pin(worktree, "1.0.12")
        self.assertIn(
            "gwz-core 1.0.12 and 2 internal gwz crate(s) from crates.io", output.getvalue()
        )

    def test_a_git_sourced_core_is_refused(self) -> None:
        git_core = lock_entry("gwz-core", "1.0.12", source=GIT_SOURCE, checksum=None)
        message = self.refused_lock(CLAP, CLI, git_core, *INTERNALS)
        self.assertIn("does not take gwz-core 1.0.12 from crates.io", message)
        self.assertIn(f"source={GIT_SOURCE!r}", message)
        self.assertIn("no checksum", message)

    def test_a_different_core_version_is_refused(self) -> None:
        message = self.refused_lock(CLAP, CLI, lock_entry("gwz-core", "1.0.11"), *INTERNALS)
        self.assertIn("(version='1.0.11';", message)

    def test_a_core_entry_without_a_checksum_is_refused(self) -> None:
        core = lock_entry("gwz-core", "1.0.12", checksum=None)
        message = self.refused_lock(CLAP, CLI, core, *INTERNALS)
        self.assertIn("(no checksum;", message)

    def test_the_workspace_path_core_is_refused(self) -> None:
        path_core = lock_entry("gwz-core", "1.0.12", source=None, checksum=None)
        message = self.refused_lock(CLAP, CLI, path_core, *INTERNALS)
        self.assertIn("(source=None, no checksum;", message)

    def test_two_core_entries_are_refused(self) -> None:
        message = self.refused_lock(CLAP, CLI, CORE, lock_entry("gwz-core", "1.0.11"), *INTERNALS)
        self.assertIn("holds 2 gwz-core entries", message)

    def test_internal_crates_from_anywhere_but_crates_io_are_refused_by_name(self) -> None:
        git_internal = lock_entry("gwz-family-model", "0.0.3", source=GIT_SOURCE, checksum=None)
        path_internal = lock_entry("gwz-repo-contract", "0.0.3", source=None, checksum=None)
        message = self.refused_lock(CLAP, CLI, CORE, git_internal, path_internal)
        self.assertIn(f"gwz-family-model 0.0.3 (source={GIT_SOURCE!r})", message)
        self.assertIn("gwz-repo-contract 0.0.3 (source=None)", message)


class FakeRegistry:
    """crates.io's answers in order (the last one repeats), and a clock only sleeping moves."""

    def __init__(self, *answers: int) -> None:
        self.answers = list(answers)
        self.urls: list[str] = []
        self.sleeps: list[float] = []
        self.now = 5000.0

    def probe(self, url: str) -> int:
        self.urls.append(url)
        if len(self.answers) > 1:
            return self.answers.pop(0)
        return self.answers[0]

    def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds

    def monotonic(self) -> float:
        return self.now


class RegistryWaitTests(ReleaseTestCase):
    """Waiting for gwz-core X.Y.Z on crates.io, with the lookup, the sleep and the clock injected."""

    def wait(self, registry: FakeRegistry, *, timeout: float = 900, interval: float = 15) -> str:
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            release.wait_for_registry_core(
                "1.0.12",
                timeout=timeout,
                probe=registry.probe,
                sleep=registry.sleep,
                monotonic=registry.monotonic,
                interval=interval,
            )
        return output.getvalue()

    def test_a_published_core_ends_the_wait_at_once(self) -> None:
        registry = FakeRegistry(200)
        output = self.wait(registry)
        self.assertEqual([CORE_URL], registry.urls)
        self.assertEqual([], registry.sleeps)
        self.assertIn("verified gwz-core 1.0.12 is on crates.io (after 0s)", output)

    def test_an_absent_core_is_polled_until_it_appears(self) -> None:
        registry = FakeRegistry(404, 404, 200)
        output = self.wait(registry)
        self.assertEqual([CORE_URL] * 3, registry.urls)
        self.assertEqual([15, 15], registry.sleeps)
        self.assertIn("not on crates.io yet (HTTP 404); checking again in 15s", output)
        self.assertIn("is on crates.io (after 30s)", output)

    def test_transient_answers_are_retried(self) -> None:
        registry = FakeRegistry(503, 0, 429, 500, 200)
        output = self.wait(registry)
        self.assertEqual(4, len(registry.sleeps))
        self.assertIn("(no answer)", output)
        self.assertIn("(HTTP 429)", output)
        self.assertIn("(HTTP 503)", output)

    def test_an_answer_waiting_cannot_change_fails_without_sleeping(self) -> None:
        for status in (400, 403, 410):
            with self.subTest(status=status):
                registry = FakeRegistry(status)
                message = self.refusal(lambda: self.wait(registry))
                self.assertIn(f"crates.io answered HTTP {status} for {CORE_URL}", message)
                self.assertEqual([CORE_URL], registry.urls)
                self.assertEqual([], registry.sleeps)

    def test_a_core_that_never_appears_times_out(self) -> None:
        registry = FakeRegistry(404)
        message = self.refusal(lambda: self.wait(registry, timeout=60, interval=15))
        self.assertIn(
            "gwz-core 1.0.12 is still not on crates.io after 60s (last answer: HTTP 404)", message
        )
        self.assertIn("crates.io publish job", message)
        self.assertIn("--registry-timeout", message)
        self.assertEqual([15, 15, 15, 15], registry.sleeps)
        self.assertEqual(5, len(registry.urls))

    def test_a_zero_timeout_looks_exactly_once(self) -> None:
        registry = FakeRegistry(404)
        message = self.refusal(lambda: self.wait(registry, timeout=0))
        self.assertIn("still not on crates.io after 0s", message)
        self.assertEqual([CORE_URL], registry.urls)
        self.assertEqual([], registry.sleeps)


class FakeResponse:
    status = 200

    def __enter__(self) -> FakeResponse:
        return self

    def __exit__(self, *exc: object) -> bool:
        return False


class RegistryStatusTests(unittest.TestCase):
    """The one real network call, with `urlopen` replaced."""

    def test_the_lookup_is_a_get_that_names_the_script_in_its_user_agent(self) -> None:
        seen = []

        def urlopen(request, timeout):
            seen.append((request, timeout))
            return FakeResponse()

        with mock.patch.object(release.urllib.request, "urlopen", urlopen):
            self.assertEqual(200, release.registry_status(CORE_URL))
        request, timeout = seen[0]
        self.assertEqual(CORE_URL, request.full_url)
        self.assertEqual("GET", request.get_method())
        agent = request.get_header("User-agent")
        self.assertIsNotNone(agent)
        self.assertIn("scripts/release.py", agent)
        self.assertIn("https://github.com/owebeeone/gwz-cli", agent)
        self.assertEqual(release.HTTP_TIMEOUT, timeout)

    def test_http_errors_become_their_status_and_silence_becomes_zero(self) -> None:
        def not_found(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 404, "Not Found", None, None)

        def unreachable(request, timeout):
            raise urllib.error.URLError("no route to host")

        def too_slow(request, timeout):
            raise TimeoutError("timed out")

        for urlopen, expected in ((not_found, 404), (unreachable, 0), (too_slow, 0)):
            with self.subTest(urlopen=urlopen.__name__):
                with mock.patch.object(release.urllib.request, "urlopen", urlopen):
                    self.assertEqual(expected, release.registry_status(CORE_URL))


class PackagingGateTests(ReleaseTestCase):
    """`cargo package --locked`, with cargo replaced: only an archive this run made passes."""

    def setUp(self) -> None:
        super().setUp()
        self.worktree = self.parent / "gwz-cli"
        self.worktree.mkdir()
        self.target = self.parent / "target"
        self.archive = self.target / "package" / "gwz-1.0.12.crate"
        self.commands: list[tuple[list[str], object]] = []

    def gate(self, *, produce: bool) -> None:
        def run(command, **options):
            self.commands.append((list(command), options.get("cwd")))
            if produce:
                self.archive.parent.mkdir(parents=True, exist_ok=True)
                self.archive.write_bytes(b"crate")

        with mock.patch.object(release, "run", run), mock.patch.object(
            release, "cargo_target_directory", lambda worktree: self.target
        ):
            release.verify_cli_package(self.worktree, "1.0.12")

    def test_the_cli_is_packaged_locked_with_verification(self) -> None:
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.gate(produce=True)
        self.assertEqual(
            [(["cargo", "package", "--locked", "--allow-dirty", "-p", "gwz"], self.worktree)],
            self.commands,
        )
        self.assertIn("packaged and verified gwz-1.0.12.crate", output.getvalue())

    def test_an_archive_left_by_an_earlier_run_does_not_pass(self) -> None:
        self.archive.parent.mkdir(parents=True)
        self.archive.write_bytes(b"stale")
        message = self.refusal(lambda: self.gate(produce=False))
        self.assertIn("finished without producing", message)
        self.assertIn("gwz-1.0.12.crate", message)
        self.assertFalse(self.archive.exists())


class ScriptSurfaceTests(unittest.TestCase):
    """The options, the commit message, and the git-pin helpers that are gone."""

    def test_the_release_commit_names_the_crates_io_pin(self) -> None:
        self.assertEqual(
            "chore(release): gwz-cli 1.0.13 (pins gwz-core 1.0.12 from crates.io)",
            release.release_commit_message("1.0.13", "1.0.12"),
        )

    def test_the_core_url_and_the_registry_timeout_are_options(self) -> None:
        defaults = release.build_parser().parse_args(["v1.0.12"])
        self.assertEqual("https://github.com/owebeeone/gwz-core", defaults.core_url)
        self.assertEqual(900, defaults.registry_timeout)
        chosen = release.build_parser().parse_args(
            ["v1.0.12", "--core-url", "/srv/git/gwz-core.git", "--registry-timeout", "60"]
        )
        self.assertEqual("/srv/git/gwz-core.git", chosen.core_url)
        self.assertEqual(60.0, chosen.registry_timeout)

    def test_the_git_pin_helpers_are_gone(self) -> None:
        source = RELEASE_PATH.read_text(encoding="utf-8")
        for retired in ("def gwz_core_url", "def verify_remote_tag", "def verify_locked_git_pin"):
            self.assertNotIn(retired, source)


if __name__ == "__main__":
    unittest.main()
