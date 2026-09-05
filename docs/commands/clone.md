# `gwz clone`

Clone a GWZ workspace root repository and materialize its members.

```text
gwz clone [OPTIONS] <url> [directory]
gwz clone --local --name <name> [dest] [--clean | --bare] [-b <branch>] [--from <name|path>]
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
| `--from <name\|path>` | Copy from this family member or path instead of the current workspace. |

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

Copy a different member instead of the workspace you are standing in. The new
clone is still registered on the root, whichever member it was copied from:

```sh
gwz clone --local --clean --from A --name B ../gwz-dev-B
gwz clone --local --from ../gwz-dev-C --name D ../gwz-dev-D
```

Complete a workspace after plain `git clone`:

```sh
gwz materialize --lock
```

## Notes

- `gwz clone` verifies that the cloned root is a GWZ workspace.
- Missing member repositories are cloned and checked out at lock revisions.
- `--dry-run` is rejected for the URL clone. On `gwz clone --local` it is a
  family operation like any other: the flag travels and core answers for it
  (today, `UnsupportedOperation`, before any destination is allocated).
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--remote`, `--progress-interval`, and `--ssh-timeout`.
- `gwz clone --local` is parsed and dispatched by this build, and answers
  `UnsupportedOperation` until the local clone engine lands. It creates
  nothing in the meantime. `--from` reaches core in the request's
  `copy_source` field and earns its own `UnsupportedOperation` until the copy
  redirect lands. The URL clone is unaffected.
- `gwz clone --local` contacts no network. It requires a quiet source for the
  whole invocation, refuses a verbatim copy while the source has an open
  coordinated merge, and defaults the destination to `../<root-dirname>-<name>`.
- `--from` names the *source* to copy, not the destination. Core resolves the
  token — a family name recorded in the index, or a filesystem path — and
  refuses one that names neither. An empty `--from` is refused by the CLI,
  because on the wire it would be indistinguishable from "copy this
  workspace".
- The new clone is registered on the workspace root, not on the workspace you
  ran the command in. List and retire family members with
  [`gwz local`](local.md).
