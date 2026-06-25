# `gwz status`

Show Git status across the workspace.

```text
gwz status [OPTIONS]
```

The default mode renders a combined workspace status. File paths are reported
relative to the workspace and prefixed by member path when available.

## Options

| Option | Meaning |
| --- | --- |
| `--combined` | Render combined workspace status. This is the default. |
| `--no-combined` | Render per-repository status with file changes. |
| `--porcelain` | Render stable script-oriented output. |
| `--no-files` | Omit file changes from combined status while keeping branch summaries. |
| `--no-branches` | Omit branch summaries from combined status while keeping file changes. |

## Examples

Show combined human status:

```sh
gwz status
```

Show per-repository status:

```sh
gwz status --no-combined
```

Show status porcelain:

```sh
gwz status --porcelain
```

Select one member:

```sh
gwz --member gwz-cli status
```

## Validation Rules

- `--porcelain` cannot be combined with `--json` or `--jsonl`.
- `--porcelain` cannot be combined with `--no-combined`.
- `--no-files` and `--no-branches` cannot both be supplied.
- `--no-files` and `--no-branches` only apply to combined status.

## Exit Code

A dirty workspace is a normal status result and exits `0`. Rejected requests
exit `2`; failed, partial, or conflicted operations exit `1`.
