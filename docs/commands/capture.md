# `gwz capture`

Record the live worktree state into the lock.

```text
gwz capture [OPTIONS]
```

`gwz capture` updates workspace lock state from the live repositories. It does
not check out, pull, push, or otherwise rewrite member worktrees.

## Examples

Capture current selected member state:

```sh
gwz capture
```

Capture all members:

```sh
gwz --all capture
```

Capture one member by path:

```sh
gwz --member-path gwz-cli capture
```

## Notes

- Use `gwz status` before capture to see whether the workspace is dirty.
- Use capture when the current checked-out member revisions are the desired
  lock state.
- Commit updated workspace metadata after capture when the lock change should be
  shared.
