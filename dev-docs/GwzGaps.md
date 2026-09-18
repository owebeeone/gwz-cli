# gwz — known gaps & deferred work

Tracked gaps that are intentionally not built yet. Active feature specs live in
the owning crate's `dev-docs/`; implemented or superseded plans live under
`dev-docs/history/`. This file collects the "not scheduled" items so they
aren't lost.

## `gwz` user preferences / aliases
- **User config file for preferences and aliases** — no implemented `.gwzrc`
  equivalent for persistent CLI preferences or command aliases such as
  `gwz st` -> `gwz status`.
- Existing proposal: `gwz-cli/dev-docs/GwzRcSpec.md` (currently names the file
  `.gwzconfig`; reconcile whether the user-facing file should be `.gwzrc`,
  `.gwzconfig`, or support both before implementation).

## `gwz add` (multi-repo staging)
- **Interactive / patch staging** — no `git add -p` equivalent (stage selected hunks).
- **Unstaging** — no `gwz restore --staged` / `gwz reset` equivalent to undo a stage.

(Implemented `gwz add` behavior and its other deferrals are recorded in
`gwz-core/dev-docs/history/GWZAddPlan.md`.)

## `gwz stash`
- Implemented (gwz-cli `9e60109`); `docs/commands/stash.md`. The spec and plan
  moved to `dev-docs/history/`. Nothing deferred.

## SSH transport: unknown host key (added 2026-09-11)
- **No host-key trust decision anywhere** — libgit2 checks the server host key
  against `~/.ssh/known_hosts` before it authenticates, and gwz installs no
  certificate callback, so a host missing from `known_hosts` fails every ssh
  clone or fetch with `GitCommandFailed: invalid or unknown remote ssh hostkey`.
  Nothing reaches the CLI or gwz-py except that error: the core-to-client
  surface during an operation is the one-way `EventSink`, and the protocol has
  no prompt/answer message. The OpenSSH "continue connecting?" prompt was never
  gwz's; gwz never runs `ssh`. Measured in the URL-scheme baseline
  (`GwzUrlSchemeBaseline-2026-09-11.md`, §4 and §11.6).
- **Decision recorded 2026-09-11: core never prompts.** Core returns a typed
  refusal carrying host, port, key type, SHA256 fingerprint, and unknown versus
  mismatched when it can tell (git2-rs discards libgit2's `valid` flag, so gwz
  must read `known_hosts` itself, including HMAC-hashed entries). Clients act on
  it: in-process (`gwz` and `gwz-py` on the machine that owns the trust store)
  a client may offer a trust-on-first-use prompt and append the entry; a remote
  gwz-core server never asks the requesting user, its operator provisions
  `known_hosts` (`ssh-keyscan`, or the fingerprints GitHub and GitLab publish).
  Automation (`--json`, JSONL, CI) gets the refusal plus a non-interactive
  remedy, for example `--trust-host <host>=SHA256:<fingerprint>`.
- Design points to keep: one decision per host, so parallel per-host member
  clones ask once and the rest wait; a request-meta bit saying whether asking
  is allowed; a mismatched key is never auto-accepted and never silently
  bypassed by switching transport (the URL-scheme `auto` fallback stays
  credential-only for that reason).
- Not part of the URL-scheme feature (plan: `GwzUrlSchemePlan.md`, pending);
  needs its own security review and plan.

## `gwz pull`: a member the manifest gained upstream (added 2026-09-19)

Measured with gwz 1.0.17 in a throwaway: workspace A pushed to bare remotes,
cloned to B with `gwz clone`, then A added member `two` and pushed.

- **One unmaterialized member refuses the whole member phase.** In B,
  `gwz pull` fast-forwards the root, so the manifest now names `two`, then
  stops with `gwz: MemberNotFound: member 'mem_two' is not materialized`, exit
  1. The existing member, which had a new upstream commit, is not pulled.
  Running `pull` again fails identically until `gwz materialize` has run.
  Nothing is lost and `gwz materialize` recovers, but the pull is partial (root
  advanced, members not) and blocking. Wanted: the missing member is a row of
  its own, every other member is pulled, and the result is `Partial`, which is
  the shape `gwz fetch` already has for this case.
- **The refusal names no remedy.** Only `gwz status` says
  `run gwz materialize --lock to complete the clone`. The pull error should
  name the command.
- **Decision needed: should `pull` materialize new members itself?** Arguments
  for an explicit step: private or very large members, and `git pull` does not
  do it for submodules either. Options: leave it explicit, add
  `pull --materialize`, or materialize by default every member the manifest
  does not mark local-only. Not decided.
- **`pull` prints no row for a root that has no upstream.** `gwz fetch` prints
  `@root . no upstream`; `gwz pull` omits the root entirely, so its output
  reads as "everything is up to date". This is how a workspace made by
  `gwz init` plus `gwz repo clone` (never pushed, never cloned) can look like a
  stale copy of another workspace: its manifest is a local file that nothing
  upstream can update, and no verb says so. Found on a developer's test
  workspace whose root had no commits and no remote.

## gwz-core runs the `git` executable in four places (added 2026-09-19)
- `gwz commit`, `gwz tag`, path-filtered `gwz log`, and a conditional fallback
  in lane import. Cases, reasons and a phased todo are in
  `gwz-core/dev-docs/GwzLibgit2Gaps.md`.
