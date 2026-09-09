# Local Clones

A local clone is a second working copy of the whole workspace on the same
machine: the root repository, every member repository, and — by default —
everything sitting in the tree, uncommitted work and build output included.
GWZ registers it by name in the workspace's **local clone family**, and that
name is how you bring its work back: `gwz merge --remote <name>`.

This page explains the model, the lifecycle, and the deletion rules. The
command reference is [`gwz local`](commands/local.md); the merge side is the
"Merging from a local clone" section of [`gwz merge`](commands/merge.md).
Throughout, a *lane* is a local clone used as an isolated place to work. The
two words mean the same thing.

## The Problem A Lane Solves

Two pieces of work that must not see each other — two agents, or you and a
long-running experiment — need two trees. Git's own answer, a linked
`git worktree`, gives each tree its own checkout but not its own repository:
worktrees share branch refs, and a coordinated GWZ merge moves refs and
records evidence per repository. Two worktrees therefore cannot each carry an
independent coordinated merge. Cloning the workspace again from its remote
isolates properly, but it costs a network transfer, loses everything that was
not committed and pushed, and leaves you with no name for the other copy.

A local clone is a separate repository for every member, made by copying the
tree as it sits. Each lane can run its own structural merges, hold its own
open merge, and keep its own stashes and records. The family gives the lanes
names, so integration is `gwz merge --remote A` rather than a path.

## What This Build Serves

The command surface is wider than what runs behind it, so this first:

- **Served end to end:** the verbatim `gwz local clone <name> [dest]`,
  `gwz local list`, `gwz merge --remote <name> [<ref>]`, `gwz local dispose
  <name>` with its preservation checks and named `--force` waivers,
  `gwz local dispose <name> --keep`, and `gwz local disband`.
- **Parsed but refused:** `--clean`, `--bare` and `--from` on
  `gwz local clone`, and family names on `gwz pull` and `gwz push`. Each
  refuses with a typed error before anything is written. The exact messages
  are under [Not Yet](#not-yet).

Nothing on this page promises when a refused form starts working.

## Disk Space and Copy Speed

Source and destination must share a reflink-capable filesystem to benefit from
copy-on-write cloning. Without that support, GWZ makes ordinary independent
copies. Later edits remain private in either case.

A Raspberry Pi test with a roughly 54 MiB, eight-repository workspace measured
one clone at **2.10 MiB of additional XFS space versus 53.72 MiB on ext4**,
repeated twice on fresh images. Clone times overlapped at 1.2–1.4 seconds.
Ten retained clones used about 20.5 MiB additional XFS space versus 537.2 MiB
on ext4. These are incremental filesystem measurements excluding the source,
not per-directory `du` totals that can double-count shared extents.

The test used 10 GiB loop images on ext4 host storage, a development build,
and normal caches on a shared ARM64 machine. Larger workspaces, build output,
subsequent writes and native partitions can behave differently.

## The Family Model

A family is one original workspace, whose reserved name is `root`, plus the
named clones made from it.

- **The index lives on the workspace root**, as the untracked file
  `.gwz/local-family.yml`. It is the only copy. Every clone carries a pointer
  back to the root instead (`.gwz/family-root`, holding the family id and the
  root's path), and reads the index from there.
- **Every family command works from any ready member.** `gwz local list`
  run inside a clone still shows the whole family, and a clone made from a
  clone is registered on the root, not on its parent. There is one namespace,
  however the clones were made.
- **Names are resolved at operation time**, from the index, on every
  command. They are never written into `gwz.conf/`, never recorded in the
  manifest or lock, and never become Git remotes. `gwz forall -- git remote
  -v` shows nothing for a family member, and `gwz repo sync` never learns of
  one.
- **`root`, `origin` and Git's reserved ref names are refused as clone
  names**, and so is a name the family already holds.
- **The family is local runtime state.** Nothing about it is committed or
  pushed; a clone of your workspace made elsewhere has no family.

## The Lifecycle

The sample paths and ids below are illustrative — a workspace at
`/Users/you/work/demo` with one member, `services/api`. Every command and
every line of output was produced by the current build; only the paths were
shortened.

### 1. Create

From inside the workspace:

```sh
gwz local clone A
```

```text
status: Ok
created local clone `A` at /Users/you/work/demo-A (verbatim; recorded as ../demo-A; 61 files copied (61 natively, 0 ordinarily), 48 directories, 0 symlinks, 37774 logical bytes; 0 remote URL(s) removed; dest-complete: 2 repositories, 15 objects verified of 20 in store, 1 ms; family fam_3fa95fa799ee9ec5b04999adba13e651, founded)
```

The destination defaults to `../<root-dirname>-<name>` beside the workspace
root; a second operand names it explicitly. The report says what happened:
`natively` counts files cloned through the filesystem's own copy-on-write
facility where it has one, `ordinarily` files copied byte by byte; the
result is a private copy either way, never a hardlink into the source.
`remote URL(s) removed` counts remotes whose URL named a filesystem path or
carried credentials, which are stripped from the copied Git configuration —
an ordinary `origin` stays. `dest-complete` is a connectivity check that
every protected root of every copied repository is readable from the
destination's own object store. `founded` appears on the create that starts
the family; later creates report the same family id without it.

**A create copies the tree as it sits.** Staged edits, unstaged edits,
untracked files and build directories are all in the clone, because that is
the point of a verbatim copy: a lane starts exactly where you were. What is
not copied is GWZ's own runtime state under `.gwz/` — the family index, merge
records, stash bundles and locks — which the clone gets fresh. One visible
consequence: a new clone can show `gwz.conf/markers/conf-integrity.yml` as
modified, because the clone writes a marker for the lock bytes it actually
holds while the source's committed marker may have gone stale through
git-produced states. Disposal discounts only the exact generated bytes when
fresh checks prove the manifest and lock unchanged in HEAD, index and worktree.
Staged, edited or uncertain marker state still blocks deletion. Source marker
work is preserved; if it cannot verify the copied configuration, creation refuses
before allocating a lane so it cannot silently overwrite that work.

**Keep the source quiet for the whole copy.** GWZ takes its family lock,
which serializes family commands, but nothing stops an editor, a build, or a
raw `git` command from writing into the source while it is being copied.
Stop those before you start, and do not resume them until the create has
reported. A create is refused while the source has an open coordinated merge:

```text
gwz: OpenOperation: local clone `Y` -> /Users/you/work/demo-Y: reserve failed: refused: source has an open gwz merge (merge_op_82653_1788669594115_0001); abort it or use --clean; effects: []; nothing was reserved
```

Finish or abort that merge first. The `--clean` the message offers is a mode
this build does not serve (see [Not Yet](#not-yet)).

A create also refuses, before anything is written, a nonempty destination, a
destination that is already a workspace or lies inside a family member, and a
name the family already holds:

```text
gwz: PathCollision: local clone `A` -> /Users/you/work/elsewhere: reserve failed: refused: name `A` already holds ../demo-A; effects: []; nothing was reserved
```

An interrupted create leaves its files and a `creating` row for you to
inspect; nothing later finishes or removes them on its own. `gwz local list`
shows the row as `creating/incomplete`, and `gwz local dispose <name> --keep`
detaches it.

### 2. Work

A lane is an ordinary GWZ workspace. Every command works there as it does at
the root, and its merges, stashes and records are its own: two lanes can each
have an open merge at the same time.

```sh
cd ../demo-A
gwz status
gwz add services/api/main.rs
gwz commit -m "Lane A change"
```

Note that `gwz commit` in a lane commits the lane's **root repository** too,
as it does anywhere — the lock moves with the member commit. That root commit
is history the original workspace does not hold, and it matters when the
lane is retired.

From any member, the family is visible:

```sh
gwz local list
```

```text
root  checkout  ready  /Users/you/work/demo
A     checkout  ready  /Users/you/work/demo-A
```

The listing is read-only: it takes no lock and repairs nothing. A workspace
that is in no family answers `no local clone family members`, which is an
answer, not an error. The state column is explained on the
[command page](commands/local.md#the-state-column).

### 3. Integrate

Back at the root (or in any other lane), merge the lane's work by name:

```sh
gwz merge --remote A
```

The result lists the root and selected members as participants. Their states
may differ: a member can fast-forward while the root requires a merge commit
to preserve both histories and record the resulting member lock.

What happened, in order:

1. GWZ paired every selected repository with its counterpart in `A` by
   recorded member identity, not by path, and refused before any transfer if
   the two workspaces were no longer the same shape.
2. For each pair it fetched the source commit — `A`'s HEAD, or the ref you
   named — into the receiving repository under one fresh import name,
   `refs/gwz/local-imports/<transfer-id>`, and verified that what arrived is
   what was captured.
3. It handed that import ref to the ordinary merge engine as the source.

Everything after step 3 is the ordinary coordinated merge described on the
[`gwz merge`](commands/merge.md) page: the same record, the same conflict
handling, `gwz merge --status`, `gwz merge --continue`, `gwz merge --abort`
and `--abort --preserve` all behave exactly as they do for a merge started
from a Git ref. A conflict looks like any other conflict, with the import
ref named as the source. This recorded member-only example shows the recovery
fields; a default merge also lists the root participant:

```text
action: merge
status: Conflicted
state: awaiting-resolution
merge: merge_op_82653_1788669594115_0001 (open)
record: v1 (open)
participants: total 1; conflicted 1
recovery commands:
  inspect:  gwz merge --status
  continue: gwz merge --continue
  abort:    gwz merge --abort
  preserve: gwz merge --abort --preserve
participants:
  services/api (mem_api)  conflicted
    source: refs/gwz/local-imports/xfer_cce736a9da9d291d6326068a2d6f2e9a @ f3a2cf19321369bbeb8eb6103a2ba734671e4ed7
    recorded: branch main; before 01c9fb66703f5bdf80fc673f2ca2efab6bb71e63; result -
    live: commit 01c9fb66703f5bdf80fc673f2ca2efab6bb71e63
    recovery: continue blocked; abort eligible
    conflicts: main.rs
```

Resolve, `gwz add` the files, and `gwz merge --continue`; or `gwz merge
--abort`. If a merge will not close, the
[Merge Recovery Runbook](MergeRecovery.md) applies unchanged.

Three things to know about the selector:

- **With a ref, the ref is resolved inside the lane.** `gwz merge --remote A
  topic` merges `A`'s `topic` branch, resolved independently in every paired
  member. Without a ref, the source is `A`'s HEAD commit, again per member.
- **Root and members participate by default.** `gwz merge --remote A` carries
  the lane's root commits as well as its member commits. Explicit selection
  stays partial; to merge members only, use:

  ```sh
  gwz --target @all --no-target @root merge --remote A
  ```

  This is an ordinary merge of the root repository, so it can conflict in
  `gwz.conf/` like any other file and is resolved the same way.
- **On `merge`, the name is family-only.** A name that is not a ready family
  member is refused and never falls back to a Git remote, so
  `gwz merge --remote origin` is a family miss rather than a fetch:

  ```text
  gwz: UnknownLocal: local family merge: no ready family member is named `origin`; `merge --remote` resolves family names only and never falls back to a Git remote
  ```

  The mirror image also holds: a bare `gwz merge A` resolves the Git
  revision `A` in each receiving repository and never means the lane.

### Integrating several lanes

When several agents work in local clones, use the original workspace as the
single integration point. Each agent commits its complete lane first. From the
root workspace, merge lanes one at a time, and close each coordinated merge
before starting the next one:

```sh
cd ~/work/demo
gwz --target @all merge --remote A
gwz --target @all merge --remote B
gwz --target @all merge --remote C
gwz merge --status       # use --continue or --abort if a merge is open
gwz push                 # publish the integrated root once
```

`merge --remote A` brings work **into the workspace where it runs**. Running
`gwz merge --remote root` inside lane `A` therefore updates `A` from root; it
does not integrate `A` into root. Use that direction only after root has
integrated the lanes, when a lane needs to catch up. `gwz push` publishes the
configured Git remotes; it does not transfer work between local-family lanes.

### 4. Retire

`gwz local dispose <name>` deletes the lane's directory — and refuses unless
the lane's history is verifiably preserved in another surviving family
member. After a completed default merge, ordinary disposal needs no separate
root merge. If you explicitly merged members only, the lane's root history may
still be unpreserved, producing a refusal like this:

```sh
gwz local dispose A
```

```text
gwz: UnwaivedHazard: local dispose `A` at /Users/you/work/demo-A: unwaived hazard(s): `@root` <unpreserved-history>: 2 protected root(s) of @root are preserved whole in no surviving family repository: Head 21bc247792bb20cfefe7ca7001ae82841b06edc9, Ref { name: "refs/heads/main" } 21bc247792bb20cfefe7ca7001ae82841b06edc9; name each accepted loss with --force <hazard,...> to delete, or --keep to detach and retain every file; nothing was removed; effects: []
```

The commit it names is the lane's root commit from step 2. Merge the root as
well, and the same command deletes:

```sh
gwz --target @root merge --remote A
gwz local dispose A
```

```text
status: Ok
deleted local clone `A`: /Users/you/work/demo-A removed, its row removed
```

There are three exits from a lane, and only one of them destroys anything:

- **Ordinary `dispose`** deletes the directory and the index row, after the
  checks pass. It is refused while any hazard is present: `open-merge` (an
  open coordinated merge or an unfinished native Git operation), `dirty`
  (staged, unstaged, untracked or ignored work, and native stash entries),
  or `unpreserved-history` (any protected root — a branch, HEAD, a tag, a
  reflog entry, a stash — that no single surviving family repository
  preserves whole).
- **`dispose --keep`** removes only the pointer and the index row.
  `--keep` deletes nothing on disk: the tree, its open merge, its stashes and
  its history stay where they are, and the directory remains usable as an
  ordinary GWZ workspace that simply belongs to no family.

  ```sh
  gwz local dispose S --keep
  ```

  ```text
  status: Ok
  detached local clone `S`: row removed, its pointer and marker removed; every file at /Users/you/work/demo-S is retained
  ```

- **`dispose --force <hazard,...>`** deletes past the hazards you name, and
  only those. It is an operator's loss waiver, not crash recovery: you are
  stating that the named work or history may be lost. Each hazard the check
  reports must be named; an unnamed one still refuses.

  ```sh
  gwz local dispose D
  ```

  ```text
  gwz: UnwaivedHazard: local dispose `D` at /Users/you/work/demo-D: unwaived hazard(s): `@root` <dirty>: unstaged change, text content (gwz.conf/markers/conf-integrity.yml); `mem_api` <dirty>: staged change, text content (main.rs), untracked file, text content (scratch.txt); name each accepted loss with --force <hazard,...> to delete, or --keep to detach and retain every file; nothing was removed; effects: []
  ```

  ```sh
  gwz local dispose D --force dirty
  ```

  ```text
  status: Ok
  deleted local clone `D`: /Users/you/work/demo-D removed, its row removed; forced past: dirty
  ```

  A bare `--force` is refused (`--force requires one or more hazard names,
  comma-separated (open-merge, dirty, unpreserved-history)`), and so is
  `--keep` together with `--force`.

**Evidence the check cannot read is never waivable.** If a lane holds
something the verifier cannot interpret, ordinary deletion is refused as
unknown evidence, and no `--force` name changes that. The case you will
actually meet is a GWZ stash record — `gwz stash push` in the lane — which
this build deliberately does not decode. The refusal says so and names both
ways out (the message repeats the finding per repository; the repetition is
elided here):

```text
gwz: UnknownEvidence: local dispose `S` at /Users/you/work/demo-S: unknown evidence: UnsupportedEvidence: gwz stash record: 1 gwz stash record(s) under /Users/you/work/demo-S/.gwz/stash/bundles: this build does not decode gwz stash coordination records (design §5.1 needs their surviving copies and referenced objects verified), so the lane cannot be deleted while any exists; pop or drop the gwz stash in the lane first, or `gwz local dispose S --keep` detaches the lane and retains every file, […]; no force name waives unknown evidence: make it interpretable, or --keep to detach and retain every file; nothing was removed; effects: []
```

Pop or drop the stash in the lane (`gwz stash pop`, `gwz stash drop
<stash-id>`) and dispose again, or `--keep` it.

Two more refusals you will meet: the root is never disposed —

```text
gwz: InvalidRequest: root is never disposed; `gwz local disband` retires the family
```

— and neither is the lane you are standing in (`… contains the working
directory`). Run `dispose` from another member.

**Retiring the family** is `gwz local disband`. It removes every pointer and
the index, and nothing else:

```sh
gwz local disband
```

```text
status: Ok
disbanded local family fam_3fa95fa799ee9ec5b04999adba13e651: 1 pointer(s) and 1 marker(s) removed, 0 member(s) held none, index removed; every tree is retained
```

Afterwards every directory is an ordinary, unrelated GWZ workspace. Family
names stop resolving: `gwz merge --remote A` becomes a family miss, and
`--remote A` on `pull` or `push` falls through to ordinary Git remote
resolution (`MissingRemote` unless a Git remote of that name exists). Disband
can be repeated after an error, and it never deletes a directory.

## The Deletion Position, In Plain Terms

- **`dispose` refuses unless the lane's history is verifiably preserved**
  in another surviving family member: every protected root of every
  repository in the lane's tree — its root repository and every member —
  must be reachable, whole, from a surviving family repository's own refs.
- **A clean working tree is not preservation proof.** Clean means nothing is
  uncommitted; it says nothing about whether the commits exist anywhere else.
  A lane with a unique commit and a spotless tree still refuses.
- **A push is not proof either.** History that exists only on a network
  remote is not certified: fetch it into a surviving family member first, or
  keep the lane.
- **`--keep` is the non-destructive exit.** `--keep` deletes nothing; it
  forgets the lane, and the directory stays intact and usable.
- **`--force <hazard,...>` is a loss waiver, not crash recovery.** It names
  the losses you accept — `open-merge`, `dirty`, `unpreserved-history` — and
  deletes an intact, ready lane past exactly those. An incomplete or
  interrupted lane is not force-deletable: `--keep` it and clean up by hand.
- **Unknown evidence is never waivable.** What the verifier cannot read, it
  will not let you delete; make it interpretable or `--keep`.
- **Nothing is kept for you.** GWZ makes no archive of a lane it deletes and
  runs no background retention. What ordinary `dispose` removes is gone,
  which is why it is only allowed when a copy provably survives.

## Import References Are Retained

Every family merge leaves one ref per receiving repository:

```sh
git -C services/api for-each-ref refs/gwz/local-imports/
```

```text
01c9fb66703f5bdf80fc673f2ca2efab6bb71e63 commit	refs/gwz/local-imports/xfer_4c05d0f1a5f847276e882e020aec9959
395c9b6f88fdca9159ab58071c1c309ed761acfb commit	refs/gwz/local-imports/xfer_c6de6dd3b1e248f0ecfaca9eb8f3dc42
f3a2cf19321369bbeb8eb6103a2ba734671e4ed7 commit	refs/gwz/local-imports/xfer_cce736a9da9d291d6326068a2d6f2e9a
```

They are ordinary Git refs, and they are retained deliberately — after a
completed merge, after an abort, after the lane is disposed or the family
disbanded. A retained import keeps the imported objects alive through
ordinary Git garbage collection, so an open merge still has its source after
the lane that supplied it is gone, and a survivor holding an import counts as
preservation when another lane asks to be deleted. The cost is disk: the
objects stay for as long as the ref does.

This build has no command that prunes them. If you remove one by hand, do it
only for a merge that is closed, and never for a ref an open merge names as
its source.

## Not Yet

Each of the following is accepted by the parser and refused by the engine,
before anything is written. The messages are what the current build prints.

- `gwz local clone <name> [dest] --clean` — a frozen checkout without
  worktree dirt:

  ```text
  gwz: UnsupportedOperation: local clone (clean mode) is not supported by this gwz-core build
  ```

- `gwz local clone <name> [dest] --bare` — a share point of bare
  repositories:

  ```text
  gwz: UnsupportedOperation: local clone (bare mode) is not supported by this gwz-core build
  ```

- `-b <branch>` is accepted only with `--clean` or `--bare`, so it is refused
  with them or without them
  (`-b <branch> is accepted only with --clean or --bare`).
- `gwz local clone <name> [dest] --from <name|path>` — copying a member
  other than the one you are standing in:

  ```text
  gwz: UnsupportedOperation: local create from an explicit copy source (--from <name|path>) is not supported by this gwz-core build
  ```

- **Family pull and push are not served by this build.** `gwz pull --head
  --remote A` and `gwz push --remote A` look the family name up and then
  fall through to Git remote resolution, so a family name that is not also a
  Git remote answers:

  ```text
  gwz: MissingRemote: missing remote 'A'
  ```

  Integrate from the receiving side with `gwz merge --remote <name>` instead.
- `--dry-run` is refused for every local family verb
  (`local family operations with dry_run is not supported by this gwz-core
  build`).
- Disposal cannot verify a GWZ stash record, so a lane holding one refuses
  ordinary deletion as unknown evidence (above).

In summary: `--clean`, `--bare` and `--from` are refused by this build, and
so are family pull and push. This page will change when that does; it does
not say when.

## Machine Output

`--json` and `--jsonl` carry the family listing under `local_family_members`
with the family root's path beside it under `local_family_root_path`, and
render every refusal above as a structured error with its typed code
(`unknown_local`, `unwaived_hazard`, `unknown_evidence`, and so on). The
shapes are on the [command page](commands/local.md#machine-output).

## About This Page

This page covers the family model, the four-step lifecycle, the deletion
rules and their reasons, the retained import refs, and what the current
build refuses. It deliberately omits the on-disk format of the family index
and pointer beyond their names, the internal steps of a create and a
disposal, the `--clean`, `--bare` and `--from` modes beyond the fact that
they refuse, and any schedule for them. Every command shown was run against
the build this page ships with; the sample paths and ids are illustrative.

### Filesystem capabilities and recovery

Windows recovery admission checks open-by-file-ID capability, a successful
nonzero 128-bit identity query and a local volume GUID, along with required
case-mode and handle probes. Filesystem names are diagnostic labels. An
unavailable name does not disable otherwise supported recovery. Missing
capabilities warn and allow ordinary `--no-ff` merges without crash recovery;
`--filesystem-strict` refuses. CLI and Python use the same core decision.
Successful block cloning proves a separate copy capability, not recovery
support.
