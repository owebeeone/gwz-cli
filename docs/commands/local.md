# `gwz local`

Inspect and retire the local clone family of this workspace.

```text
gwz local list
gwz local dispose <name> [--keep | --force <hazard,...>]
gwz local disband
```

A local clone is a second working copy of the whole workspace on the same
machine, created with [`gwz clone --local`](clone.md). The family index lives on
the workspace root and every clone carries a pointer back to it, so these
commands work from any ready family member, including a bare share point.

Family names are resolved from the index at operation time. They are never
written into `gwz.conf/gwz.yml` and never become Git remotes.

## `gwz local list`

Reports every member of the family: name, kind (checkout or bare), state, and
path — the root first, then every member in name order. The listing is
read-only: it takes no lock and repairs nothing.

```sh
gwz local list
```

```text
root  checkout  ready  .
A     checkout  ready  ../gwz-dev-A
C     checkout  ready  ../gwz-dev-C
hub   bare      ready  ../gwz-dev-hub
```

Paths are recorded relative to the workspace root that holds the index.

### The state column

Each row carries two states: the one the index *recorded* (`creating`, `ready`,
`disposing`) and the one that was *observed* on disk (`ready`, `incomplete`,
`interrupted_disposal`, `missing`, `pointer_removed`, `mismatched`,
`malformed`, `unobserved`). They are printed as one word while they agree, and
as `recorded/observed` when they do not, so an interrupted create or an
interrupted disposal is visible here rather than only after the next failure.
Any diagnostic the index recorded for a member is shown beside its row:

```text
root  checkout  ready                           .
B     checkout  creating/incomplete             ../gwz-dev-B   copy interrupted at src/
C     checkout  disposing/interrupted_disposal  ../gwz-dev-C
hub   bare      ready/pointer_removed           ../gwz-dev-hub
```

Reporting is all this does. A row that says `incomplete` or
`interrupted_disposal` stays exactly as it is until you act on it with
`gwz local dispose`.

### Machine output

`--json` and `--jsonl` carry every field of every row under
`local_family_members`, with the enum values spelled as the protocol names
them:

```json
{
  "kind": "response",
  "local_family_members": [
    {
      "name": "root",
      "kind": "Checkout",
      "recorded_state": "Ready",
      "observed_state": "Ready",
      "path": ".",
      "last_error": null
    },
    {
      "name": "B",
      "kind": "Checkout",
      "recorded_state": "Creating",
      "observed_state": "Incomplete",
      "path": "../gwz-dev-B",
      "last_error": "copy interrupted at src/"
    }
  ]
}
```

`dispose` and `disband` carry no rows, so their `local_family_members` is an
empty list.

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

- **Status:** this build parses and dispatches the whole surface described
  here. `gwz local list` is served and reports what the index holds;
  `dispose`, `disband` and `gwz clone --local` still answer
  `UnsupportedOperation` while the engine behind them lands, and nothing is
  created, moved or deleted in the meantime.
- A workspace that holds no family index lists nothing, and that is an answer
  rather than an error — no family is invented, and nothing is repaired.
- `--dry-run` travels to core for every local family verb rather than being
  answered by the CLI; core refuses it today, before any write.
- Hazard names are split on `,` and on nothing else — no trimming, no case
  folding. The vocabulary belongs to core, so an unknown name travels and core
  names it; only a bare `--force`, an empty element and `--keep` together with
  `--force` are refused before the request is encoded.
- Refusals carry a typed error code and are rendered structured under `--json`
  and `--jsonl`. `gwz merge --remote <name>` that names no ready member is
  `unknown_local`, and its message says which — an absent or reserved name, or
  a row that is `creating`/`disposing`.
