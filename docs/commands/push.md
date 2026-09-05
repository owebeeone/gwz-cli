# `gwz push`

Push workspace target refs to configured remotes.

```text
gwz push [OPTIONS]
```

`gwz push` applies one push request across selected workspace targets. Plain
`gwz push` includes `@root` plus configured member repositories.

## Examples

Push default targets:

```sh
gwz push
```

Push configured members but not the workspace root:

```sh
gwz --all --no-target @root push
```

Push to an explicit remote:

```sh
gwz --remote origin push
```

Publish into another working copy of this workspace on the same machine — a
member of the [local clone family](local.md), typically a bare share point —
by naming it instead:

```sh
gwz push --remote hub
```

Each selected repository's current branch is published to the same branch in
the named member. A name that is not in the family resolves as an ordinary Git
remote, so `--remote origin` keeps its usual meaning.

The flag and its Git meaning are unchanged; the family binding itself is still
landing, so a family name that is not also a Git remote answers
`missing_remote` in this build.

Push one member by id:

```sh
gwz --member gwz-cli push
```

Push only the workspace root:

```sh
gwz --target @root push
```

Preview planned push behavior:

```sh
gwz --dry-run push
```

## Notes

- `gwz push` has no command-specific options.
- Use global selectors to control which targets participate.
- `--all --no-target @root` is the canonical all-members-only selector.
- Use `gwz tag --push` for tag push workflows.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--progress-interval`, and `--ssh-timeout`.
