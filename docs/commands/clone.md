# `gwz clone`

Clone a GWZ workspace root repository and materialize its members.

```text
gwz clone [OPTIONS] <url> [directory]
gwz clone --local --name <name> [dest] [--clean | --bare] [-b <branch>]
```

`gwz clone` is the one-shot form of cloning the root repository and then running
`gwz materialize --lock`. The root repository is the repository that owns the
tracked `gwz.conf/` directory.

## Arguments

| Argument | Meaning |
| --- | --- |
| `<url>` | Git URL of the workspace root repository. |
| `[directory]` | Target directory. Defaults to a directory name derived from the URL. |

With `--local` there is no URL: the single positional is the destination.

| Option | Meaning |
| --- | --- |
| `--local` | Clone this workspace into a second working copy on this machine. |
| `--name <name>` | Family member name for the new clone. Required with `--local`. |
| `--verbatim` | Copy the source tree as it sits. The default local mode. |
| `--clean` | Take the frozen source state without worktree dirt. |
| `--bare` | Make the destination a share point of bare repositories (implies `--clean`). |
| `-b <branch>` | Create this branch in every destination repository. `--clean`/`--bare` only. |
| `--from <name\|path>` | Copy from another family member. Not yet supported. |

## Examples

Clone into a derived directory:

```sh
gwz clone git@github.com:org/workspace.git
```

Clone into an explicit directory:

```sh
gwz clone git@github.com:org/workspace.git work/demo
```

Clone this workspace locally, tree and Git state as they sit:

```sh
gwz clone --local --name A ../gwz-dev-A
```

Take the frozen state instead, on a new lane branch in every repository:

```sh
gwz clone --local --clean -b lane/agent-17 --name C ../gwz-dev-C
```

Make a bare share point the whole family can push to and pull from:

```sh
gwz clone --local --bare --name hub ../gwz-dev-hub
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
- `gwz clone --local` contacts no network. It requires a quiet source for the
  whole invocation, refuses a verbatim copy while the source has an open
  coordinated merge, and defaults the destination to `../<root-dirname>-<name>`.
- The new clone is registered on the workspace root, not on the workspace you
  ran the command in. List and retire family members with
  [`gwz local`](local.md).
