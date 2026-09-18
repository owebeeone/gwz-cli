# Releases

GWZ releases are distributed through GitHub Releases for the `gwz-cli`
repository:

https://github.com/owebeeone/gwz-cli/releases

The [hosted documentation](https://owebeeone.github.io/gwz-cli/) is built from
the tag of the most recently published release, so its command model matches
the released CLI rather than unreleased work on `main`.

## 1.0.17: `gwz fetch`, and lanes that dispose clean

1.0.17 (2026-09-18) adds one read-only network verb and finishes the lane
disposal clean-up that 1.0.14 and 1.0.16 began.

### `gwz fetch`

`gwz fetch` contacts every selected repository's configured remote, updates
that repository's remote-tracking refs, and prints one row each: the tracking
ref before and after, how far the current branch is ahead of and behind it, or
`new <after>` for a tracking ref that did not exist before, or `updated`,
`no change`, `no upstream`, `failed`. Plain `gwz fetch` covers `@root` plus the
configured members, and the selectors are `gwz push`'s. It answers "what moved
upstream while I was working?" across the whole workspace in one pass, without
changing a file.

```text
$ gwz fetch
status: Partial
@root      .         no change         (origin/main, +2 -0)
mem_core   gwz-core  90ef552..330a174  (origin/main, +0 -3)
mem_cli    gwz-cli   no change         (origin/main, +0 -0)
mem_local  local     no upstream
mem_priv   private   failed            RemoteRejected: member 'mem_priv' at 'private': failed to connect to 127.0.0.1: Connection refused
```

What it never does:

- **It never integrates.** No merge, no rebase, no fast-forward, no reset: no
  branch, no `HEAD`, no index, no working-tree file. Use `gwz pull` to
  integrate what a fetch showed you.
- **It never writes workspace artifacts.** No lock, no manifest, no boundary
  sync. Because of that it still runs while a coordinated merge is open,
  unlike `pull` and `push`.
- **It never prunes.**
- **It never skips the network, with one exception: `--dry-run`.** There is no
  `--check-remotes` and no "unchanged since the last fetch" short-circuit as
  there is on `gwz push`: a fetch that does not connect has answered nothing.
  `--dry-run` is not git's. `git fetch --dry-run` contacts the remote and
  declines to write; `gwz --dry-run fetch` contacts no remote at all, resolving
  the selection and printing the planned rows.

A planned row carries a token of its own, so it can never be read as an answer:

```text
$ gwz --dry-run fetch
status: Noop
@root      .         would contact origin
mem_core   gwz-core  would contact origin
mem_cli    gwz-cli   would contact origin
mem_local  local     no upstream
mem_priv   private   would contact origin
```

`would contact <remote>` is `"result": "Planned"` in the machine output, and no
live fetch prints either. `no change` and `Unchanged` keep their one meaning:
the repository was contacted and its tracking ref did not move. A repository
with no fetch remote is still `no upstream` under `--dry-run`, because that
answer needs no network.

Exit codes follow `gwz push`, with one difference worth knowing: `no change`
means contacted-and-answered rather than skipped, so a run in which one remote
failed and every other repository read cleanly exits `1`, because the report is
incomplete, even though nothing moved.

| Exit | Meaning |
| --- | --- |
| `0` | every selected repository answered |
| `1` | some answered and some failed; the report is incomplete |
| `2` | every selected repository was refused before the network, so nothing was contacted |

Exit `2` is the whole-batch refusal, for example `--remote <name>` naming a
remote no selected repository has; a refusal before there is a selection (no
workspace, an unknown member id) is a typed error and exits `1` as on every
verb. A dry run shares these codes with the live run it rehearses, and refuses
a `--remote` name a repository lacks just as the live run does.

The global `--remote <name>` selects the remote each selected repository
contacts, and `--json` carries the rows under `fetch_repos`.

**Not offered yet.** This release is the verb and nothing more. `--prune`,
`--tags`, per-remote selection and multi-remote fetch for a member that has
more than one remote are all planned and none is implemented; this note does
not say when. See [`gwz fetch`](commands/fetch.md).

### Lane disposal: an integrated lane needs no waiver

`gwz local dispose <name>` now succeeds, with no `--force` and no operator
comparison, on a lane whose work the surviving family already holds, even one
that was built in. Disposal compares the lane against the family, and against
the record the clone wrote of what it copied, so the caches, ignored user data
and stash and reflog entries a verbatim copy inherits are reported without
refusing. What refuses is what only the lane holds.

A refusal sorts what it found into four categories, printing each with its
count and with a description of what it holds, the empty ones included. Most
entries are paths; a protected root is described by its object id and the head
or ref that reaches it:

- **regenerable**: a tool made it and the same tool remakes it. Never refuses.
- **unchanged copy**: the clone copied it, the lane has not touched it, and a
  surviving member still holds it. Never refuses.
- **changed copy**: the clone copied it and it is not the family's any more.
  Refuses.
- **unique to the lane**: the lane alone holds it, including any protected
  root no single surviving family repository preserves whole. Refuses.

The refusal then prints the exact `--force <hazard,...>` command that waives
exactly what it found, and names nothing more: a lane whose only refusing entry
is dirt is offered `--force dirty` even when the same report lists regenerable
entries and unchanged copies beside it.

**Regenerable is recognised by marker and by shape, never by a directory's
name.** A directory holding a `CACHEDIR.TAG` that begins with the Cache
Directory Tagging Specification's signature line; a `__pycache__/` holding
nothing but `.pyc` and `.pyo` files; an `*.egg-info/` holding a `PKG-INFO`; a
`bazel-*` or `razel-*` symlink whose target lies outside the workspace; a file
ending `.so`, `.pyd` or `.dylib` inside a worktree; and an untagged build
directory proved by its tool's own markers (cargo's `.rustc_info.json`, a
`debug/.fingerprint` beside a `debug/deps`, a `pyvenv.cfg`). A `target` with no
cargo marker in it is not regenerable, and neither is anything a probe cannot
read. Recognition does not consult the copy record, so a cache the lane rebuilt
or created from nothing is still a cache.

**A forced deletion now reports what it was actually forced past**, not the
names you typed. A waiver that covered a refusing entry is listed after
`forced past:`; a waiver you named that covered nothing is listed after
`unused waiver:`, so an over-broad `--force` says so in its own report.

The four categories and the exact waiver command shipped in 1.0.16 with
`regenerable` always empty. 1.0.17 fills it. See
[Local Clones](LocalClones.md) and [`gwz local`](commands/local.md).

### Catching up: 1.0.14 and 1.0.16

No release notes were written for 1.0.14 (2026-09-18) or 1.0.16
(2026-09-18). What they carried:

- **Claude Code worktree hooks.** `gwz hook claude-code worktree-create` gives
  a Claude Code session started with `--worktree` a lane, a local clone of the
  whole workspace, instead of the `git worktree` Claude Code would have made,
  which in a GWZ workspace has no members. `gwz hook claude-code
  worktree-remove` disposes that lane through `gwz local dispose`, with GWZ's
  own refusals intact and never `--force` or `--keep`. `gwz hook claude-code
  setup` prints or writes the settings block (`--project`, `--project --local`
  or `--user`, with `--write`) and `--remove` takes it back out, changing no
  byte outside the block. Outside a GWZ workspace the same hooks make and
  remove the plain worktree Claude Code would have made, so the block can live
  in user-level settings without changing other projects. See
  [Claude Code](ClaudeCode.md) and [`gwz hook`](commands/hook.md).
- **`--owner <token>` and `--wait <secs>` on the family verbs.** `gwz local
  clone --owner <token>` records an opaque caller token, up to 128 bytes of
  `[A-Za-z0-9._:-]`, on the new member row, in the same index write that
  reserves the row. It never changes afterwards, `gwz local list` reports it,
  and GWZ never interprets it: it is the caller's identity for the caller's own
  reuse decisions. `--wait <secs>` makes a busy family lock retry until the
  deadline instead of refusing `Busy` at once, on every family verb and on
  `gwz merge --remote <name>`; a wait that wins rereads the index before
  acting.
- **Family index schema v2.** Recording an owner needs a place to put it, so
  the index carries `gwz.local-family/v2`: v1 plus the optional per-row
  `owner`. A 1.0.14 or later gwz reads a v1 index unchanged and writes v2 on
  its first write of any kind, a create, a dispose, a `--keep` or a family
  merge. Going the other way is a refusal rather than a downgrade: an older gwz
  refuses a v2 index as a whole, and its refusal names no gwz version at all.
  On 1.0.13 it reads ``gwz: ManifestInvalid: ... .gwz/local-family.yml is
  malformed: `schema: gwz.local-family/v2` is not `gwz.local-family/v1` (format
  version 1); this file is not in a format this store reads``. That line is
  derived from the 1.0.13 source
  (`v1.0.13:crates/family-store/src/format.rs:37-42`), not from a run on this
  machine. It calls the file malformed, which it is not, and offers no remedy;
  the remedy is to upgrade. **Once a 1.0.14 or later gwz has written a
  workspace's family index, every gwz used on that workspace must be 1.0.14 or
  later.**
- **Lane disposal clean-up, Phase 1.** `gwz local clone` records what it
  copied per repository, and `gwz local dispose` uses that record so an
  unchanged copy the family still holds is reported instead of refusing. Where
  no record exists, because the lane came from an older gwz or was copied
  outside gwz, dispose makes the comparison against the family itself. The
  identical-copy witness settled `unpreserved-history` for a merged lane: a
  copy the surviving family holds object for object is not a loss. 1.0.16
  completed the phase with the four-category refusal and the exact waiver
  command described above.
- **`gwz ls` says why a listed member is not on disk.** A member the lock
  records but that is absent is still listed, rather than quietly hidden, with
  `materialized: false` and a human note: `(private, skipped)` for the
  quiet-clone case, `recorded in the lock but absent on disk` for any other.
  Members the lock never materialized are unchanged, omitted unless
  `--unmaterialized` and carrying no note. Read `materialized`, not the note,
  to decide anything. See [`gwz ls`](commands/ls.md).

## 1.0.4: the 1.0 line ships

1.0.4 (2026-09-08) is the first published release of the 1.0 series — the
1.0.0 release candidates and 1.0.2/1.0.3 were pre-publication and packaging
iterations from the same morning. If you are upgrading from 0.14 or earlier,
read this section together with the lane-recovery notes below.

The 1.0 series delivers the **local clone family** (`gwz local`): a verbatim,
isolated second copy of the whole workspace — its own repositories, its own
runtime state, its own merges — created with `gwz local clone NAME`, integrated
back by name with `gwz merge --remote NAME`, and disposable only when its
history provably survives elsewhere. See [Local Clones](LocalClones.md) for the
lane workflow and [measured disk/speed data](LocalClones.md#disk-space-and-copy-speed)
(on reflink filesystems, ten whole-workspace lanes cost ~20.5 MiB; on ext4 the
same ten cost ~537 MiB as ordinary copies).

Release verification for 1.0.4 (2026-09-08) covered install, workspace clone,
local clone, detach, and delete on Linux ARM64 and Windows x86_64, alongside
the release pipeline's own gates.

## 1.0.0: lane recovery

This release makes the ordinary lane cycle work with one default merge:

```sh
gwz local clone A
# Work and use gwz add / gwz commit in A.
gwz merge --remote A
gwz local dispose A
```

- **Merge selection changes:** ordinary and family merges include the workspace
  root and active members by default. To merge members only, use
  `gwz --target @all --no-target @root merge`. An explicit partial merge can
  leave root history unpreserved and prevent lane disposal.
- Untouched clones can be disposed despite a regenerated configuration marker.
  Only freshly verified generated bytes are discounted; actual edits and unique
  history remain protected. The marker can still appear modified in status.
- Network publication captures source refs before transfers and withholds the
  root if selected member publication fails or locked dependencies cannot be
  proved available. Earlier member pushes may already have succeeded; repair
  the reported failure and retry.
- SSH file selection is explicit through `--identity PATH` or
  `--remote-identity NAME=PATH`, without agent fallback for the selected file.
  Exact selection of an encrypted key inside an agent remains unsupported.
- Only verbatim local clones are supported. `--clean`, `--bare`, `--from`, and
  their branch option remain refused, as do family pull/push. Use the receiving
  workspace's `merge --remote NAME` to integrate a lane.

Windows publication and fresh-lock verification, Windows/Linux lane lifecycles,
and Linux ARM64 ext4 operation have focused acceptance evidence. XFS supports
space-efficient CoW copies; ext4 uses independent ordinary copies. See
[Local Clones](LocalClones.md#disk-space-and-copy-speed) for measured scope.

The older open-merge upgrade restriction below still applies.

## Upgrading To 0.14.0

**Close any open coordinated merge before you upgrade.** GWZ 0.14 has one merge
implementation, and it cannot act on a merge record written by 0.13.x or
earlier. On 0.13.x, run `gwz merge --status`, then finish the merge with
`gwz merge --continue` or undo it with `gwz merge --abort`. Only then install
0.14.

If a pre-0.14 merge is still open after the upgrade, GWZ says so and stops.
Every merge verb, and every command a merge blocks, refuses with one sentence:

```text
gwz: OpenOperation: this is a pre-0.14 merge; use gwz 0.13.0 (the last release before 0.14) to continue or abort
```

`--status`, `--continue` and `--abort` all refuse alike, so the usual
open-merge advice does not apply and is deliberately not printed. Nothing is
wrong with the record and nothing needs repairing: reinstall a 0.13.x build,
close the merge with it, and return to 0.14. There is no conversion step and no
in-place upgrade for an open record.

Merges that were already closed are unaffected. Archived records still project
read-only through `gwz merge --status <merge-id>`, and `gwz merge --gc` never
deletes an archive it cannot read.

Starting a merge is otherwise unchanged. Ordinary, `--ff-only`,
custom-message and `--no-ff` starts now all write the same coordinated merge
record, so status, continue, abort and recovery behave identically whichever
way a merge was started.

## Compatibility Notes

Behaviour a consumer of GWZ's output or metadata has to account for. Every
note here describes **released** behaviour: the log, merge, anchor-directory
and `gwz.conf/` notes below all shipped across the 0.10 to 1.0 releases.
Unreleased work is described in the version section it will ship in, not here.

- `gwz log` adds one newest-first history across the workspace root and selected
  member repositories, with coordinated-marker and conservative heuristic
  coalescing, revision/snapshot/lock ranges, six filters, compact and full human
  rendering, and the dedicated `gwz.log/v0` JSON/JSONL record schema. The
  Python API and `gwz-py log` expose the same records and rendering contract.
- Structural workspace commands refuse uncommitted hand edits to `gwz.conf/`
  (the machine-managed manifest and lock). The refusal names the sanctioned
  `gwz repo` verbs and the acceptance path (`gwz init --update --force`);
  states produced by git itself — clone, pull, branch switch — reconcile
  silently, and read-only/list commands never gate. Workspace bootstrap also
  emits a machine-managed banner on `gwz.yml`, records digests in
  `gwz.conf/markers/conf-integrity.yml`, and writes or merges a
  `.claude/settings.json` deny rule (`Edit(/gwz.conf/**)`) for agent sessions
  started at the workspace root.
- A checked-artifact private anchor directory holding a foreign,
  non-canonical retired-anchor rendering (for example
  `.ca1-anchor-retired-007`) now refuses operations on that family until the
  foreign file is removed; such names were previously adopted silently. The
  canonical rendering is unpadded (`.ca1-anchor-retired-7`).
- First-class merge JSON and JSONL include the complete current merge-response
  key set, including finalization progress. Structured errors include
  `target_kind` and retain member id/path context even for whole-operation
  preflight failures. Durable record compatibility errors also include typed
  `record_context` rather than encoding merge id, schema/version, required
  wave, or legacy mode only in prose. Consumers must tolerate additive keys within these versioned envelopes.
- Merge status rows expose durable pending-action reconciliation as
  `NotStarted`, `ExpectedConflict`, `CompletedExactly`, or `Ambiguous`.
  Ambiguity is also reported as dedicated structured drift and remains
  mutation-blocking.
- Merge and `pull --sync merge` reject source and target commits with unrelated
  histories, matching Git porcelain. GWZ does not implicitly allow unrelated
  histories.
- While a coordinated merge is open, the accepted workspace lock remains the
  exact pre-merge baseline. Clean, conflicted, failed, and unattempted outcomes
  are retained in the local durable operation record rather than published as
  a partial composition.
- Merge commits use the quoted default message
  `Merge '<source>' into '<target-branch>'`, or the body supplied with `-m`,
  with mandatory `GWZ-Merge-ID` and `GWZ-Operation-ID` identity lines. The
  exact final message is frozen before mutation and survives restart and
  conflict resolution. A request that creates a commit supplies its
  author/committer identity when present; otherwise the target repository
  identity is used.
- Coordinated merge start, dry-run, status, continue, and safe abort are
  available together. Successful changed merges publish a checked root
  composition commit; interrupted finalization resumes idempotently.
  Recovery must not substitute raw `git merge --abort` for the coordinated
  operation.

## Install Latest

macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Windows PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "iex (irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1)"
```

## Install A Pinned Version

Replace `latest` with a concrete tag:

```text
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.sh
https://github.com/owebeeone/gwz-cli/releases/download/v0.9.0/gwz-installer.ps1
```

## Verify Assets

Release assets are checksummed and have GitHub artifact attestations. For
stronger verification:

1. Download the release archive and checksum file.
2. Compare the archive SHA-256 with the checksum.
3. Verify the GitHub artifact attestation.
4. Run `gwz --version`.
5. Run `gwz --help`.

## Smoke Test A Unix Installer

```sh
tmp="$(mktemp -d)"

curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh \
  -o "${tmp}/gwz-installer.sh"

GWZ_UNMANAGED_INSTALL="${tmp}/bin" \
GWZ_NO_MODIFY_PATH=1 \
sh "${tmp}/gwz-installer.sh"

"${tmp}/bin/gwz" --version
"${tmp}/bin/gwz" --help
```

## Smoke Test A Windows Installer

```powershell
$ErrorActionPreference = "Stop"

$tmp = Join-Path $env:TEMP "gwz-test-$([guid]::NewGuid())"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

$installer = Join-Path $tmp "gwz-installer.ps1"
Invoke-WebRequest `
  "https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1" `
  -OutFile $installer

$env:GWZ_UNMANAGED_INSTALL = Join-Path $tmp "bin"
$env:GWZ_NO_MODIFY_PATH = "1"

Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& $installer

$exe = Join-Path $env:GWZ_UNMANAGED_INSTALL "gwz.exe"
& $exe --version
& $exe --help
```

## Local Release Checks

From the development workspace:

```sh
cargo fmt --check
cargo test
cargo run -q -p gwz -- --help
python scripts/generate_cli_reference.py --check
```

When changing command docs, inspect the generated reference and spot-check
command help for the affected commands:

```sh
cargo run -q -p gwz -- help status
cargo run -q -p gwz -- help tag
```

`scripts/release.py` runs the generated CLI reference check by default before it
commits the release worktree. If the release must proceed while docs are being
reconciled separately, pass `--no-doc-check`; otherwise update the reference
with:

```sh
python scripts/generate_cli_reference.py --write
```
