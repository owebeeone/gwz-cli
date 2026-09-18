# GwzClaudeIntegrationPlan.md — remediation plan, round 1

Object: `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` at sha256
`e5325c03144af35186cb32859d5e44ee3ab154765bb0dc06493e5dd2ee6306b6`
(adopted 2026-09-17, uncommitted). Reviews:
`GwzClaudeIntegrationPlan-ReviewConsistency.md` (NO-GO: 2 P2, 7 P3) and
`GwzClaudeIntegrationPlan-ReviewSafety.md` (NO-GO: 1 P1, 6 P2, 2 P3). Both
pre-committed to GO on a revision resolving their blocking findings. One
patch resolves every finding below; the reviewers re-verdict on it.

Blind convergence: Consistency P2-1 and Safety P2-4 both attack D5's
one-placement rule from opposite directions (the rule's premise no longer
holds under D1's fixed handler; and the rule is unenforceable at pull time,
while `--command` reintroduces differing handlers). One disposition covers
both.

| ID | Disposition | Closure test |
| --- | --- | --- |
| C-P2-1 / S-P2-4 | D5 rewritten: identical handler text in more than one placement is permitted (Claude dedupes it); differing handlers are made harmless by session-keyed idempotency (the second creation for the same `session_id` and name finds the row the first made and prints the same path; a removal whose path is already gone exits zero). `setup` warns, not refuses, on a differing handler in another placement. S1.4 gains the precondition that user-level blocks are removed or identical. U4 marked resolved. | D5 no longer cites "two different commands" as its reason; S1.1 tests: two creations with one `session_id` yield one lane and one path; removal of an absent path exits zero. |
| C-P2-2 | S1.1 and S1.2 each list the generated artefacts: regenerate `docs/CLI.md` with `scripts/generate_cli_reference.py --write`, the grouping decision in `src/help.rs` (both families under `Other:`), a `docs/commands/` page and `mkdocs.yml` nav entry each. Budgets raised; S1.1 marked as splittable along create/remove. | Revised S1.1 names `docs/CLI.md` and `src/help.rs`; gwz-cli's suite is green after following the step. |
| C-P3-1 | S1.2 is the setup command only. The local `--write` run opens S1.3; the desktop `PATH` confirmation moves to S2.1 (D8 pointer updated). S1.3 retires its lanes through S3.4, stated. | No Phase 1 step title joins two deliverables. |
| C-P3-2 | Sketch gains `{ S1.3, a gwz release containing S1.1 and S1.2 } -> S1.4 -> { S4.1, S4.3 }` and a prose sentence. | Every prose dependency has an arrow. |
| C-P3-3 | U3 and U5 added to S2.1's record list; `.gwz/url-scheme.yml` and lane-local state added to S1.3's list (O2). | Every U and O string appears in a step or carries a resolved note. |
| C-P3-4 | Section 1's error-code fact deleted; trail entry says F15 folded into D1. | No `ManifestNotFound` in the object; trail names D1 for F15. |
| C-P3-5 | Lane count corrected to L1's register: 28 lanes (3 on 2026-09-10, 25 on 2026-09-15 and 16). | Count matches L1. |
| C-P3-6 | S1.0 points the timeout at S1.2's settings block. | Only S1.2 names the timeout as its output. |
| C-P3-7 | Knobs become hook command-line options baked into the handler by `setup` (`--min-free-gb`, `--max-lanes`, `--base-ref`, `--log`); environment variables dropped except for tests. D6, D10, O3 updated. | Every knob is reachable from the desktop app through the handler string. |
| S-P1-1 | Reuse only when the family index holds a `ready` row for NAME whose canonical path equals the destination, the sidecar records the same `session_id`, and the tree passes the completeness check; `creating/incomplete`, unrelated directory, or mismatched row: exit non-zero, nothing on stdout, message names the state and S3.4. Invariant stated at the head of S1.1. | Tests (a) to (d) as the review lists. |
| S-P2-1 | The create hook records `session_id` per lane in `<root>/.gwz/claude-lanes.yml` (a sidecar, never in the family index); reuse by a different session is refused naming the other session. | Test: two sessions, same name, second refused. |
| S-P2-2 | Remove hook: canonicalise `worktree_path`, find the lane root via `.gwz/family-root`, dispose only when exactly one `ready` row's canonical path equals it, with explicit `--root` and a working directory at the family root; log distinguishes hook refusal from gwz hazard refusal. | Tests as the review lists. |
| S-P2-3 | Setup writer: parse first and refuse a file that does not parse or is not a regular file; temp file, fsync, re-parse, rename; bytes outside the block unchanged. | Tests as the review lists. |
| S-P2-5 | D6 rewritten: threshold from the working-lane cost (S3.2), a ceiling on `ready` lanes in the family (default 8), free space measured on the destination parent's filesystem, message names S3.4. | Tests as the review lists. |
| S-P2-6 | Fallback contract extended: reuse an existing `.claude/worktrees/NAME` and `worktree-NAME` pair; perform the `.worktreeinclude` copy; sweep gap and remedy stated in S4.1; parity claim qualified. | Tests as the review lists. |
| S-P3-1 | D10 lists the log fields (no `transcript_path`), a size bound, and the ignored-location check with fallback to the user log. | Tests as the review lists. |
| S-P3-2 | Every non-zero exit prints one stderr line "gwz: <cause>; <remedy>"; S4.1 carries the table; S1.3 records what the user sees on an abort. | Test as the review lists. |

Not changed, surfaced to the operator instead: the Safety review's residual
risk 1 (S1.4 commits the block before GwzLaneCleanFixes R0 lands) is an
ordering decision for the operator; the patch adds the observation to S1.4
without changing the gate.
