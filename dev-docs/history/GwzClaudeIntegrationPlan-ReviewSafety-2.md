# GwzClaudeIntegrationPlan.md — SAFETY-AXIS REVIEW (round 2)

**Review object:** `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `660b7b8120ecea1e1813d179f32f43e1edfbf29453473bdf115c03033e9b7526`, 751 lines, uncommitted working-tree file (`gwz-cli` status ` M dev-docs/GwzClaudeIntegrationPlan.md`, plus the three untracked review/remediation documents). Verified identical at the start and the end of this review. Reviewed 2026-09-17.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7`; gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8` — all unchanged from round 1. Sources: full re-read of the object; `GwzClaudeIntegrationPlan-RemPlan.md`; my round-1 report (`-ReviewSafety.md`). The other axis's report exists in the tree and was **not** read — this re-verdict is formed from my own evidence, as in round 1. Round-1 tool evidence (`gwz --version` 1.0.13, `gwz local clone --help`, `gwz local dispose --help`, `gwz local list`, `git check-ignore -v .gwz/x`, `df -g .`, `LocalClones.md`, `GwzLaneIssues.md`, `GwzLaneCleanFixes.md`) is carried forward; nothing in the tree moved, so it still holds. Read-only throughout.
**Date:** 2026-09-17
**Axis:** Safety: what the text permits to go wrong. Independent, adversarial, read-only. The other axis runs in parallel; nothing here relies on it. Filed verbatim by the lane owner.

**Verdict: NO-GO** — all nine prior findings verified closed; 2 new P2 and 1 new P3, all introduced by the remediation patch itself. I pre-commit to GO on a revision that resolves {P2-1, P2-2} as specified; P3-1 is non-blocking.

---

## Prior-finding closure table

| ID | Disposition claimed | Verified on corrected tree | Status |
| --- | --- | --- | --- |
| S-P1-1 | Reuse only on a `ready` row whose canonical path equals the destination, matching sidecar `session_id`, and a passing completeness check; every other state exits non-zero with nothing on stdout; invariant stated at the head of S1.1 | D6:285-295 states the three conjunctive conditions and the five named refusal states; D1:196-199 carries the contract ("prints only a path it created or verified … nothing else is ever printed on stdout"); S1.1:444-455 lists tests (a)-(d) by name. Re-traced my original sequence: the interrupted-create retry now meets a `creating/incomplete` row → refuse; the unrelated sibling directory → "a directory with no row" → refuse; the concurrent second copy → D5:265 places the reuse check behind the family lock, so the second either waits until the row is `ready` or sees `creating` → refuse. The torn-lane-to-merge-back chain is closed at its first link | **Verified — forbidden** |
| S-P2-1 | Create hook records `session_id` per lane in `<root>/.gwz/claude-lanes.yml`, never the family index; reuse by a different session refused, naming the other session | D6:287-294 exactly as claimed; S1.1:451-454 tests "a row recorded for another `session_id`". Two live sessions can no longer be handed one lane. The sidecar is explicitly kept out of the family index, so the index's semantics are untouched | **Verified — forbidden** |
| S-P2-2 | Remove hook canonicalises, resolves via `.gwz/family-root`, disposes only when exactly one `ready` row's canonical path equals the lane root, with explicit `--root` and a working directory at the family root; log distinguishes hook refusal from gwz hazard refusal | D8:332-346 states every element, including the reason ("dispose refuses the directory the caller stands in"); S1.1:456-459 tests a removal whose process working directory is inside the lane, a basename-matching path that canonically does not, and the sibling-lane-untouched case. Both branches of my sequence — every-removal-fails and wrong-target — are now excluded by text | **Verified — forbidden** |
| S-P2-3 | Parse first; refuse a non-parsing file or a non-regular file; temp file, fsync, re-parse, rename; bytes outside the block unchanged | D1:215-219 and S1.2:478-495, including the interrupted-write test ("original bytes unchanged") and the symlink refusal. The truncated-settings state is no longer reachable from a crash or a full volume | **Verified — forbidden** |
| S-P2-4 | D5 rewritten: identical handlers dedupe; differing handlers made harmless by session-keyed idempotency; absent removal path exits zero; `setup` warns; S1.4 precondition; U4 resolved | D5:256-273, U4:162-166, D8:341-343, S1.4:532-534 all as claimed. The undefined-winner problem is genuinely dissolved — both handlers print the same path. **But the harmlessness argument does not survive the guards**, see P2-2 below: this is a *bounded* closure of the original root cause with a new one introduced beside it | **Verified — bounded, with a new adjacent root cause (P2-2)** |
| S-P2-5 | D6: threshold from working-lane cost, `--max-lanes` ceiling on `ready` rows (default 8), free space on the destination parent's filesystem, message names S3.4 | D6:274-295 states all four, and opens by naming the reason the old guard was useless ("a floor sized for the copy would never fire"). The ten-lanes-then-build sequence now hits the lane ceiling at 8 with a refusal whose stderr names S3.4. The wrong-volume case is closed explicitly ("not the workspace's volume, which can differ under a symlinked or mounted root") | **Verified — bounded** |
| S-P2-6 | Fallback reuses an existing `.claude/worktrees/NAME` + `worktree-NAME` pair; performs the `.worktreeinclude` copy; sweep gap and remedy stated; parity claim qualified | D8:310-324 carries all three behaviours and qualifies the claim ("reproduced as far as a hook can"); S4.1:638-640 carries the sweep remedy; S1.1:461-466 tests the reuse pair and the `.worktreeinclude` entries. The "strictly worse than status quo" sequence is closed. Two residuals: the Goal:26-28 sentence is still unqualified (the qualification landed only in D8), and the new copy-on-reuse behaviour is under-specified — P2-1 below | **Verified — bounded, with a new adjacent root cause (P2-1)** |
| S-P3-1 | D10 lists the log fields (no `transcript_path`), a size bound, the ignored-location check with fallback to the user log | D10:365-382: exact field list, `never transcript_path or cwd`, 1 MB bound with truncation to the newest half, and the ignore check whose rationale cites L2 and the `dirty` hazard. Stronger than I asked for | **Verified — forbidden** |
| S-P3-2 | Every non-zero exit prints one stderr line `gwz: <cause>; <remedy>`; S4.1 carries the table; S1.3 records what the user sees | D10:373-375, S1.3:525-528 ("record exactly what the user sees, and whether the hook's stderr line reaches them"), S4.1:635-637 (table of every refusal with its one-line remedy) | **Verified — closed** |

Residual risk 1 (S1.4 ordering versus R0), deliberately not changed: S1.4:540-544 now states the consequence in the step itself ("this commits routine lane creation for every clone before GwzLaneCleanFixes R0 lands, while retirement is still the L1 waiver procedure"), names the one-line alternative, and assigns the choice to the operator at the step where it is taken. **Adequate for a plan.** A plan's job here is to put the trade-off in front of the person who owns it, at the moment of the decision, with the cheaper option named — it does all three. I would not raise a finding on it in any revision that keeps this wording.

---

## Changed-range analysis

What changed, against the RemPlan's dispositions:

- **Goal (26-28):** unchanged in substance. The parity qualification the RemPlan promised for S-P2-6 landed in D8, not here. Noted below.
- **Section 1:** lane count corrected to 28 (118-122 — recount from L1's register: 3 + 4+3+3+1+1+3+2+5+3 = 28, correct); the dead error-code bullet replaced by a new facts bullet (139-147) recording `PathCollision`, the dispose hazard set, the never-disposed root, the standing-in refusal, `.gwz/family-root`, and that `/.gwz/` is ignored only through the managed exclude block. Every one of those matches what I verified directly from `--help` output and `git check-ignore` in round 1. No drift.
- **U4 (162-166):** narrowed to the half that is still open. Correct.
- **D1 (192-226):** gains the stdout/deletion contract, knobs-as-options, and the atomic writer. Within disposition.
- **D5 (256-273):** rewritten from "exactly one placement, setup refuses" to "two placements permitted, made harmless". Within disposition — and the source of P2-2.
- **D6 (274-295):** rewritten to two guards plus the strict reuse rule and the sidecar. Within disposition.
- **D8 (302-349):** fallback gains reuse, the `.worktreeinclude` copy, and the sweep disclosure; remove hook gains canonicalisation, the single-matching-row rule, `--root`/cwd discipline, absent-path-exits-zero, and the two-class log. Within disposition — and the source of P2-1.
- **D10 (359-382):** log fields, bound, ignore check, one-line stderr. Within disposition.
- **S1.0, S1.1, S1.2, S1.3, S1.4, S2.1, S4.1, section 5, O3, section 7:** all consistent with the dispositions; S1.1's test list now carries every closure test I specified, verbatim in substance. S1.3 and S2.1 each gained "Every lane this step creates is retired through S3.4 before the step closes", which is more than was asked and closes a leak I had not filed.

**Nothing changed outside the dispositions.** I found no edit that is unexplained by the RemPlan.

**New root causes introduced by the patch** (all three are consequences of remediation text, not of the original plan):

1. **P2-1** — the fallback's new copy-on-reuse is unqualified, so a reused worktree can be overwritten with `.worktreeinclude` content. Created by the S-P2-6 disposition.
2. **P2-2** — D6's guards are evaluated before D6's reuse rule, so a second parallel handler can refuse and abort a creation the first already completed. Created by the S-P2-4 disposition (permit differing handlers) meeting the S-P2-5 disposition (a lane-count ceiling). This is the blind-convergence area both axes touched; the failure is in the *ordering* of two remedies, not in either remedy alone.
3. **P3-1** — the new sidecar file gets none of the ignore-location protections the same patch gave the log.

**None of these is an ARCHITECTURAL root cause.** I considered whether P2-2 is architectural — it arises from a decision (tolerate divergent handler configurations that each perform an irreversible create) rather than from an oversight. It is not: the ordering fix is one sentence in D6 and one in S1.1, it does not disturb D5's harmlessness argument, and it requires no new mechanism. The plan's structure — one binary, two hooks, a family-locked create, a sidecar for session ownership — survives all three findings intact.

---

## 0. Evidence base

1. The object at sha256 `660b7b81…`, read in full (751 lines). Line citations below are to that file.
2. `GwzClaudeIntegrationPlan-RemPlan.md`, read in full: eighteen dispositions, nine of them mine (S-prefixed), plus the explicit non-change of residual risk 1.
3. My round-1 report at `gwz-cli/dev-docs/GwzClaudeIntegrationPlan-ReviewSafety.md`, used only to re-trace my own state sequences against the new text.
4. Round-1 tool evidence, still valid on an unmoved tree: `gwz local clone --help` (the destination refusals: nonempty, already-a-workspace, inside-a-family-member, name-held; the `creating/incomplete` row); `gwz local dispose --help` (hazards `open-merge`, `dirty`, `unpreserved-history`; root never disposed; the standing-in refusal); `LocalClones.md:75-78,104-110,127-128,166-172,401-408`; `git check-ignore -v .gwz/x` → `.git/info/exclude:4:/.gwz/`; `df -g .` → 37 GB available; `GwzLaneIssues.md` L1 (28 lanes, all refused) and L2 (an untracked file in a receiving member blocks every lane merge); `GwzLaneCleanFixes.md` R0/R0.1/R8.
5. The other axis's report was deliberately not read.

---

## 1. Findings

### [P2-1] The fallback's new `.worktreeinclude` copy is not scoped to a fresh worktree, so reuse can overwrite untracked files that git cannot restore

**Location:** D8:313-318 — "when `.claude/worktrees/NAME` already exists and is a registered worktree of the project on branch `worktree-NAME` … it is reused and printed rather than failed; the `.worktreeinclude` copy that Claude skips when a hook owns creation (section 1) is performed by the hook"; S1.1:428-429 — "creates or reuses the git worktree under `.claude/worktrees/NAME`, performs the `.worktreeinclude` copy, and prints that path".

**Root cause.** Two behaviours new in this revision — reuse of an existing worktree, and the hook performing the include copy — are stated in one breath with no ordering or scoping between them. Neither sentence says the copy runs only on a freshly created worktree.

**Violated invariant.** A tool may create untracked files in a directory it just made; it must never overwrite untracked files in a directory that already holds a user's work, because git holds no copy to restore from.

**Reproduction (each step permitted by the text).**
1. A user adopts at user level (D1:212-214, D5:260-262 both bless this) and works in a non-gwz repository whose `.worktreeinclude` lists `.env`.
2. `claude --worktree api`. The hook creates `.claude/worktrees/api` and copies `.env` in. The session edits `.env` in the worktree — adding a local database URL, a test key, whatever the file is for — and the file is untracked by construction: that is why it is in `.worktreeinclude`.
3. The session ends without removal, or removal fails, or the user simply starts a second session on the same slug — all reachable; `git worktree remove` without `--force` refuses a worktree with untracked files, which is precisely this worktree, so step 3 is the *normal* outcome here, not an edge case.
4. `claude --worktree api` again. D8 reuses the directory, and then, by the same sentence, "the `.worktreeinclude` copy … is performed by the hook". An implementer building to this text copies the project root's `.env` over the session's edited one. The edited content existed only in that file.

**Impact.** Silent, unrecoverable loss of untracked work, in the one branch of the plan that is recommended for user-level installation and therefore runs in repositories that have nothing to do with GWZ. It is a small loss per occurrence and a certain one across a fleet. It is strictly worse than the status quo it was added to restore parity with: Claude's own copy runs at creation, not at re-entry.

**Required correction.** In D8 and S1.1, scope the copy: *the `.worktreeinclude` copy is performed only for a worktree this invocation created; on reuse no file is written, and a listed file missing from the reused worktree is reported on stderr, not supplied.* State the same for the lane branch's completeness check, so neither reuse path writes into a tree it did not make.

**Closure test.** Add to S1.1's plain-repository tests: a worktree created with a `.worktreeinclude`-listed file, whose content is then modified in the worktree, is reused on a second creation with that file byte-identical to the modified version; and a listed file deleted from the reused worktree is reported on stderr and not recreated.

---

### [P2-2] D6's guards are evaluated before D6's reuse rule, so a second parallel handler aborts a creation the first has already completed, leaving an orphan lane

**Location:** S1.1:422-426 — "applies the two guards and the reuse rule (D6, with the sidecar record of `session_id`)", in that order; D6:277-295, which states the guards first and the reuse rule second; the harmlessness claim that depends on the opposite order at D5:262-268 — "Differing handlers … both run, in parallel, for the same `name` and `session_id`; the plan makes that harmless rather than forbidden. The second creation to take the family lock finds the `ready` row and sidecar record the first made … and prints the same path". Section 1:44-46 supplies the consequence of any failure: "Any non-zero exit aborts creation."

**Root cause.** The reuse rule makes a second handler harmless only if the second handler reaches it. The guards sit in front of it, and the guards are per-handler state that the first handler's own success changes.

**Violated invariant.** D1's contract has a converse the text never states: a hook that has created a lane must not let a sibling invocation turn that create into an abort. More plainly — an irreversible create must not be performed behind a decision that a parallel copy of the same hook can still refuse.

**Reproduction, needing no misconfiguration.** The machine has the committed project block (bare `gwz`, per S1.4) and a user-level block pinned with `--command /usr/local/bin/gwz` — exactly the case D1:212-214 provides the option for and D5:262 blesses. Handler text differs, so Claude's dedupe does not apply and both run in parallel (section 1:73-75). The family currently holds 7 `ready` rows; `--max-lanes` is at its default 8 in both handlers.
1. Handler A takes the family lock, counts 7 < 8, creates the lane, writes the sidecar row, prints the path, exits zero. The family now holds 8 `ready` rows.
2. Handler B, waiting on the lock, acquires it and applies its guards *first*: 8 `ready` rows ≥ `--max-lanes` 8 → refuse, one stderr line, exit non-zero.
3. Claude aborts creation on B's non-zero exit.
4. The lane A made is on disk, registered `ready`, with a sidecar row naming a session that now has no worktree. No `WorktreeRemove` will ever fire for it — Claude believes nothing was created. Under gwz 1.0.12/1.0.13 it is undisposable without the L1 waiver (section 1:118-128), and it permanently occupies one of the eight slots, so the next creation hits the ceiling one attempt sooner. Repeat: each attempt at the boundary burns another slot until every worktree creation in the workspace refuses.

The same root cause fires away from the boundary whenever the two handlers carry different option *values* (`--min-free-gb`, `--max-lanes`), which D5:262-263 explicitly contemplates, and whenever a pinned `--command` names a stale binary without the `hook` family. It also fires with a single handler: a session re-entering its own existing lane after free space has dropped below `--min-free-gb` is refused by the guard instead of being handed the lane it already owns — the guard blocks a reuse that would consume nothing.

**Impact.** Orphan registered lanes that Claude does not know exist, created by the plan's own recommended two-placement configuration, retirable only through S3.4's waiver procedure; and a lane-ceiling that ratchets down toward total refusal. It falsifies D5's central claim ("the plan makes that harmless"), which is the claim the whole permit-two-placements decision rests on.

**Required correction.** Reorder, in both D6 and S1.1: *the reuse rule is evaluated first; a handler that finds a reusable row for its own `session_id` and `name` prints that path and exits zero without applying either guard, because it consumes nothing. The guards apply only to an invocation that will actually create.* Add to D5 one sentence naming what happens when a parallel handler refuses anyway (a stale `--command`, a genuinely different option set): the created lane is an orphan, S3.4 is its inventory, and `setup`'s differing-handler warning says so.

**Closure test.** Add to S1.1: with the family at `--max-lanes` minus one, two creations for the same `name` and `session_id` under differing handler text yield one lane, two zero exits and one path — not an abort; a second creation for the same session succeeds when free space is below `--min-free-gb` (reuse consumes nothing); and a guard refusal is asserted to occur only when no lane was created by this invocation.

---

### [P3-1] The new sidecar gets none of the ignore-location protection the same patch gave the log

**Location:** D6:287-289 — "the hook's sidecar (`<root>/.gwz/claude-lanes.yml`, never the family index) records the same `session_id` for that name"; against D10:376-382, which gives the log an ignored-location check, a fallback location, and a stated reason.

**Root cause.** The patch reasoned carefully about one file it writes into the workspace and not at all about the other, although the same facts apply to both — and the plan's own section 1:145-147 records the fact that makes it matter: in gwz-dev `/.gwz/` is ignored only through the GWZ-managed block in `.git/info/exclude`, not a tracked `.gitignore` rule.

**Consequence.** In a root repository that lacks that managed block, every creation writes an untracked file into the root, which per L2 blocks every lane merge and counts as a `dirty` hazard at disposal — the exact harm D10:380-382 spells out for the log. Unlike the log, the sidecar cannot be relocated to the user-level file: reuse correctness depends on it being found beside the family. A second, smaller consequence: because D6's reuse conditions are conjunctive, a `ready` lane whose sidecar row is missing (an upgrade from a gwz that predates the feature, a hand-made lane, `.gwz/` housekeeping) can never be reused *and* cannot be re-created, since the family already holds the name. That is fail-closed and therefore safe, but it is not in D6's enumerated message list, so the user meets an abort whose cause is absent from S4.1's table.

**Required correction.** Extend D10's ignore check to cover every file the hooks write inside the workspace, naming the sidecar; state what the hook does when the sidecar's location is not ignored (refuse creation with the L2 reason, rather than silently dirtying the workspace, since relocation is not available for this file). Add "a `ready` row with no sidecar record" to D6's enumerated refusal states with its remedy (retire the lane through S3.4, or choose another name).

**Closure test.** S1.1's `git status --porcelain` assertion is extended to a fixture whose root lacks the managed exclude block: creation is refused with the stated reason and the working tree is unchanged. One test for the missing-sidecar refusal message.

---

## 2. Invariant analysis

- **The contract the plan was missing in round 1 is now stated once and tested.** D1:196-199 — the create hook prints only a path it created or verified; the remove hook deletes only what `worktree_path` canonically names — is the sentence whose absence produced S-P1-1 and S-P2-2. It is placed where an implementer reads it before writing either subcommand, and S1.1:418-419 repeats it as the step's contract. This is the single most valuable change in the patch.
- **Destructive authority is still correctly minimal.** No `--force`, no `--keep`, no `rm`: the hooks' only destructive verbs remain `gwz local dispose` and `git worktree remove` without `--force`. The patch did not widen this, and D8:332-343 narrowed the target selection to one canonical-path-matching `ready` row. I could construct no sequence in the new text in which an unattended removal reaches the main workspace, a member of it, a sibling lane other than the named one, or any directory outside the family.
- **Protection is now the plan's, not only gwz's.** In round 1 several safe outcomes depended on gwz's own refusals firing by luck. The new D6/D8 text asserts its own preconditions ahead of those refusals, so gwz's refusals are a second line rather than the only one. That is the structural improvement I was looking for.
- **One asymmetry to watch, not a finding.** Lane reuse is gated on session ownership (D6:287-289); fallback worktree reuse (D8:313-316) is not. Two sessions can therefore share a fallback worktree. I do not file it: that matches Claude Code's own documented behaviour for its default worktrees (U4's remaining half), the blast radius is bounded by `git worktree remove` refusing a dirty or untracked-carrying worktree, and imposing gwz-specific ownership on a parity branch would itself be a parity break. It is worth one sentence in S4.1 when U4 is answered.
- **The guards' end state is honest.** `--max-lanes` at 8, with dispose refusing every verbatim lane until R0, means a workspace reaches a state where no worktree session can start until the operator retires lanes by hand. That is the right failure — a clean refusal naming S3.4 (D6:277-279) instead of the 2026-09-11 disk-full incident. P2-2 is the defect that makes this ceiling ratchet downward; with P2-2 fixed the ceiling behaves as designed.
- **Deferred as instructed**, and deliberately not reported: the merits of D1–D11; whether R0–R19 are right; Windows detail (Phase 5); the object being uncommitted.

---

## 3. Risks and next action

**Residual risks, none of them blocking.**

1. *The Goal's parity sentence was not qualified.* Goal:26-28 still reads "laid out the way Claude Code lays it out by default, so it can live in user-level settings without breaking other projects", while D8:319-322 now admits the fallback cannot reproduce the automatic sweep and that hook-created worktrees accumulate until removed by hand. The disclosure landed; the summary sentence did not follow it. An implementer builds from D8, so nothing is mis-built, and S4.1 carries the remedy — but the Goal now overstates what D8 delivers. A five-word qualification ("as far as a hook can") makes the document self-consistent.
2. *S1.4 ordering versus R0* — surfaced at S1.4:540-544 as the operator's choice, with the cheaper alternative named. Adequate, as judged above. It remains the largest live risk in the plan: committing the block puts routine lane creation in every clone while retirement is still `--force dirty,unpreserved-history` after a 112-entry manual comparison.
3. *O5 is still load-bearing.* If session-written `.claude/` state inside a lane counts as user work under R8, R0 never delivers a one-command dispose for a Claude lane and S3.5's milestone is unreachable. S3.5 records the answer; nothing forces it to be sought before S1.4.
4. *`.worktreeinclude` semantics are unspecified.* The hook must now interpret a format this plan never describes. The failure modes are bounded (a file missing, or an over-broad pattern copying a large ignored tree into `.claude/worktrees/NAME`), and S1.1 tests only the simple case. Worth one sentence in S1.1 naming the pattern semantics the hook implements.

**Next action.** NO-GO on sha256 `660b7b81…`. All nine round-1 findings are genuinely closed — the patch was thorough and in several places went further than the corrections asked for. Two new P2s and one P3 were introduced by the remediation: P2-1 is a two-sentence scoping fix in D8/S1.1, P2-2 is a reordering of two clauses already present in D6 and S1.1 plus one sentence in D5, and P3-1 extends an ignore check the patch already wrote for a neighbouring file. None requires a new mechanism, a new step, or a changed decision. I pre-commit to GO on a revision that resolves {P2-1, P2-2} as specified.
