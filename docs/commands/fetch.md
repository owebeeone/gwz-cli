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
- **It never skips the network.** There is no `--check-remotes` and no
  "unchanged since the last fetch" short-circuit as there is on `gwz push`: a
  fetch that does not connect has answered nothing.

## Output

```text
$ gwz fetch
status: Partial
@root     .         no change          (origin/main, +0 -0)
mem_core  gwz-core  a1b2c3d..9f8e7d6   (origin/main, +0 -3)
mem_cli   gwz-cli   no change          (origin/main, +2 -0)
mem_local local     no upstream
mem_priv  private   failed             RemoteRejected: connection refused
```

- `<before>..<after>` — the remote-tracking ref moved, abbreviated as `git`
  abbreviates one.
- `no change` — the repository was contacted and its tracking ref did not move.
- `no upstream` — the repository has no fetch remote, or no attached branch
  whose tracking ref could move. A reported row, not a failure: one local-only
  member does not break a whole-workspace observation.
- `failed` — the remote refused, or the fetch errored, with the reason.
- `(<remote>/<branch>, +A -B)` — how far the current branch is ahead of and
  behind the tracking ref, counted after the fetch.

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

## Exit codes

Follow [`gwz push`](push.md)'s, with one difference worth knowing: `no change`
means contacted-and-answered rather than skipped, so a run in which one remote
failed and every other repository read cleanly exits `1` — the report is
incomplete — even though nothing moved.

| Exit | Meaning |
| --- | --- |
| `0` | every selected repository answered |
| `1` | some answered and some failed; the report is incomplete |
| `2` | the request was refused before any remote was contacted |

## See also

- [`gwz pull`](pull.md) — integrate what a fetch showed you.
- [`gwz push`](push.md) — the mirror verb, and the source of `fetch`'s
  selection and exit-code conventions.
- [Machine output](../MachineOutput.md#fetch-json) — the `fetch_repos` rows.
