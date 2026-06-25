# `gwz push`

Push workspace member refs to configured remotes.

```text
gwz push [OPTIONS]
```

`gwz push` applies one push request across selected workspace members.

## Examples

Push selected/default members:

```sh
gwz push
```

Push to an explicit remote:

```sh
gwz --remote origin push
```

Push one member by id:

```sh
gwz --member gwz-cli push
```

Preview planned push behavior:

```sh
gwz --dry-run push
```

## Notes

- `gwz push` has no command-specific options.
- Use global selectors to control which members participate.
- Use `gwz tag --push` for tag push workflows.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--progress-interval`, and `--ssh-timeout`.
