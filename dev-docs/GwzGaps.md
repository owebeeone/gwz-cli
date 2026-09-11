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
- Spec exists (`gwz-cli/dev-docs/GwzStashSpec.md` + `GwzStashPlan.md`), **not implemented**.

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
