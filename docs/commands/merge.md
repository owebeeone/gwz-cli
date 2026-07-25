# `gwz merge`

Merge one source ref into the current branch of each selected workspace
repository.

```text
gwz merge <source> [--dry-run]
gwz merge --status
gwz merge --continue
gwz merge --abort
```

With no selection, all active members participate and the workspace root does
not. Select the root explicitly as `@root`.

## Quick start

Preview the merge without changing any repository:

```sh
gwz merge feature/refactor --dry-run
```

Start it:

```sh
gwz merge feature/refactor
```

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
- the root merge-result commit and the later composition-evidence commit are
  reported as distinct commits.

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
- the root merge result separately from composition evidence.

Post-merge work is never silently discarded. A branch switch, new commit,
modified index, changed worktree, foreign Git operation, missing object, or
changed publication artifact blocks unsafe recovery. Preserve or remove that
work and restore the exact state named by status before retrying.

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

## Machine output

Use `--json` for one structured response or `--jsonl` for lifecycle events
followed by the terminal response:

```sh
gwz --target mem_app --target @root --jsonl merge feature/refactor
gwz --json merge --status
```

Machine results identify root rows with `target_id: "@root"`, `path: "."`, and
`target_kind: "Root"`. Errors retain the same structured target fields, so
consumers do not need to extract repository identity from human text.

## Features not yet available

Preserving post-merge work during abort (`--abort --preserve`), strategy flags,
custom merge messages, and merge-record garbage collection are not yet
available. These forms remain hidden and return typed unsupported errors if
submitted directly.

Merge also rejects unrelated operation policies supplied explicitly:
`--sync`, `--remote`, `--jobs`, `--max-per-host`,
`--progress-interval`, `--partial`, and `--force`. Diagnostics name the option
that must be removed.

`gwz branch --merge <source>` remains a deprecated compatibility spelling. It
constructs the same first-class merge request.
