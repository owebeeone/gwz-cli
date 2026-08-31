# Concepts

## Workspace Root

A GWZ workspace is a local directory that owns a tracked `gwz.conf/` metadata
directory. The root repository is a normal Git repository and records the
workspace manifest, lock, and snapshots.

## Member Repository

A member is a Git repository managed as part of the workspace. Commands can act
on all members, a selected set of members, or the workspace root when the
operation includes root state.

Members have:

- an id, used with `--member`;
- a workspace-relative path, used with `--member-path`;
- an absolute path when materialized locally;
- source and remote metadata in workspace files;
- current Git state observed by status, materialize, pull, and push.

## Manifest And Lock

The manifest is `gwz.conf/gwz.yml`. It records the workspace and configured
members.

The lock is `gwz.conf/gwz.lock.yml`. It records exact member revisions so the
workspace can be reproduced by `gwz materialize --lock` or by `gwz clone`.

Use `gwz capture` to record the live worktree state into the lock without
otherwise mutating member repositories.

## Member Listing

`gwz ls` lists materialized members by default. `gwz ls --local` prints
workspace-relative paths, which is useful in scripts. `gwz ls --unmaterialized`
also includes configured members that are not currently checked out on disk.

## Snapshot

A snapshot is a named workspace artifact that captures the current selected
member revisions. Use snapshots before risky multi-repository changes or when
you need a reproducible workspace point that is not necessarily a release tag.

Commands:

```sh
gwz snapshot before-refactor
gwz snapshot --list
gwz materialize --snapshot before-refactor
gwz pull --snapshot before-refactor
```

## Git Tag

`gwz tag` manages real Git tags across selected members. Local create, list, and
delete operations include selected members and the committed workspace root.
Remote push, fetch, list, and delete operations span member repositories.

Tags are checked out through materialization:

```sh
gwz materialize --tag v0.9.0
```

## Workspace History

`gwz log` reads the workspace root and selected member repositories as one
newest-first history. The default selection is the root plus every active
member. The same global selectors used by other commands can narrow it.

Coordinated commits carrying the same valid GWZ commit marker normally
coalesce into one workspace entry. A conservative message/author/time
heuristic can coalesce compatible unmarked commits; `--no-coalesce` exposes
the underlying per-repository commits. Machine output identifies the result as
`none`, `heuristic`, `marker:<uuid-v7>`, or `marker-invalid` provenance.

Revision and snapshot operands use the shared range grammar. For example,
`+release..HEAD` starts each repository at its `release` snapshot entry, while
`+lock..HEAD` starts at the revision recorded in the workspace lock. Pathspecs
belong after `--` and are interpreted relative to the invocation directory.
See the [`log` command page](commands/log.md) for limits, filters, output modes,
and failure policy.

## Selection

Selection flags are global:

- `--member <member-id>` selects by member id and may be repeated.
- `--member-path <member-path>` selects by workspace-relative path and may be
  repeated.
- `--all` selects all members and cannot be combined with `--member` or
  `--member-path`.

Commands also have command-specific selection in some cases. `gwz forall`, for
example, accepts positional project names that match member ids or paths.

## Planning And Failure Policy

`--dry-run` plans an operation without mutating workspace metadata or member
repositories. It is not supported for `gwz clone`.

`--partial` allows operations to complete for members that can proceed even when
another selected member fails. Without it, operations that can plan ahead reject
partial mutation when a selected member cannot proceed cleanly.

`--force` allows destructive behavior when an operation requires explicit
confirmation.

## Remotes And Sync

`--remote <name>` selects the Git remote used by operations that contact
remotes.

`--sync <mode>` selects sync behavior. Implemented values are:

```text
fetch-only
ff-only
merge
rebase
reset
driver-selected
```

The default policy is fast-forward only.

Network operations are bounded by `--jobs <n>` across the whole operation and
`--max-per-host <n>` per remote host. `--ssh-timeout <secs>` bounds stalled
SSH/network reads; `0` disables the timeout.

## Progress Events

Human mode renders live progress to stderr when stderr is a terminal.

`--jsonl` streams operation records to stdout for machine consumers. Progress
event frequency is controlled by `--progress-interval <ms>`, with the default
100 milliseconds per member and `0` meaning every update.

## Forall Execution

`gwz forall` is a CLI-local executor. It resolves selected materialized members,
then runs a command in each member directory.

In argv mode, use `--` before the command:

```sh
gwz forall -- git status --short
```

In shell mode, use `-c`:

```sh
gwz forall -c 'echo "$GWZ_MEMBER_PATH"'
```

Each child receives:

- `GWZ_MEMBER_ID`
- `GWZ_MEMBER_PATH`
- `GWZ_MEMBER_ABSPATH`
- `GWZ_ROOT`

In argv mode, `{@}` inside an argument is replaced with the member path.
