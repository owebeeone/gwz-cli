# `gwz local`

Create, inspect and retire the local clone family of this workspace.

```text
gwz local clone <name> [dest] [--clean | --bare] [-b <branch>] [--from <name|path>] [--owner <token>] [--wait <secs>]
gwz local list [--wait <secs>]
gwz local dispose <name> [--keep | --force <hazard,...>] [--wait <secs>]
gwz local disband [--wait <secs>]
```

A local clone is a second working copy of the whole workspace on the same
machine, created with `gwz local clone`. The family index lives on the
workspace root and every clone carries a pointer back to it, so these commands
work from any ready family member. [Local Clones](../LocalClones.md) explains
the model, the lifecycle and the deletion rules; this page is the reference.

Family names are resolved from the index at operation time. They are never
written into `gwz.conf/gwz.yml` and never become Git remotes. The family's own
verb owns creation, inspection and retirement together; exchange stays on the
receiving workspace's [merge](merge.md) command:
`gwz merge --remote <name> [<ref>]`. Family pull and push are unsupported. [`gwz clone`](clone.md) is the URL form and nothing else.

## `gwz local clone`

Copies the current workspace into a second working copy on this machine and
registers it, by name, in the local clone family. There is no URL and no
network.

| Argument | Meaning |
| --- | --- |
| `<name>` | Family member name for the new clone. `root`, `origin` and Git's reserved ref names are refused, and so is a name already recorded in the family. |
| `[dest]` | Destination directory. Defaults to `../<root-dirname>-<name>` beside the workspace root. |

| Option | Meaning |
| --- | --- |
| `--verbatim` | Copy the source tree as it sits, dirt and build directories included. The default, and the only mode this build serves. |
| `--clean` | Take the frozen source state without worktree dirt. **Refused by this build.** |
| `--bare` | Make the destination a share point of bare repositories (implies `--clean`). **Refused by this build.** |
| `-b <branch>` | Create this branch in every destination repository before the clone is marked ready. `--clean`/`--bare` only, so **refused by this build** either way. |
| `--from <name\|path>` | Copy from this family member or path instead of the current workspace. **Refused by this build.** |
| `--owner <token>` | Record an opaque caller token on the new member row. Up to 128 bytes of `[A-Za-z0-9._:-]`. |
| `--wait <secs>` | Keep retrying a busy family lock for this many seconds before reporting it busy. |

Clone this workspace, tree and Git state as they sit:

```sh
gwz local clone A ../gwz-dev-A
```

```text
status: Ok
created local clone `A` at /Users/you/limbo/gwz-dev-A (verbatim; recorded as ../gwz-dev-A; 61 files copied (61 natively, 0 ordinarily), 48 directories, 0 symlinks, 37774 logical bytes; 0 remote URL(s) removed; dest-complete: 2 repositories, 15 objects verified of 20 in store, 1 ms; family fam_3fa95fa799ee9ec5b04999adba13e651, founded)
```

The counts are from a small illustrative workspace. A create copies staged
edits, unstaged edits, untracked files and build directories alike; what it
does not copy is the source's `.gwz/` runtime state, which the destination
gets fresh.

The first operand is always the name and the second the destination, so
`gwz local clone dest` asks for a member called `dest` at the default
destination; a path typed in the name's place is refused by the name rules.
A verbatim copy is refused while the source has an open coordinated merge
(`source has an open gwz merge (...); abort it or use --clean`): finish or
abort the merge, since the `--clean` the message offers is not served by this
build. Keep the source quiet for the whole invocation. `--from` names the
*source* to copy, not the destination — a family name recorded in the index,
or a filesystem path; a token that names neither is refused, and so is an
empty one. In this build every `--from` is refused before that, as
unsupported.

### Lanes made by a tool

A tool that creates lanes unattended — a hook, an agent, a CI job — has two
options of its own, and neither changes what a create does.

`--owner <token>` records an opaque token on the new member row. It is written
by the same index write that reserves the row, so a row either has the token
from the moment it exists or never has one; nothing afterwards sets, changes or
clears it. GWZ stores it, reports it in `gwz local list`, and does not
interpret, match or act on it — it is the caller's identity, for the caller's
own reuse decisions. The shape is the whole of the contract: at most 128 bytes
of `[A-Za-z0-9._:-]`, refused at parse otherwise.

```sh
gwz local clone A --owner claude-code:session_7
```

`--wait <secs>` addresses the other half. The family lock is a try-lock: it is
held for the whole of a create, a dispose or a disband, and a second family
operation that meets it refuses `Busy` at once. That is right for a person and
wrong for two invocations fired for one request, so `--wait` keeps retrying the
try-lock at a short fixed interval until the deadline. There is no blocking
acquisition, so the wait is portable and bounded by the number you give. Omit
it and the behaviour is exactly what it always was.

```sh
gwz local clone A --owner claude-code:session_7 --wait 120
```

A wait that wins the lock rereads the family before acting. So two creates of
one name, fired together with `--wait`, do not make two lanes and do not answer
from a stale view: one creates the lane, and the other reports the name as
held, naming the owner the first one recorded.

```text
gwz: PathCollision: local clone `A` -> /Users/you/limbo/gwz-dev-A: ... name `A` already holds ../gwz-dev-A (owner `claude-code:session_7`) ...
```

`gwz local list` accepts `--wait` and ignores it: a listing takes no lock, so
there is nothing to wait for, and accepting it lets a wrapper pass `--wait` to
every family verb uniformly.

### Index format 2

Recording an owner needs a place to put it, so the family index carries a
second format version, `gwz.local-family/v2`: format 1 plus the optional
per-row `owner`. A gwz that reads v2 reads a v1 index unchanged and writes v2
on its **first write of any kind** — a create, a dispose, a `--keep`, a family
merge — whether or not an owner was ever recorded.

Going the other way is a refusal, not a downgrade. The row format denies
unknown fields and the schema is matched exactly, so an older gwz refuses a v2
index as a whole and its refusal names the version to install:

```text
`schema: gwz.local-family/v2` is not `gwz.local-family/v1`; this file is not in a format this store reads; `gwz.local-family/v2` is read by gwz 1.0.14 and later, so upgrade gwz to at least 1.0.14
```

Every gwz used on one workspace must therefore be at or above 1.0.14 once any
gwz at or above it has written that workspace's family index.

### Modes this build refuses

The other modes are parsed and dispatched but answer `UnsupportedOperation`
before anything is written; a refused create allocates nothing. These are the
intended forms, with what the current build prints for each.

A frozen checkout on a new lane branch in every repository:

```sh
gwz local clone C ../gwz-dev-C --clean -b lane/agent-17
```

```text
gwz: UnsupportedOperation: local clone (clean mode) is not supported by this gwz-core build
```

A bare share point:

```sh
gwz local clone hub ../gwz-dev-hub --bare
```

```text
gwz: UnsupportedOperation: local clone (bare mode) is not supported by this gwz-core build
```

A copy of a different member — the new clone would still be registered on the
root, whichever member it was copied from:

```sh
gwz local clone D ../gwz-dev-D --from ../gwz-dev-C
```

```text
gwz: UnsupportedOperation: local create from an explicit copy source (--from <name|path>) is not supported by this gwz-core build
```

## `gwz local list`

Reports every member of the family: name, kind (checkout or bare), state, and
path — the root first, then every member in name order. The listing is
read-only: it takes no lock and repairs nothing. `--wait` is accepted and
ignored here for exactly that reason.

```sh
gwz local list
```

```text
root  checkout  ready  /Users/you/limbo/gwz-dev
A     checkout  ready  /Users/you/limbo/gwz-dev-A
C     checkout  ready  /Users/you/limbo/gwz-dev-C
hub   bare      ready  /Users/you/limbo/gwz-dev-hub
```

Paths are *recorded* relative to the workspace root that holds the index. The
response carries that root as well, so the listing prints each member's actual
directory rather than a path you have to resolve yourself. Run from a clone,
the listing still names root's own directory, because the root is reached
through that clone's pointer.

### The state column

Each row carries two states: the one the index *recorded* (`creating`, `ready`,
`disposing`) and the one that was *observed* on disk (`ready`, `incomplete`,
`interrupted_disposal`, `missing`, `pointer_removed`, `mismatched`,
`malformed`, `unobserved`). They are printed as one word while they agree, and
as `recorded/observed` when they do not, so an interrupted create or an
interrupted disposal is visible here rather than only after the next failure.
Any diagnostic the index recorded for a member is shown beside its row:

```text
root  checkout  ready                           /Users/you/limbo/gwz-dev
B     checkout  creating/incomplete             /Users/you/limbo/gwz-dev-B   copy interrupted at src/
C     checkout  disposing/interrupted_disposal  /Users/you/limbo/gwz-dev-C
hub   bare      ready/pointer_removed           /Users/you/limbo/gwz-dev-hub
```

### The owner column

A family in which nobody recorded an owner renders the four columns above and
no more. As soon as any member carries the token a `gwz local clone --owner`
wrote, an `owner` column appears between `state` and `path`, and rows without
one show `-`:

```text
root  checkout  ready  -                        /Users/you/limbo/gwz-dev
A     checkout  ready  claude-code:session_7    /Users/you/limbo/gwz-dev-A
C     checkout  ready  -                        /Users/you/limbo/gwz-dev-C
```

The token is printed exactly as the index carries it. GWZ never parses,
matches or abbreviates it; a caller that wrote one is the only party that
knows what it means.

Reporting is all this does. A row that says `incomplete` or
`interrupted_disposal` stays exactly as it is until you act on it with
`gwz local dispose`.

### Machine output

`--json` and `--jsonl` carry every field of every row under
`local_family_members`, with the enum values spelled in the protocol's own
snake_case, and the family root once beside the rows under
`local_family_root_path`. Each row's `path` stays relative to that root, as the
protocol records it; join the two for the absolute path the human table prints.

```json
{
  "kind": "response",
  "local_family_root_path": "/Users/you/limbo/gwz-dev",
  "local_family_members": [
    {
      "name": "root",
      "kind": "checkout",
      "recorded_state": "ready",
      "observed_state": "ready",
      "path": ".",
      "last_error": null,
      "owner": null
    },
    {
      "name": "B",
      "kind": "checkout",
      "recorded_state": "creating",
      "observed_state": "incomplete",
      "path": "../gwz-dev-B",
      "last_error": "copy interrupted at src/",
      "owner": "claude-code:session_7"
    }
  ]
}
```

`dispose` and `disband` carry no rows, so their `local_family_members` is an
empty list and their `local_family_root_path` is `null`.

## `gwz local dispose`

Removes one member. By default the member's directory is deleted, and the
deletion is refused while any hazard is detected:

| Hazard | Meaning |
| --- | --- |
| `open-merge` | The member has an open coordinated merge. |
| `dirty` | The member has staged, unstaged, untracked or ignored work. |
| `unpreserved-history` | Some of its history is not verifiably preserved in another surviving family member. |

A clean working tree is not preservation proof, and history that exists only on
a network remote is not certified: fetch it into a survivor first, or keep the
lane. Each reported hazard must be named explicitly before the deletion
proceeds:

```sh
gwz local dispose C --force unpreserved-history
gwz local dispose C --force open-merge,dirty,unpreserved-history
```

`--force` with no hazard names is refused, and so is `--force` together with
`--keep`. The waiver covers loss, not crash recovery: an incomplete or
interrupted member is retained rather than force-deleted, and manual cleanup is
the accepted recovery path.

`--wait <secs>` keeps retrying a busy family lock until the deadline instead of
refusing at once; a dispose that waits rereads the family before acting.

`--keep` is the non-destructive alternative: `--keep` deletes nothing on
disk. It removes only the pointer and the index row; the tree, its open merge
and its history stay where they are and remain usable as an ordinary GWZ
workspace:

```sh
gwz local dispose C --keep
```

Evidence the check cannot interpret refuses as `UnknownEvidence`, and no
`--force` name waives it. The case you will meet is a GWZ stash record in the
lane (`gwz stash push`), which this build does not decode: pop or drop the
stash in the lane first, or `--keep`. And because every `gwz commit` in a lane
also commits the lane's root repository, a member-only
`gwz merge --remote <name>` leaves the lane's root history unpreserved;
`gwz --target @root merge --remote <name>` brings it across, after which the
same `dispose` deletes.

The workspace root is never disposed — use `gwz local disband` to retire the
family — and neither is the member you are standing in.

## `gwz local disband`

Removes the family pointers and the index. Every member directory survives
exactly as it is:

```sh
gwz local disband
```

Afterwards family names stop resolving, so `--remote <name>` on `pull`, `push`
and `merge` falls back to ordinary Git remote resolution. Disband may be
repeated after an error. `--wait <secs>` keeps retrying a busy family lock
until the deadline instead of refusing at once.

## Notes

- **Status:** this build serves the verbatim `gwz local clone`, `gwz local
  list`, `gwz local dispose` (ordinary deletion and `--keep`), `gwz local
  disband` and the family merge end to end. `--clean`, `--bare` and `--from`
  are parsed and dispatched but answer `UnsupportedOperation` in this build;
  a refused create allocates nothing. Family names on `pull` and `push` are
  not served either: they fall through to Git remote resolution and answer
  `MissingRemote` (see [pull](pull.md) and [push](push.md)).
- [`gwz clone`](clone.md) takes a URL and nothing else. An earlier draft of
  this feature hung creation off `gwz clone` behind a `--local` flag; that
  form was removed before any release, without an alias, and `gwz clone` now
  rejects `--local` as an unknown argument.
- A workspace that holds no family index lists nothing, and that is an answer
  rather than an error — no family is invented, and nothing is repaired.
- `--dry-run` is refused for every local family verb, `gwz local clone`
  included, before any write (`UnsupportedOperation`).
- Hazard names are split on `,` and on nothing else — no trimming, no case
  folding. An unknown name is refused by name; a bare `--force`, an empty
  element and `--keep` together with `--force` are refused before the request
  is even sent.
- Refusals carry a typed error code and are rendered structured under `--json`
  and `--jsonl`. `gwz merge --remote <name>` that names no ready member is
  `unknown_local`, and its message says which — an absent or reserved name, or
  a row that is `creating`/`disposing`.
