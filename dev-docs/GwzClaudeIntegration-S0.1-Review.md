# GWZ and Claude Code Integration: S0.1 Plan Review

- **Step:** S0.1 (Phase 0, plan review), per `GwzClaudeIntegrationPlan.md`.
- **Date:** 2026-09-12.
- **Tier / mode:** adversarial self-review by the plan's author (Fable), at
  the operator's request. Same-author reviews are weaker than peer-blind ones;
  every finding below was checked against a source outside the plan before it
  was written down.
- **Object read:** `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` (DRAFT
  2026-09-12, 307 lines, with the D8 fallback folded in).
- **Sources checked (read-only):** the Claude Code documentation pages for
  worktrees, hooks, settings reference, desktop, agent view and the tools
  reference, as fetched on 2026-09-12 (version 2.1.247 is installed);
  `gwz-cli/docs/LocalClones.md` at 1.0.11; `gwz local --help`,
  `gwz local clone --help`, `gwz local dispose --help`; the observed exit of
  `gwz --root /tmp status` (exit 1, `ManifestNotFound`); `df` on the
  workspace volume (31 GB free); the empty `~/.git` file on the operator's
  Mac. Nothing was built, changed, committed or pushed.

---

## Verdict

# GO-WITH-CONDITIONS

The plan's shape survives: hooks first, probes before documentation, dispose
rather than delete, no automatic merge-back, verbatim lanes measured before
clean lanes are considered. Its documentary claims about Claude Code check out
against the pages they cite.

It cannot be adopted as written because three of its decisions are wrong in
ways the first dogfood run would expose immediately, and one lifecycle hole is
filed as an open item when it is the main cost of the whole idea:

- **F1** (P1): the D8 fallback trigger turns any `gwz status` failure into a
  silent git worktree of the root, which is precisely the broken state the
  plan exists to prevent.
- **F2** (P1): the D6 free-space default is larger than the free space on the
  machine the plan dogfoods on, so S1.3 would refuse every lane.
- **F3** (P1): every chip, background session or subagent that produces work
  leaves a lane that only a terminal can retire, and nothing sweeps them.
- **F4** (P1): the copy runs while the parent session is live, which is what
  `LocalClones.md` tells operators never to do.

Conditions: fold F1 to F4 and the P2 findings F5 to F12 into the plan before
adoption. The P3 items are at the author's discretion. A second review round
is needed only if F7's decision changes the shape of Phase 1.

---

## Findings

### [P1 F1] D8's fallback fires on the wrong condition and would recreate the defect the plan exists to fix

**Evidence.** D8: the create hook uses a lane "only when `gwz` is on `PATH`
and the project root is a GWZ workspace (`gwz --root ROOT status` succeeds,
rather than testing for `gwz.conf/` by hand). Otherwise it reproduces Claude
Code's default". `gwz status` fails for many reasons that are not "this is
not a workspace": an open coordinated merge, an unreadable lock or marker, a
`gwz` older than the workspace's schema, a member path that resolves outside
the root. Each of those would silently produce a git worktree of the root
alone, with no members, and the session would proceed to fail at build time,
which is the opening paragraph's problem statement. The observed code for a
non-workspace is `ManifestNotFound` (exit 1); the agent skill separately names
`WorkspaceNotFound`. The plan names neither.

**Why it matters.** The fallback is the one branch of the hook that must
never be taken by mistake, and the plan makes it the default outcome of every
unexpected error.

**Required change.** D8 keys the fallback on the machine-readable error code
from `gwz --root ROOT --json status` (or a cheaper read-only command chosen in
S1.1), with the exact code set (`ManifestNotFound`, `WorkspaceNotFound`, and
whatever S1.1 pins by test) meaning "not a workspace, fall back". Any other
failure aborts creation with gwz's message on stderr. The S1.1 test suite
gains the case "gwz present, workspace present, status fails for another
reason: creation aborts, no worktree created".

### [P1 F2] D6's default refuses every lane on the dogfood machine

**Evidence.** D6 sets `GWZ_LANE_MIN_FREE_GB` to 40 by default. Section 1 of
the same plan records 31 GB free on the workspace volume. S1.3 runs the first
probe on that machine.

**Why it matters.** The first end-to-end run would fail at the guard, and the
failure would look like a hook bug rather than a policy mismatch.

**Required change.** Either derive the default from S3.2's measurement and
say so (the guard is a placeholder until then), or start with a floor sized
for the copy alone (single digits of GB on APFS) and let S3.2 raise it with
numbers. Also move a one-off measurement of lane creation time and the
`df` delta ahead of S1.3, because S1.3 also needs it for F13.

### [P1 F3] Lane accumulation is the dominant cost and is filed as an open item

**Evidence.** `LocalClones.md`: dispose "refuses unless the lane's history is
verifiably preserved", and a dirty lane is itself a hazard (`<dirty>` in the
worked example), waived only by `--force <hazard,...>` or detached by
`--keep`. The Claude docs: hook-created directories are never swept; a
subagent's worktree "with changes stays on disk"; a background session whose
removal fails "stays too". The desktop user has no terminal in the chip flow.
The plan's D3 calls the refusal the safe outcome (it is), S2.3 only "decides"
about subagents, and O2 says a periodic listing "is the interim answer".

**Why it matters.** Every chip or subagent that does any work mints a 65 GB
copy-on-write lane that persists until someone merges and disposes it from a
terminal. Ten chips in a week is ten lanes. On this machine two building lanes
exhaust the disk (section 1). The plan has no step that owns this.

**Required change.** Promote O2 to a Phase 3 step: an inventory-and-retire
procedure using `gwz local list`, a documented order (merge from the main
workspace, then dispose), and a rule for what a user does with a lane whose
session was deleted. S2.3 becomes a decision with a default written into the
plan now: subagent worktree isolation is not used in GWZ workspaces until
clean lanes exist (D7), and S4.1 says so. Consider, and record the answer to,
whether the hook can tell a subagent creation from a session creation from
the hook input (the docs show no field for it; if it cannot, the rule is
documentary only).

### [P1 F4] The copy always runs on a live source, which the lane documentation forbids

**Evidence.** `LocalClones.md`: "Keep the source quiet for the whole copy.
GWZ takes its family lock, which serializes family commands, but nothing
stops an editor, a build, or a raw `git` command from writing into the source
while it is being copied. Stop those before you start". The desktop docs:
clicking a chip starts a new session "and Claude continues your current
session uninterrupted". So the hook runs, by design, while the parent
session may be editing files or running cargo in the source tree.

**Why it matters.** A lane made during a build can carry half-written target
files; a lane made during an edit carries a torn file. Neither is corrupt
history (git writes objects atomically and `dest-complete` verifies the
stores), but the lane's build cache and working files may be inconsistent in
ways a session inside the lane cannot see.

**Required change.** State the policy in the plan and later in S4.1: a
Claude-created lane is a snapshot of the source as it sits, verified for git
objects only; build outputs and unsaved edits in flight are best effort. S1.3
adds a probe: create a lane while `cargo build` runs in the source and record
whether creation completes and whether the lane's target directory is usable
or must be rebuilt. Optionally the hook warns on stderr when a `cargo`
process holds the source's target directory open.

### [P2 F5] S1.2 ships unproven hooks to every clone of gwz-dev, and D5's "both placements" is a trap

**Evidence.** S1.2 commits the hooks into the root's `.claude/settings.json`
before S1.3 probes them. D5 says the same block "is also valid in
`~/.claude/settings.json`". The hooks docs: "All matching hooks run in
parallel. If you define the same handler in more than one settings file, it
runs once." The dedupe is by identical handler; the project copy references
`$CLAUDE_PROJECT_DIR/gwz-cli/...` and a user copy would reference a different
path, so both would run and both would print a path, and U4 already records
that the winner is unknown.

**Required change.** Resolve O1 now: S1.2 uses `settings.local.json` for the
probes and commits to `settings.json` only after S1.3. D5 states that the
hooks live in exactly one place per machine, and S4.1 says which to choose.

### [P2 F6] The committed settings depend on the gwz-cli member being present and current

**Evidence.** S1.2 references the scripts through
`$CLAUDE_PROJECT_DIR/gwz-cli/integrations/claude-code/`. A reader who cloned
with `--url-scheme https` has gwz-cli, but a workspace where the member is
detached, not yet materialized, or checked out at a commit before the scripts
exist gets a failing command, and WorktreeCreate aborts on any non-zero exit.

**Required change.** The root owns what its settings invoke: copies of the
scripts under `gwz-dev/.claude/hooks/`, refreshed from gwz-cli by a documented
step, or a settings command that checks the script exists and explains on
stderr when it does not.

### [P2 F7] Branch semantics are unaddressed

**Evidence.** Claude's default worktree is "on a new branch named
`worktree-<name>`"; the desktop's review and PR flows are branch based. A
lane copies every repository on the branch it sits on, `main` here, and
`gwz merge --remote NAME` integrates the lane's heads.

**Required change.** A decision D9: lanes keep the source branches, with the
docs stating that branch-based Claude views do not apply and integration is
`gwz merge --remote`; or the hook creates `worktree-<name>` in the root and
each member after the clone, with S1.3 checking that `gwz merge --remote`
then does what a user expects. The first is simpler and matches D4.

### [P2 F8] Windows is absent

**Evidence.** gwz ships Windows builds and the release verification runs on
Windows; Claude Code runs hooks on Windows through `powershell.exe` (hooks
docs). The plan's scripts are POSIX sh, and S1.1's Rust integration test
would execute them in gwz-cli's CI, which includes Windows.

**Required change.** Scope the plan to POSIX hosts explicitly and defer a
`.ps1` twin to a named later step, or add the twin to S1.1. Either way the
test module is unix-only behind an explicit boundary (`cfg_if!` or a platform
module), per the standing rule on conditional compilation.

### [P2 F9] `dispose --keep` and the exit-0-with-directory case are undefined

**Evidence.** `LocalClones.md`: "`dispose --keep` removes only the pointer
and the index row" and retains every file. The hooks docs define removal
failure as "a hook exits non-zero and the directory at `worktree_path` still
exists afterward". The plan's D3 says "nothing stronger" than dispose, and S3.1
asks what `--keep` does, but neither says whether the remove hook may use
`--keep`, nor what Claude does when the hook exits 0 and the directory
remains (an orphaned 65 GB tree that Claude has forgotten).

**Required change.** D3 states that the remove hook never passes `--keep` or
`--force`; S3.1 becomes a probe (new U8) of the exit-0-with-directory case so
the docs can say what happens.

### [P2 F10] The hook log cannot be switched on from the desktop app

**Evidence.** S1.1 logs to a file only when `GWZ_LANE_HOOK_LOG` is set. The
desktop docs: on macOS the app extracts `PATH` and a fixed set of variables
from the shell profile; "other variables you export there are not picked up".
S2.1 relies on the log to answer U1.

**Required change.** A fixed default log location (for example
`~/.gwz/claude-hooks.log`, or `<root>/.gwz/claude-hooks.log` when the root is
known), with the environment variable as an override only.

### [P2 F11] Existing git worktrees under the root are copied into every lane

**Evidence.** `gwz local clone` copies "dirt and build directories included".
Claude's default puts worktrees under `<root>/.claude/worktrees/`, and each
carries a `.git` file pointing at the source root's metadata. A lane made
from such a root contains those pointers, which reference another repository.

**Required change.** S1.3 probes lane creation with one plain worktree present
under the root and records what the lane contains and what Claude's identity
checks make of it. The docs advise keeping `.claude/worktrees/` empty in a
root that uses lanes; the hook may refuse or warn when it is not.

### [P2 F12] Sessions started inside a member are not considered

**Evidence.** `CLAUDE_PROJECT_DIR` "points at the project root where the
session started". A session started in `gwz-core` has that member as its
project; with user-level placement the hook would find no manifest there and
fall back to a git worktree of the member alone.

**Required change.** Decide whether the hook walks up from the project
directory to an enclosing workspace root and makes a lane of the whole
workspace (then the printed path is the lane's member directory, which S1.3
must check Claude accepts), or documents that member-rooted sessions get a
member worktree. The first is what a gwz-dev developer would expect.

### [P3 F13] The creation timeout was chosen before any measurement

1800 seconds may be short for a 65 GB copy that also verifies object stores,
or absurdly long. Measure once before S1.3 (see F2) and set it from the
number.

### [P3 F14] Command name

O2 says `gwz local ls`; the command is `gwz local list`.

### [P3 F15] `jq` is assumed

Present on this Mac at `/usr/bin/jq`, not guaranteed elsewhere. Either list
it as a prerequisite in S4.1 or have the scripts fall back to `python3` or
`sed` for the two fields they read.

### [P3 F16] No rollback and no failure story

The plan never says how to switch the hooks off quickly (delete the block, or
`worktree.bgIsolation` for background sessions) or what a user sees when
creation aborts from a chip. Both belong in S4.1.

### [P3 F17] Concurrency is documented but not stated

Two chips at once serialize on the family lock (`LocalClones.md`); the second
waits for the first copy. The plan should say so and size the timeout for two
copies, not one.

### [P3 F18] D2's parent-directory check cannot be answered by git on this machine

The empty `~/.git` file makes `git rev-parse` fail with "invalid gitfile
format" for any directory under `$HOME` outside a repository (recorded in O6
for Claude's side). The script's own check in D2 must treat that error as "no
enclosing repository", or use a directory walk for `.git` instead of asking
git.

---

## What the plan gets right

- Every quoted Claude Code behaviour in section 1 matches the fetched pages:
  the stdout-path contract, the abort-on-non-zero rule, the outside-any-
  repository rule, the removal semantics, `.worktreeinclude` being skipped,
  the sweep exemption, `CLAUDE_PROJECT_DIR`, the desktop `PATH` caveat.
- Dispose-only removal (D3) and no automatic merge-back (D4) are the correct
  safety posture; gwz's hazard model does the protecting.
- Probes precede documentation, and the unconfirmed list is honest about the
  desktop chip path (U1), which is the whole user-facing point.
- Verbatim lanes with measurement before a clean-lane feature (D7) keeps
  product work out of an integration plan.

## Summary table

| ID | Sev | Finding | Required change |
|---|---|---|---|
| F1 | P1 | D8 fallback fires on any `gwz status` failure | key on the not-a-workspace error codes; abort otherwise; test it |
| F2 | P1 | D6 default exceeds free space on the dogfood machine | derive from measurement; measure creation before S1.3 |
| F3 | P1 | lane accumulation unowned | promote O2 to a step; default: no subagent lanes until clean lanes |
| F4 | P1 | copy on a live source contradicts LocalClones.md | state snapshot policy; probe creation during a build |
| F5 | P2 | committed hooks before probing; dual placement trap | local settings for probes; one placement per machine |
| F6 | P2 | settings depend on the gwz-cli member | root-owned copies or existence check |
| F7 | P2 | branch semantics unaddressed | decision D9 |
| F8 | P2 | Windows absent | scope to POSIX or add `.ps1`; unix-only test boundary |
| F9 | P2 | `--keep` and exit-0-with-directory undefined | D3 forbids `--keep`/`--force` in hooks; S3.1 probes U8 |
| F10 | P2 | log needs an env var the desktop cannot pass | fixed default log path |
| F11 | P2 | `.claude/worktrees/` copied into lanes | probe; advise empty; hook warns |
| F12 | P2 | member-rooted sessions | walk up to the workspace, or document |
| F13 | P3 | timeout unmeasured | measure first |
| F14 | P3 | `gwz local ls` | `gwz local list` |
| F15 | P3 | `jq` assumed | prerequisite or fallback |
| F16 | P3 | no rollback or failure story | add to S4.1 |
| F17 | P3 | concurrency unstated | say it serializes; size timeout |
| F18 | P3 | parent check breaks on empty `~/.git` | treat the error as no repository |

## Next step

The author folds F1 to F12 into `GwzClaudeIntegrationPlan.md` (D3, D5, D6,
D8, a new D9, S1.1 to S1.3, S2.3, S3.1, a new Phase 3 step for F3, O1 and O2
resolved), records the fold in the plan's status line, and the operator
adopts or sends it round again.

---

## Fold record (2026-09-12, same day)

| ID | Sev | Disposition | Where it landed in `GwzClaudeIntegrationPlan.md` |
|---|---|---|---|
| F1 | P1 | **CURED** | D8 rewritten: candidate root by walk-up, final word from `gwz --json status`, fallback only on the pinned not-a-workspace codes (U11), abort on any other failure; S1.1 test cases for the unrelated-failure and each code |
| F2 | P1 | **CURED** | D6 default is a copy-only floor set from the new S1.0 measurement, capped at half the dogfood machine's free space; S3.2 raises it with build numbers |
| F3 | P1 | **CURED** | D3 forbids subagent worktree isolation in workspaces until clean lanes exist; S2.3 is that decision plus one probe (U9); new S3.4 owns inventory and retirement with `gwz local list` |
| F4 | P1 | **CURED** | D10 snapshot policy; busy-build warning; S1.3 probes creation during a running build |
| F5 | P2 | **CURED** | D5: local settings for Phases 1 and 2, one placement per machine; S1.2 is local adoption, new S1.4 commits after evidence |
| F6 | P2 | **CURED** | D1 and S1.2: root-owned copies under `gwz-dev/.claude/hooks/`, existence check in the settings command |
| F7 | P2 | **CURED** | New D9: lanes keep source branches; S1.3 records the desktop views; alternative held in reserve |
| F8 | P2 | **CURED** | New D11 (POSIX first, `cfg_if!` test boundary) and new Phase 5 / S5.1 for PowerShell twins |
| F9 | P2 | **CURED** | D3 forbids `--keep` and `--force` in hooks; S3.1 probes the exit-zero-with-directory case (U8) |
| F10 | P2 | **CURED** | D10: fixed log files, environment variable as override only |
| F11 | P2 | **CURED** | D2 warns on a non-empty `.claude/worktrees/`; S1.3 probes it; S4.1 advises |
| F12 | P2 | **CURED** | D8 walks up to the enclosing workspace; member-rooted sessions land in the lane's member directory; S1.1 test and S1.3 probe |
| F13 | P3 | **CURED** | S1.0 measures creation before S1.1 sets the timeout, sized for two serialized copies |
| F14 | P3 | **CURED** | `gwz local list` throughout |
| F15 | P3 | **CURED** | D11: `jq` or `python3`, scripts fall back between them; S4.1 lists prerequisites |
| F16 | P3 | **CURED** | S4.1: switch-off recipe and the failed-creation story from S2.1 |
| F17 | P3 | **CURED** | Section 1 states the family-lock serialization; S1.0 sizes the timeout for two copies |
| F18 | P3 | **CURED** | D2: the parent check walks for `.git` and treats "invalid gitfile format" as no repository |

No finding was rejected or deferred. Round 2 is not required: F7's D9 keeps
Phase 1's shape. The plan awaits the operator's adoption.
