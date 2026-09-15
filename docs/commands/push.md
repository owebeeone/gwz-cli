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

Read every selected remote and every root dependency, including repositories
unchanged since the last fetch or push:

```sh
gwz push --check-remotes
```

## Notes

- `--check-remotes` is the only command-specific option.
- Use global selectors to control which targets participate.
- `--all --no-target @root` is the canonical all-members-only selector.
- Use `gwz tag --push` for tag push workflows.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--progress-interval`, and `--ssh-timeout`.

## Publication and authentication

GWZ captures the selected source refs before transfer. A failed selected member
push withholds root publication, and a push that contacts the root, root-only
pushes included, must first prove that the committed lock's dependencies are
available. A refusal can follow successful member transfers, so read the
per-target results, repair the failure and retry.

- Root dependencies are proven by this operation's own reads or accepted
  pushes, never by remote-tracking refs.
- By default, repositories unchanged since the last fetch or push are not
  checked for changes or pushed. Each reports `Noop`; human output counts them
  in one summary line, and `--verbose` shows each reason. A push that contacts
  the root still reads each dependency.
- `--check-remotes` reads every selected remote and every root dependency,
  pushes repositories whose remote lacks their branch's commit, and proves a
  selected root even when it has nothing to push.

See [Publication](../Concepts.md#publication) for the URL each dependency is
read at, what each `Noop` reason means, and when to use `--check-remotes`.

Select a private-key file for an SSH destination:

```sh
gwz --identity ~/.ssh/project_key push
gwz --remote-identity "origin=$HOME/.ssh/project_key" push
```

A selected file does not fall back to another agent identity if it fails.
Exact encrypted-agent key selection is unsupported. File selection does not
grant access: the destination must authorize that key. Use `--ssh-timeout` to
bound stalled SSH reads; setting it to zero disables that timeout.

A `--remote-identity` for a remote that this push reaches over HTTPS, as its
own destination or to read a root dependency, is refused before any transfer;
use `--identity PATH`, which HTTPS destinations ignore, or no override for that
remote.
