---
name: gwz
description: "Operate a GWZ multi-repository workspace: select repositories, inspect changes, commit work, manage members, and integrate or retire local lanes."
---

# Working with GWZ

GWZ coordinates a root Git repository and its registered member repositories.
Use GWZ for workspace operations. Files under `gwz.conf/` and `AGENTS_GWZ.md`
are managed by GWZ; do not hand-edit them.

## Find the workspace and select the work

Use the root the user named. An inherited directory or similarly named checkout
is not proof. `gwz --root ROOT --json ls` identifies member IDs and paths;
`gwz --root ROOT status` inspects current work. `WorkspaceNotFound` calls for
finding the correct root, not automatically initializing another workspace.

`--root ROOT` chooses the workspace. `--target @root` chooses only its root repo;
`--target @all` chooses root plus members. A member ID/path selects that member;
`--no-target` excludes targets. The default selection includes root and members.

**Combined status paths are workspace-relative, not all owned by the root.**
For example, `packages/api/file.rs` belongs to member `packages/api`, with
repository-relative path `file.rs`. Match the longest registered member path;
unmatched paths belong to the root. Use `--json status` and `--json ls` for
machine reports: JSON status separates member changes from root changes.
Staged, unstaged and untracked are different states.

## Stage, commit and verify

`gwz add PATH...` stages requested paths in their owning repositories.
`gwz add -A` stages everything in the selected targets: use it only when that
whole scope is intended. `--all` selects targets; it does **not** mean stage-all.
`gwz commit -m "message"` commits staged work; `-a` stages tracked edits first.
Preserve unrelated work and staging. A metadata-only root commit does not mean
member edits were committed.

Verify the requested outcome before reporting success: inspect status and the
relevant files/history (`gwz log --full`, `gwz diff`). Check every requested
repository, including the root. A successful command or plausible summary is
not proof that all requested steps happened. Once verified, finish rather than
repeating inspections indefinitely.

## Add, detach and restore members

- `gwz repo add PATH`: register an existing Git checkout.
- `gwz repo create PATH`: create and register a new repository.
- `gwz repo clone URL PATH`: clone and register a repository.
- `gwz repo detach MEMBER`: mark its designation inactive; keep its checkout.
- `gwz repo attach MEMBER_ID`: reactivate a previously detached designation.
  It does not register an arbitrary new checkout.
- `gwz repo sync MEMBER`: refresh metadata from local Git configuration.

Verify both sides of a requested detach/attach change: the retired designation
is inactive, the restored designation is active, and existing files/history
remain. `gwz --json ls` lists active members; managed metadata can also be read.
Use positional member operands for detach/attach, without global target flags.

## Local lanes

Use `gwz local clone NAME DEST` for an independent workspace lane. The source
must be a valid GWZ workspace; DEST must be outside it. Copies include dirty,
untracked and ignored files, including build output; copy-on-write is used where
available. Keep the source quiet during copying. An untouched lane is still a
whole workspace, not a Git linked worktree.

Work and commit in the lane, then integrate **from the receiving workspace**:

```sh
gwz --root LANE add -A
gwz --root LANE commit -m "completed work"
gwz --root ROOT --target @all merge --remote NAME
gwz --root ROOT local dispose NAME
```

Here `-A` assumes all lane changes are intended. Include `@all` when integrating
the complete lane so root history travels with member history. Integrate multiple
lanes serially. Verify committed contents and completed merge before disposal.
`gwz push` publishes configured Git remotes; it is not lane-to-root integration.

`gwz local list` inspects the family. `local dispose NAME` deletes a preserved
lane; `local dispose NAME --keep` detaches it while keeping files. These differ
from **member** `repo detach`. `local disband` retires the family, keeping trees.
If deletion refuses dirty or unpreserved work, investigate and preserve it;
`--keep` does not satisfy a request to delete the directory.

## Recovery and less common commands

For an open coordinated merge: inspect `gwz merge --status`, then use
`--continue` after resolution or `--abort` when appropriate. Do not manually
repair managed metadata or waive loss hazards merely to claim completion.

Look up the specific command with `gwz help COMMAND` or `COMMAND --help` for
branch, tag, stash, snapshot, materialize, authentication or execution options.
Dry-run support varies; check the specific command’s help. Do not assume every
mutating command implements dry-run.

Check `gwz --version` when behavior differs. For installation, consult the
[published Install page](https://owebeeone.github.io/gwz-cli/Install/).
