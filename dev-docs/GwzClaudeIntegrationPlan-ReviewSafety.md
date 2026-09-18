# GwzClaudeIntegrationPlan.md — SAFETY-AXIS REVIEW

**Review object:** `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `e5325c03144af35186cb32859d5e44ee3ab154765bb0dc06493e5dd2ee6306b6`, 594 lines, uncommitted working-tree file (`gwz-cli` status ` M dev-docs/GwzClaudeIntegrationPlan.md`, the only modification). Verified identical at the start and the end of this review. Reviewed 2026-09-17.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7`; gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8`. Sources read read-only with Read/Grep and with `gwz --version` (1.0.13), `gwz local clone --help`, `gwz local dispose --help`, `gwz local list`, `git check-ignore -v .gwz/x`, `df -g .`. Nothing was written, built, or mutated.
**Date:** 2026-09-17
**Axis:** Safety: what the text permits to go wrong. Independent, adversarial, read-only. The other axis runs in parallel; nothing here relies on it. Filed verbatim by the lane owner.

**Verdict: NO-GO** — 1 P0, 0 P1 (see below: 1 P1, 0 P0), 6 P2, 2 P3. Correct counts: **0 P0, 1 P1, 6 P2, 2 P3.** I pre-commit to GO on a revision that resolves {P1-1, P2-1, P2-2, P2-3, P2-4, P2-5, P2-6} as specified; P3-1 and P3-2 are non-blocking.

---

## 0. Evidence base

Facts established by inspection, used by the findings below.

1. `gwz local clone --help` (1.0.13) confirms the destination guards the plan's reuse rule bypasses: a create "refuses, before anything is written, a nonempty destination, a destination that is already a workspace or lies inside a family member, and a name the family already holds" (`gwz-cli/docs/LocalClones.md:170-172`, and the help text's `PathCollision` example). It also confirms "An interrupted create leaves its files and a `creating` row for you to inspect; nothing later finishes or removes them on its own" (`LocalClones.md:166-168`).
2. `gwz local dispose --help` confirms the real refusal set: `open-merge`, `dirty`, `unpreserved-history`, each waived only by name with `--force`; `--keep` is the non-destructive alternative; "The workspace root is never disposed, and neither is the member you are standing in." `LocalClones.md:401-408` gives the second refusal's text: "… contains the working directory"; "Run `dispose` from another member."
3. The family index lives on the source root; a clone gets `.gwz/family-root` holding the family id and the root's path and reads the index from there (`LocalClones.md:75-78`). `.gwz/` itself is not copied into a lane (`LocalClones.md:127-128`). Confirmed on disk: `/Users/owebeeone/limbo/gwz-dev/.gwz/local-family.yml` (`members: {}`), and `gwz local list` shows only the root.
4. `git check-ignore -v .gwz/x` at the gwz-dev root resolves to `.git/info/exclude:4:/.gwz/` — the GWZ-managed member block, an untracked per-clone file, **not** a tracked `.gitignore` rule. `.claude/settings.json` *is* tracked (`git ls-files .claude`), and gwz-dev is public.
5. `df -g .` reports 37 GB available on the data volume against the plan's 65 GB workspace (section 1 quotes 31 GB at drafting). `LocalClones.md:104-110` confirms the copy is copy-on-write where the filesystem has one ("natively" vs "ordinarily"), i.e. a fresh lane's real cost is near zero.
6. gwz-dev `dev-docs/GwzLaneIssues.md` L1 confirms the disposal facts the plan rests on (19 lanes, every one refused, remedy = operator comparison then `--force dirty,unpreserved-history`), and L2 confirms that an untracked file in a receiving member blocks every lane merge.
7. gwz-core `dev-docs/GwzLaneCleanFixes.md` R0/R0.1 confirm that a one-command dispose of an integrated lane is unimplemented, and R8 plus its section 6 confirm that changed ignored data (agent state such as `.claude/`) still refuses even after R0.
8. Round-1 findings F1–F18 (`GwzClaudeIntegration-S0.1-Review.md`) were read to avoid re-filing closed ground. None of the findings below duplicates one: F3 (accumulation), F5/F6 (placement, version), F9 (`--keep`/exit-0), F11 (copied worktrees) are adjacent but each finding here has a different root cause, stated explicitly.

---

## 1. Findings

### [P1-1] The idempotent-reuse rule is keyed on directory existence, so it hands a session a directory that is not a complete, ready lane

**Location:** line 341 — "reuses an existing lane directory if present (idempotent, covers U4)"; supporting text at lines 105-107 (interrupted creates leave files and a `creating` row) and 111-112 (family commands serialise on the lock).

**Root cause.** Reuse is conditioned on the *presence of a directory* at the default destination, not on the family index holding a `ready` row for `NAME` whose recorded path is that directory.

**Violated invariant.** A path printed to Claude Code as a created worktree must be a complete, verified workspace (`dest-complete` passed, family row `ready`) or nothing must be printed at all. `gwz local clone` enforces this by refusing a nonempty destination, an existing workspace, a path inside a family member, and a held name (evidence 1); the reuse branch takes none of those checks, because it never calls clone.

**Reproduction (each step is permitted by the text).**
1. `claude --worktree fix` in gwz-dev. The create hook begins the copy of a 65 GB workspace.
2. The command hook exceeds its timeout (default 600 s; S1.0 only *sizes* one, line 333-334) or the session is interrupted. Per section 1 the interrupted create leaves its files and a `creating/incomplete` row, and nothing finishes or removes them.
3. `claude --worktree fix` again. The directory `../gwz-dev-fix` exists, so the hook reuses it and prints it as the last stdout line. A retry that tried to clone instead would be refused ("name `fix` already holds ../gwz-dev-fix"), so reuse is the only path the text leaves — and it is unguarded.
4. The session works, builds and runs `gwz add` / `gwz commit` in a half-copied workspace whose object stores were never connectivity-checked (`dest-complete` never ran). Commits made over a torn checkout record deletions and truncations as real changes.
5. The operator integrates with `gwz merge --remote fix` from the main workspace (D4/S3.4). The torn state is merged into gwz-dev.

A second, concurrent variant needs no crash at all: section 1 states two creations serialise on the family lock, but S1.1 places the reuse test *before* the clone, so a second creation with the same slug (trivially reachable — two `--worktree fix` invocations, or both handler placements firing in parallel, see P2-4) observes the directory the first is still writing and prints it immediately.

A third variant: any pre-existing sibling directory named `<root-dirname>-<slug>` that is not a lane at all (an unrelated checkout, a manual copy, another workspace) is reused and handed to the session; the "already a workspace" and "inside a family member" refusals never run.

**Impact.** A Claude session is given a workspace that is incomplete, concurrently being written, or not a lane, with no signal that anything is wrong. The merge-back step converts that into modifications of the real workspace. This is the only path in the plan by which Claude-originated state reaches the source workspace, and it has no integrity gate on it.

**Required correction.** Rewrite line 341 to: the hook reuses a destination only when the family index holds a `ready` row for `NAME` whose recorded path canonicalises to the destination and whose tree passes the same completeness check a fresh create reports (`dest-complete`); a `creating/incomplete` row, a directory with no row, or a row whose path differs must make the hook exit non-zero with a message naming the row's state and the S3.4 retirement procedure. State explicitly that the hook never prints a path it did not either create or verify.

**Closure test.** S1.1's test list gains: (a) a fixture with a `creating/incomplete` row plus its files — hook exits non-zero, nothing printed on stdout, message names the row; (b) a fixture with an unrelated non-lane directory at the default destination — same; (c) a fixture whose index row for `NAME` points elsewhere — same; (d) the existing idempotent-reuse test is tightened to assert the row was `ready` and the path matched.

---

### [P2-1] Reuse of a live lane hands two "isolated" sessions the same working tree, and each one's exit fires a removal on the other's work

**Location:** line 341 (the same sentence, different failure) with D8's remove classification at lines 277-281 and S1.1 at lines 347-352.

**Root cause.** Nothing in the text makes a lane exclusive to one session: there is no occupancy record, no lock the hook takes for the session's lifetime, and no rule that a name already bound to a running session is refused.

**Violated invariant.** Worktree isolation — the property the whole feature exists to provide — requires that the path handed to a session is used by that session alone for its lifetime.

**Reproduction.** Session A: `claude --worktree fix` in gwz-dev, lands in `../gwz-dev-fix`, edits and builds. Session B (another terminal, a background session, or a desktop chip whose auto-generated slug collides): `claude --worktree fix`. The directory exists and is a ready lane, so B is given the identical path. Both sessions now edit one tree, each believing it is isolated; Claude's in-worktree isolation checks pass for both. When A exits with removal, the remove hook runs `gwz local dispose fix` against a tree B is actively writing.

**Impact.** Concurrent clobbering edits and interleaved builds in one tree; a merge-back that carries both sessions' work under one lane name with no record that two authors touched it. The deletion itself is currently blocked by gwz (B's uncommitted work reports `dirty`, and under R0.1 unique work must still refuse), so this is a corruption-and-confusion defect rather than a loss defect — but the protection comes from gwz's refusals, not from the plan, and the plan's own S3.5 goal is to make an integrated lane dispose without any refusal.

**Required correction.** Require the create hook to record the requesting `session_id` against the lane (the hook already receives it; D10 already writes a log) and to refuse a reuse whose recorded session differs and is still live, with a message naming the other session. Alternatively, state as a decision that lane names are never reused across concurrent sessions and make the hook mint a disambiguated name, documenting the consequence for `gwz merge --remote`.

**Closure test.** A test that two creations with the same `name` and different `session_id` values, without an intervening removal, produce a refusal from the second (or two distinct lane paths), and that a removal naming a lane whose recorded session is not the caller's is refused.

---

### [P2-2] The remove hook's dispose invocation is unspecified: nothing binds the disposed name to `worktree_path`, and the standing-in refusal can make every removal fail

**Location:** lines 277-281 (D8's remove classification) and lines 347-352 ("disposed as `gwz local dispose NAME` would").

**Root cause.** The text names the command but not its operands: it never says how `NAME` is recovered from `worktree_path`, that the recovered row's path must equal `worktree_path`, or from which working directory / `--root` the dispose runs.

**Violated invariant.** An unattended removal must delete exactly the directory Claude named, or nothing at all.

**Reproduction / state sequence.**
- *Every removal fails.* Claude Code's documentation, as quoted in section 1, does not fix the hook's working directory (it only says relative paths resolve against "the hook's directory"). If the remove hook is invoked with, or leaves, a working directory inside `worktree_path` — the natural implementation for a hook that was just told a worktree path — `gwz local dispose` refuses with "… contains the working directory" (`LocalClones.md:401-408`). The hook exits non-zero; per section 1 the removal fails and the session is kept. This is indistinguishable, in the hook log and to the user, from the hazard refusal the plan *wants* (D3), so it will be read as expected behaviour and will survive S3.5: after R0 lands and integrated lanes should dispose in one step, they still will not, and the plan's acceptance criterion for S3.5 will fail for a reason nobody is looking for.
- *Wrong target.* Under the member-path case (D8 prints `<lane>/<member>`), `NAME` must be recovered by reading `<lane>/.gwz/family-root` and matching the row whose path is the lane. The text permits the cheaper implementation — strip the `<root-dirname>-` prefix from a basename — which is a string operation with no verification that the family row for that name points at `worktree_path`. gwz's own protections (root never disposed, name must exist in the index) contain the blast radius to *some registered lane of the same family*, but the plan supplies no invariant of its own here, and a removal that deletes a different lane of the same family is not excluded by anything in the text.

**Impact.** Either every unattended removal silently fails (lanes accumulate, the D3 "refusal is the safe outcome" story masks a bug), or a removal deletes a lane other than the one Claude named.

**Required correction.** Specify in D8/S1.1: the remove hook canonicalises `worktree_path`, resolves the owning workspace root, reads the family index, and disposes only when exactly one `ready` row's canonical path equals the resolved lane root; it invokes dispose with an explicit `--root` and a working directory outside the lane; anything else is refused with a distinct exit message. Require the log line to distinguish "refused by gwz hazard" from "refused by the hook's own classification".

**Closure test.** S1.1 tests: a removal whose process working directory is inside the lane still disposes (proving the chdir/`--root` discipline); a `worktree_path` that is a member directory disposes the correct lane and the sibling lane is untouched; a `worktree_path` whose basename matches a family name but whose canonical path does not is refused; the log distinguishes the two refusal classes.

---

### [P2-3] The settings writer has no atomicity or validity requirement, so a failed write can break every Claude session in the project

**Location:** lines 372-381 (S1.2: "merges the block into the chosen file …, preserving every other key, creating the file when absent").

**Root cause.** The step specifies the *content* transformation and says nothing about how the file is replaced or what happens to a file that does not parse.

**Violated invariant.** A settings file must be either the old valid document or the new valid document at every instant; a tool that edits another program's configuration must never leave it unparseable.

**Reproduction.** `gwz claude-code setup --project --write` is run while the volume is at the free-space floor D6 exists to detect (evidence 5: 37 GB free against a 65 GB workspace, and the plan's own 2026-09-11 disk-full incident). An in-place truncate-and-write fails part way; `.claude/settings.json` is left truncated. Every subsequent Claude Code session in that project fails to load settings. In gwz-dev that file is tracked (evidence 4), so the damaged file is also a dirty tracked change in the root — which, per L2, blocks lane merges until it is resolved. A second sequence needs no crash: the user's existing settings file contains a comment or a trailing comma (a form Claude Code's own loader tolerates in practice); a strict parse-and-reserialise either refuses or silently drops the tolerated syntax and reorders every key, which the "preserved byte-for-byte outside the block" test is written to forbid but the prose does not require.

**Impact.** A tool whose purpose is convenience can disable Claude Code for a repository, and in the committed-settings case (S1.4) can propagate a damaged file or an unexpected reserialisation to every clone.

**Required correction.** State in S1.2: the writer parses the existing file first and refuses (exit non-zero, nothing written, message naming the offending construct) if it does not parse; it writes to a temporary file in the same directory, fsyncs, re-parses the result, and renames over the original; it never rewrites bytes outside the inserted block. Say explicitly that `--write` is refused when the target is not a regular file (symlink, directory).

**Closure test.** S1.2's test list gains: a write interrupted before rename leaves the original bytes unchanged; a file that does not parse is refused with the original untouched; a file with an unknown top-level key and unusual formatting is byte-identical outside the inserted block after `--write`.

---

### [P2-4] D5's one-placement rule is unenforceable exactly where it matters, and `--command` defeats Claude's same-handler dedupe

**Location:** lines 190-198 (D1: fixed handler text so the dedupe holds; `--command PATH` for a pinned absolute binary), lines 236-242 (D5: exactly one settings file per machine, setup refuses a second placement), lines 413-420 (S1.4: commit the project block).

**Root cause.** The refusal is a check performed by `setup` at the moment it writes. The project block does not arrive on other machines via `setup`; it arrives via `git pull` / `gwz clone` of the committed root (S1.4). No check runs then, and the plan gives the hook itself no duplicate detection.

**Violated invariant.** The plan's own rule: the hooks live in exactly one settings file per machine, because two path-printing handlers both run and the winner is undefined (U4).

**Reproduction.** Machine B adopts at user level during Phase 1/2 as D1 recommends for the desktop app, using `--command /usr/local/bin/gwz` because the desktop app's `PATH` did not carry `gwz`. S1.4 later commits the project block with the bare `gwz` handler. Machine B pulls. The two handler strings differ, so Claude's "same handler runs once" dedupe (section 1, lines 73-75) does not apply; both `WorktreeCreate` hooks run, in parallel, for the same `name`. Both attempt a lane with the same destination — which is precisely the concurrent-reuse race of P1-1 — and Claude takes one of the two printed paths by an undefined rule. On removal both `WorktreeRemove` handlers run; the loser sees a path that no longer exists and, per D8's "anything else is refused", exits non-zero.

**Impact.** A defect that the plan deliberately designed against is reintroduced by the plan's own recommended adoption sequence, on exactly the machines the `--command` escape hatch exists for.

**Required correction.** Make the duplicate detectable at hook time, not only at setup time: the hook must detect that it is one of two path-printing handlers for the same event (for example by a marker file it writes for the `session_id` before doing work) and refuse the second with a message naming the two placements. Add to S1.4 the precondition that every machine's user-level block be removed before the project block is committed, and to S4.1 a stated migration for a machine that already pinned `--command`. State that a removal whose `worktree_path` no longer exists exits zero, not non-zero.

**Closure test.** A test that two distinct handler commands for the same event produce exactly one lane and one printed path, or an explicit refusal; and a test that removing an already-absent `worktree_path` exits zero.

---

### [P2-5] D6's free-space guard cannot bound the failure it cites, and measures the wrong volume

**Location:** lines 243-248 (D6: refuse when free space on the workspace volume is below `GWZ_LANE_MIN_FREE_GB`; "The default is a floor for the copy alone").

**Root cause.** The guard's threshold is sized against the cost of the copy, but on the filesystem the plan targets the copy costs almost nothing — section 1 (lines 132-136) and `LocalClones.md:104-110` both state the verbatim copy is a copy-on-write clone, cheap until the lane builds. A floor sized for a near-zero cost is a guard that never fires.

**Violated invariant.** A resource guard must be sized against the resource the protected operation actually consumes. The incident D6 names (2026-09-11, disk full) was caused by lane *builds*, not lane creation.

**Reproduction.** Free space 37 GB (measured). Ten lanes are created over an afternoon by chips and background sessions; each create moves free space by megabytes, so the guard stays green throughout and refuses nothing. Each lane then builds (the plan's own S1.3/S3.2 expect `cargo build -p gwz` and the gwz-core suite in a lane). Space is exhausted mid-build, with ten lanes on disk, none of which `gwz local dispose` will remove without `--force dirty,unpreserved-history` (evidence 6), and with the settings writer of P2-3 now operating on a full volume.

Second defect in the same sentence: "free space on the workspace volume" is not the quantity that matters. The lane is written to `../<root-dirname>-NAME`, the parent of the root, which is not guaranteed to be the root's volume (a symlinked root, a root mounted under a different filesystem from its parent). The guard must `statfs` the destination's parent.

**Impact.** The guard passes its own tests and provides no protection against the cited incident; the plan will read as having addressed round-1 F2/F13 when it has not.

**Required correction.** Restate D6: the guard's threshold is derived from the *post-creation* cost of a working lane (the S3.2 build numbers), not the copy; add a second bound the copy cost cannot defeat — a ceiling on concurrently registered lanes, or a refusal when the family already holds N lanes that `gwz local list` reports as `ready` — and state that free space is measured on the filesystem containing the destination's parent directory. Say explicitly that the guard's stderr message names the S3.4 retirement procedure.

**Closure test.** A test with a forced threshold proving the guard fires on an apparent-size basis (or on a lane-count basis) and not only on the bytes the copy writes; a test with a destination parent on a different filesystem from the root asserting which volume is measured.

---

### [P2-6] The fallback is specified as one `git worktree add` line but is claimed to reproduce Claude's default, which it does not

**Location:** line 28 ("so it can live in user-level settings without breaking other projects"), lines 263-266 (D8: "Claude Code's default is reproduced, `git worktree add -b worktree-NAME .claude/worktrees/NAME <base>`"), against the plan's own section 1 at lines 68-70 and U7 at lines 165-169.

**Root cause.** D8 specifies the fallback as a single command and then asserts parity with a default that section 1 says includes two behaviours that command does not have, and that omits any handling of a destination that already exists.

**Violated invariant.** A fallback offered for user-level installation must be no worse than the behaviour it replaces, in every project on the machine.

**Reproduction / state sequence.** A user adopts at user level (`gwz claude-code setup --user --write`), as D1 and D5 permit, and works in an unrelated non-gwz repository.
1. *Existing destination.* That repository already has `.claude/worktrees/review` and the branch `worktree-review` from Claude's own earlier creation. `git worktree add -b worktree-review .claude/worktrees/review <base>` fails on both counts; the hook exits non-zero; Claude aborts creation. Before the hook was installed, the same request worked. The idempotent-reuse rule of S1.1 is written for the lane branch only — the fallback branch has no reuse rule at all.
2. *Untracked includes.* `.worktreeinclude` is not processed when a hook replaces creation (section 1, lines 68-69), and D8 does not require the fallback to perform that copy. Every new worktree in every non-gwz project silently loses the untracked files that mechanism exists to carry (local env and config files), so builds in those worktrees fail for a reason the user cannot connect to gwz.
3. *No sweep.* A hook-created directory is never swept automatically (lines 69-70). Worktrees now accumulate in every project on the machine, with no gwz command that knows about them and no equivalent of S3.4 for them.

**Impact.** The Goal's load-bearing claim — a user-level installation that does not break other projects — is false as specified, and the breakage is silent and machine-wide.

**Required correction.** Either (a) extend D8's fallback contract to the full default: reuse an existing `.claude/worktrees/NAME` / `worktree-NAME` pair rather than failing, perform the `.worktreeinclude` copy, and state the sweep gap and its manual remedy in S4.1; or (b) withdraw the parity claim: state in the Goal and D8 that the fallback is a *reduced* reproduction, enumerate what it drops, and restrict the recommended placement to project settings of GWZ workspaces until S1.3 has measured the difference. Do not leave both the one-line specification and the parity claim standing.

**Closure test.** S1.1's plain-repository tests gain: creation with `.claude/worktrees/NAME` and `worktree-NAME` already present succeeds and prints the existing path; a repository with a `.worktreeinclude` listing an untracked file has that file present in the created worktree (or the test is explicitly marked as the documented divergence). S1.3's fallback probe records the sweep behaviour for a hook-created worktree.

---

### [P3-1] The hook log's contents, growth and ignore status are unspecified, and the workspace-root location can dirty the root

**Location:** lines 297-303 (D10: one line per decision to `<root>/.gwz/claude-hooks.log`, `~/.claude/gwz-lane-hooks.log` for the fallback, `GWZ_LANE_HOOK_LOG` overrides the location).

**Root cause.** The decision fixes a path and a cadence but not the record's fields, its retention, or the precondition that the path be ignored by the repository it sits in.

**Violated invariant.** A tool that writes into a workspace must not change what `git status` reports there, and a fixed log must have a stated retention bound.

**Consequences, concretely.** (a) `<root>/.gwz/` is ignored in gwz-dev only through the GWZ-managed block in `.git/info/exclude` (evidence 4) — a per-clone untracked file, not a tracked `.gitignore` rule. A root repository obtained without that block, or a `GWZ_LANE_HOOK_LOG` pointed anywhere else in the tree, makes every hook invocation create an untracked file in the receiving repository, which per L2 blocks every lane merge, and which `gwz local dispose` counts as a `dirty` hazard. (b) The hook input carries `session_id`, `cwd` and `transcript_path`; the text does not say which fields are logged, so an implementer may record `transcript_path` (a path into the user's transcript store) in a file inside a public repository's working tree. gwz-dev is public and `.claude/settings.json` there is already tracked. (c) No rotation or size bound is stated for a file written on every creation and every removal.

**Required correction.** State the log record's fields explicitly (event, timestamp, name, resolved classification, outcome, exit code — and say whether `session_id` is included and that `transcript_path` is not), give it a size bound with truncation behaviour, and require the hook to verify that the chosen location is ignored by the enclosing repository and to fall back to the user-level log with a stderr note when it is not.

**Closure test.** A test asserting the exact field set of a log line; a test that a `GWZ_LANE_HOOK_LOG` pointing at a tracked location is refused or redirected, and that `git status --porcelain` in the fixture workspace is unchanged after a creation and a removal.

---

### [P3-2] Aborted creations give the user no route back

**Location:** lines 243-248 (D6's "plain stderr message"), lines 267-272 (workspace-that-cannot-be-used aborts with gwz's message), lines 282-283 (every decision is logged).

**Root cause.** The plan requires the hook to fail loudly on stderr but never requires any failure message to name the recovery, and section 1 does not establish that the user sees hook stderr at all when creation aborts (S2.1 is scheduled to find out what a user sees when a chip's creation aborts — after the mechanism ships).

**Consequence.** The reachable failures are numerous — the free-space guard, an open coordinated merge, an unreadable lock, a manifest newer than the binary, a missing `gwz` on the desktop `PATH`, a name refused by GWZ, the P1-1 reuse refusals once they exist — and the user's whole experience of each is Claude aborting worktree creation. The retirement procedure that actually unblocks the free-space case lives in S3.4, and nothing routes the user to it.

**Required correction.** Add to S1.1 that every non-zero exit prints a single stderr line of the form "gwz: <cause>; <the one command that resolves it>", enumerate the causes and their remedies in a table in S4.1, and state in D10 that the same line is what the log records. Move S2.1's "record what a user sees when creation aborts from a chip" earlier, or add the equivalent observation to S1.3, so the message design is informed before S1.4 commits the block.

**Closure test.** A test asserting that each classified failure prints exactly one stderr line containing its named remedy and nothing on stdout.

---

## 2. Invariant analysis

What the plan gets right, and where the protection actually comes from — this matters because several of the findings above are cases where gwz, not the plan, is doing the protecting.

- **Nothing in the text lets an unattended removal reach the main workspace.** The remove hook's only destructive verbs are `gwz local dispose NAME` and `git worktree remove` without `--force`. `dispose` refuses the root and the member the caller stands in; `git worktree remove` refuses the main working tree and refuses a dirty worktree. A `worktree_path` naming the main workspace root, or a member inside it, therefore ends in a refusal. I could construct no sequence in which the text permits the main checkout or its members to be deleted. The caveat is that this rests entirely on gwz's and git's refusals: the plan asserts no invariant of its own about what the removal is allowed to touch (P2-2).
- **`--force` and `--keep` are correctly excluded from the hook** (D3, lines 217-221, and S1.1 line 350). The loss waiver stays with the operator. This is the single most important safety property in the document and it is stated twice, consistently.
- **The refusal-keeps-the-session contract is correct** and matches the documented behaviour in section 1: a non-zero exit with the directory still present means the removal fails and the session is kept. The plan chose the direction that loses nothing.
- **Credential handling is not a gap.** `gwz local clone` strips remote URLs that name filesystem paths or carry credentials (`LocalClones.md:117-119`). No finding on that axis.
- **The lane never carries GWZ runtime state.** `.gwz/` is not copied (evidence 3), so the family index cannot be forked by a lane, and a lane's gwz commands resolve back to the real family through `.gwz/family-root`. This closes the "a lane's copied index disposes the wrong family member" attack I looked for; it does not close P2-2, which is about the operands the hook passes.
- **The invariant the document is missing** is a single sentence it never states: *the create hook prints only a path it created and verified, and the remove hook deletes only the path it was given.* P1-1 and P2-2 are both instances of that absence. Adding it as a stated contract at the head of S1.1 would make the two corrections mechanical.
- **Deferred as instructed**, and deliberately not reported: the merits of D1–D11; whether R0–R19 are the right requirements; Windows detail (Phase 5); the object being uncommitted.

---

## 3. Risks and next action

**Residual risks the plan acknowledges but which remain live after the corrections above.**

1. *Retirement is a loss waiver, and adoption is not gated on R0.* Until GwzLaneCleanFixes R0 lands, the documented retirement of every Claude-created lane is the L1 remedy: an operator comparison of 112 entries followed by `--force dirty,unpreserved-history` (evidence 6). S1.4 commits the hook block to every clone of the workspace as soon as a gwz release contains S1.1 — not as soon as R0 lands. The plan therefore schedules routine, multi-user lane creation ahead of the ability to retire a lane safely. D3 restricts only subagents. I do not file this as a separate finding because round-1 F3 is folded into D3/S2.3/S3.4 and the plan states the situation honestly in section 1 and S3.4 — but the *ordering* is a decision the operator should take deliberately: keeping the block in `settings.local.json` until R0 is a one-line change to S1.4 and removes the whole class.
2. *O5 is load-bearing and unresolved.* Under R8, agent state a session changed inside its lane (`.claude/`, and the plan's own `<root>/.gwz/claude-hooks.log` pattern) is changed ignored data and still refuses. Claude sessions write such state by definition. If O5 resolves as "user work", R0 never delivers a one-command dispose for a Claude lane at all, and the plan's S3.5 milestone is unreachable. S3.5 records the answer; nothing forces the answer to be sought before the adoption in S1.4.
3. *U8 remains open at the point of first use.* The behaviour when the remove hook exits zero and the directory remains is undocumented and is probed in S3.1 — which runs after S1.3, i.e. after the hooks are live.

**Next action.** NO-GO. Revise the text for {P1-1, P2-1, P2-2, P2-3, P2-4, P2-5, P2-6}; each correction above is a bounded edit to an existing decision or step and adds no new phase. P3-1 and P3-2 should be folded in the same pass but do not block. I pre-commit to GO on a revision that resolves the seven blocking findings as specified.
