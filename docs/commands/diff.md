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

When release or checkpoint tags exist in only part of the workspace, use
`--tagged` to select their intersection automatically:

```sh
gwz diff --tagged v0.10.2 v0.10.3
```

Each operand is treated as an exact local tag name, and only repositories that
contain every supplied tag are diffed. The workspace root is treated like any
other candidate: it is included when it has all the tags and otherwise
excluded. Global or explicit `--target` selection establishes the candidate set
before tag filtering. In JSON and JSONL output, omitted candidates appear in
`excluded_targets` with reason `TagMissing` (`tag_missing` in the protocol
schema).

One tag compares that tag with the worktree (or index with `--cached`). Two
tags, or a closed `A..B`/`A...B` range, compare tag-to-tag. Snapshot operands
and open ranges are not accepted in this mode. Outside `--tagged`, revision
resolution remains strict in every selected repository.

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
