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

Merge diverged repository heads:

```sh
gwz --sync merge pull --head
```

Allow partial completion:

```sh
gwz --sync merge --partial pull --head
```

Select a remote:

```sh
gwz --remote origin pull --head
```

## Notes

- With `--sync merge`, GWZ fetches and then predicts true-merge conflicts
  before changing any selected local branch, index, worktree, or workspace
  lock. It freezes the exact fetched source commit and prepared clean result
  for every changing repository, then revalidates the complete selection
  immediately before the first local mutation. A predicted conflict or
  pre-execution drift rejects a non-partial pull before local mutation.
- Fetch may update remote-tracking refs during preflight. If one changes again
  before the final barrier, GWZ rejects the pull. If it changes after that
  barrier, checked execution still integrates only the exact commit that was
  fetched, predicted, and prepared.
- With `--sync merge --partial`, a predicted-conflict member is reported as
  skipped while clean selected members may proceed. Fetch updates to
  remote-tracking refs can already have occurred during either preflight.
- Pull uses this prediction to avoid beginning a conflict; it does not open the
  coordinated `gwz merge` state machine. Use `gwz merge` when you need
  workspace-wide status, continue, abort, and preservation.
- Other sync policies retain their existing behavior. In particular, rebase
  can still stop in native Git conflict state.
- Use `--dry-run` before large pulls.
- Use `--jobs`, `--max-per-host`, `--progress-interval`, and `--ssh-timeout`
  to tune network behavior.
