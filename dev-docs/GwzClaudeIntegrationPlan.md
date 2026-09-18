# GWZ and Claude Code Integration Plan

Status: **ADOPTED 2026-09-17; S0.1 done; accepted at sha256 `c0bba7fa…`
(with gwz-core `GwzLaneCleanFixes.md` at `c5f06e41…`) after
`GwzClaudeIntegrationPlan-ReviewConsistency-6.md` and
`-ReviewSafety-6.md` both reported GO; this accepts the plan text and the
R20 to R22 requirements only, no code.** Released: S1.1 and S1.2 (the hook subcommands and `setup`, in the `gwz hook claude-code` shape of amendment A2) and R20 to R22 shipped in **gwz 1.0.14** on 2026-09-18, installed the same day; S1.3 onward can run on the installed gwz. Drafted 2026-09-12 by Fable at
the operator's request after the 1.0.11 release; reviewed the same day
(`GwzClaudeIntegration-S0.1-Review.md`: GO-WITH-CONDITIONS, 4 P1, 8 P2,
6 P3) and every finding folded (trail in section 7); updated 2026-09-17 for
the lane disposal clean-up requirements. The operator confirmed D2 to D7
and D9 to D11 as written on 2026-09-17, replaced D1 (gwz itself is the hook,
and a gwz command writes the settings) and simplified D8 (one test: is the
project a GWZ workspace). No code or settings have changed yet.
Implementation is chartered for non-Fable agents (Opus builders) under the
usual review loop: S1.1 onward once GwzLaneCleanFixes R20 and R21 are in
an installed gwz (the redesign route chosen on 2026-09-17 after the
round-3 review; section 7). Amended A1 on 2026-09-17 by the operator's
decision, without re-review: the copy-cost guard estimates at run time by
filesystem and S1.0 is withdrawn (D6, S1.2, S3.2, section 7).

## Goal

When Claude Code needs an isolated working copy of a GWZ workspace, it should
get a GWZ lane (`gwz local clone NAME`) rather than a git worktree of the root
alone. Today the Claude desktop app offers "fix in worktree" task chips, and
the CLI offers `--worktree`, subagents with `isolation: "worktree"`, and
background sessions. All of them create a git worktree of the root repository.
In a GWZ workspace the members are separate repositories, git-ignored by the
root, so such a worktree has no members: nothing builds, nothing tests, and
the session cannot do the work it was spawned for. A lane is a whole
workspace, and its lifecycle (integrate, dispose) stays under GWZ's
protections. The hook is `gwz` itself (D1). Where the project is not a GWZ
workspace, the same hook falls back to a plain git worktree laid out the way
Claude Code lays it out by default, as far as a hook can (D8 names what it
cannot), so it can live in user-level settings without breaking other
projects.

## 1. Facts this plan rests on

Claude Code 2.1.247, from the official documentation
(<https://code.claude.com/docs/en/worktrees> and
<https://code.claude.com/docs/en/hooks#worktreecreate>, read 2026-09-12):

- `WorktreeCreate` fires "when a worktree is being created via `--worktree`,
  `isolation: "worktree"`, or for a background session", and "configuring a
  WorktreeCreate hook replaces that default git behavior".
- The hook reads JSON on stdin with the common fields (`session_id`, `cwd`,
  `transcript_path`, `hook_event_name`; `session_id` is a UUID in every
  observed hook payload, to be confirmed in S1.3, and D6 validates it
  before use) plus `name`, "a slug identifier for
  the new worktree, either specified by the user or auto-generated". There is
  no branch or path field, and no field says whether the caller is a session,
  a background session or a subagent.
- A command hook "must return the path to the created worktree directory" as
  "the last non-empty line of stdout"; everything else belongs on stderr. Any
  non-zero exit aborts creation. Relative paths resolve against the hook's
  directory; absolute paths with `.` or `..` segments, or passing through a
  symlink below the repository root, are refused.
- The directory must sit outside any repository: "if the hook creates the
  directory inside a repository, git resolves it to that repository's checkout
  and Claude Code refuses it". A directory that is its own repository is
  accepted.
- `WorktreeRemove` receives the created path as `worktree_path`. It fires when
  a `--worktree` session exits and the user chooses removal, when a subagent
  with worktree isolation finishes, and when a background session whose
  worktree the hook created is deleted. "When a hook exits non-zero and the
  directory at `worktree_path` still exists afterward, the removal fails" and
  the session is kept. What happens when the hook exits zero and the
  directory remains is not documented (U8). Headless `-p` runs never call it.
- The hook inherits the parent environment (so `gwz` on `PATH` works) and
  receives `CLAUDE_PROJECT_DIR`, the project root the session started from.
  On macOS the desktop app reads `PATH` from the shell profile but not other
  exported variables, so a hook launched by the desktop app cannot rely on
  any environment variable of ours. The default command timeout is 600
  seconds.
- Hooks fire "wherever it runs: sessions in the terminal, IDE extensions, the
  Desktop app, and Claude Code on the web". On Windows, hooks run through
  `powershell.exe`. `.worktreeinclude` is not processed when a hook replaces
  creation. A hook-created directory is never swept automatically, and its
  transcript stays at the launch directory. A subagent's worktree "with
  changes stays on disk", and a background session whose removal fails stays
  listed.
- "All matching hooks run in parallel. If you define the same handler in more
  than one settings file, it runs once." The dedupe is by identical handler;
  two different commands that both print a path both run.
- Isolation is still enforced inside a hook-created directory: edits into the
  main checkout are blocked, and "Claude Code blocks a Bash or Monitor command
  when it can't verify from the command text that any git the command runs
  stays inside the worktree". Sessions must be launched or resumed from the
  main checkout, not from inside the hook-created directory. While an agent
  runs, Claude Code "holds a `git worktree lock` on its worktree"; a lane is
  not a git worktree of the main repository, so what that lock does there is
  unknown (U10).
- `worktree.bgIsolation: "none"` in settings lets background sessions edit
  the main checkout instead; it is the documented escape hatch when
  worktrees are impractical, and the quickest way to switch background
  isolation off if the hooks misbehave.
- Claude Code's own default, which the fallback reproduces: "the worktree is
  created under `.claude/worktrees/<name>/` at your repository root, on a new
  branch named `worktree-<name>`", branched from `origin/<default-branch>`
  when `worktree.baseRef` is `"fresh"` (the default) or from `HEAD` when it is
  `"head"`. A directory that carries git metadata of its own, as
  `git worktree add` produces, passes the identity check; only a bare
  directory inside a repository is refused. The desktop app's chip "starts
  that work in a new session with its own worktree; Claude continues your
  current session uninterrupted".

GWZ, from `gwz local --help`, `gwz local clone --help` and
`docs/LocalClones.md` at 1.0.11:

- `gwz local clone NAME [DEST]` copies the workspace verbatim, "dirt and build
  directories included", to `../<root-dirname>-NAME` by default, and registers
  the lane in the local clone family. `--clean`, `--bare`, `-b` and `--from`
  are parsed but refused. Creation refuses during an open coordinated merge,
  refuses a nonempty destination, and refuses a name the family already
  holds. An interrupted create leaves its files and a `creating` row that
  `gwz local list` shows as `creating/incomplete`.
- "Keep the source quiet for the whole copy. GWZ takes its family lock, which
  serializes family commands, but nothing stops an editor, a build, or a raw
  `git` command from writing into the source while it is being copied."
  The lock is advisory, on `<root>/.gwz/local-family.lock`, taken with
  `try_lock` (`flock(LOCK_EX | LOCK_NB)`), and refused as `Busy` at once
  when held; nothing in gwz 1.0.13 waits or retries, and `local list`
  reads without it. A clone holds it for the whole copy, so a second
  family command during a copy fails rather than queues. R21 (below) adds
  an opt-in wait.
- Integration is `gwz merge --remote NAME` run from the receiving workspace.
  `gwz local dispose NAME` "refuses unless the lane's history is verifiably
  preserved"; a dirty lane is a hazard in its own right; `--force <hazard,...>`
  waives named hazards; `--keep` "removes only the pointer and the index row"
  and retains every file. The root is never disposed.
- Every verbatim lane of gwz-dev has refused disposal so far: 28 lanes in
  L1's register, three on 2026-09-10 and 25 merged on 2026-09-15 and
  2026-09-16, each after its merge, with gwz 1.0.12 and then 1.0.13 (gwz-dev
  `dev-docs/GwzLaneIssues.md`, L1; the register records that 1.0.13,
  installed 2026-09-16, changes nothing for L1). Each
  reported `dirty` and `unpreserved-history`; from the second round on, the
  same 112 entries every time, all inherited by the copy: build and cache
  directories, `__pycache__`, stash entries and commits only a reflog still
  reaches. Retiring such a lane takes the operator's own comparison against
  the family, then `gwz local dispose NAME --force dirty,unpreserved-history`.
  Dispose has no check-only mode.
- The gwz-core requirements that would let an integrated lane dispose in one
  command, and make `--clean` work, are drafted in gwz-core
  `dev-docs/GwzLaneCleanFixes.md` (R0 to R19). None is implemented; this plan
  cites them by number. On 2026-09-17 this plan added three of its own to
  that document: R20, an owner token recorded on the family row in the
  same write that reserves it and shown by `local list`; R21, an opt-in
  `--wait <secs>` on every family command that retries a `Busy` lock until
  a deadline; R22, the concurrent-create and older-reader tests. R20 bumps
  the index schema to `gwz.local-family/v2`; an older gwz refuses a v2
  index as a whole, naming the minimum version, so every gwz used on a
  workspace must be at or above the R20 release once any such gwz has
  written the family index (a dispose, a `--keep` or a family merge is a
  write too, not only a create), and the acceptance runbook's pin moves
  with it. The two carry
  gwz-cli work as well as gwz-core work (the clap arguments, the generated
  `docs/CLI.md`, the long help, the `local list` sample and the
  machine-output contract; §3.6's scope note). S1.1 onward needs R20 and
  R21 in the installed gwz.
- The gwz-dev workspace is 65 GB on disk, most of it cargo target trees. On
  APFS the verbatim copy is a copy-on-write clone, cheap until a lane builds;
  a full build in a lane then writes its own tens of gigabytes (the 2026-09-11
  disk-full incident came from exactly this). Free space at the time of
  writing: 31 GB.
- Block sharing on copy is a filesystem property and never crosses a
  filesystem boundary: APFS (`clonefile`), XFS and btrfs (`FICLONE`),
  OpenZFS 2.2 and bcachefs, and ReFS (block cloning) share; ext4 and NTFS
  copy every byte. gwz's copy backend already takes the clone path where
  the platform offers it (LocalClones.md: "natively" versus "ordinarily").
  A clone's wall time is bound by file count, not bytes, on every sharing
  filesystem.
- `gwz` refuses `root`, `origin` and git's reserved ref names as lane names.
- Creation also refuses a destination that is already a workspace or lies
  inside a family member (`PathCollision`). Dispose refuses `open-merge`,
  `dirty` and `unpreserved-history`, each waived only by name, never
  disposes the root, and refuses the member the caller is standing in
  ("contains the working directory"). A lane does not carry `.gwz/`; it
  holds `.gwz/family-root` naming the family and the root's path, and the
  family index lives on the root. In gwz-dev `/.gwz/` is ignored only through
  the GWZ-managed block in `.git/info/exclude`, not a tracked `.gitignore`
  rule.

## 2. What is not confirmed

These are the questions the probes in Phases 1 and 2 must answer before the
integration is documented for other users.

- U1. Answered 2026-09-18: yes. A "fix in worktree" chip started from a
  gwz-dev session ran `gwz hook claude-code worktree-create` (launched by the
  desktop app's helper), the family gained a `ready` row owned by the chip
  session's id, the lane held every member, and no `.claude/worktrees/`
  entry appeared. Without the hooks installed the same chip had made a
  member-less git worktree under `.claude/worktrees/` earlier that day.
  Claude shows nothing while the hook runs: the session looked stalled for
  the ~2 min the hook took (lane directory born 7 s after the click, tree
  copy 106 s, copy record and index write 10 s; no other lane or build was
  running), so the guide should say so. A second timing on the same workspace
  the same day: a `gwz local clone` run by hand, with one other lane present,
  took 146 s. These are two observations, not a measured range; the guide
  states them as observations. Originally: whether the chips create their
  session through `WorktreeCreate`; the docs never name the chips.
- U2. How the isolation check that inspects command text for git treats a
  wrapper such as `gwz`, which runs git internally, when invoked inside a
  lane.
- U3. Whether an interactive session's `EnterWorktree` fires the hook.
- U4. Whether Claude Code checks for an existing directory before calling the
  hook for a reused name. The second half of the original question (which
  path wins when two settings files both define a `WorktreeCreate` hook) was
  resolved 2026-09-17 by D5: identical handlers run once, and differing
  handlers print the same path for the same session, so no winner is
  needed. That holds because gwz records the owner in the same index write
  as the row (R20) and the second handler keeps polling while the first's
  `creating` row is its own session's (D6's attempt loop); the
  resolution is conditional on R20 being in the installed gwz.
- U5. What the desktop app's "Worktree location" and branch-prefix settings do
  once a hook owns creation.
- U6. **Answered 2026-09-18 by S1.3**, `GwzClaudeIntegration-Probe-20260918.md`
  §6: the refusal does name the remedy, and names it as this plan's
  two-command sequence. It was also 3,556 bytes on one line — the dispose
  report in full with the remedy stapled on — which was folded back into
  S1.1 as F3 and fixed: the hook now says how many hazards there are, across
  how many repositories, and of which class, and leaves the report to
  `gwz local dispose`. GwzLaneCleanFixes R9 and R10 still set what the report
  itself should say.
- U7. Whether a hook-created git worktree under `.claude/worktrees/<name>`
  behaves identically to one Claude Code creates itself: the docs say the
  automatic sweep skips hook-created worktrees, and the `.worktreeinclude`
  copy step is not run, so the fallback must do that copy itself if parity
  matters.
- U8. What Claude Code does when the remove hook exits zero but the directory
  at `worktree_path` still exists: forgets it (an orphaned tree) or treats
  the removal as failed.
- U9. Whether anything in the hook input or environment distinguishes a
  subagent's creation from a session's. The documented input carries only
  `name`; if nothing does, D3's subagent rule is documentary only.
- U10. What `git worktree lock`, which Claude Code takes on a running agent's
  worktree, does when the directory is a lane rather than a git worktree of
  the main repository.
- U11. Resolved 2026-09-17 by D8: the hook is gwz, so it finds the workspace
  the way `--root` resolution does, in process, and never interprets its own
  error codes. The question was only meaningful for an external script.

## 3. Scope decisions (confirmed by the operator on 2026-09-17; D1 and D8 changed at adoption)

- D1. **gwz is the hook, and gwz writes the settings.** Two subcommands in
  gwz-cli, `gwz hook claude-code worktree-create` and
  `gwz hook claude-code worktree-remove`, read Claude's hook JSON on stdin,
  do the work, log (D10), and print the path as the last stdout line. Their
  contract, stated once here and tested in S1.1: **the create hook prints
  only a path it created or verified in this invocation, and the remove hook
  deletes only the lane or worktree that `worktree_path` canonically names.**
  Nothing else is ever printed on stdout, and nothing else is ever deleted.
  Every knob the hooks have is a command-line option on the hook
  (`--min-free-gb`, `--max-lanes`, `--wait-secs`, `--base-ref`, `--log`),
  because the
  desktop app passes no environment variables of ours (section 1); the
  environment variables the tests use to force values are not user
  configuration. A third subcommand,
  `gwz hook claude-code setup --project | --user [--local] [--write]
  [--command PATH] [hook options]`, prints the hooks block and, with
  `--write`, merges it idempotently into `.claude/settings.json` (or
  `settings.local.json` with `--local`) of the workspace root, or
  `~/.claude/settings.json`. The default handler is the bare command
  `gwz hook ...` with no options, resolved through `PATH`, so a committed
  project block is machine-independent and identical everywhere, which is
  what Claude's same-handler dedupe keys on; `--command PATH` pins an
  absolute binary in a user's own settings if the desktop app's `PATH` lacks
  it, and hook options baked into the handler make it differ too (D5 says
  why that is harmless). The writer edits another program's configuration,
  so it parses the existing file first and refuses one that does not parse
  or is not a regular file; writes a temporary file beside the target,
  fsyncs, re-parses it, and renames over the original; and changes no byte
  outside the inserted block. Consequences: no shell scripts, no `jq` or
  `python3`, no copy step, and the same binary serves Windows, where Claude
  Code runs the command through `powershell.exe` (D11). Hook behaviour is
  versioned with the installed gwz, which is the right coupling: the lane
  semantics are gwz's. Original D1 (scripts in
  `gwz-cli/integrations/claude-code/`, copied by users) was replaced at
  adoption; the rejected alternative of baking hooks into the agent skill
  stands rejected, because hooks are settings, not skill content.
- D2. **The lane name is Claude's `name` slug**, validated by the hook
  against GWZ's refused names and characters outside `[A-Za-z0-9._-]`, and the
  destination is GWZ's default sibling `../<root-dirname>-NAME`. The hook
  checks that the parent of the workspace is not inside a repository, by
  walking up looking for `.git` rather than by asking git, because on the
  operator's Mac an empty `~/.git` file makes `git rev-parse` fail with
  "invalid gitfile format" for every directory under `$HOME` that is not in a
  repository; that error counts as "no enclosing repository". The hook
  warns on stderr when `<root>/.claude/worktrees/` is non-empty, because a
  verbatim copy carries those worktrees' `.git` pointers into the lane (S1.3
  probes what that does; S4.1 advises keeping the directory empty).
- D3. **Removal is `gwz local dispose NAME`, nothing stronger and nothing
  softer.** The remove hook never passes `--force` or `--keep`. A refusal
  (unpreserved history, or dirt) makes the hook exit non-zero, which keeps the
  lane and the session. That is the safe outcome and is documented as such.
  A fourth outcome is not a refusal: the family lock still busy when the
  hook's wait deadline passes (D8). It also exits non-zero and keeps the
  session, but it is logged and printed as "family busy; retry" and never
  as a hazard, so nobody is sent to S3.4 for a lane that needed a minute.
  With gwz 1.0.12 and 1.0.13 it is also the only outcome for a verbatim lane,
  merged or not (section 1), so every Claude-created lane is retired through the
  terminal procedure (S3.4) until GwzLaneCleanFixes R0 lands. After that, a
  refusal means the lane holds something the family does not, and the hook
  still waives nothing: the narrower waiver names of R11 remain the
  operator's choice. Subagent worktree isolation is not used in GWZ
  workspaces until an integrated lane disposes in one command
  (GwzLaneCleanFixes R0, reached through verbatim-lane fixes or clean lanes;
  D7, S3.5): each subagent would otherwise leave a lane behind. The hook
  cannot enforce that rule unless U9 finds a signal, so it is a documented
  rule for agent briefs and the skill.
- D4. **No automatic merge-back.** Integration remains the operator's
  `gwz merge --remote NAME` from the receiving workspace. A helper the
  operator runs by hand may come later; hooks never integrate.
- D5. **gwz-dev adopts first, locally, then commits.** Phases 1 and 2 run
  with the hooks in the operator's `.claude/settings.local.json`; the block
  moves into the root's committed `.claude/settings.json` only after S1.3 has
  evidence (S1.4). A block may sit in both the project and the user settings
  of one machine: identical handler text runs once (section 1), and that is
  the normal state of a gwz-dev clone on a machine whose user settings also
  carry the block. Differing handlers (one pinned with `--command`, or
  carrying options) both run, in parallel, for the same `name` and
  `session_id`; the plan makes that harmless rather than forbidden, through
  one gwz-core facility this plan requires (GwzLaneCleanFixes R20, section
  1), the owner token gwz records on the family row in the same write that
  reserves it, and D6's attempt loop in the hook. (R21's `--wait` is the
  remove hook's dependency, D8; creation does not use it.) The second
  handler sees the first's `creating` row
  owned by its own session and keeps polling (D6's attempt loop) while the
  first holds the lock for the whole copy; when the row is `ready` it
  reuses it and prints the same path, so Claude receives one path twice
  (D6). On removal, the hook whose `worktree_path` is already gone exits
  zero and logs it. A parallel handler can still refuse for a reason of its
  own (a pinned `--command` naming a binary without the `hook` family, a
  malformed handler, or a wait that outlives its deadline): then Claude
  aborts the creation the first handler completed, and that lane is an
  orphan whose inventory is `gwz local list` and whose retirement is S3.4;
  `gwz hook claude-code setup` warns, and does not refuse, when another
  placement on the machine carries a differing handler, naming both files
  and this consequence.
  S1.4 requires user-level blocks on the dogfood machines to be removed or
  identical before the project block is committed, so the parallel case is
  rare in practice. S4.1 recommends one placement and explains this.
- D6. **Two guards in the create hook, and a strict reuse rule.** (Amended
  A1, 2026-09-17.) The guard protects the copy, not what a session builds
  afterwards: a build that fills the disk is a failure mode Claude's own
  worktrees already have, and the operator has ruled it out of scope. The
  copy's cost depends on the filesystem, so the hook estimates it at run
  time rather than carrying a measured constant. Estimate: walk the source
  once for apparent size and file count (the walk the completeness check
  needs anyway); probe the destination's parent by writing one 64 MB file
  there and cloning it with the platform's reflink call (`clonefile`,
  `FICLONE`, ReFS block cloning), reading the `df` delta; a failed clone,
  or a destination on a different filesystem from the source, means no
  sharing. Cost = apparent size × (1 − share) + a fixed margin, with
  share from a pessimistic table by filesystem: 94% for APFS, XFS and
  btrfs; 70% for ReFS; 70% for any other filesystem whose probe clones
  successfully; 0% for ext4, NTFS and any filesystem whose probe fails to
  clone. The table's numbers are placeholders until S3.2 measures them.
  Creation therefore refuses, with one stderr line naming the S3.4
  retirement procedure, when either holds: free space on the filesystem
  that holds the destination's parent directory (not the workspace's
  volume, which can differ under a symlinked or mounted root) is below the
  estimated cost, or below `--min-free-gb` when the handler carries that
  option as a floor; or the family already holds `--max-lanes` rows in
  state `ready` (default 8). Both are hook options baked into the
  handler by `setup` (D1), with compiled-in defaults, so the bare handler
  is guarded too. The guards apply only to an attempt that will create; the
  reuse rule is evaluated first, and a reuse consumes nothing, so it is
  never refused by a guard. The hook takes no lock of its own and keeps no
  state of its own. It runs a bounded attempt loop until its deadline, the
  `--wait-secs` option (compiled-in default 300 s, which `setup` raises,
  and only raises, to the estimated copy time when that is larger: file
  count × a per-file cost from the same table, pessimistic. The floor was
  added 2026-09-18 as S1.3's F5: the estimate models the copy and not the
  queue behind the family lock, which is what the wait is for, and on
  gwz-dev a baked 132 s met a 207 s copy of which all but ~30 s was
  queueing behind two other lanes. Claude's create timeout is one wait plus
  one estimated copy plus 60 s, A1): each
  attempt re-reads the family index lock-free (as `local list` does),
  evaluates the table below, re-evaluates both guards, and only then runs
  `local clone` in process with `--owner <session_id>` (R20) and no wait;
  a `Busy` result sleeps briefly and loops. An attempt that reaches the
  clone pays the clone's pre-lock source inventory before it can learn the
  lock is busy (the clone inventories the source at its step 4 and takes
  the lock at step 5), so against an unrelated family command the
  effective poll period is that inventory, not the sleep; the per-file
  cost the estimate uses covers it, and S3.2 measures it. The clone holds the family lock
  for the copy and records the owner in the same index write as the row,
  so there is no second store and no window between the row and its
  owner. The guards are therefore fresh at every attempt; the residual
  bound, stated here and accepted, is that two attempts admitted in the
  same instant can exceed `--max-lanes` by one. The hook validates
  `session_id` against R20's token grammar before using it and refuses
  with its own message otherwise. Reuse of an existing destination is
  allowed only when all of these hold: the index has a `ready` row for
  `NAME` whose recorded path canonicalises to the destination; the row's
  owner equals this invocation's `session_id`; and the lane's repositories
  pass a completeness check, which writes nothing. That check is
  deliberately cheap, and S1.3's F2 pinned down what it is: the row's own
  `observed_state`, which is gwz-core's observation of the lane's pointer
  and allocation marker, plus one existence test per repository the source
  workspace lists (the directory, and its `.git`). It is not a tree walk,
  a status or a fetch: a reuse has to stay a reuse, and on a
  one-repository fixture the check costs about 0.15 ms. Every other terminal combination refuses: exit non-zero, nothing
  on stdout, one stderr line naming the state and its remedy. The table,
  evaluated on every attempt:
  - no row, nothing at the destination: attempt the create.
  - no row, a directory at the destination (an unrelated tree, or a
    manual copy): refuse; choose another name or move the directory.
  - a `creating` row owned by this session: another handler for this
    session is mid-copy; keep looping until it settles or the deadline
    passes.
  - a `creating` row with no owner or another owner (an interrupted or
    foreign create): refuse; retire it through S3.4.
  - a `disposing` row: refuse; retire it through S3.4.
  - a `ready` row whose path differs from the destination: refuse; the
    name is taken by a lane elsewhere.
  - a `ready` row at the destination with no owner (a lane made by hand,
    or before this workspace's first owned lane): refuse, fail-closed;
    retire it through S3.4 or choose another name.
  - a `ready` row at the destination owned by another session: refuse,
    naming that session.
  - a `ready` row at the destination owned by this session, complete:
    reuse, print the path.
  - a `ready` row at the destination owned by this session, incomplete:
    refuse; retire it through S3.4.
  - the lock still busy, or the row still `creating`, when the deadline
    passes: refuse, naming the wait; Claude aborts, and any lane the other
    holder completes is the orphan D5 describes.
  A hook that Claude kills at its timeout mid-copy leaves a `creating` row
  and its files and writes no log line; `gwz local list` is the inventory
  and S3.4 the retirement, and the next attempt for that name refuses as
  above. The hook never prints a path it did not create or verify. The
  guards are not a substitute for S3.4.
- D7. **Verbatim lanes for now.** Clean lanes (`--clean`) are a GWZ product
  feature. Their requirements (R14 to R16), with those that let a verbatim
  lane dispose without a waiver (R1 to R8), are in gwz-core
  `dev-docs/GwzLaneCleanFixes.md`. This plan consumes them and implements
  neither: it measures the verbatim cost first (S3.2), and records at S3.3
  which route Claude use waits on.
- D8. **One test: is the project a GWZ workspace.** The create hook resolves
  the workspace root from `CLAUDE_PROJECT_DIR` exactly as `--root` resolution
  does for every other gwz command (the nearest enclosing manifest, loaded in
  process). Outcomes:
  - A workspace: a lane of it. When the session started inside a member, the
    printed path is that member's directory inside the lane, so the session
    lands where it started (S1.3 checks Claude accepts it: the directory is
    its own repository).
  - Not a workspace: Claude Code's default is reproduced as far as a hook
    can. `git worktree add -b worktree-NAME .claude/worktrees/NAME <base>`
    in the project repository, with the base taken from `--base-ref` when
    the handler carries it, else `origin/<default-branch>` when it resolves
    and `HEAD` otherwise; when `.claude/worktrees/NAME` already exists and
    is a registered worktree of the project on branch `worktree-NAME`
    (Claude's own earlier creation, or this hook's), it is reused and
    printed rather than failed, with no session check, unlike a lane: a
    plain worktree is cheap and recreatable, `git worktree remove` without
    `--force` protects a dirty tree, and it also refuses a locked worktree,
    which is what Claude holds on a running agent's worktree (section 1;
    S1.3 records that the lock is present for a fallback worktree, U10),
    so the plan accepts that two sessions naming the same slug share it
    (what Claude's own default does for a reused name is unconfirmed, U4;
    S1.3's fallback probe records it); the `.worktreeinclude` copy that Claude
    skips when a hook owns creation (section 1) is performed by the hook
    only for a worktree this invocation created (the same gitignore-style
    patterns Claude documents, matched against the project-root-relative
    path of the project root's own untracked files: the enumeration
    excludes the project's `.git` and `<root>/.claude/worktrees/`, so no
    worktree's included files are ever seen as the project's and copied
    into another worktree — S1.3's F7, where an unanchored `secrets.env`
    matched the copy inside an existing worktree and leaked it one
    directory deeper into the next); on reuse nothing is written, and a
    listed file missing from the reused worktree is reported on stderr,
    not supplied;
    and that path is printed. What the fallback cannot reproduce is the
    automatic sweep, which never runs for hook-created worktrees, so they
    accumulate until removed by hand (`git worktree list`, then
    `git worktree remove`); S4.1 states this. S1.1 picks the git CLI or
    git2's worktree API; the layout and these three behaviours are what
    matter.
  - A workspace that cannot be used (an open coordinated merge, an unreadable
    lock or marker, a manifest newer than the binary): creation aborts,
    non-zero, with gwz's message on stderr. These are ordinary gwz errors,
    not a third branch: a member-less worktree of the root is the failure
    this plan exists to prevent, so a workspace never falls back.
  There is no "`gwz` missing" branch: if `gwz` is not on the desktop app's
  `PATH` the hook does not run, Claude aborts creation with its own error,
  and S2.1 confirms `PATH` once. The remove hook decides by inspection of
  `worktree_path`, canonicalised first: a registered git worktree of the
  project is removed with `git worktree remove` (never `--force`; a dirty
  worktree makes the hook fail and keeps the session, matching D3); a path
  that is a lane root, or a member directory inside one, is resolved to its
  lane root through `<lane>/.gwz/family-root`, and disposed only when
  exactly one `ready` row of that family has a canonical path equal to the
  lane root; the dispose runs with an explicit `--root` naming the family
  root, `--wait <wait-secs>` (R21, the same deadline as the create hook)
  and a working directory at the family root, never inside the lane
  (dispose refuses the directory the caller stands in). A path that no
  longer exists exits zero and logs it (the other placement's handler, or a
  hand retirement, removed it first), and so does a dispose that finds the
  row already gone after classification; both are logged under D10's
  `already-absent`, which is a success class and not one of the three
  refusal classes (S1.3's F4). Anything else is refused. Every
  decision is logged (D10), and the log line distinguishes three classes:
  a refusal by the hook's own classification, a hazard refusal by gwz, and
  the family lock still busy at the deadline (D3), so a session that
  expected a lane and got a worktree, or a removal that failed for a reason
  D3 does not intend, can be seen for what it is. Original D8 (an external script
  probing `gwz --json status` and interpreting not-a-workspace codes) was
  simplified at adoption.
- D9. **Lanes keep the source branches.** The hook does not create
  `worktree-NAME` branches in the lane's repositories; a lane is the workspace
  as it sits, on whatever branches it sits on, and integration is
  `gwz merge --remote NAME`. Claude's branch-based views (the desktop's diff
  and PR flows) therefore do not describe a lane session; S2.1 records what
  the desktop shows for one and S4.1 says so. Alternative held in reserve:
  the hook creates the branch in the root and every member after the copy,
  if S2.1 shows the branch-based views matter more than the extra commands
  per lane.
- D10. **A lane is a snapshot of a live source.** The hook always runs while
  the parent session is alive and possibly building; LocalClones.md's quiet
  rule cannot be met from a hook. Policy: the lane's git object stores are
  verified by gwz (`dest-complete`); unsaved edits and build outputs in
  flight are copied best effort and a session in the lane may need to
  rebuild. The hook warns on stderr when a `cargo` process holds the source's
  target directory open. Both hooks log one line per decision to a fixed
  file, `<root>/.gwz/claude-hooks.log` for a workspace and
  `~/.claude/gwz-lane-hooks.log` for the fallback, because the desktop app
  cannot pass an environment variable to switch logging on; the `--log`
  hook option overrides the location. A log line carries exactly: a
  timestamp, the event, `name`, `session_id`, the resolved classification
  (lane, member-in-lane, fallback-worktree, already-absent,
  refused-by-hook, refused-by-hazard, family-busy; the last three are the
  refusal classes D8 names, and `already-absent` was added 2026-09-18 by
  S1.3's F4 for the exit-zero removal, which was being filed as a
  refusal), the path printed or acted on, the outcome, and the exit code;
  never `transcript_path` or `cwd`. `name` is the lane name: on the remove
  path, which Claude's payload gives no name at all, it is the name the
  hook read off the family row, and the destination's basename only when
  no row was found (F4). `path` is the path the hook printed or acted on,
  including on a refusal whose path it had already canonicalised (F4).
  Every non-zero exit also prints one stderr line of the form
  `gwz: <cause>; <the one command that resolves it>`, and the log records
  that same line — one line, not a report: a dispose hazard refusal is
  summarised as `<n> hazards across <m> repositories (<class>)` with the
  full report left to `gwz local dispose`, which the remedy names (F3). The file is bounded at 1 MB and truncated to its newest
  half when it passes that. The log is the only file the hook writes
  inside a workspace on its own account; the in-process create also
  writes, at the root and under `/.gwz/`, the family index (where the
  owner lives, R20) and the lock file, and it regenerates the root's
  managed `.git/info/exclude` block, which is what keeps all of them out of
  `git status`. The ignore check below governs the log alone; S1.1's
  fixture asserts the exclude block's content before and after a creation
  so the other writes are seen for what they are. Before writing the log,
  the hook checks that the location is ignored by the
  enclosing repository (in gwz-dev `/.gwz/` is ignored through the managed
  exclude block, not a tracked rule), because an untracked file in a
  member blocks every lane merge (gwz-dev L2) and counts as a `dirty`
  hazard at disposal. A log location that would appear in `git status` is
  not used, the user-level log is used instead, and a stderr note says so.
- D11. **POSIX hosts first, Windows from the same binary.** The hooks are gwz
  subcommands (D1), so there is nothing platform-specific to write twice;
  the platform-bound pieces (free-space measurement for D6, the busy-build
  check for D10, `HOME` resolution for the fallback log) live behind
  explicit `cfg_if!` boundaries per the standing rule on conditional
  compilation, and the fallback test runs on every platform. Windows is
  verified, not implemented, in a named later step (S5.1): Claude Code runs
  the same `gwz hook ...` command through `powershell.exe`.

## 4. Phases

Foundational work first; Phase 2 probes and Phase 3 measurements can run in
parallel once S1.3 has produced a working lane. Budgets are aspirational
targets, not limits.

### Phase 0: adoption (milestone: the plan is reviewed and adopted)

- **S0.1: review.** Done 2026-09-12 (section 7). The operator confirms D1 to
  D11 and the unconfirmed list. Output: this file's status line updated with
  a dated adoption note.

### Phase 1: hook subcommands and the CLI path (milestone: `claude --worktree NAME` in gwz-dev lands in a GWZ lane, and exiting with removal runs dispose, keeping the lane whenever dispose refuses)

- **S1.0: withdrawn by amendment A1 (2026-09-17).** The creation-cost
  measurement it described is replaced by the run-time estimate in D6 (a
  source walk, a reflink probe, and a pessimistic table by filesystem),
  which `setup` also uses to bake `--wait-secs` and both handlers'
  timeouts (S1.2). No lane is created and retired for measurement; S3.2
  validates the estimate against a real clone once the hooks exist. The
  number is kept so cross-references stay stable.
- **S1.1: the hook subcommands** *(gwz-cli, new `hook` command family;
  ~450 lines plus ~350 lines of test, above the aspiration; an implementer
  may split it along create/remove, landing create with its tests first)*.
  The contract is D1's: print only a created or verified path; delete only
  what `worktree_path` names. `gwz hook claude-code worktree-create` reads
  the hook JSON on stdin and takes `name` and `session_id`, validates the
  name (D2), resolves the workspace root from `CLAUDE_PROJECT_DIR` as D8
  says (falling back to the process's working directory when the variable
  is absent, so the command can be run by hand), validates `session_id`
  against R20's grammar, issues the busy-build warning (D10), and runs
  D6's attempt loop until `--wait-secs`: on each attempt a lock-free read
  of the index, the reuse table, then, only for an attempt that will
  create, the two guards, then the local clone in process with `--owner
  <session_id>` (R20) and all progress on stderr, sleeping and looping on
  `Busy` or on a `creating` row of its own session; it prints the
  canonical lane path (or the member path inside it) as the last stdout
  line. In the D8 fallback
  case it creates or reuses the git worktree under `.claude/worktrees/NAME`,
  performs the `.worktreeinclude` copy only when it created the worktree,
  and prints that path; on any error it exits non-zero with D10's one
  stderr line and nothing on stdout. `gwz hook claude-code worktree-remove` reads
  `worktree_path`, canonicalises and classifies it (D8): a registered git
  worktree of the project is removed with `git worktree remove`; a lane
  root, or a member directory inside one, is resolved through
  `.gwz/family-root` to the one `ready` row whose path matches and disposed
  with explicit `--root` from the family root, never `--keep` or `--force`
  (D3); an absent path exits zero; anything
  else is refused. Both log to the fixed file (D10). Also part of this
  step, because gwz-cli enforces them: regenerate `docs/CLI.md` with
  `scripts/generate_cli_reference.py --write` (the `g00` test compares it
  byte-for-byte and the release gate re-checks it); place the `hook` family
  under `Other:` in the curated grouping in `src/help.rs`; add
  `docs/commands/hook.md` and its `mkdocs.yml` nav entry.
  Tests, in gwz-cli's existing test tree, against a fixture workspace built
  in a temporary directory: a workspace (lane path printed and canonical,
  name validation including GWZ's refused names, each guard with a forced
  value, the cost estimate on a same-filesystem destination whose probe
  clones (share from the table) and on a destination whose probe fails or
  lies on another filesystem (share 0, the guard firing on apparent
  size), the `--min-free-gb` floor applying on top of the estimate, the busy-build warning, same-session reuse printing the same path
  with the row still `ready`, dispose exit codes on a clean and a dirty
  lane, a member-rooted `CLAUDE_PROJECT_DIR` resolving to the lane's member
  directory and its removal disposing the right lane with a sibling lane
  untouched, stdout carrying exactly one line); reuse refusals (a
  `creating/incomplete` row with its files; an unrelated directory at the
  destination; a row whose path points elsewhere; a row owned by another
  `session_id`; a `ready` row with no owner; a `ready` row owned by this
  session whose lane fails the completeness check; a `creating` row with
  another owner; a `disposing` row; a `session_id` outside R20's grammar),
  each exiting non-zero with nothing on stdout and a message naming the
  state; two creations with the same `name` and `session_id` where the
  second starts only after the first's `creating` row is visible in the
  index (one lane, two zero exits, one path, no abort), repeated with the
  family at `--max-lanes` minus one under differing handler text; the same
  with the first create killed mid-copy (the second refuses fail-closed,
  nothing on stdout); two creations of different names with the family at
  `--max-lanes` minus one, the second starting only after the first's
  `creating` row is visible in the index (exactly one new lane, the second
  refused by the ceiling once the first is `ready`, nothing copied for it,
  logged; the same-instant race D6 concedes is outside this test's scope); a
  wait that passes its deadline behind a lock held by a stub (refused,
  nothing created, the message naming the wait); a same-session reuse
  succeeding with free space forced below `--min-free-gb`; a guard refusal
  asserted to occur only when this attempt created nothing; the root's
  managed exclude block asserted before and after a creation; a removal whose process
  working directory is inside the lane still disposing; a `worktree_path`
  whose basename matches a family name but whose canonical path does not
  (refused); an absent `worktree_path` (exit zero, logged); a removal
  against a family whose lock a stub holds past the deadline (refused,
  classified busy, printed as "family busy; retry", lane intact); a
  removal whose row disappears between classification and dispose (exit
  zero, logged); a workspace
  with an open coordinated merge (creation aborts, nothing created,
  logged); a plain git repository (worktree created under
  `.claude/worktrees/NAME` on branch `worktree-NAME` from
  `origin/<default>` when present and `HEAD` otherwise, `--base-ref`
  honoured, an existing worktree and branch pair reused, reused again by a
  second `session_id` with the same path printed, a `.worktreeinclude`
  file's untracked entries present in a created worktree, an included file
  modified in the worktree surviving reuse byte-identical, an included
  file deleted from the worktree reported on stderr and not recreated,
  removed cleanly, refused when dirty, refused when `git worktree lock`
  is held on it); a `worktree_path` that is neither
  (refused); the log line's exact field set, the three refusal classes
  distinguished, `git status --porcelain` unchanged in the fixture after a
  creation and a removal, and a `--log` location that would show in
  `git status` redirected to the user log; the field-set assertion admits
  the three refusal classes of D10, and the busy-removal test asserts
  `family-busy`. Malformed or missing stdin
  fields exit non-zero with a message naming the field. The fallback tests
  run on every platform; only the platform-bound helpers sit behind
  `cfg_if!` (D11).
- **S1.2: the setup command** *(gwz-cli, `hook claude-code setup`; ~200
  lines plus ~150 lines of test)*. `gwz hook claude-code setup` prints the
  hooks block
  (`WorktreeCreate` and `WorktreeRemove`, each one command handler with
  its own timeout, computed by `setup` at write time from D6's estimate
  for the workspace it runs in (A1): `--wait-secs` baked as the estimated
  copy time, one wait plus one estimated copy plus 60 s for create, one
  wait plus 60 s for remove; outside a workspace, the compiled-in default
  and 600 s; hook options passed through into the handler text). With `--write` it merges the block into the chosen file
  (`--project` for `<root>/.claude/settings.json`, `--project --local` for
  `settings.local.json`, `--user` for `~/.claude/settings.json`) the way D1
  requires: parse first and refuse a file that does not parse or is not a
  regular file, naming the offending construct; temporary file, fsync,
  re-parse, rename; every byte outside the inserted block unchanged; the
  file created when absent; nothing done when the block is already present.
  It warns, naming both files, when another placement on this machine
  carries a differing handler (D5), and `--command PATH` substitutes an
  absolute binary for the bare `gwz` (D1). Also: regenerate `docs/CLI.md`,
  place `claude-code` under `Other:` in `src/help.rs`, add
  `docs/commands/claude-code.md` and its nav entry. Tests: a fresh file; a
  file with unrelated hooks, unknown top-level keys and unusual formatting
  (byte-identical outside the block after `--write`); a second run (no
  change); a file that does not parse (refused, untouched); a write
  interrupted before the rename (original bytes unchanged); a symlink
  target (refused); the differing-handler warning; `--command` and hook
  options appearing in the handler text; both handlers' timeouts present
  and consistent with the block's `--wait-secs`.
- **S1.3: local adoption and the CLI probe** *(evidence only; one command
  at the gwz-dev root, not committed, then ~2 hours of operator time)*.
  Run `gwz hook claude-code setup --project --local --write` at the gwz-dev
  root
  (D5). Then, from the gwz-dev root, run `claude --worktree probe-YYYYMMDD`.
  **The hand-run create probe and the `claude --worktree` probe must use
  different lane names**, or the second must run first: D6's reuse rule is
  keyed on `session_id`, a real session mints its own, and a lane a hand
  invocation claimed is refused to it (the 2026-09-18 run met exactly that,
  §5 of the note).
  Record: the session's working directory is the lane; `gwz status` and
  `gwz ls` work inside it; what lane-local state the copy carried that
  should not travel, such as `.gwz/url-scheme.yml` (O2); a
  `cargo build -p gwz` inside the lane succeeds and what it cost in time and
  disk; whether Bash commands that run `gwz` or `git` inside a member are
  allowed by the isolation check (U2), with the exact refusal text if not;
  an edit plus `gwz add` and `gwz commit` in the lane; exit with removal,
  expecting the dispose refusal to keep the lane, and the exact refusal text
  (U6); then `gwz merge --remote probe-YYYYMMDD` from the main workspace and
  a second removal. With gwz 1.0.13 the second removal is refused too, for
  the hazards the copy inherited (section 1): record its text, retire the
  lane through S3.4, and leave the removal that succeeds to S3.5. Repeat
  creation while `cargo build` runs in the source and record whether it
  completes and whether the lane's target directory is usable (D10). Repeat
  creation with one plain git worktree
  present under `<root>/.claude/worktrees/` and record what the lane contains
  and what Claude makes of it (D2). Start a session inside `gwz-core` and
  create a lane from there, confirming the member path inside the lane is
  accepted (D8). Confirm Claude's outside-any-repository check is not
  confused by the empty `~/.git` file (O4). Then probe the fallback: run
  `claude --worktree probe-plain` in a throwaway plain git repository with
  the hooks in user settings, confirming it lands in
  `.claude/worktrees/probe-plain` and is removed on exit (U7), and
  recording, while the session runs, whether `git worktree lock` is held
  on that worktree and whether `git worktree remove` refuses it (U10 for
  the fallback case, the protection D8 leans on); and once in
  gwz-dev with `gwz` hidden from `PATH`, recording what Claude reports when
  the hook command cannot run (D8 says creation aborts; confirm no
  member-less worktree appears). For every abort met along the way (the
  guards, the hidden `gwz`, a refused name), record exactly what the user
  sees, and whether the hook's stderr line reaches them, so S4.1's table
  describes observed text. Every lane this step creates is retired through
  S3.4 before the step closes. Write the findings to
  `gwz-cli/dev-docs/GwzClaudeIntegration-Probe-YYYYMMDD.md`, with owner
  tokens (session ids) redacted from every `gwz local list` transcript
  and hook-log excerpt, and fold any hook fix back into S1.1.
- **S1.4: committed adoption** *(gwz-dev root; ~10 lines)*. With S1.3
  evidence in hand, remove the block from `settings.local.json`, remove or
  make identical any user-level block on the dogfood machines (D5), run
  `gwz hook claude-code setup --project --write`, and commit the root's
  `.claude/settings.json` through `gwz commit --target @root`, so every
  clone of the workspace has it (D5). A clone whose installed `gwz` predates
  the `hook` family then carries a hook command that does not exist, and
  Claude aborts creation there; so S1.4 waits for a gwz release that
  contains S1.1 and S1.2, and S4.3 states the minimum version. Note for
  the operator: this commits routine lane creation for every clone before
  GwzLaneCleanFixes R0 lands, while retirement is still the L1 waiver
  procedure; keeping the block in `settings.local.json` until R0 is the
  one-line alternative, and the choice is the operator's at S1.4.

### Phase 2: desktop chips and background sessions (milestone: a "fix in worktree" chip and a background session start in a lane, or the gap is recorded)

- **S2.1: the chip probe** *(evidence; needs the desktop app)*. First
  confirm the desktop app's environment supplies `PATH` with `gwz` (section
  1, D8) by triggering one creation from the app and reading the hook log.
  Then trigger a task chip from a gwz-dev session, accept it, and check the
  new session's working directory and the hook log (D10). If the chip does
  not go through the hook (U1), record the observed behaviour, report it to
  Anthropic through the app's feedback channel, and note the interim
  workaround: start the spun-off work with `claude --worktree NAME` from
  the root instead. Record what a user sees when creation aborts from a
  chip; whether an interactive session's `EnterWorktree` fires the hook
  (U3); what the app's "Worktree location" and branch-prefix settings do
  once the hook owns creation (U5); and what the desktop's diff and PR views
  show for a lane session (D9). Every lane this step creates is retired
  through S3.4 before the step closes.
- **S2.2: the background-session probe** *(evidence)*. Start a background
  session (`/bg` or the desktop's parallel session) from gwz-dev; confirm the
  lane, the isolation behaviour, what the agent lock does to a lane (U10),
  and that deleting the session calls the remove hook with the dispose
  refusal semantics from S1.3, leaving the session listed.
- **S2.3: the subagent rule** *(decision plus one probe)*. D3 already sets
  the default: no subagent worktree isolation in GWZ workspaces until an
  integrated lane disposes in one command (GwzLaneCleanFixes R0). The probe
  launches one subagent with `isolation: "worktree"` from a gwz-dev session
  to record creation time, whether anything in the hook input identifies it
  (U9), and what its finished-with-changes lane looks like. A second subagent
  that changes nothing records what the remove hook does when it finishes:
  with gwz 1.0.13 dispose refuses and the lane stays. If U9 finds a signal,
  the hook refuses subagent creations in workspaces with a message naming the
  rule; if not, the rule lives in S4.1, S4.2 and agent briefs.

### Phase 3: lifecycle and cost (milestone: Claude-created lanes are cheap enough to make routinely and safe to retire)

- **S3.1: dispose semantics probe** *(evidence, then gwz-cli docs and
  possibly gwz-core; ~40 lines)*. Establish U8 by experiment: run a remove
  hook that exits zero and leaves the directory, and record whether Claude
  forgets the path. Make sure the refusal text a user sees through Claude
  names the remedy (`gwz merge --remote NAME` from the main workspace; U6). If
  the message lacks the remedy, record the gap against GwzLaneCleanFixes R9
  and R10, which already require hazards reported by category and the exact
  waiver command, rather than opening a separate gwz-core change.
- **S3.2: cost measurement** *(evidence; ~60 lines in the probe note; postponed indefinitely by operator decision D8 on 2026-09-18: the placeholder share table and per-file cost stand, and nothing in Phases 1, 2 or 4 waits on this step)*. On
  the gwz-dev volume, measure: lane creation wall time, quiet and under
  load, against the estimate `setup` baked (file count × per-file cost);
  the clone's pre-lock inventory interval; apparent and actual disk use of
  a fresh lane (`du` versus `df` deltas) against D6's estimate, giving the
  measured share for APFS; the same on the Linux (aarch64) and Windows
  hosts the acceptance runbook already uses, for ext4 or XFS and NTFS or
  ReFS (btrfs was measured on the Linux host on 2026-09-18 and shares
  within 1% of XFS, so the two keep one table entry and need no separate
  measurement); growth after `cargo build -p gwz` and after the gwz-core suite in
  the lane, for the docs only (out of the guard's scope, A1). Replace D6's
  placeholder table entries with the measured values, keeping them
  pessimistic, and state in the docs what a building lane costs.
- **S3.3: disposal route decision** *(decision only)*. From S3.2 and S2.3,
  decide which GwzLaneCleanFixes route Claude use waits on: verbatim lanes
  that dispose once integrated (R1 to R8), clean lanes from
  `gwz local clone --clean` (R14 to R16, which start with cold builds), or
  both. Record the choice here and in that document. The gwz-core work is
  planned there, not here.
- **S3.4: inventory and retirement** *(gwz-cli docs; ~60 lines)*. The
  procedure for lanes Claude cannot retire: chips and background sessions
  whose removal was refused, headless runs that never call the hook, crashed
  sessions, and `creating/incomplete` rows. `gwz local list` is the
  inventory (its owner column is a session id: redact it from anything
  committed); the order is `gwz merge --remote NAME` from the main workspace,
  then `gwz local dispose NAME --wait <secs>`, with `--force <hazard>` or
  `--keep` as the operator's explicit choices, never the hook's. The
  `--wait` is not optional in practice: with any other family operation in
  flight the bare command fails in 12 ms on the family lock (S1.3, §8).
  Until GwzLaneCleanFixes R0 lands, dispose refuses every verbatim lane
  even after the merge, so the procedure includes the L1 check from gwz-dev
  `dev-docs/GwzLaneIssues.md` before `--force dirty`. At gwz 1.0.14 a
  **merged** lane no longer raises `unpreserved-history` — the
  identical-copy witness does its job, and S1.3 saw `dirty` alone on all
  six refusing repositories — so the waiver for one is `dirty`;
  keep `dirty,unpreserved-history` for an **unmerged** lane until that is
  measured. After R0, a refusal means the
  lane holds unique work. Once the check-only mode of R12 exists, it is the
  inventory's report for each lane. A weekly look at `gwz local list` is the
  recommended habit until a Claude-side listing exists.
- **S3.5: adopt the disposal fixes** *(evidence, then doc edits; waits on an
  installed gwz that meets GwzLaneCleanFixes R0; ~40 lines in the probe
  note)*. Repeat S1.3's removal sequence and S2.3's subagent probes, timing
  the first disposal that succeeds, and size the remove handler's timeout
  in S1.2's block as one wait plus that measured dispose plus 60 s (until
  then it is one wait plus 60 s, adequate because a hook's dispose only
  refuses, in 5 to 16 s on gwz-dev; forced disposals of 27 to 238 s in
  gwz-dev L1 are the operator's, never the hook's). Confirm
  that an unintegrated lane still refuses, that an integrated lane disposes
  through the remove hook in one step, and that a subagent lane finishing
  without changes is removed. Record whether sessions write Claude state
  inside their lanes (O5). If every check passes, lift D3's subagent rule
  and update S3.4, S4.1 and S4.2 to match.

### Phase 4: documentation and the skill (milestone: any GWZ user can adopt the integration from the docs)

- **S4.1: the docs page** *(gwz-cli, `docs/ClaudeCode.md`, linked from
  `docs/AgentBootstrap.md` and the nav; ~200 lines)*. What the hooks do, the
  settings block, `gwz hook claude-code setup`, the recommended single
  placement
  and why two placements are harmless (D1, D5) together with the one case
  where they are not (a second handler that refuses for its own reason
  orphans the first's lane; `gwz local list` is the inventory, S3.4 the
  retirement), the migration for a machine that pinned `--command` before
  the project block was committed,
  prerequisites (`gwz` on the `PATH` the desktop app sees; the minimum gwz
  version; POSIX hosts until S5.1, D11), the hook options and their
  defaults (D6, D10; environment variables are not user configuration), the
  lane lifecycle from Claude's point of view, the snapshot policy (D10),
  branch semantics (D9), the isolation behaviour observed in S1.3 and S2.x,
  the copy-cost estimate's placeholder table stated as a conservative estimate (S3.2 is postponed, D8), the retirement procedure (S3.4, before and
  after GwzLaneCleanFixes R0), the subagent rule (D3), a table of every
  refusal the hooks can print with its one-line remedy (D10, as observed in
  S1.3), including "family busy; retry" as its own row (D3), the caveats (launch from the main root, headless runs never
  remove, hooks never merge, keep `.claude/worktrees/` empty in a root that
  uses lanes, what the fallback drops: no automatic sweep, so `git worktree
  list` and `git worktree remove` by hand, and no session ownership, so two
  sessions naming the same slug share one worktree), how to switch the hooks off
  quickly (delete the block; `worktree.bgIsolation: "none"` for background
  sessions), and what a failed creation looks like from a chip (S2.1).
- **S4.2: the skill** *(gwz-cli, `skills/gwz/SKILL.md`, "Local lanes"
  section; ~30 lines)*. Tell an agent that its session may already be running
  in a lane created by Claude Code, how to identify the source workspace, that
  integration happens from the receiving workspace, that it must not dispose
  the lane it is working in, and that it must not ask for subagent worktree
  isolation in a GWZ workspace until S3.5 lifts that rule (D3).
- **S4.3: pointers** *(gwz-dev `README.md` and `AGENTS_GWZ.md`; ~10 lines)*.
  **Done 2026-09-18.** One paragraph each pointing at the docs page, next to
  the existing install and clone instructions, naming the minimum gwz version
  that carries the `hook` family (S1.4: gwz 1.0.14). The `README.md` paragraph
  is written. `AGENTS_GWZ.md` is a gwz-managed generated file and was not
  edited: its body is
  `gwz-core/src/workspace_ops/agents_gwz_template.md`, rendered by
  `managed_agents_gwz_contents()` in
  `gwz-core/src/workspace_ops/workspace_bootstrap.rs`, so the pointer sentence
  is a gwz-core change routed by the lane owner, and every managed root picks
  it up on its next `gwz init --update`.

### Phase 5: Windows (deferred; milestone: the same behaviour through `powershell.exe`)

- **S5.1: Windows verification** *(evidence, plus fixes behind `cfg_if!`
  boundaries if any; ~40 lines in the probe note)*. Run the S1.1 tests on
  the windows-msvc job of the workspace matrix, then repeat S1.3's creation,
  removal and fallback probes on the Windows host the release process
  already uses, where Claude Code runs the same `gwz hook ...` command
  through `powershell.exe`. Record path canonicalisation (drive letters,
  `\\?\` prefixes) in the printed path, the free-space and busy-build helpers,
  and the fallback log location. Not scheduled until Phases 1 to 4 are done
  on POSIX.

## 5. Step dependency sketch

```
{ S0.1, GwzLaneCleanFixes R20 and R21 in an installed gwz } -> S1.1 -> S1.2 -> S1.3
{ S1.3, a gwz release containing S1.1 and S1.2 } -> S1.4 -> { S4.1, S4.3 }
S1.3 -> { S2.1, S2.2, S2.3, S3.1, S3.4 } -> S3.3 -> { S4.1 } -> S5.1   (S3.2 postponed, D8; S4.2 and S4.3 done)
{ S2.3, S3.3, GwzLaneCleanFixes R0 in an installed gwz } -> S3.5 -> revisions of S3.4, S4.1, S4.2
```

S2.x, S3.1, S3.2 and S3.4 are
independent of each other and can be picked up by different agents; S4.x
waits for their evidence so the docs describe measured behaviour. Three
dependencies lie outside this plan's steps: S1.1 waits for an installed
gwz that carries R20 and R21 (gwz-core work plus the gwz-cli surface
named in §3.6's scope note, landed in one lane; S1.0 is withdrawn, A1); S1.4 waits for a gwz release that carries the `hook` and
`claude-code` families; and S3.5 waits on gwz-core work for R0. Nothing
else waits on S3.5, and its revisions follow whenever it lands.

## 6. Open items

- O1. `claude --worktree NAME --tmux` opens the lane in a tmux session; worth
  a line in S4.1 once the lane path works.
- O2. Whether the hook should strip lane-local state that should not travel.
  **Answered in part 2026-09-18 by S1.3** (§4, §6): it is not `.gwz/`, which
  the lane regenerates and which holds nothing of the root's log, locks or
  merge state, and `.gwz/url-scheme.yml` does not exist in gwz-dev at all.
  What travels and should not is **`.claude/`**: `settings.local.json`
  (which, under local adoption, is the hook block itself, pointed at the
  lane) and `.cc-writes/`. Both become `dirty` hazards at disposal. The
  open half is what to do about it — strip it in the clone, exclude it, or
  leave it — which belongs with GwzLaneCleanFixes R8 and S3.3.
- O3. The fallback cannot read `worktree.baseRef` from settings without
  parsing them; it follows the documented default (`origin/<default-branch>`,
  else `HEAD`). If the operator sets `baseRef: "head"`, the fallback differs
  from Claude's own behaviour; S4.1 says so, and the `--base-ref` hook
  option, baked into the handler by `setup` (D1), is the override; an
  environment variable would not reach the desktop app (section 1).
- O4. The stray empty `~/.git` file on the operator's Mac makes git report
  "invalid gitfile format" for any directory under `$HOME` that is not inside
  a repository. D2 handles the hook's own check; S1.3 confirms Claude
  Code's outside-any-repository check is not confused by it either.
- O5. A session may write Claude state inside its lane, for example under
  `.claude/`. Under GwzLaneCleanFixes R8 that is changed ignored data, so the
  lane would still refuse after integration. That document leaves open
  whether such state is user work or tool state (its section 6). S3.5 records
  whether sessions write it, and the answer decides whether Claude lanes can
  dispose in one command. **S1.3 answered half of it early (§4, §6): a
  session is not the only writer, and it is not the first.** The adoption
  method itself puts `.claude/settings.local.json` in the workspace, the
  verbatim copy carries it and `.claude/.cc-writes/` into the lane
  byte-identical, and both are named as `dirty` hazards by the dispose
  refusal — before any session in the lane has written anything. On gwz-dev
  it changes no outcome, because `.venv/`, `bazel-out`, `target/` and six
  native stash entries already make every lane unconditionally dirty.

## 7. Adoption trail

- 2026-09-12: drafted (Fable). Fallback to a git worktree when GWZ is absent
  added the same day at the operator's request (D8).
- 2026-09-12: S0.1 round 1, adversarial self-review,
  `GwzClaudeIntegration-S0.1-Review.md`: GO-WITH-CONDITIONS, F1 to F18. All
  eighteen folded the same day: F1 and F12 into D8 and S1.1; F2 and F13 into
  D6 and the new S1.0; F3 into D3, S2.3 and the new S3.4; F4 into D10 and
  S1.3; F5 into D5, S1.2 and the new S1.4; F6 into D1 and S1.2; F7 into the
  new D9; F8 into D11 and the new Phase 5; F9 into D3 and S3.1; F10 into
  D10; F11 into D2 and S1.3; F14 into S3.4 and section 1; F15 into D11
  (since 2026-09-17, into D1: no `jq` or `python3` at all); F16
  and F17 into S4.1 and section 1; F18 into D2. Old O1 and O2 are resolved
  by D5 and S3.4; the open items were renumbered. Awaiting the operator's
  adoption.
- 2026-09-17: updated at the operator's request for the lane disposal
  clean-up requirements, gwz-core `dev-docs/GwzLaneCleanFixes.md` (R0 to
  R19). Section 1 records that every verbatim lane has refused disposal, even
  after its merge. U6, D3, D7, S1.0, S1.3, S2.3, S3.1, S3.3, S3.4, S4.1, S4.2
  and the dependency sketch now cite those requirements; S3.5 and O5 are new.
- 2026-09-17: **adopted** (S0.1 done). The operator confirmed D2 to D7 and
  D9 to D11 as written, replaced D1 (gwz is the hook, through
  `gwz hook claude-code worktree-create|worktree-remove`, and
  `gwz claude-code setup` writes the settings block), and simplified D8 to a
  single test (workspace: lane; otherwise: Claude's default git worktree;
  no `gwz`-missing branch). Folded into the Goal, U11 (resolved), D1, D2,
  D5, D8, D11, S1.1 (subcommands and Rust tests replace scripts and a stub),
  S1.2 (setup command), S1.3 (fallback probes), S1.4 (minimum-version
  condition), S4.1, S4.3 and S5.1 (verification, not PowerShell twins).
  Version stamps note that gwz 1.0.13 changes nothing for L1.
- 2026-09-17: round-2 dual peer-blind review of the adopted text,
  `GwzClaudeIntegrationPlan-ReviewConsistency.md` (NO-GO: 2 P2, 7 P3) and
  `GwzClaudeIntegrationPlan-ReviewSafety.md` (NO-GO: 1 P1, 6 P2, 2 P3).
  Both axes converged blind on D5's one-placement rule. All eighteen
  findings folded in one patch per `GwzClaudeIntegrationPlan-RemPlan.md`:
  the hook contract and knobs-as-options into D1; placement into D5 and
  U4; the two guards, the strict reuse rule and the session sidecar into
  D6; the fallback's reuse, `.worktreeinclude` copy and sweep gap, and the
  remove hook's canonical resolution, into D8; log fields, bound, ignore
  check and the one-line stderr remedy into D10; generated CLI artefacts,
  the full test list and the atomic settings writer into S1.1 and S1.2;
  local adoption moved into S1.3 and the desktop `PATH` check into S2.1
  (with U3, U5 and D9's views); S1.4's release gate into the sketch with
  the R0 ordering left to the operator; O2 into S1.3; O3 and section 1's
  dead error-code fact corrected; the lane count set to L1's 28.
- 2026-09-17: round-2 re-verdicts, `-ReviewConsistency-2.md` (all nine
  prior findings cured; NO-GO on 1 new P2, classified architectural, and
  2 new P3) and `-ReviewSafety-2.md` (all nine closed; NO-GO on 2 new P2,
  1 new P3). Every new finding came from the round-1 patch. Folded in one
  patch per `GwzClaudeIntegrationPlan-RemPlan-2.md`, the last remediation
  round the loop allows: D6 now holds the family lock across the clone and
  the session record and enumerates the full row-by-record table, with
  reuse evaluated before the guards; D5 names the lock and the orphan case;
  U4's resolution is conditional on that; the fallback's
  `.worktreeinclude` copy is scoped to a created worktree and its
  unsession-keyed reuse is stated with its reason; D10's ignore check
  covers the sidecar; D9 points at S2.1; the Goal's parity sentence is
  qualified; S1.1's tests extended.
- 2026-09-17: round-3 re-verdicts, `-ReviewSafety-3.md` (GO, one P3) and
  `-ReviewConsistency-3.md` (NO-GO: a second NEW ARCHITECTURAL root cause,
  A-P2-1: the round-2 cure assumed a family lock that waits and can be held
  across an in-process clone; gwz-core's lock is `try_lock` only and the
  clone takes it itself). The review loop's cap stopped the lane; the
  operator chose the redesign route on the same day: two gwz-core
  requirements, R20 (owner token on the family row, written with the row)
  and R21 (opt-in `--wait` on family commands), with R22 as their test,
  added to `GwzLaneCleanFixes.md`. This revision rests on them: the hook
  takes no lock and keeps no sidecar; D5, D6, D10, U4, S1.1 and the sketch
  rewritten; section 1's lock sentence corrected to the contract; the
  round-3 P3s folded (D8's unsourced default-behaviour claim marked
  unconfirmed under U4; S4.1 lists the orphan case and the shared fallback
  worktree; the lock-nesting note is moot).
- 2026-09-17: round-4 fresh dual review of the route-3 text,
  `-ReviewConsistency-4.md` (NO-GO: 3 P2, 4 P3) and `-ReviewSafety-4.md`
  (NO-GO: 4 P2, 4 P3); no architectural finding on either axis; both
  converged blind on the remove hook's dispose having no wait and no busy
  class. All fifteen folded in one patch per
  `GwzClaudeIntegrationPlan-RemPlan-3.md`: the create hook now runs its
  own attempt loop (reuse table and guards re-evaluated before every
  attempt, a `creating` row of its own session means wait, `Busy` means
  loop) with a compiled-in `--wait-secs` default of 300 s and Claude's
  timeout derived from it; the remove hook's dispose carries `--wait` and
  "family busy" is a fourth outcome in D3, D8, D10, S1.1 and S4.1; R20
  states its schema contract (v2, older readers refuse naming the minimum
  version) and §3.6 names the gwz-cli surface it carries; D6's table
  covers `disposing`, the incomplete same-owner lane, the timeout-kill
  residue and token validation; D10 names every file a create writes at
  the root; owner tokens are redacted from committed notes; the fallback's
  shared-worktree reason cites `git worktree remove` refusing a locked
  worktree.
- 2026-09-17: round-5 re-verdicts, `-ReviewSafety-5.md` (GO; 3 new P3)
  and `-ReviewConsistency-5.md` (all seven prior findings closed; NO-GO on
  3 new P2, 3 new P3, none architectural). Folded in one patch per
  `GwzClaudeIntegrationPlan-RemPlan-4.md`, the last remediation round on
  this object: S1.0 rewritten to D6's timeout formula and made to measure
  the clone's pre-lock inventory; D10's classification vocabulary carries
  the three refusal classes and S1.1's assertions match; the different-name
  concurrency test sequenced so it is deterministic; D5 and U4 attribute
  creation to R20 plus the attempt loop and R21 to the remove hook; S1.3's
  fallback probe records the worktree lock (U10); D6 states the
  per-attempt cost; R20's minimum-version rule keys on any index write;
  S1.2 gives the remove handler its own timeout and D1 lists
  `--wait-secs`.
- 2026-09-17: round-6 final re-verdicts, `-ReviewSafety-6.md` (GO, no
  findings) and `-ReviewConsistency-6.md` (GO, 2 P3). **Accepted** at
  sha256 `c0bba7fa…` with `GwzLaneCleanFixes.md` at `c5f06e41…`. The two
  P3s were folded after the GO, as the loop permits for P3s: section 1
  names the family merge among the writes that trigger R20's
  minimum-version rule, and S3.5 times the first succeeding dispose and
  sizes the remove handler's timeout from it. Nothing else in the accepted
  text changed. Across six rounds: 15 round-1 findings on the adopted
  text, 6 on the first patch, 4 on the second (one architectural: a lock
  the plan assumed and gwz-core lacks, which stopped the lane and led to
  the R20 to R22 redesign), then on the redesigned text 15, 6 and 2, all
  closed with re-traced counterexamples. Implementation may start once
  R20 and R21 are in an installed gwz.
- 2026-09-18: the probe-free part of Phase 4 landed in a lane. S4.3 done:
  gwz-dev `README.md` gained a Claude Code paragraph beside the install and
  clone instructions naming gwz 1.0.14; `AGENTS_GWZ.md` was not edited,
  because it is generated from
  `gwz-core/src/workspace_ops/agents_gwz_template.md` (rendered by
  `managed_agents_gwz_contents()` in
  `gwz-core/src/workspace_ops/workspace_bootstrap.rs`) and that change is
  routed to gwz-core by the lane owner. In `docs/ClaudeCode.md` the version
  caveat became a plain prerequisite (gwz 1.0.14 or later, with the format-2
  local-family index consequence stated), and, under D8, the copy-cost
  guard's share table and per-file time cost are now stated as conservative
  estimates rather than marked **Unmeasured**; the probe markers for S1.3,
  S2.1, S2.2, S2.3 and S3.5 are untouched.
- 2026-09-18: decision D8 (`GwzOpenDecisions.md`): S3.2's measurements are
  postponed indefinitely; S4.1 takes the placeholder cost table as a stated
  estimate and S3.3 draws on S2.3 alone; the sketch updated accordingly.
- 2026-09-18: S1.3's defects folded back in a lane. F7 (the fallback copied
  one worktree's `.worktreeinclude` files into the next), F3 (the 3.5 KB
  hazard refusal), F4 (the remove path's `name=`, `path=` and the exit-zero
  `absent` class), F2 (the reuse ran no completeness check), F5 (the baked
  wait had no floor, and `setup` estimated in a plain repository) and F1
  (the merge wrote one near-minified line in the wrong event order) are
  fixed in gwz-cli with tests. Folded here: U6 (answered), O2 and O5
  (rewritten from §4 and §6), D6 (the wait's floor, and what the
  completeness check is), D8 (the include enumeration's scope, and the
  `already-absent` class), D10 (the vocabulary and the log fields, and the
  one-line hazard summary), S1.3 (two lane names) and S3.4 (`--wait`, and
  `dirty` alone for a merged lane at 1.0.14). `docs/ClaudeCode.md` and
  `docs/commands/hook.md` follow. F6 (`--force` reporting the flags rather
  than the waived hazards) is gwz-core's and is not fixed here.
- 2026-09-17: **amendment A1**, at the operator's decision and, by the
  operator's instruction, without re-review. The free-space guard no
  longer sizes itself from a one-off measured clone or from what a lane
  builds afterwards (a build filling the disk is a failure mode Claude's
  own worktrees already have, ruled out of scope): the hook estimates the
  copy's cost at run time from a source walk, a reflink probe at the
  destination's parent, and a pessimistic share table by filesystem (94%
  APFS, XFS, btrfs; 70% ReFS and any other filesystem that clones; 0%
  ext4, NTFS and any that does not), and `setup` bakes `--wait-secs` and
  both handler timeouts from the same estimate. S1.0 is withdrawn, its
  number retained; S3.2 measures the placeholders on the three hosts and
  replaces them. Touched: D6, S1.0, S1.1 (tests), S1.2, S3.2, section 1
  (a block-sharing fact), section 5, the status line.
- 2026-09-18: amendment A2 (operator decision, Surface review):
  `gwz claude-code setup` moved under `gwz hook claude-code setup`;
  `--remove` added; S4.2 skill text landed.
