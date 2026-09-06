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

Naming another working copy of this workspace on the same machine — a member
of the [local clone family](local.md) — is the intended form, with each
selected repository's current branch published to the same branch there:

```sh
gwz push --remote hub
```

**This build does not serve it.** The family name is looked up and the flag
then falls through to ordinary Git remote resolution, so a family name that
is not also a Git remote fails every selected target with
`MissingRemote: missing remote 'hub'` (`missing_remote` in this build's
machine output). `--remote origin` keeps its usual meaning. Integrate from the
receiving side with `gwz merge --remote <name>` instead; see
[`gwz merge`](merge.md) and [Local Clones](../LocalClones.md).

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
