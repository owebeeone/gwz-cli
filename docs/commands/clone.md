# `gwz clone`

Clone a GWZ workspace root repository and materialize its members.

```text
gwz clone [OPTIONS] <url> [directory]
```

`gwz clone` is the one-shot form of cloning the root repository and then running
`gwz materialize --lock`. The root repository is the repository that owns the
tracked `gwz.conf/` directory.

## Arguments

| Argument | Meaning |
| --- | --- |
| `<url>` | Git URL of the workspace root repository. |
| `[directory]` | Target directory. Defaults to a directory name derived from the URL. |

## Examples

Clone into a derived directory:

```sh
gwz clone git@github.com:org/workspace.git
```

Clone into an explicit directory:

```sh
gwz clone git@github.com:org/workspace.git work/demo
```

Complete a workspace after plain `git clone`:

```sh
gwz materialize --lock
```

## Notes

- `gwz clone` verifies that the cloned root is a GWZ workspace.
- Missing member repositories are cloned and checked out at lock revisions.
- `--dry-run` is rejected for `gwz clone`.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--remote`, `--progress-interval`, and `--ssh-timeout`.
