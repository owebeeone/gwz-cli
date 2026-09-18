# GwzClaudeIntegrationPlan.md — remediation plan, round 2

Object: `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` at sha256
`660b7b8120ecea1e1813d179f32f43e1edfbf29453473bdf115c03033e9b7526`.
Round-2 reviews: `GwzClaudeIntegrationPlan-ReviewConsistency-2.md` (all
nine prior findings cured; NO-GO on 1 new P2, 2 new P3) and
`GwzClaudeIntegrationPlan-ReviewSafety-2.md` (all nine prior findings
closed; NO-GO on 2 new P2, 1 new P3). Every new finding was introduced by
the round-1 patch. The consistency axis classified its N-P2-1 as a NEW
ARCHITECTURAL root cause (the reuse predicate spans the family index and a
sidecar under no stated lock); the safety axis classified its P2-2 (guards
evaluated before reuse) as not architectural. This is the second and last
remediation round the review loop allows on this object; one architectural
root cause has been counted against it.

Blind convergence again: both axes flag the Goal's unqualified parity
sentence (safety residual 1, consistency changed-range note) and both flag
the fallback's unsession-keyed reuse (safety invariant note, consistency
N-P3-2).

| ID | Disposition | Closure test |
| --- | --- | --- |
| C-N-P2-1 (architectural) | D6 states that the create hook holds the family lock from before the clone until after the sidecar row is written, so the `ready` row and the sidecar record become visible together; the full outcome table of row state × sidecar state is enumerated, including `ready`-with-no-record (refuse, fail-closed, with the S3.4 remedy). D5 names the lock. U4 qualified. | D6 enumerates every combination; D5's sentence names the lock; S1.1 gains the "ready row before any sidecar row" test. |
| S-P2-2 | Reuse is evaluated before the guards; a same-session reuse consumes nothing and skips both guards; D5 names the orphan case when a parallel handler refuses anyway (stale `--command`), with S3.4 as its inventory. | S1.1 tests: two creations at `--max-lanes` minus one under differing handlers yield one lane and two zero exits; same-session reuse below `--min-free-gb` succeeds; a guard refusal only when nothing was created. |
| S-P2-1 | The `.worktreeinclude` copy runs only for a worktree this invocation created; on reuse nothing is written and a missing listed file is reported on stderr. The lane completeness check likewise writes nothing. Pattern semantics named in S1.1. | S1.1 tests: a modified included file survives reuse byte-identical; a deleted one is reported, not recreated. |
| S-P3-1 | D10's ignore check covers every file the hooks write in the workspace, naming the sidecar; a sidecar location that would show in `git status` refuses creation with the L2 reason. `ready`-with-no-record is in D6's table. | S1.1 test: a root without the managed exclude block refuses creation with the tree unchanged; a missing-sidecar refusal message. |
| C-N-P3-1 | D9's two pointers → S2.1. | No "S1.3" inside D9. |
| C-N-P3-2 / safety note | D8's fallback bullet states why structural reuse suffices there (a plain worktree is cheap and recreatable, matches Claude's own default, and `git worktree remove` without `--force` protects a dirty tree); the two-session case added to S1.1's fallback tests. | D8 states the reason; S1.1 lists the case. |
| Goal parity (both axes) | Goal sentence qualified: "as far as a hook can". | Goal matches D8. |
