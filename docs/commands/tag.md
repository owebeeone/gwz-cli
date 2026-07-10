# `gwz tag`

Manage real Git tags across the workspace.

```text
gwz tag [OPTIONS] [name]
```

`gwz tag` fans out tag operations the way `gwz commit` fans out commits. Local
create, list, and delete operations span the selected members plus the committed
workspace root. Remote push, fetch, list, and delete operations span members.

## Operations

| Operation | Command |
| --- | --- |
| Create lightweight tag | `gwz tag <name>` |
| Create annotated tag | `gwz tag <name> -m <message>` |
| Create signed tag | `gwz tag <name> -s` |
| List local tags | `gwz tag` or `gwz tag --list` |
| Delete local tag | `gwz tag --delete <name>` |
| Push one tag | `gwz tag --push <name>` |
| Push all tags | `gwz tag --push` |
| Fetch tags | `gwz tag --fetch` |
| List remote tags | `gwz tag --list --remote <name>` |
| Materialize a tag | `gwz materialize --tag <name>` |

## Options

| Option | Meaning |
| --- | --- |
| `--list` | List tags. This is the default with no name. |
| `--delete` | Delete the named tag. |
| `--push` | Push a named tag, or all tags if no name is supplied. |
| `--fetch` | Fetch tags from the selected remote. |
| `-m <message>` | Annotated tag message. |
| `-s`, `--sign` | Create a signed tag. |

Use global `--remote <name>` for remote tag operations.

## Examples

Create a tag:

```sh
gwz tag v0.9.0
```

Create an annotated tag:

```sh
gwz tag v0.9.0 -m "GWZ v0.9.0"
```

List local tags:

```sh
gwz tag
```

Delete a local tag:

```sh
gwz tag --delete v0.9.0
```

Push a tag:

```sh
gwz tag --push v0.9.0
```

Fetch tags:

```sh
gwz tag --fetch
```

List remote tags:

```sh
gwz tag --list --remote origin
```

Materialize the workspace at a tag:

```sh
gwz materialize --tag v0.9.0
```

## Notes

- Tags are Git refs, not GWZ-specific tag artifacts.
- Ensure the workspace root is committed before creating a tag that should
  include root metadata.
- Use `--dry-run` to inspect planned effects before mutating tags.
