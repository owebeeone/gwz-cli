# `gwz add`

Stage file contents across workspace repositories.

```text
gwz add [OPTIONS] [pathspec]...
```

`gwz add` is the multi-repository equivalent of `git add`. Each pathspec is
resolved relative to the current directory, routed to the member or workspace
root repository that owns it, and staged there.

## Arguments And Options

| Item | Meaning |
| --- | --- |
| `[pathspec]...` | Paths to stage, resolved like `git add`. |
| `-A`, `--all` | Stage all changes across every workspace repository. |

## Examples

Stage one path:

```sh
gwz add gwz-cli/README.md
```

Stage paths in different repositories:

```sh
gwz add gwz-cli/README.md gwz-core/src/lib.rs
```

Stage everything:

```sh
gwz add -A
```

## Notes

- Pair `gwz add` with `gwz commit`.
- Use `gwz repo add` to register an existing repository as a workspace member.
- `--dry-run` plans the staging operation without changing indexes.
