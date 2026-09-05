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
path. The listing is read-only — it takes no lock, and repairs nothing.

```sh
gwz local list
```

```text
root  checkout  ready  /work/gwz-dev
A     checkout  ready  /work/gwz-dev-A
C     checkout  ready  /work/gwz-dev-C
hub   bare      ready  /work/gwz-dev-hub
```

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
  here, and every local clone family operation answers `UnsupportedOperation`
  — the engine behind it is still landing. Nothing is created, moved or
  deleted in the meantime.
- These verbs need a family: with no local clone family recorded, they answer
  that the family is missing rather than inventing one.
- `--dry-run` is not supported for the local family verbs.
- Refusals carry a typed error code and are rendered structured under `--json`
  and `--jsonl`.
