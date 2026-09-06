# `gwz local`

Create, inspect and retire the local clone family of this workspace.

```text
gwz local clone <name> [dest] [--clean | --bare] [-b <branch>] [--from <name|path>]
gwz local list
gwz local dispose <name> [--keep | --force <hazard,...>]
gwz local disband
```

A local clone is a second working copy of the whole workspace on the same
machine, created with `gwz local clone`. The family index lives on the
workspace root and every clone carries a pointer back to it, so these commands
work from any ready family member, including a bare share point.

Family names are resolved from the index at operation time. They are never
written into `gwz.conf/gwz.yml` and never become Git remotes. The family's own
verb owns creation, inspection and retirement together; exchange stays on the
verbs that already do it, with a family-resolved remote:
`gwz merge --remote <name> [<ref>]`, `gwz pull --head --remote <name>` and
`gwz push --remote <name>` (see [merge](merge.md), [pull](pull.md) and
[push](push.md)). [`gwz clone`](clone.md) is the URL form and nothing else.

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
| `--verbatim` | Copy the source tree as it sits, dirt and build directories included. The default. |
| `--clean` | Take the frozen source state without worktree dirt. |
| `--bare` | Make the destination a share point of bare repositories (implies `--clean`). |
| `-b <branch>` | Create this branch in every destination repository before the clone is marked ready. `--clean`/`--bare` only. |
| `--from <name\|path>` | Copy from this family member or path instead of the current workspace. |

Clone this workspace, tree and Git state as they sit:

```sh
gwz local clone A ../gwz-dev-A
```

Take the frozen state instead, on a new lane branch in every repository:

```sh
gwz local clone C ../gwz-dev-C --clean -b lane/agent-17
```

Make a bare share point the whole family can push to and pull from:

```sh
gwz local clone hub ../gwz-dev-hub --bare
```

Copy a different member instead of the workspace you are standing in. The new
clone is still registered on the root, whichever member it was copied from:

```sh
gwz local clone B ../gwz-dev-B --clean --from A
gwz local clone D ../gwz-dev-D --from ../gwz-dev-C
```

The first operand is always the name and the second the destination, so
`gwz local clone dest` asks for a member called `dest` at the default
destination; a path typed in the name's place is refused by core's name rules.
A verbatim copy is refused while the source has an open coordinated merge:
abort it, or use `--clean`. Keep the source quiet for the whole invocation.
`--from` names the *source* to copy, not the destination; core resolves the
token -- a family name recorded in the index, or a filesystem path -- and
refuses one that names neither. An empty `--from` is refused by the CLI,
because on the wire it would be indistinguishable from "copy this workspace".

## `gwz local list`

Reports every member of the family: name, kind (checkout or bare), state, and
path — the root first, then every member in name order. The listing is
read-only: it takes no lock and repairs nothing.

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
      "last_error": null
    },
    {
      "name": "B",
      "kind": "checkout",
      "recorded_state": "creating",
      "observed_state": "incomplete",
      "path": "../gwz-dev-B",
      "last_error": "copy interrupted at src/"
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

`--keep` is the non-destructive alternative. It removes only the pointer and
the index row; the tree, its open merge and its history stay on disk and remain
usable as an ordinary GWZ workspace:

```sh
gwz local dispose C --keep
```

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
repeated after an error.

## Notes

- **Status:** this build serves the verbatim `gwz local clone`, `gwz local
  list`, `gwz local dispose` (ordinary deletion and `--keep`), `gwz local
  disband` and the family merge end to end. `--clean`, `--bare` and `--from`
  are parsed and dispatched but still answer `UnsupportedOperation` while the
  engine behind them lands; a refused create allocates nothing.
- [`gwz clone`](clone.md) takes a URL and nothing else: it has no local form
  and rejects `--local` as an unknown argument (operator ruling 2026-09-06; the
  local create moved here without an alias, nothing having been released).
- A workspace that holds no family index lists nothing, and that is an answer
  rather than an error — no family is invented, and nothing is repaired.
- `--dry-run` travels to core for every local family verb, `gwz local clone`
  included, rather than being answered by the CLI; core refuses it today,
  before any write. Only the URL clone keeps a CLI-side `--dry-run` refusal.
- Hazard names are split on `,` and on nothing else — no trimming, no case
  folding. The vocabulary belongs to core, so an unknown name travels and core
  names it; only a bare `--force`, an empty element and `--keep` together with
  `--force` are refused before the request is encoded.
- Refusals carry a typed error code and are rendered structured under `--json`
  and `--jsonl`. `gwz merge --remote <name>` that names no ready member is
  `unknown_local`, and its message says which — an absent or reserved name, or
  a row that is `creating`/`disposing`.
