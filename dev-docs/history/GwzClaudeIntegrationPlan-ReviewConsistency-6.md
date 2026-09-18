# GwzClaudeIntegrationPlan.md — CONSISTENCY-AXIS FINAL RE-VERDICT, ROUND 6

**Review object:** WORKING-TREE `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `c0bba7fafec38dfb5bbbf896f3357e6fc0d196d4698970fda7a9936e7b077d20`, 969 lines.
**Baseline:** gwz-dev root HEAD `5152f637c4a2169b15e0c7d97393b1ed3a2afe28`; gwz-cli HEAD `865f89c5787b5e1470415a718003dc4682f448a7`; gwz-core HEAD `0f1aad6f0b9c687a315afc11c78b22472feb88e8`; controlling working-tree `gwz-core/dev-docs/GwzLaneCleanFixes.md` sha256 `c5f06e41e672ea8a48ac73ce7eebee443729e8a58b790e46c2edcfd97a94e331`, 193 lines. Tuple verified identical at review start and at review end.
**Date:** 2026-09-17 **Axis:** CONSISTENCY — the document against its controlling graph.

**Verdict: GO on sha256 `c0bba7fa…`** — 0 P0, 0 P1, 0 P2, 2 P3. **All six round-5 findings (N-P2-1..3, N-P3-1..3) are closed and verified against my own counterexamples.** **No new architectural root cause**, and none has been introduced at any point on this revision. The two new P3s are bounded and non-blocking: both are one-clause edits the operator may fold at leisure or carry as known gaps into S1.0/S3.5.

---

## Prior-finding closure table

| ID | Round-5 finding | Cure located | Counterexample re-traced | Status |
| --- | --- | --- | --- | --- |
| N-P2-1 | S1.0 carried the pre-attempt-loop timeout formula ("two copies back to back") while D6 cited S1.0 for a different one, and S1.0 did not produce `--wait-secs` | S1.0 `:523-530`, rewritten: "Also time the clone's pre-lock source inventory on its own… The numbers set D6's interim `--min-free-gb` default, **revise D6's `--wait-secs` default** (at least one copy plus the inventory), and set the create timeout in S1.2's settings block to **one wait plus one copy plus 60 s, D6's formula**, with the remove handler's timeout at one wait plus 60 s." | Re-ran the arithmetic counterexample: with a 60 s copy, S1.0 and D6 `:332-333` and S1.2 `:622-625` now yield the same number instead of 120 s versus 420 s. `grep` confirms "two copies back to back" is gone from the document. One formula, stated in three places identically, and S1.0 now names `--wait-secs` among the values it revises. | **CLOSED** |
| N-P2-2 | D10's log-line field set was closed at one refusal value and S1.1 asserted two, while D8 required three — the "family busy" cure had no field to live in | D10 `:473-476`: classification "(lane, member-in-lane, fallback worktree, **refused-by-hook, refused-by-hazard, family-busy; the three refusal classes D8 names**)"; S1.1 `:611` "the **three** refusal classes distinguished" and `:614-616` "the field-set assertion admits the three refusal classes of D10, and the busy-removal test asserts `family-busy`" | Re-traced the contradiction: an implementer obeying D10's "carries exactly" now has a value for each of D8 `:446-448`'s three classes, and S1.1's field-set assertion no longer collides with its own busy-removal assertion at `:596-597`. D3 `:269-272`'s promise ("logged and printed as 'family busy; retry' and never as a hazard") is now deliverable through a named field. `grep` confirms "two refusal classes" is gone. | **CLOSED** |
| N-P2-3 | S1.1's different-name concurrency test asserted an outcome D6 explicitly concedes the design does not guarantee, with no sequencing stipulation | S1.1 `:583-587`: "two creations of different names with the family at `--max-lanes` minus one, **the second starting only after the first's `creating` row is visible in the index** (exactly one new lane, the second **refused by the ceiling once the first is `ready`**, nothing copied for it, logged; **the same-instant race D6 concedes is outside this test's scope**)" | Re-ran the race: both handlers can no longer read the index before either reserves. I then checked the new sequence is actually deterministic *and* still tests what it was written for — at the moment the second starts, the first's row is `creating`, so the ceiling (which counts `ready` rows, D6 `:325-326`) is not yet reached; the second passes the guard, reaches the clone, takes `Busy` at create.rs step 5 with nothing written, loops, and is refused on a later attempt once the first is `ready`. The assertion "nothing copied for it" is correct because `Busy` precedes reservation. The test is deterministic and still demonstrates guards being re-evaluated per attempt. | **CLOSED** |
| N-P3-1 | D5 and U4 still attributed the parallel-*creation* resolution to R21's `--wait` | D5 `:295-299`: "through **one** gwz-core facility this plan requires (GwzLaneCleanFixes R20…), the owner token…, and D6's attempt loop in the hook. **(R21's `--wait` is the remove hook's dependency, D8; creation does not use it.)**"; U4 `:192` "conditional on **R20** being in the installed gwz" | Searched every creation sentence for `--wait`: none remains. The sketch `:828` still gates S1.1 on R20 *and* R21, which is correct and not a contradiction — S1.1 builds the remove hook, which does consume R21 (D8 `:440`). | **CLOSED** |
| N-P3-2 | D8 cited S1.3 for a `git worktree lock` observation S1.3 did not collect | S1.3 `:671-674`: "…and removed on exit (U7), **and recording, while the session runs, whether `git worktree lock` is held on that worktree and whether `git worktree remove` refuses it (U10 for the fallback case, the protection D8 leans on)**" | D8 `:409-410`'s forward pointer now names work S1.3 performs, and it is scoped to the fallback case (U10's lane case stays with S2.2 `:715`+). The evidence D8's shared-worktree acceptance leans on will exist before S4.1 documents the caveat. | **CLOSED** |
| N-P3-3 | D6's "sleeps briefly and loops" understated the per-attempt cost, because the clone inventories the source before reaching the lock | D6 `:337-342`: "An attempt that reaches the clone pays the clone's pre-lock source inventory before it can learn the lock is busy (**the clone inventories the source at its step 4 and takes the lock at step 5**), so against an unrelated family command the effective poll period is that inventory, not the sleep; **S1.0 measures it and the `--wait-secs` default is sized against it**"; S1.0 `:523-526` | Verified the code citation is exact: `gwz-core/src/local_clone/create.rs:14-16` step 4 "the source inventory and snapshot, **before the family lock**", step 5 "the family lock". The statement is now true, scoped correctly to the unrelated-command case (the D5 parallel-handler case still loops off the lock-free read without calling the clone, D6 `:360-362`), and the cost is routed to a measuring step. | **CLOSED** |

---

## Changed-range analysis

Every range the coordinator named was inspected; two carry new, bounded defects, filed below.

- **S1.0 (`:518-530`)** — rewritten as specified; the stale rationale ("since two creations serialize on the family lock") is gone with the stale formula. Clean.
- **D10 (`:472-477`)** — classification vocabulary extended to the three classes and cross-referenced to D8. Clean.
- **S1.1 (`:583-587`, `:611`, `:614-616`)** — all three edits landed; the D6-table-to-test mapping I built in round 5 is unaffected and still complete across all eleven rows.
- **D5 (`:295-299`) and U4 (`:192`)** — attributions corrected in both directions (creation → R20 + loop; R21 → remove hook). Clean.
- **S1.3 (`:671-674`)** — the fallback lock observation added. Clean.
- **D6 (`:337-342`)** — the per-attempt cost sentence added, with an exact and correct citation of create.rs's step order. Clean.
- **D1 (`:227`)** — `--wait-secs` added to the option list, so D1's enumeration now matches D6's and S1.2's usage. Clean.
- **S1.2 (`:622-625`, `:643-644`)** — the remove handler gains its own timeout ("one wait plus 60 s"), both derived from the same `--wait-secs`, with a test asserting both. **The second term is unsourced — N-P3-1 below.**
- **Section 1 R20 paragraph (`:145-154`)** — restates the new minimum-version trigger, but drops one of the three writes R20 names. **N-P3-2 below.**
- **Trail (`:957-969`)** — the round-5 entry matches my filed report exactly (all seven prior closed; NO-GO on 3 new P2 and 3 new P3; "none architectural") and matches RemPlan-4's eight dispositions.
- **GwzLaneCleanFixes R20 (`:115-131`)** — the trigger now reads "once any gwz at or above it has written the family index, and a dispose, a `--keep`, or a family merge is such a write, not only a create." Coherent with the code: `gwz merge` writing member rows is exactly what GwzLaneIssues L4 recorded (rotating member-row fields until 1.0.13), so naming merge is correct and necessary.

**NO NEW ARCHITECTURAL ROOT CAUSE.** The mechanism is unchanged from the text I verified in depth last round against `local_clone/create.rs`, `family-store-contract`, `family-model` and `workspace-install`; this patch touched only step prose, a field vocabulary, two test stipulations and two attributions. Nothing in it disturbs the loop's realisability.

---

## 0. Evidence base

Read in full: the revised object (969 lines); `GwzClaudeIntegrationPlan-RemPlan-4.md` (25 lines, all eight dispositions); the revised `GwzLaneCleanFixes.md` R20/R21 and §3.6 scope note. Re-read targeted ranges: `gwz-dev/dev-docs/GwzLaneIssues.md` L1 occurrence list and timings (`:22-36`).

Code facts re-confirmed (all read-only, carried from rounds 4–5 and re-checked where this patch cites them): `gwz-core/src/local_clone/create.rs:1-31` (step 4 inventory before the lock, step 5 the lock, pre/post-reservation residue rules — the basis of D6 `:337-342` and of N-P2-3's "nothing copied for it"); `crates/family-store-contract/src/lib.rs:276-305` (`try_lock`/`Busy`, lock-free `read_view`, the crash-recoverable pointer-then-row removal order); `crates/family-model/src/lib.rs:207-211` (three `MemberState` variants, all named in D6's table); `crates/workspace-install/src/admit.rs:16-71`, `:150-168`; `gwz-cli/src/clirequest/local.rs:21-52`; `gwz-cli/src/tests/g00.rs:35-40`; `gwz-cli/docs/LocalClones.md:191-201`; `gwz-cli/docs/MachineOutput.md:558-561`.

Stale-string sweep on the revised text: "two copies back to back" — gone; "two refusal classes" — gone; "sidecar" — survives only in trail entries `:903`, `:923`, `:934`, the last of which negates it. The one surviving "owned lane" (`:369`) is D6's no-owner table row, which is correct and distinct from the minimum-version trigger.

Re-verified carried-forward invariants: section 1's lock facts (`:116-121`); the 28-lane count and 112 entries against L1; all F1–F18 pointers; every trail entry against the eight filed review documents; U1–U11 and O1–O5 each assigned or resolved; §3.6's six named gwz-cli artefacts, each still accurate to the tree.

---

## 1. Findings

### [P3-1] The remove handler's timeout term is unsourced: nothing in the plan measures a dispose, and L1 already records disposals up to 238 s against a 60 s margin

- **Root cause.** S1.2's new remove-handler timeout was given a fixed 60 s margin over the wait, but no step measures how long the dispose it wraps actually takes.
- **Location.** S1.2, `GwzClaudeIntegrationPlan.md:622-625` ("each one command handler with its own timeout from S1.0: one wait plus one copy plus 60 s for create, **one wait plus 60 s for remove**"). S1.0, `:526-530`, which times the *clone* and its pre-lock inventory and then hands S1.2 both timeouts — it measures nothing about disposal. Against `gwz-dev/dev-docs/GwzLaneIssues.md:33-36`: "Plain refused in 5–16 s; **forced took 28–44 s, except split-ca at 238 s**"; round 9, "Plain refused in 6–15 s; forced took 27–33 s."
- **Violated invariant.** Every number the settings block carries is derived from a measurement the plan names (the rule S1.0's rewrite just established for the create side).
- **Reproduction.** Read S1.2`:623-624` → remove timeout = wait + 60 s; with D6`:332`'s default that is 360 s. Read D8`:440` → the dispose itself may consume the full `--wait <wait-secs>` waiting for the lock. Then read L1`:33-35` for what a dispose that actually deletes costs on gwz-dev: up to 238 s. Worst case 300 s wait + 238 s dispose = 538 s against a 360 s budget. Search the plan for any step that measures dispose duration (`grep -i dispose` cross-checked against S1.0, S3.2, S1.3, S3.5): none does; S1.3 records refusal *text*, S3.2 measures creation only.
- **Impact.** Bounded, and only post-R0. Today the hook's dispose always refuses in 5–16 s (it never forces, D3`:266`), so the budget holds. After GwzLaneCleanFixes R0 lands and S3.5 makes a successful one-step disposal possible, a remove hook can be killed at its timeout mid-dispose. The outcome is recoverable and fail-safe — gwz's removal order is crash-recoverable (`family-store-contract/src/lib.rs:296-305`), Claude keeps the session (section 1`:61-63`), and a resulting `disposing` row is refused fail-closed by D6`:365` and routed to S3.4 — but it leaves a partially retired lane with **no log line** (the hook was killed, D6`:380-382`), which is the one state D10's log cannot explain.
- **Required correction.** Have S1.0 (or S3.5, where a succeeding dispose first becomes observable) time a dispose, and size the remove handler's timeout as one wait plus a measured dispose plus margin, citing L1's 5–238 s range as the interim basis.
- **Closure test.** The remove handler's timeout is derived from a measurement a named step performs, as the create handler's now is.

### [P3-2] Section 1's restatement of R20's minimum-version trigger drops "a family merge" — the one write gwz-dev performs most

- **Root cause.** The plan's restatement of the revised R20 enumerates two of the three writes R20 names.
- **Location.** Plan section 1, `:146-150` ("every gwz used on a workspace must be at or above the R20 release once any such gwz has written the family index (**a dispose or a `--keep` is a write too, not only a create**)"). Against `GwzLaneCleanFixes.md:128-131` ("once any gwz at or above it has written the family index, and **a dispose, a `--keep`, or a family merge** is such a write, not only a create").
- **Violated invariant.** A restatement of a controlling requirement must not be narrower than the requirement.
- **Reproduction.** Read R20's final sentence, then the plan's `:148-150`. "A family merge" is absent from the plan's list. That omission matters specifically here: GwzLaneIssues L4 records that `gwz merge` rewrote the fields of every member row until the 1.0.13 fix, and L1's register shows 25 of the 28 lanes reaching disposal through `gwz merge --remote` — merge is the family write gwz-dev performs most often, and the one most likely to be the first v2 write on a workspace.
- **Impact.** Bounded but concrete. The plan is where the gwz-dev operator will read this rule; S4.1`:786-787` and S4.3 then document "the minimum gwz version" from it. An operator running a mixed-version fleet concludes a v1 index survives routine merges and is surprised when the first `gwz merge --remote` bumps the schema and an older binary on a second machine refuses the whole index.
- **Required correction.** Add "or a family merge" to section 1`:149`, so the plan's list matches R20's three writes.
- **Closure test.** The plan and R20 enumerate the same writes.

---

## 2. Invariant analysis

**The object is internally consistent and agrees with its controlling graph.** Every axis of attack I have available is now clean:

- **Mechanism realisability** — unchanged and verified: looping on `Busy` is residue-free because `Busy` occurs at create.rs step 5, before reservation, and the header guarantees "A refusal before reservation leaves nothing"; serialization holds even pre-founding because `try_lock` creates the lock file independently of the index; the hook takes no lock and keeps no state (D6`:330-331`), so the round-2 sidecar and the round-3 lock-nesting architecture are both permanently gone.
- **D6 table ↔ S1.1 tests** — all eleven rows covered one-for-one (mapping built in round 5, re-checked here; the only test-list changes this round were the sequencing clause and the field-set clause, neither of which disturbs the mapping). All three `MemberState` variants are named.
- **One number, one definition** — the create timeout now reads identically in D6`:332-333`, S1.0`:528-530` and S1.2`:622-624`; `--wait-secs` appears in D1's option list, has a compiled-in default that the bare committed handler uses, and has a named step that revises it.
- **Three refusal classes** — D3, D8, D10 and S1.1 now agree on the count and on the vocabulary, and S1.1's two relevant assertions no longer collide.
- **Citations** — every R-number resolves and says what the plan says (R20's revised text checked clause by clause; the one narrowing is P3-2); L1 and L2 correct; the 28 lanes and 112 entries reconcile arithmetically; §3.6's six gwz-cli artefacts each verified against the tree, including the `g00` byte-compare that is the one with teeth; all eight trail entries match the filed reports' verdicts and counts, including this round's.
- **Plan rule** — phases are milestones, each step one goal with a budget, foundational first (S1.0 explicitly ungated and runnable now), parallel-friendly with the independent sets named. S1.1's over-budget disclosure and split offer remain honest and unchanged.

**Sub-threshold observations**, recorded without findings: whether S1.0's "revise D6's `--wait-secs` default" means changing the compiled-in constant during S1.1 or baking `--wait-secs` into the block at S1.2 is left open, but both paths are workable and the same looseness has ridden along harmlessly on `--min-free-gb` since round 1; R20 still says "member row" where the plan says "family row" (same object); R20's `MUST never change after creation` binds every member-row writer and §3.6 now at least names merge as one, which closes the gap I noted last round.

---

## 3. Risks and next action

This patch did what a final remediation round should: it took three narrow contradictions and two dangling pointers and closed each at the exact line I named, without introducing a new mechanism and without weakening a test to make an assertion pass. The different-name concurrency test is the one I checked hardest, because a sequencing stipulation is an easy way to make a test deterministic and useless — it is not: the sequence chosen still forces the guard to be re-evaluated between attempts, which is what the test exists to demonstrate.

The residual risk on this axis is small and named. **P3-2 is a one-clause edit** (add "or a family merge" to `:149`) and I would take it now, since it costs nothing and the plan is where the operator will read the rule. **P3-1 is a real gap in a number, but it cannot bite until GwzLaneCleanFixes R0 lands and S3.5 runs** — the natural place to fold it is S3.5's repeat of the removal sequence, where a succeeding dispose is observable for the first time. Carrying it as a known gap into S3.5 is a defensible choice; so is timing a dispose in S1.0.

**No P0, P1 or P2 is open. No architectural root cause was found on this revision, at any round.** The object is consistent with `GwzLaneCleanFixes.md` R20–R22 as they now read, with the gwz-core code those requirements modify, with gwz-dev's L1/L2 register, and with itself.

**Verdict: GO on sha256 `c0bba7fafec38dfb5bbbf896f3357e6fc0d196d4698970fda7a9936e7b077d20`.**
