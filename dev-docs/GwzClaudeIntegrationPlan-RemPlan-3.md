# GwzClaudeIntegrationPlan.md — remediation plan, round 3 (route-3 revision, round 1)

Object: `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` at sha256
`32bc9c74d831308515e31ab55e88129c5087219224dfeda11231b24e12788bcf`
with gwz-core `dev-docs/GwzLaneCleanFixes.md` §3.6 at sha256
`ad8b23cde45b42f12939986a06e250e16fc79e077e5d9070c8ebf7440f9a166b`.
Fresh dual review of the redesigned mechanism:
`GwzClaudeIntegrationPlan-ReviewConsistency-4.md` (NO-GO: 3 P2, 4 P3) and
`GwzClaudeIntegrationPlan-ReviewSafety-4.md` (NO-GO: 4 P2, 4 P3). Neither
axis found an architectural root cause; both pre-committed to GO. This is
the first remediation round on the redesigned object.

Blind convergence: C-P2-2 and S-P2-3 (the remove hook's dispose has no
wait and no `Busy` class) from both axes.

Design change in this patch, driving several dispositions: the create hook
no longer relies on `--wait` inside the clone. It runs its own bounded
attempt loop: each iteration re-reads the index lock-free, re-evaluates
the reuse table and both guards, and only then attempts the clone without
a wait; `Busy` sleeps and loops until the hook's deadline. That closes
S-P2-1 (a `creating` row owned by this session is "wait", not "refuse")
and S-P2-2 (guards are fresh at every attempt) without new gwz-core work.
R21 stays, for the remove hook's dispose and for the wider workflow.

| ID | Disposition | Closure test |
| --- | --- | --- |
| C-P2-1 | `--wait-secs` gets a compiled-in default (300 s) that the bare handler uses; S1.0 revises it and sets the Claude timeout to wait + one copy + 60 s; S1.2 prints the timeout, and bakes `--wait-secs` only as an override. D6's "both baked by setup" reconciled with D1. | D6 names the default; the bare handler waits by it. |
| C-P2-2 / S-P2-3 | The remove hook's dispose carries `--wait <wait-secs>` (R21); a fourth refusal class "family busy" added to D3, D8's log classes, D10's stderr form, S1.1's tests and S4.1's table; a row that disappears between classification and dispose exits zero. | S1.1 tests as both reviews specify. |
| C-P2-3 | §3.6 gains a scope paragraph naming the gwz-cli artefacts R20/R21 carry (clap args, `docs/CLI.md`, `local_long.rs`, `docs/commands/local.md`, the `local list` sample, `MachineOutput.md`); section 5's "outside this plan" sentence restated. | Both documents name the artefacts. |
| S-P2-1 | D6's table gains "`creating` row owned by this session: keep waiting until the deadline, then re-evaluate"; the attempt loop above. | The concurrency test starts B after A's `creating` row is visible. |
| S-P2-2 | Guards re-evaluated at every attempt, immediately before the clone; the residual bound (two attempts admitted in the same instant can exceed the ceiling by one) stated in D6. | Different-name concurrency test: second refused by the ceiling after its wait. |
| S-P2-4 | R20 states the compatibility contract: schema bumps to `gwz.local-family/v2`, `owner` optional, an older reader refuses the whole index naming the minimum version; the plan states that every gwz used on a workspace must be at or above the R20 release, and D6's no-owner row reads "made by hand, or before this workspace's first owned lane". | R22 extended with the older-reader case. |
| C-P3-1 | Same-owner-but-incomplete test added. | S1.1 lists six refusal rows. |
| C-P3-2 | `disposing` row added to D6's table (refuse; S3.4) and tested. | All three `MemberState` variants named. |
| C-P3-3 | Status line and section 1: "S1.1 onward" not "Phase 1". | No Phase-1-wide gate. |
| C-P3-4 | Section 1 records `session_id` as a UUID (to confirm in S1.3); D6: the hook validates the token against R20's grammar and refuses with its own message. | One S1.1 test. |
| S-P3-1 | D10 names every file a create writes at the root (index, lock file, managed exclude block) and which the ignore check governs. | Fixture asserts the exclude block content. |
| S-P3-2 | S1.3 and S3.4: owner tokens redacted from committed notes. | Rule stated. |
| S-P3-3 | D8: `git worktree remove` refuses a locked worktree, and Claude locks a running agent's worktree (section 1); S1.3 records it (U10). | Fallback removal test with a locked worktree. |
| S-P3-4 | D6/D10: a timeout kill leaves a `creating` row and files, no log line; `gwz local list` is the inventory. | Kill-mid-copy test. |
