# `gwz commit`

Commit staged changes across members and the workspace root.

```text
gwz commit [OPTIONS] --message <message>
```

The commit message is applied to every repository that receives a commit.

## Options

| Option | Meaning |
| --- | --- |
| `-m`, `--message <message>` | Commit message applied to every committed repository. Required. |
| `-a`, `--all` | Stage tracked modifications first, like `git commit -a`. |

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
- Run `gwz status` before and after commit to confirm what changed.
