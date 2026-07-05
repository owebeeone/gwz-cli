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
