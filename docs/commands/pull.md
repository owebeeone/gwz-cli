# `gwz pull`

Move workspace members forward to an explicit target.

```text
gwz pull [OPTIONS]
```

The default target is repository heads. The default sync policy is
fast-forward only.

## Targets

| Option | Meaning |
| --- | --- |
| `--head` | Pull repository heads. This is the default. |
| `--snapshot <name>` | Pull a workspace snapshot. |

Only one target flag may be supplied.

## Examples

Pull heads:

```sh
gwz pull --head
```

Pull a snapshot:

```sh
gwz pull --snapshot integration-baseline
```

Fetch without updating worktrees:

```sh
gwz --sync fetch-only pull --head
```

Allow partial completion:

```sh
gwz --partial pull --head
```

Select a remote:

```sh
gwz --remote origin pull --head
```

## Notes

- GWZ rejects operations that cannot update selected members cleanly unless
  policy flags such as `--partial`, `--force`, or `--sync` change the behavior.
- Use `--dry-run` before large pulls.
- Use `--jobs`, `--max-per-host`, `--progress-interval`, and `--ssh-timeout`
  to tune network behavior.
