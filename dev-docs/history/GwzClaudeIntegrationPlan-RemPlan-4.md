# GwzClaudeIntegrationPlan.md — remediation plan, round 4 (route-3 revision, round 2)

Object: `gwz-cli/dev-docs/GwzClaudeIntegrationPlan.md` at sha256
`d5356fec0330e3c718166a9716887588b684e8145ece5ba87179c6ac40c347db`, with
gwz-core `dev-docs/GwzLaneCleanFixes.md` at
`3026e35199a8df41e2a40109d27544a90fefcbfad753b267043362208bb503b1`.
Re-verdicts: `-ReviewSafety-5.md` (GO; 3 new P3) and
`-ReviewConsistency-5.md` (all seven prior findings closed; NO-GO on 3 new
P2, 3 new P3). No architectural finding on either axis. This is the second
and last remediation round on the redesigned object.

Convergence: consistency N-P2-1 and safety N-P3-3 are the same defect
(S1.0's timeout formula predates the attempt loop).

| ID | Disposition | Closure test |
| --- | --- | --- |
| C-N-P2-1 / S-N-P3-3 | S1.0's output sentence rewritten to D6's formula: it sets `--min-free-gb`, revises `--wait-secs`, measures the clone's pre-lock inventory, and sets Claude's create timeout to one wait plus one copy plus 60 s. | One formula in the document. |
| C-N-P2-2 | D10's classification vocabulary carries the three refusal classes D8 names; S1.1's field-set assertion says three. | D8, D10, S1.1 agree. |
| C-N-P2-3 | The different-name concurrency test starts the second create only after the first's `creating` row is visible; the same-instant race is stated as out of the test's scope. | Test deterministic; no assertion contradicts D6's bound. |
| C-N-P3-1 | D5 and U4 attribute the creation resolution to R20 plus D6's loop; R21 named as the remove hook's dependency. | No creation sentence cites `--wait`. |
| C-N-P3-2 | S1.3's fallback probe records whether `git worktree lock` is held on a fallback worktree and whether remove refuses it (U10, fallback case). | S1.3 carries the observation. |
| C-N-P3-3 | D6 states that an attempt reaching the clone pays the clone's pre-lock inventory before learning the lock is busy; S1.0 measures that inventory. | D6 states the cost; S1.0 measures it. |
| S-N-P3-1 | R20 and section 1 key the minimum-version rule on any index write by an R20-release gwz, naming dispose and `--keep` as writes. | Both documents say "written", not "owned lane". |
| S-N-P3-2 | S1.2's block carries a timeout for the `WorktreeRemove` handler too (wait plus 60 s); D1's option list gains `--wait-secs`. | S1.2's handler-text test asserts both timeouts. |
