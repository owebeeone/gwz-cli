# GwzClaudeIntegrationPlan.md — SAFETY-AXIS CONFIRMATION, ROUND 6

**Review object:** WORKING-TREE `/Users/owebeeone/limbo/gwz-dev/gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md`, sha256 `c0bba7fafec38dfb5bbbf896f3357e6fc0d196d4698970fda7a9936e7b077d20`, 969 lines.
**Controlling:** WORKING-TREE `gwz-core/dev-docs/GwzLaneCleanFixes.md`, sha256 `c5f06e41e672ea8a48ac73ce7eebee443729e8a58b790e46c2edcfd97a94e331`, 193 lines.
**Baseline:** gwz-dev root `5152f637…`, gwz-cli `865f89c5…`, gwz-core `0f1aad6f…` — unchanged. Both object hashes verified at start and end; nothing moved.
**Date:** 2026-09-17 **Axis:** SAFETY.

**Verdict: GO** — 0 P0, 0 P1, 0 P2, 0 P3. All three round-5 P3s closed; no new finding on my axis from any changed range. **No NEW ARCHITECTURAL root cause.** My round-5 GO stands, now unqualified.

---

## Prior-finding closure table

| Finding | Required correction | New text | Re-trace | Status |
| --- | --- | --- | --- | --- |
| **N-P3-1** — R20's summary sentence keyed the older-binary lockout on "once any lane has been made by one", while the specified trigger is any index write by a v2-aware binary; an operator following it would believe a lane-free workspace was still safe for the pinned older gwz | Key both statements on the write, naming dispose/`--keep`/merge as writes | `GwzLaneCleanFixes.md` R20:128-132 — "once any gwz at or above it has written the family index, and a dispose, a `--keep`, or a family merge is such a write, not only a create"; plan section 1:147-149 carries the same ("a dispose or a `--keep` is a write too, not only a create") | My sequence: lane-free workspace, one `gwz local dispose` run with the R20 release, then the pinned 1.0.13. The text now predicts the refusal instead of excluding the case, and the refusal names the minimum version (R20:126-128). The one mixed-version surprise the amendment had left open is closed in both documents, so neither can be read alone and mislead | **Verified — closed** |
| **N-P3-2** — the remove handler had a wait but no timeout of its own, so a raised `--wait-secs` put it past Claude's 600 s default and made the new "family busy; retry" class unreachable; `--wait-secs` was also absent from D1's canonical option list | Set the remove handler's timeout from the same number; list the option | S1.2:622-625 — each handler carries "its own timeout from S1.0: one wait plus one copy plus 60 s for create, one wait plus 60 s for remove, both derived from the same `--wait-secs` the block carries or the compiled-in default"; S1.0:529-530 states both; S1.2:643-644 tests "both handlers' timeouts present and consistent with the block's `--wait-secs`"; D1:227 now lists `--wait-secs` | My sequence: S1.0 raises `--wait-secs` above ~540 s; the remove handler's timeout now moves with it, so the hook reaches its own deadline, classifies, prints "family busy; retry" and logs `family-busy` (D10:474-475, S1.1:596-597, 614-616) rather than being killed silently. The fourth outcome class is reachable in every configuration S1.0 can produce, and a test pins the arithmetic | **Verified — closed** |
| **N-P3-3** — S1.0 still sized the create timeout by the formula D6 replaced ("two copies back to back"), so an implementer following S1.0 would come up 60 s short of D6's need and lose the enumerated wait refusal to a bare timeout | One rule, stated once | S1.0:526-530 — the stale parenthetical is gone; S1.0 now revises `--wait-secs` (at least one copy plus the inventory) and sets "the create timeout in S1.2's settings block to one wait plus one copy plus 60 s, **D6's formula**, with the remove handler's timeout at one wait plus 60 s" | The two places that stated a budget now derive it from D6's single rule, and S1.0 is the step that measures its inputs. The kill-instead-of-refuse path I traced needs a number S1.0 no longer produces | **Verified — closed** |

## Changed-range analysis

Each changed range read and attacked on the safety axis; none introduces a defect, and one materially improves the honesty of the wait design.

- **D6:337-342 (per-attempt cost)** — new and correct against the code: `src/local_clone/create.rs:184` inventories the source at step 4, `:216` takes the lock at step 5, so an attempt that reaches the clone pays the pre-lock inventory before it can learn the lock is busy. The text draws the right boundary: this applies only "against an unrelated family command", because the own-session `creating` case (D6:360-362) loops on the cheap lock-free index read and never reaches the clone. The consequence is a coarser poll period, not a hazard — the inventory is a read, the loop stays bounded by the deadline, and S1.0:523-526 now measures the interval with `--wait-secs` sized against it. This converts an unstated cost into a measured one.
- **D10:473-476** — the classification vocabulary now spells the three refusal classes (`refused-by-hook`, `refused-by-hazard`, `family-busy`), so D8's three-way distinction is expressible in the field set rather than only asserted in prose. S1.1:611-616 asserts the field set admits them and that the busy-removal test asserts `family-busy`.
- **S1.1:583-587** — the different-name ceiling test is now sequenced (second starts after the first's `creating` row is visible) and explicitly scopes out the same-instant race D6 concedes. This matches what I found in round 5: because the lock is held from reservation through promotion (`create.rs:216-218,272-277`), the reachable behaviour is "the second is refused by the ceiling once the first is `ready`", and that is exactly what the test now asserts. No test now claims a property the mechanism cannot deliver.
- **D5:298-300 and U4:189-192** — attribution corrected: creation rests on R20 plus the attempt loop, R21 on the remove hook ("creation does not use it"). Safety-relevant, and in the conservative direction: a defect in R21's wait can no longer reach lane creation, and U4's conditional resolution is narrowed to R20 accordingly. S1.1's gate on both requirements remains right, since S1.1 builds both hooks.
- **S1.3:671-674** — the fallback `git worktree lock` observation is now taken *while the session runs* and records both whether the lock is held and whether `git worktree remove` refuses it, naming it as "the protection D8 leans on" (U10). The assumption under my round-4 S-P3-3 is now measured at the point where it matters rather than inferred.
- **D1:227, section 1:145-154, trail:946-969** — option list, schema trigger, and an accurate record of the round.

## 0. Evidence base

Full read of the changed ranges in both documents plus the surrounding decisions they bind (D1, D5, D6, D8, D10, U4, S1.0–S1.3, S4.1, section 5, trail); the round-5 state sequences re-traced against the new text. Code re-verified read-only for the one new factual claim: `src/local_clone/create.rs:184` ("Step 4: the source is inventoried and captured before the lock") and `:216` ("Step 5: the family lock"). No builds, no mutations, no `gwz` command that writes.

## 1. Findings

None. No P0, P1, P2 or P3 on the safety axis for sha256 `c0bba7fa…`.

## 2. Invariant analysis

The safety properties I certified in round 5 are unchanged and now better instrumented: the create hook re-derives index, table and guards at every attempt and carries no state across a blocking call; every unowned direction of the reuse table is fail-closed, with the single loop keyed to the hook's own session and bounded by a deadline; the remove path keeps its canonicalise-resolve-match-`--root`-never-force discipline and now has a busy class that is barred from the hazard channel and reachable in every configuration; nothing sensitive leaves an ignored file, and the owner token is redacted wherever the plan's own steps commit an inventory; the mixed-version lockout is specified, diagnosable by a refusal that names the minimum version, and now triggered by the condition the text states.

## 3. Risks and next action

Nothing on this axis blocks S1.0 or S1.1. The residual risk is ordinary implementation risk that the plan has already targeted with tests: the wait arithmetic (S1.2:643-644), the three-class log (S1.1:611-616), the sequenced concurrency cases (S1.1:578-587), and the two probes that convert this revision's remaining assumptions into evidence (S1.0's pre-lock inventory interval; S1.3's fallback worktree lock, U10). If either probe contradicts its assumption, the affected sentence is D6:337-342 or D8's shared-worktree reason — both local, neither structural.

Next action: none from me. **GO on sha256 `c0bba7fafec38dfb5bbbf896f3357e6fc0d196d4698970fda7a9936e7b077d20`.**
