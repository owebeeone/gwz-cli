#!/usr/bin/env python3
"""Reconcile the gwz-cli ``release`` branch for a new release, in a throwaway worktree.

gwz-cli ships from its ``release`` branch, which differs from ``main`` in exactly one
way: the gwz-core dependency. On ``main`` it is a local ``path``; on ``release`` it is an
exact crates.io pin, ``gwz-core = "=X.Y.Z"`` (gwz-core dev-docs/GwzCratesIoPlan.md D6), so
the branch builds standalone and the ``gwz`` package itself can be published. This script
automates RELEASE.md steps 2-5 for a given tag:

  1. Verify the gwz-core ``<core-tag>`` exists at ``--core-url``: the CLI/core parity tests
     read gwz-core's fixtures from a clone at that tag. Then wait for gwz-core ``X.Y.Z`` (the
     core tag without its ``v``) on crates.io, polling
     ``https://crates.io/api/v1/crates/gwz-core/X.Y.Z`` for up to ``--registry-timeout``
     seconds. gwz-core's release workflow publishes it; this script never publishes.
  2. In a temporary ``git worktree`` checked out on ``release``, merge ``main`` (the
     gwz-core dep line auto-resolves to release's form because main never edits it -- the
     "merge gotcha" documented in RELEASE.md).
  3. Reconcile Cargo.toml: pin ``gwz-core = "=X.Y.Z"`` and set the package version (and the
     two Bazel artifact versions) to match. A ``release`` branch still carrying the ``git`` +
     ``tag`` pin of 1.0.11 and earlier is migrated to the registry pin; any other shape of the
     dependency is refused.
  4. Regenerate Cargo.lock and assert it takes gwz-core ``X.Y.Z`` and every internal ``gwz-*``
     crate from crates.io (not from git, and not as the workspace ``path`` dependency);
     ``cargo build`` + ``cargo test`` in that standalone worktree; check the generated CLI
     reference is current; then ``cargo package --locked``, which rebuilds the ``gwz`` package
     from its packaged sources against crates.io alone, as ``cargo install gwz`` will -- and
     only then commit the merge as
     ``chore(release): gwz-cli X.Y.Z (pins gwz-core X.Y.Z from crates.io)``.
  5. Tag that commit ``<tag>`` (lightweight). An existing tag is NEVER moved -- if ``<tag>``
     already points elsewhere the script aborts rather than re-pointing a release tag.

The worktree is removed afterward (kept only with ``--keep-worktree``). The ``release``
branch advances ONLY if every step succeeds; on any failure nothing is committed/tagged and
``release`` is left untouched. Re-running after a successful release is an idempotent no-op
(and will create the tag if a prior run stopped before tagging). Pushing is left to you
unless ``--push`` is given.

This operates on your LOCAL ``main`` and ``release`` refs and does not fetch; it warns if
either is behind its upstream. Pull first if you want the latest.

Use ``--core-tag`` when only the CLI advances; otherwise both tags match.

The helpers are unit tested by ``scripts/test_release.py`` (Python 3.10 or later).

Usage:
    python scripts/release.py vX.Y.Z              # reconcile + verify + commit + tag (no push)
    python scripts/release.py vX.Y.Z --push       # also push the release branch + tag to origin
    python scripts/release.py vX.Y.Z --no-test    # skip `cargo test` (still builds and packages)
    python scripts/release.py vX.Y.Z --no-doc-check  # skip generated CLI reference freshness check
    python scripts/release.py vX.Y.Z --registry-timeout 1800  # wait longer for crates.io
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Callable

# scripts/release.py -> the gwz-cli repo root is one level up.
REPO = Path(__file__).resolve().parent.parent

DEFAULT_CORE_URL = "https://github.com/owebeeone/gwz-core"
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
REGISTRY_API = "https://crates.io/api/v1/crates/gwz-core/{version}"
# crates.io refuses a request without a User-Agent that identifies the client, so name the
# script and the repository rather than urllib's default.
USER_AGENT = "gwz-cli-release (scripts/release.py; https://github.com/owebeeone/gwz-cli)"
DEFAULT_REGISTRY_TIMEOUT = 900
REGISTRY_POLL_SECONDS = 15
HTTP_TIMEOUT = 30
REGISTRY_FOUND = 200

# The two shapes the release branch's gwz-core line may have when a release starts, each
# matched against the whole line: the exact registry pin this script writes, and the git + tag
# pin every release up to 1.0.11 carried (migrated once, at the first release on the registry).
CORE_REGISTRY_PIN = re.compile(r'gwz-core\s*=\s*"=[^"\s]+"')
CORE_GIT_TAG_PIN = re.compile(
    r'gwz-core\s*=\s*\{\s*(?:git\s*=\s*"[^"]+"\s*,\s*tag\s*=\s*"[^"]+"'
    r'|tag\s*=\s*"[^"]+"\s*,\s*git\s*=\s*"[^"]+")\s*\}'
)
TABLE_HEADER = re.compile(r"\s*(\[\[?[^\]]*\]\]?)")
LOCK_FIELD = re.compile(r'([A-Za-z0-9_-]+)\s*=\s*"([^"]*)"')


def fail(msg: str):
    print(f"release: error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def log(msg: str):
    print(f"release: {msg}")


def run(cmd, *, cwd=None, capture=False, check=True) -> subprocess.CompletedProcess:
    printable = " ".join(str(c) for c in cmd)
    log(f"$ {printable}")
    result = subprocess.run(
        [str(c) for c in cmd],
        cwd=str(cwd) if cwd is not None else None,
        capture_output=capture,
        text=True,
    )
    if check and result.returncode != 0:
        if capture and result.stderr:
            print(result.stderr, file=sys.stderr)
        fail(f"command failed ({result.returncode}): {printable}")
    return result


def git(args, **kw) -> subprocess.CompletedProcess:
    return run(["git", "-C", REPO, *args], **kw)


def git_wt(worktree, args, **kw) -> subprocess.CompletedProcess:
    return run(["git", "-C", worktree, *args], **kw)


def branch_exists(branch: str) -> bool:
    return git(["rev-parse", "--verify", "--quiet", branch], capture=True, check=False).returncode == 0


def is_ancestor(ancestor: str, descendant: str) -> bool:
    """True if `ancestor` is already contained in `descendant` (so a merge would be a no-op)."""
    return git(["merge-base", "--is-ancestor", ancestor, descendant], capture=True, check=False).returncode == 0


def warn_if_behind_upstream(branch: str):
    upstream = git(["rev-parse", "--abbrev-ref", "--symbolic-full-name", f"{branch}@{{u}}"],
                   capture=True, check=False)
    if upstream.returncode != 0 or not upstream.stdout.strip():
        return  # no upstream configured
    name = upstream.stdout.strip()
    behind = git(["rev-list", "--count", f"{branch}..{name}"], capture=True, check=False).stdout.strip()
    if behind and behind != "0":
        log(f"WARNING: local {branch} is {behind} commit(s) behind {name} "
            f"(tracking ref; run `git fetch` for current state) -- releasing local {branch}")


def verify_core_tag(url: str, tag: str):
    """The core tag must exist at `url`: the parity tests read gwz-core's fixtures from a clone of it."""
    result = run(["git", "ls-remote", "--tags", url, f"refs/tags/{tag}"], capture=True)
    if not result.stdout.strip():
        fail(f"gwz-core tag {tag} not found at {url} -- push the gwz-core release first")
    log(f"verified gwz-core {tag} exists at {url}")


def registry_status(url: str) -> int:
    """The HTTP status crates.io answers a read-only lookup with; 0 when it did not answer."""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT}, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT) as response:
            return int(response.status)
    except urllib.error.HTTPError as error:
        return int(error.code)
    except (urllib.error.URLError, TimeoutError, OSError):
        return 0


def registry_status_may_change(status: int) -> bool:
    """Whether waiting can turn this answer into 200: not published or not visible yet (404),
    throttled (429), a crates.io outage (5xx) or no answer at all (0). Anything else -- 403 for
    a refused User-Agent, say -- will not change by polling."""
    return status in (0, 404, 429) or 500 <= status <= 599


def describe_status(status: int) -> str:
    return "no answer" if status == 0 else f"HTTP {status}"


def wait_for_registry_core(
    version: str,
    *,
    timeout: float = DEFAULT_REGISTRY_TIMEOUT,
    probe: Callable[[str], int] = registry_status,
    sleep: Callable[[float], None] = time.sleep,
    monotonic: Callable[[], float] = time.monotonic,
    interval: float = REGISTRY_POLL_SECONDS,
):
    """Wait until crates.io serves gwz-core `version`, or fail once `timeout` seconds have passed.

    gwz-core's release workflow publishes the core and its internal crates after its Linux
    verification job, so a CLI release started soon after the core's release may have to wait
    for that job. Runs before the worktree exists, so a timeout leaves nothing behind.
    """
    url = REGISTRY_API.format(version=version)
    log(f"waiting up to {timeout:.0f}s for gwz-core {version} on crates.io ({url})")
    start = monotonic()
    while True:
        status = probe(url)
        elapsed = monotonic() - start
        if status == REGISTRY_FOUND:
            log(f"verified gwz-core {version} is on crates.io (after {elapsed:.0f}s)")
            return
        if not registry_status_may_change(status):
            fail(
                f"crates.io answered {describe_status(status)} for {url}, which waiting cannot "
                "change (only 404, 429, 5xx and no answer are retried) -- check the version, the "
                "network and crates.io's status, then re-run"
            )
        if elapsed >= timeout:
            fail(
                f"gwz-core {version} is still not on crates.io after {elapsed:.0f}s (last answer: "
                f"{describe_status(status)}) -- finish the gwz-core release, including its "
                "crates.io publish job, then re-run (or wait longer with --registry-timeout)"
            )
        log(f"gwz-core {version} is not on crates.io yet ({describe_status(status)}); "
            f"checking again in {interval:.0f}s")
        sleep(interval)


def checkout_gwz_core(url: str, tag: str, target: Path):
    """Check out the exact core release beside the standalone CLI worktree.

    CLI/core machine-output parity tests intentionally read core's canonical
    fixtures through ``../gwz-core``. Keeping the matching tag at that boundary
    preserves the one-fixture contract in both GWZ development and standalone
    release verification.
    """
    run(["git", "clone", "--depth", "1", "--branch", tag, url, target])


def release_branch_is_free(release: str):
    git(["worktree", "prune"], check=False)  # clear stale entries left by a hard-killed prior run
    porcelain = git(["worktree", "list", "--porcelain"], capture=True).stdout
    if re.search(rf"^branch refs/heads/{re.escape(release)}$", porcelain, re.M):
        fail(f"branch '{release}' is checked out in another worktree -- free it first "
             f"(`git switch` away, or `git worktree remove` it), then re-run")


def merge_head_exists(worktree) -> bool:
    return git_wt(worktree, ["rev-parse", "-q", "--verify", "MERGE_HEAD"], capture=True, check=False).returncode == 0


def do_merge(worktree, main: str, release: str):
    """Merge `main` into the release worktree (stopping before commit). Resolves a Cargo.lock-only
    conflict (cargo regenerates it); aborts + fails on any other conflict or genuine merge error."""
    already = is_ancestor(main, release)
    merge = run(["git", "-C", worktree, "merge", "--no-ff", "--no-commit", main], capture=True, check=False)
    conflicts = [c for c in git_wt(worktree, ["diff", "--name-only", "--diff-filter=U"],
                                   capture=True).stdout.split() if c]
    if conflicts:
        other = [c for c in conflicts if c != "Cargo.lock"]
        if other:
            git_wt(worktree, ["merge", "--abort"], check=False)
            fail("merge produced conflicts beyond Cargo.lock (only the gwz-core dep should ever "
                 "differ between main and release):\n  " + "\n  ".join(other))
        # Cargo.lock conflict is benign here: the lock is regenerated from the reconciled Cargo.toml.
        git_wt(worktree, ["checkout", "--theirs", "--", "Cargo.lock"], check=False)
        git_wt(worktree, ["add", "Cargo.lock"])
        log("resolved a Cargo.lock merge conflict (cargo generate-lockfile will regenerate it)")
    elif not merge_head_exists(worktree):
        # No conflicts and no merge in progress: either up to date, or a genuine merge error.
        if already:
            log(f"{release} already contains {main}; reconciling version/dep only")
        else:
            git_wt(worktree, ["merge", "--abort"], check=False)
            fail(f"`git merge {main}` did not produce a merge:\n{merge.stdout}{merge.stderr}")


def keyed_lines(lines: list[str], key: str) -> list[tuple[int, str | None]]:
    """Each manifest line that assigns `key`, with the header of the table it sits in."""
    pattern = re.compile(rf"\s*{re.escape(key)}\s*=")
    table = None
    found = []
    for index, line in enumerate(lines):
        header = TABLE_HEADER.match(line)
        if header:
            table = header.group(1)
        elif pattern.match(line):
            found.append((index, table))
    return found


def core_dependency_index(lines: list[str]) -> int:
    """The index of the one line of `[dependencies]` that declares gwz-core."""
    tables = [line.strip() for line in lines if TABLE_HEADER.match(line) and "gwz-core" in line]
    found = keyed_lines(lines, "gwz-core")
    if tables or len(found) != 1 or found[0][1] != "[dependencies]":
        where = [f"line {index + 1} in {table or 'no table'}" for index, table in found]
        where += [f"table {table}" for table in tables]
        fail("Cargo.toml must declare gwz-core on exactly one line of [dependencies] (found: "
             f"{', '.join(where) or 'none'}) -- the release branch carries "
             '`gwz-core = "=X.Y.Z"` there')
    return found[0][0]


def reconcile_cargo_toml(worktree, core_version: str, version: str) -> bool:
    """Pin gwz-core to exactly `core_version` from crates.io and set the package version, in
    Cargo.toml and the two CLI Bazel artifacts. Returns True if it changed a file, False if both
    were already reconciled (idempotent re-run).

    Accepts the registry pin this script writes and, once, the git + tag pin of the releases up
    to 1.0.11. Every other shape fails -- main's `path` form carried in by the merge, a `branch`
    or `rev` pin, extra keys such as `features`, a requirement that is not exact -- because
    rewriting it would silently change what the release builds.
    """
    path = worktree / "Cargo.toml"
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")

    index = core_dependency_index(lines)
    current = lines[index].strip()
    if CORE_REGISTRY_PIN.fullmatch(current):
        migrated = False
    elif CORE_GIT_TAG_PIN.fullmatch(current):
        migrated = True
    else:
        fail(f"unexpected gwz-core dependency on the release branch: `{current}` -- expected the "
             'registry pin `gwz-core = "=X.Y.Z"` or, before the first registry release, '
             '`gwz-core = { git = "...", tag = "vX.Y.Z" }` (did main edit the dependency line?)')
    lines[index] = f'gwz-core = "={core_version}"'

    versions = [found for found, table in keyed_lines(lines, "version") if table == "[package]"]
    if len(versions) != 1:
        fail(f"expected one `version` line in Cargo.toml's [package] table, found {len(versions)}")
    lines[versions[0]], replaced = re.subn(
        r'^(\s*version\s*=\s*)"[^"]*"', rf'\g<1>"{version}"', lines[versions[0]], count=1
    )
    if replaced != 1:
        fail(f"Cargo.toml's package version is not a quoted string: `{lines[versions[0]].strip()}`")
    updated = "\n".join(lines)

    bazel_path = worktree / "BUILD.bazel"
    bazel = bazel_path.read_text(encoding="utf-8")
    updated_bazel, artifact_count = re.subn(
        r'^(\s*version\s*=\s*)"[^"\n]*"', rf'\g<1>"{version}"', bazel, flags=re.M
    )
    if artifact_count != 2:
        fail("expected exactly two CLI Bazel artifact versions")

    if updated == text and updated_bazel == bazel:
        log(f'Cargo.toml already reconciled (version = {version}, gwz-core = "={core_version}")')
        return False
    if updated != text:
        path.write_text(updated, encoding="utf-8", newline="\n")  # force LF regardless of core.autocrlf
    if updated_bazel != bazel:
        bazel_path.write_text(updated_bazel, encoding="utf-8", newline="\n")
    log(f'reconciled Cargo.toml: version = {version}, gwz-core = "={core_version}" from crates.io'
        + (" (migrated from the git + tag pin)" if migrated else ""))
    return True


def lock_packages(lock: str) -> list[dict[str, str]]:
    """The `[[package]]` entries of a Cargo.lock, each as its quoted string fields."""
    packages = []
    current = None
    for line in lock.splitlines():
        stripped = line.strip()
        if stripped == "[[package]]":
            current = {}
            packages.append(current)
        elif stripped.startswith("["):
            current = None
        elif current is not None:
            field = LOCK_FIELD.fullmatch(stripped)
            if field:
                current[field.group(1)] = field.group(2)
    return packages


def verify_locked_registry_pin(worktree, core_version: str):
    """After `cargo generate-lockfile`, the standalone lock must take gwz-core `core_version` and
    every internal `gwz-*` crate from crates.io -- not from git, and not as the workspace `path`
    dependency (which would happen if the worktree resolved inside the gwz-dev cargo workspace)."""
    packages = lock_packages((worktree / "Cargo.lock").read_text(encoding="utf-8"))
    cores = [package for package in packages if package.get("name") == "gwz-core"]
    if len(cores) != 1:
        fail(f"Cargo.lock holds {len(cores)} gwz-core entries; a release resolves exactly one, "
             f"gwz-core {core_version} from crates.io")
    core = cores[0]
    problems = []
    if core.get("source") != CRATES_IO_SOURCE:
        problems.append(f"source={core.get('source')!r}")
    if core.get("version") != core_version:
        problems.append(f"version={core.get('version')!r}")
    if not core.get("checksum"):
        problems.append("no checksum")
    if problems:
        fail(f"after `cargo generate-lockfile`, Cargo.lock does not take gwz-core {core_version} "
             f"from crates.io ({', '.join(problems)}; expected source={CRATES_IO_SOURCE!r}, "
             "that version and a checksum) -- the worktree may have resolved inside a cargo "
             "workspace or through a [patch]; the build would not exercise the published release")
    # Internal crates by prefix (`gwz` itself, the root package, has none) so a new or retired
    # internal needs no edit here.
    internals = [package for package in packages
                 if package.get("name", "").startswith("gwz-") and package.get("name") != "gwz-core"]
    stray = [f"{package.get('name')} {package.get('version')} (source={package.get('source')!r})"
             for package in internals if package.get("source") != CRATES_IO_SOURCE]
    if stray:
        fail("Cargo.lock takes internal gwz crate(s) from somewhere other than crates.io: "
             + "; ".join(stray) + " -- a published gwz-core resolves its internals from the registry")
    log(f"verified Cargo.lock takes gwz-core {core_version} and {len(internals)} internal gwz "
        "crate(s) from crates.io")


def verify_cli_reference_docs(worktree):
    """The release branch must not ship stale generated CLI reference docs."""
    script = worktree / "scripts" / "generate_cli_reference.py"
    result = run([sys.executable, script, "--check"], cwd=worktree, capture=True, check=False)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, file=sys.stderr, end="")
    if result.returncode != 0:
        fail(
            "generated CLI reference is out of date. Run "
            "`python scripts/generate_cli_reference.py --write` from the gwz-cli repo, "
            "commit the updated docs/CLI.md, then rerun the release. "
            "Use `--no-doc-check` only to bypass this check intentionally."
        )
    log("verified docs/CLI.md matches current Clap help")


def cargo_target_directory(worktree) -> Path:
    """Where cargo writes for this worktree, honouring CARGO_TARGET_DIR and cargo configuration."""
    result = run(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=worktree, capture=True)
    try:
        return Path(json.loads(result.stdout)["target_directory"])
    except (ValueError, KeyError, TypeError) as error:
        fail(f"cannot read the target directory from `cargo metadata`: {error}")


def verify_cli_package(worktree, version: str):
    """Package the CLI as crates.io receives it, and build it from that package.

    `cargo package --locked` assembles the manifest's `include` list and then compiles the
    extracted package on its own against registry dependencies only -- the build crates.io and
    `cargo install gwz` perform. It runs before the release commit so `release` never advances
    past a CLI that cannot be packaged, hence `--allow-dirty`: the tree it packages is the merge
    the commit then records. Any stale archive is removed first, so only this run can pass.
    """
    archive = cargo_target_directory(worktree) / "package" / f"gwz-{version}.crate"
    archive.unlink(missing_ok=True)
    run(["cargo", "package", "--locked", "--allow-dirty", "-p", "gwz"], cwd=worktree)
    if not archive.is_file():
        fail(f"`cargo package --locked -p gwz` finished without producing {archive} -- the gwz "
             "package cannot be published from this release")
    log(f"packaged and verified {archive.name} against crates.io ({archive.stat().st_size} bytes)")


def release_commit_message(version: str, core_version: str) -> str:
    return f"chore(release): gwz-cli {version} (pins gwz-core {core_version} from crates.io)"


def ensure_tag(tag: str, target: str):
    """Create the lightweight tag `tag` at commit `target`, or no-op if it already points there.
    NEVER moves an existing tag -- released tags are immutable."""
    existing = git(["rev-parse", "-q", "--verify", f"refs/tags/{tag}^{{commit}}"], capture=True, check=False)
    if existing.returncode == 0:
        if existing.stdout.strip() == target:
            log(f"tag {tag} already points at {target[:10]} -- leaving it")
            return
        fail(f"tag {tag} already exists at {existing.stdout.strip()[:10]}, not the release commit "
             f"{target[:10]} -- refusing to move a release tag (delete it yourself if this is intentional)")
    git(["tag", tag, target])
    log(f"created tag {tag} -> {target[:10]}")


def push_release(release: str, tag: str):
    """Push the release branch + tag together, atomically (both land or neither)."""
    result = run(["git", "-C", REPO, "push", "--atomic", "origin", release, tag], capture=True, check=False)
    if result.returncode != 0:
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        fail(f"`git push --atomic origin {release} {tag}` failed -- with --atomic the remote is left "
             "unchanged; inspect `git ls-remote origin` and retry")
    log(f"pushed {release} + {tag} to origin (atomic)")


def remove_worktree(worktree):
    result = git(["worktree", "remove", "--force", worktree], capture=True, check=False)
    if result.returncode != 0:
        log(f"WARNING: `git worktree remove` failed for {worktree}: {result.stderr.strip()}")
        shutil.rmtree(worktree, ignore_errors=True)
    git(["worktree", "prune"], check=False)
    if Path(worktree).exists():
        log(f"WARNING: worktree dir still present at {worktree} -- remove it manually, "
            "then run `git worktree prune`")


def release_version(tag: str) -> str:
    """Accept stable releases and numbered release candidates, with no leading zeroes."""
    number = r"(?:0|[1-9][0-9]*)"
    if not re.fullmatch(rf"v{number}\.{number}\.{number}(?:-rc\.[1-9][0-9]*)?", tag):
        fail(f"tag must look like vX.Y.Z or vX.Y.Z-rc.N, got '{tag}'")
    return tag[1:]


def release_versions(tag: str, core_tag: str | None) -> tuple[str, str]:
    version = release_version(tag)
    core_tag = core_tag or tag
    release_version(core_tag)
    return version, core_tag


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Reconcile the gwz-cli release branch for a release tag, in a temp worktree."
    )
    parser.add_argument("tag", help="release tag, e.g. v0.3.0")
    parser.add_argument("--core-tag", help="existing core release tag (default: CLI release tag)")
    parser.add_argument("--core-url", default=DEFAULT_CORE_URL,
                        help="gwz-core git repository that must hold the core tag; the parity-fixture "
                             "clone comes from it (default: %(default)s)")
    parser.add_argument("--registry-timeout", type=float, default=DEFAULT_REGISTRY_TIMEOUT,
                        help="seconds to wait for gwz-core X.Y.Z to appear on crates.io before "
                             "failing (default: %(default)s)")
    parser.add_argument("--main", default="main", help="source branch to merge from (default: main)")
    parser.add_argument("--release", default="release", help="release branch to reconcile (default: release)")
    parser.add_argument("--no-test", action="store_true",
                        help="skip `cargo test` (still runs `cargo build` and `cargo package`)")
    parser.add_argument("--no-doc-check", action="store_true",
                        help="skip the generated docs/CLI.md freshness check")
    parser.add_argument("--push", action="store_true", help="also push the release branch + tag to origin")
    parser.add_argument("--keep-worktree", action="store_true",
                        help="leave the temp worktree in place (you must `git worktree remove` it before re-running)")
    return parser


def main():
    args = build_parser().parse_args()

    tag = args.tag
    version, core_tag = release_versions(tag, args.core_tag)
    core_version = release_version(core_tag)
    if args.registry_timeout < 0:
        fail("--registry-timeout must not be negative")

    for tool in ("git", "cargo"):
        if not shutil.which(tool):
            fail(f"`{tool}` not found on PATH")
    for branch in (args.main, args.release):
        if not branch_exists(branch):
            fail(f"branch '{branch}' does not exist in {REPO}")
    release_branch_is_free(args.release)
    warn_if_behind_upstream(args.main)
    warn_if_behind_upstream(args.release)

    core_url = args.core_url
    verify_core_tag(core_url, core_tag)
    wait_for_registry_core(core_version, timeout=args.registry_timeout)

    # If the tag already exists, the release is already cut: never advance release past it and never
    # move it. Checking here -- before any commit -- also removes any commit-but-no-tag window.
    existing = git(["rev-parse", "-q", "--verify", f"refs/tags/{tag}^{{commit}}"], capture=True, check=False)
    if existing.returncode == 0:
        head = git(["rev-parse", args.release], capture=True).stdout.strip()
        if existing.stdout.strip() != head:
            fail(f"tag {tag} already exists at {existing.stdout.strip()[:10]} but {args.release} HEAD is "
                 f"{head[:10]} -- inconsistent; resolve the tag manually before re-running")
        log(f"{tag} already exists at {args.release} HEAD ({head[:10]}); release already cut")
        if args.push:
            push_release(args.release, tag)
        return

    temp_root = Path(tempfile.mkdtemp(prefix=f"gwz-cli-{tag}-"))
    worktree = temp_root / "gwz-cli"
    core_checkout = temp_root / "gwz-core"
    git(["worktree", "add", worktree, args.release])
    try:
        do_merge(worktree, args.main, args.release)
        merged = merge_head_exists(worktree)
        changed = reconcile_cargo_toml(worktree, core_version, version)
        # Regenerate the lock from the reconciled manifest rather than update it: the merged lock
        # can carry main's path-shaped gwz-core entry (the workspace-member era) or the previous
        # release's git-sourced one, which `cargo update -p` cannot address in a standalone
        # worktree ("package ID specification did not match").
        (Path(worktree) / "Cargo.lock").unlink(missing_ok=True)
        run(["cargo", "generate-lockfile"], cwd=worktree)
        # Before any build: only ever build, package, commit or tag a lock that takes the
        # published core and its internal crates from crates.io.
        verify_locked_registry_pin(worktree, core_version)
        changed = changed or bool(git_wt(worktree, ["status", "--porcelain"], capture=True).stdout)
        if merged or changed:
            # Build before adding the sibling used by tests: the release must build with only
            # the CLI checkout and its crates.io dependencies.
            run(["cargo", "build", "--locked"], cwd=worktree)
        checkout_gwz_core(core_url, core_tag, core_checkout)
        if merged or changed:
            if not args.no_test:
                run(["cargo", "test", "--locked"], cwd=worktree)
            if not args.no_doc_check:
                verify_cli_reference_docs(worktree)
            verify_cli_package(worktree, version)
            git_wt(worktree, ["add", "-A"])
            message = release_commit_message(version, core_version)
            git_wt(worktree, ["commit", "-m", message])
            sha = git_wt(worktree, ["rev-parse", "HEAD"], capture=True, check=False).stdout.strip()
            log(f"{args.release} reconciled -> {sha[:10] if sha else '(committed)'}  "
                f"(gwz-cli {version}, gwz-core {core_version} from crates.io)")
        else:
            log(f"{args.release} already reconciled for {tag}; no new commit needed")
            if not args.no_doc_check:
                verify_cli_reference_docs(worktree)
            # Only ever tag a commit that packages and builds against crates.io.
            verify_cli_package(worktree, version)

        # Tag the worktree's HEAD (== release HEAD). The tag was confirmed absent above, so this
        # creates it; ensure_tag still refuses to move a tag if one raced in concurrently.
        target = git_wt(worktree, ["rev-parse", "HEAD"], capture=True).stdout.strip()
        ensure_tag(tag, target)

        if args.push:
            push_release(args.release, tag)
        else:
            log("next step (not done without --push):")
            log(f"  git -C {REPO} push origin {args.release} {tag}")
    finally:
        if args.keep_worktree:
            log(
                f"left release checkouts under {temp_root} "
                f"(remove {worktree} with `git worktree remove` before re-running)"
            )
        else:
            remove_worktree(worktree)
            shutil.rmtree(temp_root, ignore_errors=True)


if __name__ == "__main__":
    main()
