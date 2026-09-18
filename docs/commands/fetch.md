# `gwz fetch`

Contact every selected repository's remote and report what moved, integrating
nothing.

```text
gwz fetch
```

`gwz fetch` fetches the configured remote of each selected workspace target,
updates that repository's remote-tracking refs, and reports one line per
repository. Plain `gwz fetch` includes `@root` plus configured member
repositories, and the selectors are [`gwz push`](push.md)'s.

It is the answer to "what moved upstream while I was working?" across the whole
workspace, in one pass and one voice, without changing a single file.

## What it does not do

- **It never integrates.** No merge, no rebase, no fast-forward, no reset: no
  branch, no `HEAD`, no index, no working-tree file. Use [`gwz pull`](pull.md)
  to integrate what a fetch showed you.
- **It never writes workspace artifacts** — no lock, no manifest, no boundary
  sync. Because of that it still runs while a merge is open, unlike `pull` and
  `push`.
- **It never prunes.**
- **It never skips the network, with one exception: `--dry-run`.** There is no
  `--check-remotes` and no "unchanged since the last fetch" short-circuit as
  there is on `gwz push`: a fetch that does not connect has answered nothing.
  The exception is [`gwz --dry-run fetch`](#a-dry-run-contacts-nothing), which
  resolves the selection, prints one planned row per repository it would have
  contacted, and stops before the network.

## Output

```text
$ gwz fetch
status: Partial
@root      .         no change         (origin/main, +2 -0)
mem_core   gwz-core  90ef552..330a174  (origin/main, +0 -3)
mem_cli    gwz-cli   no change         (origin/main, +0 -0)
mem_local  local     no upstream
mem_priv   private   failed            RemoteRejected: member 'mem_priv' at 'private': failed to connect to 127.0.0.1: Connection refused
```

- `<before>..<after>` — the remote-tracking ref moved, abbreviated as `git`
  abbreviates one.
- `new <after>` — the remote-tracking ref did not exist before this fetch, so
  there is no left-hand side to show. This is the row a first fetch prints.
- `updated` — the ref moved but neither object id is known, so there is
  nothing to print on either side.
- `no change` — the repository was contacted and its tracking ref did not move.
- `no upstream` — the repository has no fetch remote, or no attached branch
  whose tracking ref could move. A reported row, not a failure: one local-only
  member does not break a whole-workspace observation.
- `failed` — the remote refused, or the fetch errored, with the reason.
- `would contact <remote>` — `--dry-run` only. The repository was not
  contacted; see below.
- `(<remote>/<branch>, +A -B)` — how far the current branch is ahead of and
  behind the tracking ref, counted after the fetch.

### A dry run contacts nothing

`gwz --dry-run fetch` resolves the selection and reports the repositories it
would contact, contacting none of them. Its rows say so in their own words:

```text
$ gwz --dry-run fetch
status: Noop
@root      .         would contact origin
mem_core   gwz-core  would contact origin
mem_cli    gwz-cli   would contact origin
mem_local  local     no upstream
mem_priv   private   would contact origin
```

`would contact <remote>` is a plan, not a result, and no live fetch prints it.
Its machine value is `"result": "Planned"`. A repository with no fetch remote is
still `no upstream` here, because that answer needs no network. A
`--remote <name>` that a repository does not have is refused here too, as
`failed` with `MissingRemote`, because that answer is also local: a dry run
never promises to contact a remote the live run cannot. Nothing in a dry-run
row comes from a remote, so a dry run cannot tell you what moved: run
`gwz fetch` for that. A dry run exits with the same codes as the live run of
the same selection.

## Examples

Fetch every repository, root included:

```sh
gwz fetch
```

Members only:

```sh
gwz --no-target @root fetch
```

One member:

```sh
gwz --member mem_app fetch
```

Another remote, for every selected repository:

```sh
gwz --remote upstream fetch
```

Machine output, one object per repository under `fetch_repos`:

```sh
gwz --json fetch
```

Resolve the selection and report what would be contacted, contacting nothing:

```sh
gwz --dry-run fetch
```

Every row reads `would contact <remote>`, or `no upstream` where there is no
fetch remote to contact.

## Exit codes

Follow [`gwz push`](push.md)'s, with one difference worth knowing: `no change`
means contacted-and-answered rather than skipped, so a run in which one remote
failed and every other repository read cleanly exits `1` — the report is
incomplete — even though nothing moved.

| Exit | Meaning |
| --- | --- |
| `0` | every selected repository answered |
| `1` | some answered and some failed; the report is incomplete |
| `2` | every selected repository was refused before the network, so nothing was contacted |

Exit `2` is the whole-batch refusal: for example `--remote <name>` naming a
remote that no selected repository has. A refusal that stops the request
before it has a selection at all, such as no workspace under the current
directory or an unknown member id, is a typed error and exits `1`, as it does
on every verb. A dry run shares these codes with the live run it rehearses.

## See also

- [`gwz pull`](pull.md) — integrate what a fetch showed you.
- [`gwz push`](push.md) — the mirror verb, and the source of `fetch`'s
  selection and exit-code conventions.
- [Machine output](../MachineOutput.md#fetch-json) — the `fetch_repos` rows.
