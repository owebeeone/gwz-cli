#!/usr/bin/env python3
"""Reconcile the gwz-cli ``release`` branch for a new release, in a throwaway worktree.

gwz-cli ships from its ``release`` branch, which differs from ``main`` in exactly one
way: the gwz-core dependency. On ``main`` it is a local ``path``; on ``release`` it is a
pinned ``git`` + ``tag``. This script automates RELEASE.md steps 2-4 for a given tag:

  1. Verify the gwz-core ``<tag>`` actually exists on the gwz-core remote (so the pinned
     dependency will be fetchable).
  2. In a temporary ``git worktree`` checked out on ``release``, merge ``main`` (the
     gwz-core dep line auto-resolves to release's git+tag form because main never edits
     it -- the "merge gotcha" documented in RELEASE.md).
  3. Reconcile Cargo.toml: pin the gwz-core dependency to ``<tag>`` and set the package
     version to match.
  4. ``cargo build`` + ``cargo test`` in that standalone worktree -- this refreshes
     Cargo.lock against gwz-core@``<tag>`` (and the script then asserts the lock really
     pins the git tag), proves gwz-cli compiles against the pinned release, and checks
     the generated CLI reference is current -- then commit the merge as
     ``chore(release): gwz-cli X.Y.Z (pins gwz-core vX.Y.Z)``.
  5. Tag that commit ``<tag>`` (lightweight). An existing tag is NEVER moved -- if ``<tag>``
     already points elsewhere the script aborts rather than re-pointing a release tag.

The worktree is removed afterward (kept only with ``--keep-worktree``). The ``release``
branch advances ONLY if every step succeeds; on any failure nothing is committed/tagged and
``release`` is left untouched. Re-running after a successful release is an idempotent no-op
(and will create the tag if a prior run stopped before tagging). Pushing is left to you
unless ``--push`` is given.

This operates on your LOCAL ``main`` and ``release`` refs and does not fetch; it warns if
either is behind its upstream. Pull first if you want the latest.

Usage:
    python scripts/release.py vX.Y.Z              # reconcile + verify + commit + tag (no push)
    python scripts/release.py vX.Y.Z --push       # also push the release branch + tag to origin
    python scripts/release.py vX.Y.Z --no-test    # skip `cargo test` (still builds)
    python scripts/release.py vX.Y.Z --no-doc-check  # skip generated CLI reference freshness check
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# scripts/release.py -> the gwz-cli repo root is one level up.
REPO = Path(__file__).resolve().parent.parent


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


def gwz_core_url(release: str) -> str:
    """The gwz-core git URL pinned on the release branch's Cargo.toml."""
    toml = git(["show", f"{release}:Cargo.toml"], capture=True).stdout
    match = re.search(r'gwz-core\s*=\s*\{[^}\n]*\bgit\s*=\s*"([^"]+)"', toml)
    if not match:
        fail(f"no `git = \"...\"` gwz-core dependency found on the {release} branch")
    return match.group(1)


def verify_remote_tag(url: str, tag: str):
    result = run(["git", "ls-remote", "--tags", url, f"refs/tags/{tag}"], capture=True)
    if not result.stdout.strip():
        fail(f"gwz-core tag {tag} not found at {url} -- push the gwz-core release first")
    log(f"verified gwz-core {tag} exists at {url}")


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
        # Cargo.lock conflict is benign here: cargo build regenerates it from the reconciled Cargo.toml.
        git_wt(worktree, ["checkout", "--theirs", "--", "Cargo.lock"], check=False)
        git_wt(worktree, ["add", "Cargo.lock"])
        log("resolved a Cargo.lock merge conflict (cargo build will regenerate it)")
    elif not merge_head_exists(worktree):
        # No conflicts and no merge in progress: either up to date, or a genuine merge error.
        if already:
            log(f"{release} already contains {main}; reconciling version/dep only")
        else:
            git_wt(worktree, ["merge", "--abort"], check=False)
            fail(f"`git merge {main}` did not produce a merge:\n{merge.stdout}{merge.stderr}")


def reconcile_cargo_toml(worktree, tag: str, version: str) -> bool:
    """Pin the gwz-core dep to `tag` and set the package version. Returns True if it changed the
    file, False if it was already reconciled (idempotent re-run)."""
    path = worktree / "Cargo.toml"
    text = path.read_text(encoding="utf-8")
    # Top-level package version (dependency `version =` entries are indented, so `^version` skips them).
    updated = re.sub(r'^(version\s*=\s*)"[^"]*"', rf'\g<1>"{version}"', text, count=1, flags=re.M)
    # The gwz-core git dependency's tag (git=/tag= order within the inline table is irrelevant).
    updated = re.sub(r'(gwz-core\s*=\s*\{[^}\n]*\btag\s*=\s*)"[^"]*"', rf'\g<1>"{tag}"', updated)

    already_correct = f'version = "{version}"' in updated and f'tag = "{tag}"' in updated
    if updated == text:
        if already_correct:
            log("Cargo.toml already reconciled (version + gwz-core tag already match)")
            return False
        fail("Cargo.toml reconcile changed nothing and the expected lines are absent -- the merged "
             "gwz-core dependency may not be in git+tag form (did main edit the dependency line?), "
             "or the file format is unexpected")
    if not already_correct:
        fail(f"Cargo.toml reconcile did not yield version={version} + gwz-core tag={tag}")
    path.write_text(updated, encoding="utf-8", newline="\n")  # force LF regardless of core.autocrlf
    log(f"reconciled Cargo.toml: version = {version}, gwz-core tag = {tag}")
    return True


def verify_locked_git_pin(worktree, tag: str):
    """After cargo build, the standalone lock must pin gwz-core via the git tag -- not the workspace
    `path` dep (which would happen if the worktree resolved inside the gwz-dev cargo workspace)."""
    lock = (worktree / "Cargo.lock").read_text(encoding="utf-8")
    match = re.search(r'\[\[package\]\]\nname = "gwz-core"\nversion = "[^"]*"\n(?:source = "([^"]+)"\n)?', lock)
    source = match.group(1) if match else None
    if not source or "git+" not in source or f"tag={tag}" not in source:
        fail(f"after `cargo build`, Cargo.lock does not pin gwz-core via git tag {tag} "
             f"(source={source!r}) -- the worktree may have resolved inside a cargo workspace; "
             "the build did not actually exercise the pinned release")
    log(f"verified Cargo.lock pins gwz-core via {source}")


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


def main():
    parser = argparse.ArgumentParser(
        description="Reconcile the gwz-cli release branch for a release tag, in a temp worktree."
    )
    parser.add_argument("tag", help="release tag, e.g. v0.3.0")
    parser.add_argument("--main", default="main", help="source branch to merge from (default: main)")
    parser.add_argument("--release", default="release", help="release branch to reconcile (default: release)")
    parser.add_argument("--no-test", action="store_true", help="skip `cargo test` (still runs `cargo build`)")
    parser.add_argument("--no-doc-check", action="store_true",
                        help="skip the generated docs/CLI.md freshness check")
    parser.add_argument("--push", action="store_true", help="also push the release branch + tag to origin")
    parser.add_argument("--keep-worktree", action="store_true",
                        help="leave the temp worktree in place (you must `git worktree remove` it before re-running)")
    args = parser.parse_args()

    tag = args.tag
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        fail(f"tag must look like vX.Y.Z, got '{tag}'")
    version = tag[1:]

    for tool in ("git", "cargo"):
        if not shutil.which(tool):
            fail(f"`{tool}` not found on PATH")
    for branch in (args.main, args.release):
        if not branch_exists(branch):
            fail(f"branch '{branch}' does not exist in {REPO}")
    release_branch_is_free(args.release)
    warn_if_behind_upstream(args.main)
    warn_if_behind_upstream(args.release)

    verify_remote_tag(gwz_core_url(args.release), tag)

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

    worktree = Path(tempfile.gettempdir()) / f"gwz-cli-{tag}-{os.getpid()}"
    git(["worktree", "add", worktree, args.release])
    try:
        do_merge(worktree, args.main, args.release)
        merged = merge_head_exists(worktree)
        changed = reconcile_cargo_toml(worktree, tag, version)
        if merged or changed:
            # The worktree lives outside the gwz-dev workspace, so cargo resolves gwz-core via
            # git+tag: this refreshes Cargo.lock against the pinned release and verifies the build.
            run(["cargo", "build"], cwd=worktree)
            if not args.no_test:
                run(["cargo", "test"], cwd=worktree)
            verify_locked_git_pin(worktree, tag)
            if not args.no_doc_check:
                verify_cli_reference_docs(worktree)
            git_wt(worktree, ["add", "-A"])
            message = (
                f"chore(release): gwz-cli {version} (pins gwz-core {tag})\n\n"
                "Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
            )
            git_wt(worktree, ["commit", "-m", message])
            sha = git_wt(worktree, ["rev-parse", "HEAD"], capture=True, check=False).stdout.strip()
            log(f"{args.release} reconciled -> {sha[:10] if sha else '(committed)'}  "
                f"(gwz-cli {version}, gwz-core {tag})")
        else:
            log(f"{args.release} already reconciled for {tag}; no new commit needed")
            verify_locked_git_pin(worktree, tag)  # only ever tag a commit whose lock pins the git tag
            if not args.no_doc_check:
                verify_cli_reference_docs(worktree)

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
            log(f"left worktree at {worktree} (remove it before the next run: git worktree remove)")
        else:
            remove_worktree(worktree)


if __name__ == "__main__":
    main()
