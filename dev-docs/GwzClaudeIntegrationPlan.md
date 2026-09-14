# GWZ and Claude Code Integration Plan

Status: **DRAFT 2026-09-12, round-1 review folded.** Written by Fable at the
operator's request after the 1.0.11 release; reviewed the same day
(`GwzClaudeIntegration-S0.1-Review.md`: GO-WITH-CONDITIONS, 4 P1, 8 P2,
6 P3) and every finding folded below (trail in section 7). Not adopted; no
code, settings or scripts have changed. Implementation is chartered for
non-Fable agents (Opus builders) under the usual review loop once the
operator confirms the decisions in section 3.

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
protections. Where `gwz` is not installed, or the project is not a GWZ
workspace, the same hooks fall back to a plain git worktree laid out the way
Claude Code lays it out by default, so they can live in user-level settings
without breaking other projects.

## 1. Facts this plan rests on

Claude Code 2.1.247, from the official documentation
(<https://code.claude.com/docs/en/worktrees> and
<https://code.claude.com/docs/en/hooks#worktreecreate>, read 2026-09-12):

- `WorktreeCreate` fires "when a worktree is being created via `--worktree`,
  `isolation: "worktree"`, or for a background session", and "configuring a
  WorktreeCreate hook replaces that default git behavior".
- The hook reads JSON on stdin with the common fields (`session_id`, `cwd`,
  `transcript_path`, `hook_event_name`) plus `name`, "a slug identifier for
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
  Family commands serialize on that lock, so two creations at once run one
  after the other rather than failing.
- Integration is `gwz merge --remote NAME` run from the receiving workspace.
  `gwz local dispose NAME` "refuses unless the lane's history is verifiably
  preserved"; a dirty lane is a hazard in its own right; `--force <hazard,...>`
  waives named hazards; `--keep` "removes only the pointer and the index row"
  and retains every file. The root is never disposed.
- The gwz-dev workspace is 65 GB on disk, most of it cargo target trees. On
  APFS the verbatim copy is a copy-on-write clone, cheap until a lane builds;
  a full build in a lane then writes its own tens of gigabytes (the 2026-09-11
  disk-full incident came from exactly this). Free space at the time of
  writing: 31 GB.
- `gwz` refuses `root`, `origin` and git's reserved ref names as lane names.
- `gwz --root DIR status` in a directory without a manifest exits 1 with
  `ManifestNotFound`; the agent skill also names `WorkspaceNotFound`. Status
  fails for other reasons too (an open coordinated merge, an unreadable lock
  or marker, a binary older than the workspace's schema), which are not
  "this is not a workspace".

## 2. What is not confirmed

These are the questions the probes in Phases 1 and 2 must answer before the
integration is documented for other users.

- U1. Whether the desktop app's task chips ("fix in worktree") create their
  session through `WorktreeCreate`. The docs never name the chips; they say
  hooks fire in the desktop app and that a chip starts "a new session with its
  own worktree".
- U2. How the isolation check that inspects command text for git treats a
  wrapper such as `gwz`, which runs git internally, when invoked inside a
  lane.
- U3. Whether an interactive session's `EnterWorktree` fires the hook.
- U4. Whether Claude Code checks for an existing directory before calling the
  hook for a reused name, and which path wins if two settings files both
  define a path-returning `WorktreeCreate` hook.
- U5. What the desktop app's "Worktree location" and branch-prefix settings do
  once a hook owns creation.
- U6. How the dispose refusal reads when Claude reports it, and whether the
  message names the remedy.
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
- U11. The exact machine-readable error codes that mean "not a workspace"
  (`ManifestNotFound` observed; `WorkspaceNotFound` named by the skill); S1.1
  pins the set by test.

## 3. Scope decisions (proposed; the operator confirms or changes them at S0.1)

- D1. **Ship the integration from gwz-cli.** New directory
  `gwz-cli/integrations/claude-code/` holding the two hook scripts, a
  `settings.hooks.json` fragment, and a README. Users copy the scripts into
  the place their settings invoke; the docs page in Phase 4 explains it.
  Alternative rejected: bake the scripts into the agent skill, because hooks
  are settings, not skill content, and the skill is copied to
  `~/.claude/skills`.
- D2. **The lane name is Claude's `name` slug**, validated by the script
  against GWZ's refused names and characters outside `[A-Za-z0-9._-]`, and the
  destination is GWZ's default sibling `../<root-dirname>-NAME`. The script
  checks that the parent of the workspace is not inside a repository, by
  walking up looking for `.git` rather than by asking git, because on the
  operator's Mac an empty `~/.git` file makes `git rev-parse` fail with
  "invalid gitfile format" for every directory under `$HOME` that is not in a
  repository; that error counts as "no enclosing repository". The script
  warns on stderr when `<root>/.claude/worktrees/` is non-empty, because a
  verbatim copy carries those worktrees' `.git` pointers into the lane (S1.3
  probes what that does; S4.1 advises keeping the directory empty).
- D3. **Removal is `gwz local dispose NAME`, nothing stronger and nothing
  softer.** The remove hook never passes `--force` or `--keep`. A refusal
  (unpreserved history, or dirt) makes the hook exit non-zero, which keeps the
  lane and the session. That is the safe outcome and is documented as such.
  Retiring a refused lane is a terminal procedure (S3.4). Subagent worktree
  isolation is not used in GWZ workspaces until clean lanes exist (D7): each
  subagent that changes anything would otherwise leave a lane behind. The
  hook cannot enforce that rule unless U9 finds a signal, so it is a
  documented rule for agent briefs and the skill.
- D4. **No automatic merge-back.** Integration remains the operator's
  `gwz merge --remote NAME` from the receiving workspace. A helper the
  operator runs by hand may come later; hooks never integrate.
- D5. **gwz-dev adopts first, locally, then commits.** Phases 1 and 2 run
  with the hooks in the operator's `.claude/settings.local.json`; the block
  moves into the root's committed `.claude/settings.json` only after S1.3 has
  evidence (S1.4). The hooks live in exactly one settings file per machine,
  project or user, never both: two different commands that both print a path
  both run, and the winner is undefined (U4). S4.1 tells users which to
  choose.
- D6. **A free-space guard in the create hook.** Creation refuses, with a
  plain stderr message, when free space on the workspace volume is below
  `GWZ_LANE_MIN_FREE_GB`. The default is a floor for the copy alone, set from
  the S1.0 measurement and no higher than half the free space on the dogfood
  machine at that time; S3.2 raises it with build numbers and states what a
  building lane costs. The guard is not a substitute for S3.4.
- D7. **Verbatim lanes for now.** Clean lanes (`--clean`) are a GWZ product
  feature with its own plan; this plan measures the verbatim cost first
  (S3.2) and records whether clean lanes are needed for Claude use.
- D8. **Fall back to a git worktree only when GWZ is provably not in play.**
  The create hook resolves the workspace root by walking up from
  `CLAUDE_PROJECT_DIR` to the nearest directory holding `gwz.conf/gwz.lock.yml`
  (a candidate filter only), then asks `gwz --root CANDIDATE --json status`
  for the final word. Outcomes:
  - `gwz` on `PATH` and status succeeds: a lane of that workspace. When the
    session started inside a member, the printed path is that member's
    directory inside the lane, so the session lands where it started (S1.3
    checks Claude accepts it: the directory is its own repository).
  - `gwz` missing, or no candidate root, or status fails with one of the
    not-a-workspace codes pinned in S1.1 (U11): Claude Code's default is
    reproduced, `git worktree add -b worktree-NAME .claude/worktrees/NAME
    <base>` in the project repository, with the base taken from
    `origin/<default-branch>` when it resolves and `HEAD` otherwise, and that
    path is printed.
  - status fails for any other reason: creation aborts, non-zero, with gwz's
    message on stderr. The fallback is never the answer to an unexpected
    error, because a member-less worktree of the root is the failure this
    plan exists to prevent.
  The remove hook decides by inspection of `worktree_path`: a registered git
  worktree of the project is removed with `git worktree remove` (never
  `--force`; a dirty worktree makes the hook fail and keeps the session,
  matching D3); a sibling lane of a workspace root, or a member directory
  inside one, is disposed with GWZ. Anything else is refused. Every decision
  is logged (D10) so a session that expected a lane and got a worktree can
  see why.
- D9. **Lanes keep the source branches.** The hook does not create
  `worktree-NAME` branches in the lane's repositories; a lane is the workspace
  as it sits, on whatever branches it sits on, and integration is
  `gwz merge --remote NAME`. Claude's branch-based views (the desktop's diff
  and PR flows) therefore do not describe a lane session; S1.3 records what
  the desktop shows for one and S4.1 says so. Alternative held in reserve:
  the hook creates the branch in the root and every member after the copy,
  if S1.3 shows the branch-based views matter more than the extra commands
  per lane.
- D10. **A lane is a snapshot of a live source.** The hook always runs while
  the parent session is alive and possibly building; LocalClones.md's quiet
  rule cannot be met from a hook. Policy: the lane's git object stores are
  verified by gwz (`dest-complete`); unsaved edits and build outputs in
  flight are copied best effort and a session in the lane may need to
  rebuild. The hook warns on stderr when a `cargo` process holds the source's
  target directory open. Both scripts log one line per decision to a fixed
  file, `<root>/.gwz/claude-hooks.log` for a workspace and
  `~/.claude/gwz-lane-hooks.log` for the fallback, because the desktop app
  cannot pass an environment variable to switch logging on;
  `GWZ_LANE_HOOK_LOG` overrides the location when it is set.
- D11. **POSIX hosts first.** The scripts are POSIX sh and need `jq` or
  `python3` (they fall back from one to the other for the two fields they
  read). Windows, where Claude Code runs hooks through `powershell.exe`, is a
  named later step (S5.1), not an omission; the Rust test that executes the
  scripts is unix-only behind an explicit `cfg_if!` boundary, per the
  standing rule on conditional compilation.

## 4. Phases

Foundational work first; Phase 2 probes and Phase 3 measurements can run in
parallel once S1.3 has produced a working lane. Budgets are aspirational
targets, not limits.

### Phase 0: adoption (milestone: the plan is reviewed and adopted)

- **S0.1: review.** Done 2026-09-12 (section 7). The operator confirms D1 to
  D11 and the unconfirmed list. Output: this file's status line updated with
  a dated adoption note.

### Phase 1: hook scripts and the CLI path (milestone: `claude --worktree NAME` in gwz-dev lands in a GWZ lane, and exiting with removal disposes it)

- **S1.0: creation cost, measured once** *(evidence; independent of S1.1;
  ~30 minutes)*. From the gwz-dev root, `gwz local clone probe-cost`, timed,
  with `df` before and after and `du -sh` of the lane; then dispose it. The
  numbers set D6's default and the creation timeout in S1.1 (sized for two
  copies back to back, since two creations serialize on the family lock).
- **S1.1: the scripts** *(gwz-cli, `integrations/claude-code/`; ~160 lines
  of POSIX sh plus ~200 lines of test)*. `gwz-lane-create.sh` reads `name`
  (`jq`, else `python3`), validates it (D2), resolves the workspace root as
  D8 says, applies the free-space guard (D6) and the busy-build warning
  (D10), reuses an existing lane directory if present (idempotent, covers
  U4), runs `gwz --root ROOT local clone NAME` with all output on stderr, and
  prints the lane path (or the member path inside it) as the last line with
  `pwd -P`. In the D8 fallback case it creates the git worktree under
  `.claude/worktrees/NAME` and prints that path; in the abort case it exits
  non-zero with gwz's message. `gwz-lane-remove.sh` classifies
  `worktree_path` (D8): a registered git worktree is removed with
  `git worktree remove`; a lane, or a member directory inside one, is
  disposed with `gwz --root ROOT local dispose NAME`, never `--keep` or
  `--force` (D3); anything else is refused. Exit codes propagate. Both scripts
  log to the fixed file (D10).
  Tests: a Rust integration test in `gwz-cli/tests/claude_code_hooks.rs`,
  unix-only behind a `cfg_if!` boundary (D11), runs both scripts with a stub
  `gwz` on `PATH` that replays recorded JSON: a workspace (lane path printed,
  name validation, guard, idempotent reuse, dispose exit codes, member-rooted
  session resolving to the lane's member directory); `gwz` present and a
  workspace present but status failing with an unrelated code (creation
  aborts, nothing created); each not-a-workspace code (fallback taken and
  logged, U11 pinned here); an empty `PATH` in a plain git repository
  (worktree created under `.claude/worktrees/NAME` on branch
  `worktree-NAME`, removed cleanly, refused when dirty); `jq` absent
  (`python3` fallback). `settings.hooks.json` carries the two hook entries
  with the S1.0 timeout, and the README documents the copy step (D1, S1.2).
- **S1.2: local adoption in gwz-dev** *(gwz-dev root; ~30 lines, not
  committed)*. Copy the scripts to `gwz-dev/.claude/hooks/` (the root owns
  what its settings invoke; a missing or detached gwz-cli member must not
  break creation) and add the hooks block to the operator's
  `.claude/settings.local.json` (D5). The settings command first checks the
  script exists and otherwise explains on stderr. Confirm the desktop app's
  environment supplies `PATH` with `gwz` (section 1).
- **S1.3: the CLI probe** *(evidence only; ~2 hours of operator time)*. From
  the gwz-dev root run `claude --worktree probe-YYYYMMDD`. Record: the
  session's working directory is the lane; `gwz status` and `gwz ls` work
  inside it; a `cargo build -p gwz` inside the lane succeeds and what it cost
  in time and disk; whether Bash commands that run `gwz` or `git` inside a
  member are allowed by the isolation check (U2), with the exact refusal text
  if not; what the desktop's diff and PR views show for a lane session (D9);
  an edit plus `gwz add` and `gwz commit` in the lane; exit with removal,
  expecting the dispose refusal to keep the lane, and the exact refusal text
  (U6); then `gwz merge --remote probe-YYYYMMDD` from the main workspace and
  a second removal that succeeds. Repeat creation while `cargo build` runs in
  the source and record whether it completes and whether the lane's target
  directory is usable (D10). Repeat creation with one plain git worktree
  present under `<root>/.claude/worktrees/` and record what the lane contains
  and what Claude makes of it (D2). Start a session inside `gwz-core` and
  create a lane from there, confirming the member path inside the lane is
  accepted (D8). Confirm Claude's outside-any-repository check is not
  confused by the empty `~/.git` file (O4). Then probe the fallback: run
  `claude --worktree probe-plain` in a throwaway plain git repository, and
  once more in gwz-dev with `gwz` hidden from `PATH`, confirming both land in
  `.claude/worktrees/probe-plain` and are removed on exit (U7). Write the
  findings to `gwz-cli/dev-docs/GwzClaudeIntegration-Probe-YYYYMMDD.md` and
  fold any script fix back into S1.1.
- **S1.4: committed adoption** *(gwz-dev root; ~10 lines)*. With S1.3
  evidence in hand, move the hooks block from `settings.local.json` to the
  root's committed `.claude/settings.json` through
  `gwz commit --target @root`, so every clone of the workspace has it (D5).

### Phase 2: desktop chips and background sessions (milestone: a "fix in worktree" chip and a background session start in a lane, or the gap is recorded)

- **S2.1: the chip probe** *(evidence; needs the desktop app)*. Trigger a
  task chip from a gwz-dev session, accept it, and check the new session's
  working directory and the hook log (D10). If the chip does not go through
  the hook (U1), record the observed behaviour, report it to Anthropic
  through the app's feedback channel, and note the interim workaround: start
  the spun-off work with `claude --worktree NAME` from the root instead.
  Record what a user sees when creation aborts from a chip.
- **S2.2: the background-session probe** *(evidence)*. Start a background
  session (`/bg` or the desktop's parallel session) from gwz-dev; confirm the
  lane, the isolation behaviour, what the agent lock does to a lane (U10),
  and that deleting the session calls the remove hook with the dispose
  refusal semantics from S1.3, leaving the session listed.
- **S2.3: the subagent rule** *(decision plus one probe)*. D3 already sets
  the default: no subagent worktree isolation in GWZ workspaces until clean
  lanes exist. The probe launches one subagent with `isolation: "worktree"`
  from a gwz-dev session to record creation time, whether anything in the
  hook input identifies it (U9), and what its finished-with-changes lane
  looks like. If U9 finds a signal, the hook refuses subagent creations in
  workspaces with a message naming the rule; if not, the rule lives in S4.1,
  S4.2 and agent briefs.

### Phase 3: lifecycle and cost (milestone: Claude-created lanes are cheap enough to make routinely and safe to retire)

- **S3.1: dispose semantics probe** *(evidence, then gwz-cli docs and
  possibly gwz-core; ~40 lines)*. Establish U8 by experiment: run a remove
  hook that exits zero and leaves the directory, and record whether Claude
  forgets the path. Make sure the refusal text a user sees through Claude
  names the remedy (`gwz merge --remote NAME` from the main workspace; U6). If
  the message lacks the remedy, that is a gwz-core message change with its
  own test.
- **S3.2: cost measurement** *(evidence; ~60 lines in the probe note)*. On
  the gwz-dev volume, measure: lane creation wall time (S1.0 repeated under
  load); apparent and actual disk use of a fresh lane (`du` versus `df`
  deltas, since APFS clones share blocks); growth after `cargo build -p gwz`
  and after the gwz-core suite in the lane. Set the D6 default from the
  numbers, and state in the docs what a building lane costs.
- **S3.3: clean lanes decision** *(decision only)*. From S3.2 and S2.3,
  decide whether Claude use needs `gwz local clone --clean` (a lane without
  build outputs and dirt). If yes, open a separate plan in gwz-core; this
  plan does not implement it.
- **S3.4: inventory and retirement** *(gwz-cli docs; ~60 lines)*. The
  procedure for lanes Claude cannot retire: chips and background sessions
  whose removal was refused, headless runs that never call the hook, crashed
  sessions, and `creating/incomplete` rows. `gwz local list` is the
  inventory; the order is `gwz merge --remote NAME` from the main workspace,
  then `gwz local dispose NAME`, with `--force <hazard>` or `--keep` as the
  operator's explicit choices, never the hook's. A weekly look at
  `gwz local list` is the recommended habit until a Claude-side listing
  exists.

### Phase 4: documentation and the skill (milestone: any GWZ user can adopt the integration from the docs)

- **S4.1: the docs page** *(gwz-cli, `docs/ClaudeCode.md`, linked from
  `docs/AgentBootstrap.md` and the nav; ~200 lines)*. What the hooks do, the
  settings block and the one-placement rule (D5), where the scripts live and
  the copy step (D1), prerequisites (`jq` or `python3`; POSIX hosts, D11),
  the lane lifecycle from Claude's point of view, the snapshot policy (D10),
  branch semantics (D9), the isolation behaviour observed in S1.3 and S2.x,
  the cost numbers from S3.2, the retirement procedure (S3.4), the subagent
  rule (D3), the caveats (launch from the main root, headless runs never
  remove, hooks never merge, keep `.claude/worktrees/` empty in a root that
  uses lanes, `baseRef` parity for the fallback), how to switch the hooks off
  quickly (delete the block; `worktree.bgIsolation: "none"` for background
  sessions), and what a failed creation looks like from a chip (S2.1).
- **S4.2: the skill** *(gwz-cli, `skills/gwz/SKILL.md`, "Local lanes"
  section; ~30 lines)*. Tell an agent that its session may already be running
  in a lane created by Claude Code, how to identify the source workspace, that
  integration happens from the receiving workspace, that it must not dispose
  the lane it is working in, and that it must not ask for subagent worktree
  isolation in a GWZ workspace (D3).
- **S4.3: pointers** *(gwz-dev `README.md` and `AGENTS_GWZ.md`; ~10 lines)*.
  One paragraph each pointing at the docs page, next to the existing install
  and clone instructions.

### Phase 5: Windows (deferred; milestone: the same behaviour through `powershell.exe`)

- **S5.1: PowerShell twins** *(gwz-cli, `integrations/claude-code/*.ps1`;
  ~200 lines plus a Windows-only test module)*. The same decisions, the same
  log, the same fallback; probed on the Windows host the release process
  already uses. Not scheduled until Phases 1 to 4 are done on POSIX.

## 5. Step dependency sketch

```
S0.1 -> { S1.0, S1.1 } -> S1.2 -> S1.3 -> S1.4
S1.3 -> { S2.1, S2.2, S2.3, S3.1, S3.2, S3.4 } -> S3.3 -> { S4.1, S4.2, S4.3 } -> S5.1
```

S1.0 and S1.1 are independent of each other. S2.x, S3.1, S3.2 and S3.4 are
independent of each other and can be picked up by different agents; S4.x
waits for their evidence so the docs describe measured behaviour.

## 6. Open items

- O1. `claude --worktree NAME --tmux` opens the lane in a tmux session; worth
  a line in S4.1 once the lane path works.
- O2. Whether the hook should strip lane-local state that should not travel,
  such as `.gwz/url-scheme.yml`; S1.3 decides from what the lane looks like.
- O3. The fallback cannot read `worktree.baseRef` from settings without
  parsing them; it follows the documented default (`origin/<default-branch>`,
  else `HEAD`). If the operator sets `baseRef: "head"`, the fallback differs
  from Claude's own behaviour; S4.1 says so, and a `GWZ_LANE_BASE_REF`
  environment variable is the cheap override if anyone needs it.
- O4. The stray empty `~/.git` file on the operator's Mac makes git report
  "invalid gitfile format" for any directory under `$HOME` that is not inside
  a repository. D2 handles the script's own check; S1.3 confirms Claude
  Code's outside-any-repository check is not confused by it either.

## 7. Adoption trail

- 2026-09-12: drafted (Fable). Fallback to a git worktree when GWZ is absent
  added the same day at the operator's request (D8).
- 2026-09-12: S0.1 round 1, adversarial self-review,
  `GwzClaudeIntegration-S0.1-Review.md`: GO-WITH-CONDITIONS, F1 to F18. All
  eighteen folded the same day: F1 and F12 into D8 and S1.1; F2 and F13 into
  D6 and the new S1.0; F3 into D3, S2.3 and the new S3.4; F4 into D10 and
  S1.3; F5 into D5, S1.2 and the new S1.4; F6 into D1 and S1.2; F7 into the
  new D9; F8 into D11 and the new Phase 5; F9 into D3 and S3.1; F10 into
  D10; F11 into D2 and S1.3; F14 into S3.4 and section 1; F15 into D11; F16
  and F17 into S4.1 and section 1; F18 into D2. Old O1 and O2 are resolved
  by D5 and S3.4; the open items were renumbered. Awaiting the operator's
  adoption.
