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

On `clone` and `materialize`, `--url-scheme <manifest|ssh|https>` chooses the
URL form used for repositories this operation clones on the known hosts
github.com, gitlab.com, and bitbucket.org; `GWZ_URL_SCHEME` sets the same
choice. The manifest is never rewritten, and a member that is already checked
out keeps its remotes.

The flag wins over `GWZ_URL_SCHEME`, which wins over the preference an earlier
`ssh` or `https` run recorded in `<workspace>/.gwz/url-scheme.yml`; `manifest`
applies when none of them is set. An explicit `--url-scheme manifest` clears
that record. The file is local runtime state: it is not part of `gwz.conf/`,
not in the manifest, and not committed.

On `gwz merge`, `--remote <name>` does not name a Git remote at all: it names
a member of the local clone family, described below.

## Publication

`gwz push` sends each selected repository to its own configured remote. A push
that contacts the root proves, before the root transfer, that every member
commit the committed lock names is available, from this operation's own reads of
the member remotes or its accepted pushes of branches that contain the commit,
never from remote-tracking refs. It reads every dependency; like its transfers,
its reads run in parallel within `--jobs` and `--max-per-host`.

The proof reads each dependency at the push URL, else the fetch URL, of the
member's remote (the one the manifest names, usually `origin`) when that URL is
the committed URL or the same URL in the other scheme on github.com, gitlab.com
or bitbucket.org; at the committed URL in the scheme `.gwz/url-scheme.yml`
records when the member is not checked out; and at the committed URL otherwise.
A workspace cloned with `gwz clone --url-scheme https` therefore publishes over
HTTPS end to end, with credentials from your Git credential helper.

To switch a workspace cloned over SSH, clone it again with `--url-scheme https`,
or point the root (`.`) and each member at HTTPS yourself:

```sh
git -C <path> remote set-url origin https://github.com/<owner>/<repo>.git
```

Push takes no `--url-scheme` and does not read `GWZ_URL_SCHEME`. A workspace
switched by hand has no `.gwz/url-scheme.yml`, so members that are not checked
out are still read at their committed URLs until a `--url-scheme https` clone or
materialize records one. The manifest keeps its URLs: `gwz repo sync` keeps the
manifest URL for a remote that differs from it only by scheme, unless run with
`--force`.

By default a push contacts only repositories that changed since the last fetch
or push. It compares each branch with its remote-tracking ref, such as
`origin/main`, not with the branch's upstream, and neither checks for changes
nor pushes a repository whose branch equals that ref or is behind it. A
repository without a usable remote-tracking ref (a branch never fetched or
pushed, a single-branch or shallow clone, or a push URL that names another
repository) is always contacted. Each repository left alone reports `Noop` with
a reason that names the remote and branch:

- `already on origin`: this push read the remote, and its branch already points
  at the local commit;
- `up to date with origin/main as of the last fetch or push`: the branch equals
  its remote-tracking ref, so the remote was not contacted;
- `behind origin/main as of the last fetch or push`: the remote-tracking ref is
  ahead of the branch, so there is nothing to publish and the remote was not
  contacted.

When repositories were not contacted, human output ends with a line such as the
following, and `--verbose` adds each row's reason:

```text
7 repositories unchanged since the last fetch or push were not checked for changes; --check-remotes to verify
```

`gwz push --check-remotes` skips that comparison: it reads every selected remote
and every root dependency, pushes repositories whose remote lacks their branch's
commit, and proves a selected root even when it has nothing to push. Use it when
someone else may have rewound a remote, before a release, or to check that an
unchanged root is still sound. Remote-tracking refs can go stale; see
[Troubleshooting](Troubleshooting.md#push-skips-a-member-whose-remote-changed).

## Local Clone Family

A local clone is a second working copy of the whole workspace on the same
machine, made with `gwz local clone <name> [dest]` and registered by name in
the workspace's local clone family. A create copies the tree as it sits,
uncommitted work and build output included. The family index lives on the
workspace root; each clone carries a pointer back to it, so family commands
work from any ready member. Names resolve at operation time, are never
written into `gwz.conf/`, and never become Git remotes.

Work comes back by name: `gwz merge --remote <name> [<ref>]` imports the
clone's commits into each receiving repository under a retained
`refs/gwz/local-imports/<transfer-id>` ref and runs the ordinary coordinated
merge from there. On `merge` the name is family-only. On `pull` and `push` a
family name is not served by this build and falls through to Git remote
resolution.

A clone is deleted with `gwz local dispose <name>` only when its history is
verifiably preserved in another surviving member; a clean working tree is not
proof. `--keep` forgets the clone without deleting anything, `--force
<hazard,...>` names the losses you accept, and `gwz local disband` retires the
family while keeping every tree. See [Local Clones](LocalClones.md).

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
