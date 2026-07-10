# `gwz diff`

Show changes from the workspace root and active members as one unified diff
whose paths are relative to the workspace.

```sh
gwz diff
gwz diff --cached
gwz diff HEAD
gwz diff main...topic -- gwz-core/src
gwz diff +before-refactor
```

The root is rendered first, followed by members in manifest order. Revisions,
ranges, and `+snapshot` ids are classified independently in each repository.
Put literal pathspecs after `--`.

Useful summary modes mirror Git:

```sh
gwz diff --stat
gwz diff --name-only
gwz diff --name-status
```

`--exit-code` returns 1 when differences exist. `--quiet` suppresses patch
output and implies `--exit-code`, making it useful in scripts:

```sh
gwz diff --quiet
```

Human patch output uses a pager on a terminal and writes directly when piped.
Use `--no-pager` to force direct output. See `gwz help diff` or the generated
[CLI Reference](../CLI.md#gwz-diff) for all patch formatting and selection
options.
