# Claude Code

Status: draft written from the accepted plan before its probes have run.
Paragraphs marked **Unmeasured** describe intended behaviour that no probe has
yet confirmed; the plan's steps S1.3, S2.1, S2.2 and S3.2 replace them with
observed behaviour.

## What it does

Claude Code makes an isolated working copy when a session starts with
`--worktree`, when a subagent asks for worktree isolation, or for a background
session. By default that copy is a `git worktree` of the repository it was
started in. In a GWZ workspace the members are separate repositories, ignored
by the root, so a worktree of the root has no members: nothing builds and the
session cannot do its work.

The two hooks change what Claude Code makes. `WorktreeCreate` runs
`gwz hook claude-code worktree-create`, which gives the session a **lane**, a
local clone of the whole workspace (`gwz local clone`), registered in the
workspace's local clone family. `WorktreeRemove` runs
`gwz hook claude-code worktree-remove`, which disposes the lane through
`gwz local dispose`, with GWZ's own refusals intact. Outside a GWZ workspace
the same hooks make and remove the plain worktree Claude Code would have made,
so they can live in user-level settings without changing other projects.

## Prerequisites

- **gwz 1.0.14 or later**, the release that carries the `gwz hook` family.
  One consequence is workspace-wide: once any 1.0.14 gwz has written a
  workspace's local-family index, that index is format 2, so every gwz used on
  that workspace from then on must also be 1.0.14 or later — an older one
  refuses the whole index and says so.
- `gwz` on the `PATH` the Claude Code desktop app sees. On macOS the app reads
  `PATH` from the shell profile and nothing else; no environment variable of
  ours reaches a hook, which is why every knob is an option on the handler.
- POSIX hosts. Windows runs the same command through `powershell.exe` and is
  verified later (the plan's Phase 5); until then the Windows fallback reports
  no block sharing and the guard is conservative.
- Nothing else: no scripts to copy, no `jq`, no Python.

## Setting it up

Print the settings block, then write it where you want it:

```sh
gwz hook claude-code setup --project
```

```sh
gwz hook claude-code setup --project --local --write
```

```sh
gwz hook claude-code setup --user --write
```

`--project` writes `<root>/.claude/settings.json`, `--project --local` writes
`settings.local.json` (not committed), `--user` writes
`~/.claude/settings.json`. The writer parses first, writes a temporary file
beside the target, fsyncs, re-parses and renames, and changes no byte outside
the block, and `--remove` takes it back out again; see
[`gwz hook`](commands/hook.md).

**Which placement.** One is the recommendation. A gwz-dev clone whose machine
also carries the block in user settings is fine: identical handler text runs
once. Two *differing* handlers (one pinned with `--command`, or carrying
options) both run for the same name and session; creation stays harmless
because the second reuses the first's lane and prints the same path, but a
handler that refuses for a reason of its own (a pinned binary without the
`hook` family, say) makes Claude abort the creation the other completed. That
lane is then an orphan: `gwz local list` is its inventory and
[retirement](#retiring-lanes) its remedy. `setup` warns, naming both files,
when it finds a differing handler.

After writing the block, install or refresh the agent skill too: copy
`skills/gwz/SKILL.md` to `~/.claude/skills/gwz/`.

**Migrating a machine that pinned `--command`** before the project block was
committed: remove or align the user-level block first, then pull.

Inside a GWZ workspace, `setup` computes the handlers' timeouts and
`--wait-secs` from the same cost estimate the create hook uses: the wait is
the compiled-in 300 s, which the estimated copy time raises but never lowers,
and the timeouts are one wait plus one estimated copy plus 60 s for creation
and one wait plus 60 s for removal. Outside a workspace — in a plain
repository, where the block still serves the fallback — no estimate is run and
the compiled-in defaults stand: a 300 s wait and 600 s timeouts.

## The lane lifecycle, from Claude's side

1. **Creation.** The hook resolves the workspace from the directory the session
   started in, exactly as `--root` resolution does. A workspace gets a lane at
   `../<root-dirname>-<name>`; when the session started inside a member, the
   printed path is that member's directory inside the lane, so the session lands
   where it started. The lane is registered with the session's id as its owner.
2. **Work.** The session edits, builds and tests inside the lane. Commit in the
   lane with `gwz add` and `gwz commit` from the lane root. The lane is on
   whatever branches the workspace was on: the hook creates no `worktree-<name>`
   branches, so Claude Code's branch-based views (the desktop's diff and PR
   flows) do not describe a lane session. **Unmeasured:** what those views show
   for a lane session (plan S2.1).
3. **Integration is yours.** The hooks never merge. From the main workspace:

   ```sh
   gwz --target @all merge --remote <name>
   ```

4. **Removal.** When the session exits with removal, the hook runs
   `gwz local dispose <name>` from the family root, never `--keep` and never
   `--force`. A refusal keeps the lane and the session. See
   [retiring lanes](#retiring-lanes) for what happens next.

**A lane is a snapshot of a live source.** The hook runs while the parent
session is alive and possibly building. Git object stores are verified by gwz
at creation; unsaved edits and build outputs in flight are copied best effort,
and a session in the lane may need to rebuild. The hook warns on stderr when a
`cargo` process holds the source's target directory open. **Unmeasured:**
whether a lane created during a build has a usable target directory (S1.3).

**Isolation.** Claude Code blocks edits into the main checkout from inside a
hook-created directory, and blocks shell commands it cannot verify keep git
inside the worktree. **Unmeasured:** how that check treats `gwz`, which runs
git internally, when invoked inside a lane (S1.3), and what Claude's
`git worktree lock` does to a lane (S2.2).

**Subagents.** Do not use subagent worktree isolation in a GWZ workspace until
an integrated lane disposes in one command: each subagent would leave a lane
behind that only the retirement procedure removes. The hook cannot enforce
this (nothing in its input identifies a subagent), so it is a rule for agent
briefs and the skill. **Unmeasured:** whether the hook input ever identifies a
subagent (S2.3).

## Reuse and the guards

An existing lane of the same name is reused only when the family holds a
`ready` row at that path, the row's owner is this session, and the lane passes
the completeness check — gwz's own observation of the lane, plus one existence
test per repository (the directory, and its `.git`), which is cheap enough
that a reuse still answers in under a millisecond. Every other state refuses with one line naming the
state and its remedy: an unfinished create, a lane at another path, a lane with
no owner (made by hand or before the workspace's first owned lane), a lane
owned by another session, an incomplete lane. A `creating` row of this session
means another handler is mid-copy; the hook waits for it.

Two guards protect the copy, never a reuse:

- **Free space** on the filesystem holding the destination's parent, against a
  run-time estimate of the copy's cost: a walk of the source for apparent size
  and file count, a block-sharing probe at the destination's parent, and a
  pessimistic share by filesystem (APFS, XFS and btrfs 94%; ReFS 70%; ext4 and
  NTFS 0%). `--min-free-gb` is a floor on top of it. The share figures and the
  per-file time cost are conservative estimates, not measurements; no probe
  has measured them and none is scheduled.
- **A ceiling of ready lanes**, `--max-lanes`, default 8. Until the lane
  clean-up lands (below), every lane counts against it until you retire it by
  hand, so a busy workspace will meet this ceiling; the refusal names the
  retirement procedure.

What a session builds afterwards is not guarded, and is out of scope: a build
that fills the disk is the same failure Claude Code's own worktrees have.

## Retiring lanes

`gwz local list` is the inventory. Its owner column is the Claude session id.
The order is:

```sh
gwz --target @all merge --remote <name>
```

```sh
gwz local dispose <name>
```

Add `--wait <secs>` to the dispose: with any other family command in flight
the bare form fails in milliseconds on the family lock.

With gwz 1.0.14, `dispose` still refuses every verbatim lane of a large
workspace even after its merge, because the copy inherited build caches,
stashes and ignored user data that disposal counts as `dirty` hazards — and
under local adoption `.claude/settings.local.json` and `.claude/.cc-writes/`
are among them, carried into the lane by the copy before any session runs.
What 1.0.14 no longer raises for a *merged* lane is `unpreserved-history`:
the identical-copy witness does its job, so the waiver for one is `dirty`
alone. Keep `dirty,unpreserved-history` for an unmerged lane. Retiring such a
lane takes your own comparison against the family, then
`gwz local dispose <name> --force dirty --wait 600`. `--force` and
`--keep` are always your choice, never the hook's. The clean-up requirements
(`GwzLaneCleanFixes`, R0 to R19) remove those false hazards; their first phase
is merged and unreleased. **Unmeasured:** the release in which an integrated
lane disposes in one command (S3.5).

Lanes the hook cannot retire: chips and background sessions whose removal was
refused, headless `-p` runs (which never call the remove hook), crashed
sessions, and `creating/incomplete` rows from a hook killed at its timeout.
Look at `gwz local list` weekly until a Claude-side listing exists.

## What the fallback drops

In a non-GWZ project the hook reproduces Claude Code's default as far as a hook
can: a `git worktree` under `.claude/worktrees/<name>` on branch
`worktree-<name>`, from `--base-ref` when the handler carries it, else
`origin/<default-branch>`, else `HEAD`; the `.worktreeinclude` copy is
performed for a worktree the hook created, never on reuse, and matches the
patterns against the project's own untracked files only — never against what
is inside another worktree. It cannot
reproduce two things:

- **No automatic sweep.** Hook-created worktrees accumulate until removed by
  hand: `git worktree list`, then `git worktree remove`. Worktrees *and
  branches* — `git worktree remove` leaves the `worktree-<name>` branch
  behind, so `git branch -d worktree-<name>` is the second half of the
  clean-up (observed).
- **No session ownership.** Two sessions naming the same slug share one
  worktree. `git worktree remove` without `--force` refuses a dirty or locked
  worktree, which is the protection. **Unmeasured:** that Claude's lock is
  present on a fallback worktree (S1.3).

If `worktree.baseRef` is set to `"head"` in Claude's settings, the fallback
differs from Claude's own behaviour unless the handler carries `--base-ref`.

## Caveats

- Launch or resume sessions from the main checkout, never from inside a lane.
- Headless `-p` runs never call the remove hook.
- Hooks never merge.
- Keep `<root>/.claude/worktrees/` empty in a workspace that uses lanes: a
  verbatim copy carries those worktrees' `.git` pointers into every lane.
  **Unmeasured:** what a lane makes of them (S1.3).
- A hook killed at its timeout mid-copy leaves a `creating` row and its files
  and writes no log line; `gwz local list` shows it.
- **Unmeasured:** whether the desktop app's task chips ("fix in worktree") go
  through the hook at all (S2.1); if not, start the spun-off work with
  `claude --worktree <name>` from the root instead.

## Switching it off

Take the block back out with the same placement flags:

```sh
gwz hook claude-code setup --project --local --remove
```

`--remove` changes no byte outside the two entries it takes out, drops an array
or a `hooks` object left empty behind them, and never deletes the settings file
itself; a file that does not carry the block is left alone and the command says
so. Deleting the block by hand does the same job. For background sessions
alone, `worktree.bgIsolation: "none"` in Claude's settings lets them edit the
main checkout instead.

## Refusals

Every non-zero exit prints one line, `gwz: <cause>; <remedy>`, and logs the
same line. **Observed** (2026-09-18, gwz 1.0.14, Claude Code 2.1.247): the
hook's line reaches the user intact, wrapped by Claude as

```text
Error creating worktree: WorktreeCreate hook failed: <handler>: <our line>
```

and a non-zero create hook aborts the session before it starts. Two further
orderings were observed at the same time: `--worktree` combines with `-p`,
and the create hook runs *before* authentication, so a lane is made even for
a session that then fails to authenticate. A dispose hazard refusal is
summarised — `gwz: <n> hazards across <m> repositories (<class>); <remedy>` —
with the full report left to `gwz local dispose`.

| Cause | Remedy |
| --- | --- |
| name refused (reserved, or characters outside `[A-Za-z0-9._-]`) | choose another name |
| session id outside the owner-token grammar | report it; the hook refuses rather than guess |
| free space below the estimate or the floor | retire lanes (`gwz local list`), or free space |
| `--max-lanes` ready rows already | retire lanes |
| unfinished create, `disposing` row, lane at another path | retire it through the procedure above |
| lane with no owner, or owned by another session | retire it, or choose another name |
| lane of this session incomplete | retire it |
| family lock still busy at the deadline (create or remove) | retry; another gwz family command is running |
| workspace cannot be used (open coordinated merge, unreadable lock or marker, manifest newer than the binary) | gwz's own message says what to do |
| dispose hazard refusal (dirt, unpreserved history) | merge, compare, then `--force` by hand |
| fallback worktree dirty or locked | finish or clean the other session's work |

## The log

`<root>/.gwz/claude-hooks.log` in a workspace, `~/.claude/gwz-lane-hooks.log`
otherwise, one line per decision: timestamp, event, name, session id,
classification (`lane`, `member-in-lane`, `fallback-worktree`,
`already-absent`, `refused-by-hook`, `refused-by-hazard`, `family-busy`),
path, outcome, exit code. Never the transcript path or the working directory. Bounded at 1 MB. The
session id is the lane's owner token: redact it from anything you publish.

Reference: [`gwz hook`](commands/hook.md), [Local Clones](LocalClones.md).
