# `gwz commit`

Commit staged changes across the selected targets.

```text
gwz commit [OPTIONS] --message <message>
```

The commit message is applied to every repository that receives a commit.

## Selection Semantics

The default selection is the workspace root plus every active member, so a plain
`gwz commit` commits both. A narrowed selection commits only what it names: with
`gwz --member <id> commit`, the root is **not** committed — its index (including
the refreshed `gwz.conf` lock) is left staged for a later commit. The root is
committed only when the selection includes it (default, `--target @root`, or
`--all`), and it is always committed last so it records the post-commit
composition.

Every commit produced by one `gwz commit` carries the same
`GWZ-Commit-ID`/`GWZ-Workspace-ID` message trailers, which is what `gwz log`
coalesces on. The marker artifact under `gwz.conf/markers/` is written only when
the root is committed, so a marker is never left pending; pass
`--no-commit-marker` to skip both the trailers and the marker for one commit.

## Options

| Option | Meaning |
| --- | --- |
| `-m`, `--message <message>` | Commit message applied to every committed repository. Required. |
| `-a` | Stage tracked modifications first, like `git commit -a`. |
| `--all` | The global target selector (`@all`), not the staging flag. |
| `--commit-marker` | Force GWZ commit marker creation (the default when the root is committed). |
| `--no-commit-marker` | Disable the marker and the message trailers for this commit. |

## Examples

Commit staged changes:

```sh
gwz commit -m "Update workspace docs"
```

Stage tracked modifications and commit:

```sh
gwz commit -a -m "Refresh generated files"
```

Commit only selected members:

```sh
gwz --member gwz-cli commit -m "Update CLI docs"
```

## Notes

- Use `gwz add` before `gwz commit` when you need to stage new files or
  selected pathspecs.
- `-a` stages tracked modifications only; it does not stage new untracked
  files.
- A member-scoped commit leaves whatever else is staged in the root index
  staged; it never sweeps it into a commit.
- `--dry-run` plans the commit and mutates nothing.
- Run `gwz status` before and after commit to confirm what changed.
