# GwzClaudeIntegrationPlan.md — CONSISTENCY-AXIS RE-VERDICT, ROUND 5

**Review object:** WORKING-TREE `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `d5356fec0330e3c718166a9716887588b684e8145ece5ba87179c6ac40c347db`, 933 lines.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7`; gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8`; controlling working-tree `gwz-core/dev-docs/GwzLaneCleanFixes.md` sha256 `3026e35199a8df41e2a40109d27544a90fefcbfad753b267043362208bb503b1`, 192 lines. Tuple verified identical at review start and at review end.
**Date:** 2026-09-17 **Axis:** CONSISTENCY — the document against its controlling graph.

**Verdict: NO-GO on sha256 `d5356fec…`** — 0 P0, 0 P1, **3 P2**, 3 P3. **All seven round-4 findings (C-P2-1..3, C-P3-1..4) are closed and verified against my own counterexamples.** **No new architectural root cause** — the hook-owned attempt loop is realisable against `local_clone/create.rs` as written (see §2). All three new P2s are text-level contradictions the patch left behind in the ranges it touched. **I pre-commit to GO on a revision that resolves {N-P2-1, N-P2-2, N-P2-3} as specified**; the three P3s are non-blocking.

---

## Prior-finding closure table

| ID | Round-4 finding | Cure located | Counterexample re-traced | Status |
| --- | --- | --- | --- | --- |
| C-P2-1 | Wait deadline had no default; D1's bare handler carried no `--wait-secs`, and D6's "sized from S1.0" was unsourced | D6 `:329-331` "the `--wait-secs` option (**compiled-in default 300 s**…)"; D6 `:324-326` "Both are hook options baked into the handler by `setup` (D1), **with compiled-in defaults, so the bare handler is guarded too**" | Re-read D1`:235` (bare handler) → D6`:330` (300 s default) → S1.4`:665` (`setup --project --write`). The committed bare block now waits 300 s; D5's parallel resolution no longer collapses into an immediate `Busy`. | **CLOSED** (residual S1.0 mismatch filed separately as N-P2-1 — a different root cause, not a reopening) |
| C-P2-2 | Remove path had no wait and no `Busy` refusal class | D3 `:268-271` (fourth outcome, "family busy; retry", "never as a hazard"); D8 `:432-434` (`--wait <wait-secs>` (R21, the same deadline as the create hook)); D8 `:438-441` (three log classes); D8 `:436-438` (row gone after classification → exit zero); S1.1 `:580-584`; S4.1 `:771` | Re-ran my two-parallel-remove-handler trace: B now waits 300 s for A's lock; if A finished and removed the row, B's dispose finds the row gone and exits zero; if the lock is genuinely held past the deadline, the outcome is printed and logged as busy, not as a hazard, so nobody is routed to S3.4. | **CLOSED** (D10/S1.1 vocabulary not carried along — filed as N-P2-2) |
| C-P2-3 | R20/R21's gwz-cli CLI-surface impact unowned by either document | GwzLaneCleanFixes `:145-153` scope note; plan `:149-153`; section 5 `:813-819` | Verified each named artefact against my own evidence: `src/clirequest/local.rs` (clap args — confirmed at `:21-52`), `docs/CLI.md` generated and byte-compared (confirmed `src/tests/g00.rs:35-40`) and re-checked by the release gate, `src/local_long.rs` / `docs/commands/local.md`, the `local list` sample (confirmed `docs/LocalClones.md:191-201`), `local_family_members` (confirmed `docs/MachineOutput.md:558-561`). All six correct; ownership assigned ("land in gwz-cli, in the same lane as the gwz-core change, under gwz-cli's review loop"). | **CLOSED** |
| C-P3-1 | S1.1 omitted D6's "`ready` row, this session, incomplete" | S1.1 `:561-562` "a `ready` row owned by this session whose lane fails the completeness check" | Re-enumerated D6's table against S1.1's list: all eleven rows now have a test (mapping in §2). | **CLOSED** |
| C-P3-2 | D6's table omitted `MemberState::Disposing` | D6 `:358` "a `disposing` row: refuse; retire it through S3.4"; S1.1 `:563` | All three `family-model/src/lib.rs:207-211` variants now named in D6. | **CLOSED** |
| C-P3-3 | Status line and section 1 gated all of Phase 1 on R20/R21, contradicting section 5 | Status `:12-13` "S1.0 now, S1.1 onward once…"; section 1 `:152-153` "S1.1 onward needs R20 and R21 in the installed gwz; S1.0 does not" | No Phase-1-wide gate remains; consistent with the sketch `:804` and prose `:814-817`. | **CLOSED** |
| C-P3-4 | `session_id` passed unvalidated into R20's constrained `--owner` grammar | Section 1 `:43-45` (UUID in every observed payload, to be confirmed in S1.3); D6 `:340-342` (hook validates against R20's grammar, refuses with its own message); S1.1 `:563` | The assumption is now recorded as a fact with a confirming step, and the hook's own refusal replaces an opaque gwz abort. | **CLOSED** |

---

## Changed-range analysis

I attacked each range the coordinator named, plus the design change RemPlan-3 records.

**The design change (create hook runs its own attempt loop; clone called without `--wait`) is sound and is NOT a new architectural root cause.** I verified it against `gwz-core/src/local_clone/create.rs:1-31`:
- Looping on `Busy` is safe and residue-free: the lock is taken at **step 5**, and the header states "A refusal before reservation leaves nothing; a family this invocation founded and did not use is un-founded again (its empty index removed) so that stays true." A `Busy` at step 5 is before reservation, so a failed attempt writes nothing and the loop is idempotent.
- Serialization holds even on a family-less workspace: the lock file `.gwz/local-family.lock` is created by `try_lock` independently of whether an index exists (`family-store-contract/src/lib.rs:279-282`, `read_view` observes "without writing anything, **including the lock file**"), so two handlers founding a family still serialize.
- D6's lock-free per-attempt read is exactly `read_view`, the path `local list` uses.
- The timeout-kill residue D6 `:373-375` describes matches create.rs verbatim: "A failure after reservation leaves the `creating` row with a diagnostic and the destination as it stands, for inspection; nothing is cleaned up, resumed or promoted."
- The design removes creation's dependency on R21 entirely, which is a simplification, not a hazard.

**Status line, section 1, U4, D3, D8 (remove path), S3.4, section 5 prose:** correct and mutually consistent; see the closure table.
**Section 1 R20/R21 paragraph vs the new R20:** `:145-149` restates the v2 schema contract accurately against GwzLaneCleanFixes `:122-130` (schema `gwz.local-family/v2`, `owner` optional, older gwz refuses the whole index naming the minimum version, every binary on the workspace at or above the R20 release). R22's second test (`:141-143`, older-reader case) matches.
**D8 fallback reason:** `:400-405` cites section 1's `git worktree lock` fact accurately (`:85-88`) — but assigns S1.3 a recording task S1.3 does not carry (N-P3-2).
**D10:** the "every file a create writes" addition (`:472-479`) is verbatim-correct against create.rs step 5 (the lock, the index, the regenerated managed `.git/info/exclude` block). But D10's log-line field set was not carried forward for D8's third class (N-P2-2).
**S1.1:** three test-list additions verified present; the full D6-table correspondence is now complete. Two defects introduced: the different-name concurrency test asserts more than D6 guarantees (N-P2-3), and the field-set assertion still says "two refusal classes" (part of N-P2-2).
**S1.0:** **not in the coordinator's changed-range list, and that is the defect** — the attempt-loop design invalidated S1.0's timeout formula and D6 now names S1.0 as the source of a value S1.0 does not produce (N-P2-1).

---

## 0. Evidence base

Read in full: the revised object (933 lines); the revised `GwzLaneCleanFixes.md` §3.6 (`:107-153`) and §§4–6; `GwzClaudeIntegrationPlan-RemPlan-3.md` (41 lines, all 15 dispositions). Carried forward from round 4 (re-verified where load-bearing): `gwz-dev/dev-docs/GwzLaneIssues.md` L1; `GwzLaneDisposalAudit-2026-09-10.md`.

Code re-verified read-only this round: `gwz-core/src/local_clone/create.rs:1-31` (step order; step 4 source inventory/snapshot **before** the family lock, step 5 the lock, step 6 `install`; the pre/post-reservation residue rules); `gwz-core/crates/family-store-contract/src/lib.rs:276-295`; `gwz-core/crates/family-model/src/lib.rs:207-211`, `:259-271`, `:53`; `gwz-core/src/filesystem/native/platform_lock.rs:10-12`; `gwz-core/crates/workspace-install/src/admit.rs:16-71`, `:150-168`; `gwz-cli/src/clirequest/local.rs:21-52`; `gwz-cli/src/tests/g00.rs:35-40`; `gwz-cli/docs/LocalClones.md:191-201`; `gwz-cli/docs/MachineOutput.md:558-561`.

Remnant sweep on the revised text (`grep -n -i "sidecar|two copies back to back|two refusal classes|refused)|--wait"`): `sidecar` survives only in trail entries `:880`, `:900`, `:911`, the last of which negates it, and D6 `:328-329` now states "The hook takes no lock of its own **and keeps no state of its own**". Three live stale strings remain and are filed below: `:516`, `:467`, `:596`.

---

## 1. Findings

### [N-P2-1] S1.0 was not updated for the attempt-loop design: it states a Claude-timeout formula that contradicts D6's, and does not produce the `--wait-secs` value D6 says it produces

- **Root cause.** The patch rewrote D6 around the hook's own wait but left S1.0 — the step D6 names as the source of both the timeout and the revised `--wait-secs` — on its pre-attempt-loop text.
- **Location.** D6, `GwzClaudeIntegrationPlan.md:330-331` ("compiled-in default 300 s; **S1.0 revises it and sets Claude's create timeout to one wait plus one copy plus 60 s**"). Against S1.0, `:514-517` ("The numbers set D6's interim `--min-free-gb` default and the creation timeout in S1.2's settings block (**sized for two copies back to back, since two creations serialize on the family lock**)") — byte-identical to the round-4 text I reviewed. S1.2, `:605-606` ("the create timeout from S1.0"). RemPlan-3 row C-P2-1 called for exactly this ("S1.0 revises it and sets the Claude timeout to wait + one copy + 60 s"); the edit landed in D6 only.
- **Violated invariant.** One value, one definition: the create timeout S1.2 bakes into the settings block has a single formula.
- **Reproduction.** Read D6`:330-331` → timeout = 300 s + copy + 60 s. Read S1.0`:514-517` → timeout = 2 × copy. Read S1.2`:605-606` → S1.2 takes it "from S1.0". Then note that S1.0's stated outputs are `--min-free-gb` and the timeout; nothing in S1.0 mentions `--wait-secs`, which D6 says S1.0 revises. S1.0's parenthetical rationale ("since two creations serialize on the family lock") is the pre-redesign justification for a two-copy budget and no longer describes what a waiting handler does.
- **Impact.** Concrete and quantifiable. With a measured copy of 60 s, S1.0's formula gives 120 s and D6's gives 420 s. An implementer following S1.0 — as S1.2 instructs — bakes 120 s into the handler while the hook's compiled-in deadline is 300 s. Claude then kills the hook mid-wait or mid-copy, which by D6`:373-375` "leaves a `creating` row and its files and writes no log line", and the next attempt for that name refuses fail-closed and must be retired through S3.4. The wrong formula manufactures precisely the residue D6 warns about, on every contended creation, in the configuration S1.4 commits.
- **Required correction.** Rewrite S1.0's output sentence to the attempt-loop design: the measurement sets `--min-free-gb`, revises `--wait-secs`, and sets Claude's create timeout to one wait plus one copy plus 60 s; delete "sized for two copies back to back, since two creations serialize on the family lock".
- **Closure test.** S1.0 and D6 state the same timeout formula; S1.0 names `--wait-secs` among the values it revises; S1.2's "from S1.0" resolves to one definition.

### [N-P2-2] D10's log-line field set is closed at one refusal value and S1.1 asserts two, while D8 now requires three classes — the "family busy" cure has no field to live in

- **Root cause.** D8's log-classification sentence was extended from two classes to three; D10, which defines the log line, and S1.1's field-set assertion, were not.
- **Location.** D8, `:438-441` ("the log line distinguishes **three classes**: a refusal by the hook's own classification, a hazard refusal by gwz, and the family lock still busy at the deadline (D3)"). Against D10, `:465-468` ("A log line carries **exactly**: a timestamp, the event, `name`, `session_id`, the resolved classification (lane, member-in-lane, fallback worktree, **refused**), the path printed or acted on, the outcome, and the exit code"). Against S1.1, `:596` ("the log line's exact field set, **the two refusal classes** distinguished").
- **Violated invariant.** D10 is the single decision that fixes the log line's fields, and it says "exactly"; every other section's logging claim must be satisfiable within it.
- **Reproduction.** Read D8`:439-441` → three distinguishable classes required. Read D10`:465-468` → the classification field has four enumerated values, exactly one of which covers refusal, and the list is closed by the word "exactly". Read S1.1`:596` → the test that pins the field set asserts two refusal classes. Then read S1.1`:580-582`, which asserts a removal is "classified busy" — an assertion the field-set test as specified would contradict.
- **Impact.** Diagnosability, and it lands on the cure for C-P2-2. D3`:268-271` promises that a busy removal "is logged and printed as 'family busy; retry' and never as a hazard, so nobody is sent to S3.4 for a lane that needed a minute". That promise is delivered by the log class. An implementer obeying D10 literally logs `refused` for all three, and the two S1.1 assertions cannot both pass. The operator reading `<root>/.gwz/claude-hooks.log` after a contended removal cannot tell a transient lock from a real hazard — the exact discrimination D8`:441-443` says the log exists to provide.
- **Required correction.** Extend D10's classification vocabulary to carry the three refusal classes D8 names (or add a distinct class field), and change S1.1`:596` from "the two refusal classes" to the three D8 requires, so the field-set test and the busy-removal test agree.
- **Closure test.** D10, D8 and S1.1 name the same number of refusal classes; S1.1's field-set assertion admits the value its busy-removal test asserts.

### [N-P2-3] S1.1's different-name concurrency test asserts an outcome D6 explicitly concedes the design does not guarantee, and lacks the sequencing stipulation its sibling test carries

- **Root cause.** D6 states a residual race in the `--max-lanes` ceiling and accepts it; the test written to close the same finding asserts the ceiling is never exceeded.
- **Location.** D6, `:338-340` ("The guards are therefore fresh at every attempt; **the residual bound, stated here and accepted, is that two attempts admitted in the same instant can exceed `--max-lanes` by one**"). Against S1.1, `:570-572` ("two concurrent creations of different names with the family at `--max-lanes` minus one (**exactly one new lane**, the second refused by the ceiling after its wait, nothing copied for it, logged)"). Contrast the sibling test one clause earlier, `:565-567`, which *does* stipulate its ordering: "two creations with the same `name` and `session_id` **where the second starts only after the first's `creating` row is visible in the index**".
- **Violated invariant.** A plan's test section must be satisfiable as written, and must not assert a property the decision it tests disclaims.
- **Reproduction.** Set the family at `--max-lanes` minus one, so there is room for exactly one lane. Start two creates of different names truly concurrently, as the test says. Per D6's loop, each attempt does a lock-free index read, then the guards, then the clone. If both read the index before either reserves — the instant D6`:339-340` names — both pass the ceiling guard and both proceed; the clones serialize on the family lock but both reserve, giving **two** new lanes and a ceiling exceeded by one. The test's "exactly one new lane" then fails. Nothing in the test forces the second handler's first attempt to be `Busy`, which is what "refused by the ceiling **after its wait**" silently presumes.
- **Impact.** The test is not deterministically constructible. Either it lands flaky in gwz-cli's suite — where `src/tests/g00.rs`-style assertions are expected to be exact — or an implementer "fixes" the flake by adding mutual exclusion around the guard that the design does not have and D6 explicitly declined to require. Both outcomes are worse than the honest bound D6 states.
- **Required correction.** Give the different-name test the same sequencing stipulation as its sibling — the second create starts only after the first's `creating` row is visible in the index — so the ceiling refusal is deterministic; and state in S1.1 that the same-instant race D6 concedes is out of the test's scope, so the two sections agree.
- **Closure test.** S1.1's different-name test states the ordering that makes its assertion deterministic, and no S1.1 assertion contradicts D6's stated residual bound.

### [N-P3-1] D5 and U4 still attribute the parallel-creation resolution to R21's `--wait`, which the create path no longer uses

- **Root cause.** The design change moved creation off `--wait` onto the hook's own loop; D5's and U4's framing sentences were half-updated.
- **Location.** D5, `:294-300` ("the plan makes that harmless rather than forbidden, through **two** gwz-core facilities this plan requires (GwzLaneCleanFixes R20 and R21…): the owner token … **and the `--wait` that retries a `Busy` lock until a deadline**. The second handler sees the first's `creating` row owned by its own session and **keeps polling (D6's attempt loop)**…"). U4, `:188-192` ("gwz records the owner in the same index write as the row (R20) and the second handler keeps polling while the first's `creating` row is its own session's (D6's attempt loop); the resolution is conditional on **those two requirements**"). Against D6, `:333-335` ("runs `local clone` in process with `--owner <session_id>` (R20) and **no wait**").
- **Violated invariant.** A decision's stated dependencies must be the ones its mechanism actually consumes.
- **Reproduction.** Read D5`:295-297` (two facilities, one of them `--wait`), then D5`:298-300` (the resolution, which is the hook's loop), then D6`:334` ("and no wait"). The create resolution now depends on R20 alone.
- **Impact.** Bounded but concrete: a reader concludes the parallel-creation resolution is gated on R21 and will validate it through R22's concurrent-create test (`GwzLaneCleanFixes.md:138-141`), which exercises a `--wait`-driven mechanism the hook does not use. It also obscures which requirement actually gates S1.1's create half. (The S1.1 gate on "R20 and R21" in the sketch remains correct, because S1.1 also implements the remove hook, which does use R21 — that part needs no change.)
- **Required correction.** In D5 and U4, attribute the creation resolution to R20 plus D6's attempt loop, and name R21 as the remove hook's dependency where it belongs.
- **Closure test.** No sentence describing the parallel-*creation* resolution cites `--wait`.

### [N-P3-2] D8 cites S1.3 for an observation S1.3 does not collect (the `git worktree lock` on a fallback worktree)

- **Root cause.** RemPlan-3's S-P3-3 disposition had two halves; only the D8 and S1.1 halves landed.
- **Location.** D8, `:401-405` ("it also refuses a locked worktree, which is what Claude holds on a running agent's worktree (section 1; **S1.3 records that the lock is present for a fallback worktree, U10**)"). Against S1.3's fallback probe, `:648-654`, which records only that the session "lands in `.claude/worktrees/probe-plain` and is removed on exit (U7)" and what Claude reports when `gwz` is hidden from `PATH`. U10 is assigned to S2.2 (`:694`), for a lane, not a fallback worktree. RemPlan-3 row S-P3-3: "S1.3 records it (U10)".
- **Violated invariant.** Every forward pointer to an evidence step must name work that step actually performs.
- **Reproduction.** Read D8`:402-403`, then search S1.3 (`:625-661`) for `U10`, `lock`, or `git worktree lock`: none occurs.
- **Impact.** D8's acceptance of two sessions sharing one fallback worktree now rests on the lock being held there, and no step collects that evidence. S4.1`:773-775` will then document the shared-worktree caveat citing S1.3 observations that were never made.
- **Required correction.** Add to S1.3's fallback probe: record whether `git worktree lock` is held on the hook-created fallback worktree while a session runs in it, and whether `git worktree remove` refuses it (U10 for the fallback case).
- **Closure test.** S1.3's text contains the observation D8 attributes to it.

### [N-P3-3] D6's "sleeps briefly and loops" understates the per-attempt cost, because the clone does the full source inventory before it reaches the lock

- **Root cause.** The attempt loop's poll period is described as a sleep, but one class of attempt runs the clone's entire pre-lock work before discovering `Busy`.
- **Location.** D6, `:332-335` ("each attempt re-reads the family index lock-free …, evaluates the table below, re-evaluates both guards, and only then runs `local clone` in process … ; **a `Busy` result sleeps briefly and loops**"). Against `gwz-core/src/local_clone/create.rs:14-16`, step 4: "the source inventory and snapshot, **before the family lock**, so a design §4.0 hazard refuses with nothing written at all"; step 5 is the lock.
- **Violated invariant.** A stated cost must match the code path it describes.
- **Reproduction.** Read create.rs's step order. Then take the case D6's table sends to the clone: no row for `NAME`, but the family lock held by an unrelated command (a `merge`, a `dispose`, or a clone of a different name — nothing that puts a row under this name into the index). The lock-free read finds nothing, the guards pass, the clone runs, does the step-4 inventory and snapshot of the 65 GB source (section 1`:154`), then hits `Busy` at step 5. The hook sleeps and repeats — inventory included. Note that the D5 parallel-handler case is *not* affected: there the second handler sees the first's `creating` row on the lock-free read and loops without calling the clone (`:353-355`), which is correct and efficient.
- **Impact.** Bounded but real: against an unrelated family command the effective poll period is the pre-lock inventory, not a brief sleep, so a 300 s deadline buys far fewer attempts than the wording implies, and the waiting handler competes for I/O with the work it is waiting on. Nothing in S1.0 or S3.2 measures the pre-lock inventory, so the adequacy of the 300 s default is unmeasured for this case.
- **Required correction.** State in D6 that an attempt reaching the clone costs the clone's pre-lock inventory before it can learn the lock is busy (citing create.rs's step order), and add that inventory time to what S1.0 measures, so `--wait-secs` is sized against it.
- **Closure test.** D6 states the per-attempt cost for the clone-reaching case; S1.0 measures it.

---

## 2. Invariant analysis

**The mechanism remains realisable, and I found no architectural defect.** I re-attacked the new attempt loop specifically, because a design change on a re-verdict is where a fresh root cause would hide:

- **Loop safety.** Every `Busy` occurs at create.rs step 5, before reservation, and the header guarantees "A refusal before reservation leaves nothing" and un-founds a family the invocation founded but did not use. Repeated attempts therefore leave no accumulating residue — the property the loop's correctness depends on, and it holds.
- **No lock nesting.** D6`:328` "The hook takes no lock of its own and keeps no state of its own." The round-3 architectural finding A-P2-1 stays cured under the new design, and the round-2 sidecar is gone from all live text.
- **Serialization is complete.** `try_lock` creates the lock file (implied by `read_view`'s explicit "without writing anything, **including the lock file**"), so two handlers racing to found a family still serialize.
- **Fail-closed reuse.** The positive reuse rule (`:342-346`) still requires all three conditions — `ready` row, path canonicalises to the destination, owner equals this `session_id`, plus the write-free completeness check — and `:347` now reads "Every other **terminal** combination refuses", the precise wording the looping row required.
- **Table/test correspondence is complete.** I mapped all eleven D6 rows (`:350-372`) onto S1.1's list: no row → `:552`; directory at destination → `:559`; `creating` own session → `:565-567`; `creating` no/other owner → `:559`, `:562`; `disposing` → `:563`; `ready` path differs → `:560`; `ready` no owner → `:561`; `ready` other owner → `:560`; `ready` own session complete → `:554`; `ready` own session incomplete → `:561-562`; deadline passed → `:573-574` (lock) and `:568-570` (row still `creating`). Every row is covered — the round-4 C-P3-1 and C-P3-2 gaps are genuinely closed, not papered over.
- **R20's compatibility contract** (`GwzLaneCleanFixes.md:122-130`) is coherent with the code it binds: `deny_unknown_fields` on the row format and exact schema matching are what make a silent field addition impossible, so the v2 bump with an optional `owner` and a whole-index refusal by older readers is the right shape. R22's second test (`:141-143`) covers it. The plan restates it faithfully at `:145-153`.
- **§3.6's scope note** is accurate artefact-by-artefact against the gwz-cli tree, including the one that actually bites (`docs/CLI.md` byte-compared by `src/tests/g00.rs:35-40` and re-checked by the release gate).

**Carried-forward verifications still hold** on the revised text: section 1's lock facts (`:116-121`); the 28-lane count and the 112 entries against L1; all F1–F18 trail pointers; the round-2, round-3 and the new round-4 trail entry (`:916-933`), whose counts ("3 P2, 4 P3" consistency; "4 P2, 4 P3" safety; "All fifteen folded") match my filed report and RemPlan-3's disposition table exactly; every U1–U11 and O1–O5 assigned or resolved; the plan rule (milestones, one goal per step, foundational first, parallel-friendly) intact, with S1.1's over-budget disclosure and split offer unchanged and still honest.

**Sub-threshold observations**, recorded without findings: R20 says "member row" where the plan says "family row" (same object); R20's "MUST never change after creation" binds every writer of member rows, and GwzLaneIssues L4 records that `gwz merge` rewrote member-row fields until 1.0.13 — §3.6 still does not name merge as a writer it constrains; the v2 schema bump creates a minimum-version obligation broader than the `hook`-family one S4.3`:786-787` names (it binds every binary touching a workspace that has ever held an owned lane, hook adopter or not), though section 1`:147-149` does state it; S1.1's kill-mid-copy test (`:568-570`) runs for the full `--wait-secs` deadline unless the test forces a short one, which D1`:229` permits.

---

## 3. Risks and next action

The patch did the substantive work: all seven of my round-4 findings are closed on their merits, the blind convergence item (the remove hook's wait and busy class) is properly cured across D3, D8, S1.1 and S4.1, and the design change that replaced `--wait` with a hook-owned loop is a genuine simplification that I verified against gwz-core's code rather than accepting on assertion.

What remains is the familiar shape of a large single-patch fold: **three sections the patch should have touched and did not.** S1.0 was left on the old timing model while D6 started citing it (N-P2-1). D10's closed field list and S1.1's "two refusal classes" were left behind while D8 moved to three (N-P2-2). And S1.1's new concurrency test over-asserts against a bound D6 had just honestly conceded (N-P2-3). None is architectural; each is a one-to-three-sentence edit, and two of them (N-P2-1, N-P2-2) are stale strings I can name by line: `:516`, `:467`, `:596`.

I record explicitly: **no new architectural root cause on this axis.** The object's mechanism is now, in my assessment, coherent with `family-model`, `family-store-contract`, `workspace-install` and `local_clone/create.rs`, and coherent with R20–R22 as those requirements now read.

Next action: fold N-P2-1, N-P2-2 and N-P2-3 (and, cheaply, the three P3s — N-P3-1 and N-P3-2 are one clause each) and re-file. **I pre-commit to GO on a revision resolving {N-P2-1, N-P2-2, N-P2-3} as specified above.**
