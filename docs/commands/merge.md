# `gwz merge`

Merge one source ref into the current branch of each selected workspace
repository.

```text
gwz merge <source> [--dry-run] [--ff-only] [--no-ff] [-m <message>]
gwz merge --status [<merge-id>]
gwz merge --continue
gwz merge --abort [--preserve]
gwz merge --gc [<merge-id>]
```

With no selection, all active members participate and the workspace root does
not. Select the root explicitly as `@root`.

## Quick start

Preview the merge without changing any repository:

```sh
gwz merge feature/refactor --dry-run
```

The preview performs an in-memory merge for every participant that would need
a true merge. It reports whether each such merge is predicted to be clean and
lists predicted conflict paths. The result is advisory: starting the merge
repeats preflight under the workspace mutation lock.

Git paths that are not safely representable as ordinary UTF-8 are shown in a
quoted, byte-safe form such as `"config-\xFF.toml"`. That spelling is a stable
diagnostic, not a path value to copy back into a filesystem command.

Start it:

```sh
gwz merge feature/refactor
```

Supply a custom body for any merge commits created by the operation:

```sh
gwz merge feature/refactor -m "Merge the refactor series"
```

GWZ normalizes CRLF and bare CR line endings to LF, removes trailing newlines,
and appends its mandatory merge and operation identity lines. Empty,
whitespace-only, and NUL-containing messages are rejected before any
repository is changed. The exact final message is recorded once and reused by
restart and conflict resolution. `-m` does not change the separate root
composition-publication message.

If a repository conflicts, inspect the coordinated operation:

```sh
gwz merge --status
```

Resolve and stage the reported files, then continue:

```sh
gwz add path/to/resolved-file
gwz merge --continue
```

Or safely restore every participant to its pre-merge state:

```sh
gwz merge --abort
```

If successful participants have accumulated safe post-merge work, preserve it
before rollback:

```sh
gwz merge --abort --preserve
```

Do not substitute raw `git merge --abort`. It knows about only one repository
and cannot restore a coordinated workspace operation.

## Choosing participants

The source ref is resolved independently in every selected repository.

```sh
# All active members; root excluded
gwz merge feature/refactor

# Two members only
gwz --target mem_app --target mem_docs merge feature/refactor

# Workspace root only
gwz --target @root merge feature/refactor

# Members plus the workspace root
gwz --target mem_app --target @root merge feature/refactor
```

Selection is frozen from the pre-merge manifest. Members remain in manifest
order and an explicitly selected root is appended last. A root merge cannot
add, remove, reorder, or rename participants in the operation already under
way.

`@all` and bare `--all` retain merge's member-only default. Use
`--target @root` when root participation is intended.

## The coordinated state machine

Most successful merges pass through `Executing` and `Finalizing` too quickly to
notice. The other states exist so interruption and partial progress remain
recoverable.

| State | Meaning | What to do |
| --- | --- | --- |
| `Idle` | No merge is open. | Start a merge. |
| `Executing` | GWZ is applying the frozen plan in order. | If interrupted, run `gwz merge --status`. |
| `AwaitingResolution` | At least one repository has an expected Git conflict. Other independent participants may already have completed. | Resolve and stage conflicts, then continue or abort. |
| `Halted` | An unexpected Git or host failure stopped later participants. | Inspect status, repair the reported cause, then continue or abort. |
| `Finalizing` | Participant results are verified and workspace composition evidence is being published. | Continue after interruption; GWZ resumes the recorded publication step. |
| `Preserving` | A preserve-abort is recording and verifying backup refs or coordinated stashes before rollback. | If interrupted, rerun `gwz merge --abort --preserve`. |
| `RecoveryRequired` | Live Git state no longer exactly matches a safe recovery point. | Follow the drift report and restore the exact expected state before retrying. |
| `RollingBack` | A coordinated abort is being persisted and applied in reverse order. | Retry `gwz merge --abort` after interruption. |
| `Completed` | Results and composition evidence were verified and the operation was closed. | No recovery action is needed. |
| `Aborted` | Every affected repository was restored and the operation was closed. | No recovery action is needed. |

Each participant also has its own state, such as `UpToDate`,
`FastForwarded`, `Merged`, `Conflicted`, `Continued`, `Failed`,
`Unattempted`, `RolledBack`, or `Aborted`. This is why one workspace operation
can be awaiting resolution while another repository has already merged
successfully.

## What happens when a merge starts

GWZ first preflights every selected repository before mutating any of them. It
requires an attached, born target branch, a clean index and worktree, no
unrelated Git sequencer state, a resolvable source ref, and shared source/target
history.

It then:

1. persists the frozen participant plan and exact pre-merge state;
2. executes members in manifest order;
3. continues past expected conflicts so independent later members can run;
4. stops after an unexpected backend or host failure and marks later
   participants `Unattempted`;
5. executes an explicitly selected workspace root last;
6. verifies every recorded result; and
7. publishes the updated lock and merge marker in one checked root composition
   commit.

The accepted workspace lock remains at its exact pre-merge baseline while the
operation is open. Durable participant results, rather than a partially
advanced lock, are the source of truth for status and recovery.

True merge commits use:

```text
Merge '<source>' into '<target-branch>'

GWZ-Merge-ID: <merge-id>
GWZ-Operation-ID: <operation-id>
```

GWZ rejects unrelated histories; it does not implicitly enable Git's
`--allow-unrelated-histories`.

## Requiring fast-forwards everywhere

Use `--ff-only` when the entire selected workspace must advance without
creating a merge commit:

```sh
gwz merge feature/refactor --ff-only
```

This is a selection-wide guarantee. GWZ completes preflight for every
participant and rejects the whole operation before local mutation if any
changing repository would need a true merge. Up-to-date repositories remain
valid no-ops.

## Resolving conflicts

An expected conflict remains in that participant's ordinary Git merge state.
Edit the conflicted files normally, but stage them with `gwz add`. While a merge
is open, `gwz add` accepts only repositories recorded as conflicted and rejects
the entire add if the selection includes a clean or unrelated repository.

```sh
gwz merge --status
gwz add repos/app/src/config.rs
gwz merge --continue
```

Before each Git mutation, GWZ records the exact pending action. After a crash
or lost connection, status classifies that action as not started, at the
expected conflict, completed exactly, or ambiguous. Only exact states are
adopted automatically.

## When the workspace root participates

Root participation is opt-in because the root contains the manifest, lock, and
composition history used to coordinate the workspace.

The root must already have a commit and must pass the same attached-branch and
clean-state checks as a member. Its pre-merge manifest and lock are read from
the recorded root commit, so recovery does not depend on the current files
remaining parseable.

Consequently:

- `gwz merge --status`, `gwz add`, `gwz merge --continue`, and
  `gwz merge --abort` still work when a root conflict has left the live
  manifest or lock with conflict markers;
- finalization reloads merged root metadata only after the root merge succeeds;
- selected member identities, paths, and source identities must still match
  the frozen operation; and
- the root participant keeps its merge-result commit while GWZ verifies the
  later composition evidence internally during publication and recovery.

If root metadata attempts to redefine an in-flight member, finalization fails
closed. The operation remains open and can be inspected or aborted using its
durable pre-merge record.

## Status and drift

`gwz merge --status` is strictly read-only. It reports:

- the operation and publication states;
- recorded before, source, result, and live commits;
- conflict paths and pending-action state;
- participant and operation drift;
- continue and abort eligibility; and
- the root merge result and the current publication step. Composition evidence
  is verified internally but is not currently a separate response field.

Post-merge work is never silently discarded. A branch switch, new commit,
modified index, changed worktree, foreign Git operation, missing object, or
changed publication artifact blocks unsafe recovery. Preserve or remove that
work and restore the exact state named by status before retrying.

With no id, status inspects the open merge or reports `Idle`. Closed records
can be inspected while they are retained:

```sh
gwz merge --status merge_20260725_1234
```

An id-qualified closed response is historical: it has `open: false` and does
not inspect or reopen the repositories.

## Coordinated abort

Abort preflights every participant and all root publication evidence before its
first mutation. If any repository is unsafe to roll back, nothing is changed.

Rollback follows the reverse of execution:

1. remove an incomplete composition-evidence commit, if present;
2. restore the explicitly merged root to its pre-merge commit;
3. restore members in reverse execution order;
4. verify the exact baseline manifest and lock bytes; and
5. close the operation as `Aborted`.

If abort itself is interrupted, rerun the same command. Durable rollback
progress makes the operation restart-safe.

If abort refuses, if it reports success and the result still looks wrong, or if
neither continue nor abort can close the operation, see the
[Merge Recovery Runbook](../MergeRecovery.md).

## Preserving work before abort

`gwz merge --abort --preserve` is an explicit, conservative escape hatch for
work created after a participant merged successfully. It accepts only an
attached recorded target branch, no unresolved entries, no active or foreign
Git operation, and a live commit equal to or descended from the recorded merge
result. A rewound or divergent branch, partial conflict resolution, branch
switch, or ambiguous state rejects the whole preserve attempt before rollback.
GWZ verifies that every still-conflicted repository retains its original
conflict index and conflict-marker file contents; edit or stage that work only
after deciding to continue, or preserve it manually before aborting.
If Git created a conflict but GWZ was interrupted before recording the original
marker snapshot, `--continue` may still reconcile and resolve the conflict, but
preserve-abort refuses to infer the original from later live bytes.

The root is also eligible when a member-only merge has already recorded its
composition commit. Root work created before that composition evidence exists
is rejected with manual-preservation guidance so candidate metadata cannot be
mistaken for user work. Automatic root worktree preservation is likewise
conservative when the composition commit created the root's first commit;
committed descendants can still be retained through the reported backup ref.

Depending on what changed, GWZ creates:

- `refs/gwz/merge/<merge-id>/<member-id>/head` for committed member work;
- `refs/gwz/merge/<merge-id>/root/head` for committed root work; and
- one coordinated `stash_<merge-id>` bundle for staged, unstaged, and
  untracked work. Ignored files are not included.

The operation enters `Preserving` before artifact creation. Every required
artifact is verified and recorded before the existing reverse-order abort path
can begin. If creation is interrupted, rerun the same preserve command; GWZ
checks the recorded ref targets and stash object ids rather than duplicating
them. Plain `gwz merge --abort` rejects an operation in `Preserving`, because
it must not bypass artifact reconciliation or verification.

A mutating GWZ command holds the workspace mutation lock for its complete
service call. Do not edit a selected repository or run a separate mutating Git
command while that call is executing. The lock serializes cooperating GWZ
commands, but it cannot freeze an editor or a raw filesystem/Git writer during
native stash creation; such concurrent writes have the same unsupported race
as running `git stash` while another process edits the checkout.

Preserved work is not reapplied automatically because it was created against
the post-merge tree. The human, JSON, and JSONL responses report every ref,
commit, stash id, and native stash object id. Inspect or branch from a backup
ref with Git. Restore the coordinated stash deliberately after abort:

```sh
gwz stash apply stash_<merge-id>
```

This restore works for preservation rows owned by members and by `@root`.
After applying and checking the recovered work, the same deterministic bundle
can be dropped, including an explicit root selection:

```sh
gwz --target @root stash drop stash_<merge-id>
```

## Retention and cleanup

GWZ keeps the latest 20 ordinary closed merge records for local diagnostics.
Records that own preservation evidence are exempt from automatic retention.

```sh
# Apply ordinary retention only
gwz merge --gc

# Remove one retained record and its verified private backup refs
gwz merge --gc merge_20260725_1234
```

Explicit GC preflights every recorded ref before deleting any of them and
refuses to run while a coordinated merge is open. A missing ref is accepted on
retry, but a ref pointing at a different commit fails closed. After its refs
are deleted, the archived merge record is removed. Successful GC output lists
only preservation artifacts that remain, such as native stash object ids; it
does not present deleted backup refs as recoverable.

GC never deletes native stashes or coordinated stash bundles. After recovering
or intentionally abandoning the preserved changes, remove those separately:

```sh
gwz stash drop stash_<merge-id>
```

## Machine output

Use `--json` for one structured response or `--jsonl` for lifecycle events
followed by the terminal response:

```sh
gwz --target mem_app --target @root --jsonl merge feature/refactor
gwz --json merge --status
gwz --json merge --status merge_20260725_1234
gwz --json merge --gc merge_20260725_1234
```

Machine results identify root rows with `target_id: "@root"`, `path: "."`, and
`target_kind: "Root"`. Errors retain the same structured target fields, so
consumers do not need to extract repository identity from human text.
Record-version failures likewise include `record_context` with the merge id,
readable schema/version pair, required semantic wave when known, and legacy
mode when applicable.

## `--no-ff`

`--no-ff` always creates a merge commit, even where a fast-forward is
possible. It is the counterpart to `--ff-only`; supplying both together
is rejected. A `--no-ff` start writes a v1 coordinated merge record and
publishes a two-parent integration commit. Ordinary and
custom-message starts continue to write v0 records.

Merge also rejects unrelated operation policies supplied explicitly:
`--sync`, `--remote`, `--jobs`, `--max-per-host`,
`--progress-interval`, `--partial`, and `--force`. Diagnostics name the option
that must be removed.

`gwz branch --merge <source>` remains a deprecated compatibility spelling. It
constructs the same first-class merge request.
