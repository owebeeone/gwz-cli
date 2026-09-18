# GWZ Log S2.5 round-1 independent review

**Verdict: NO-GO.**

- **Core base / sole parent:** `2214eace46b72915f76ab28e03e16716ce9d1a60`
- **Core candidate:** `ba525e91c8b0363fcd59c33410635e7cc2781b55`
- **Normative CLI authority:** `19f1e3d471c70385f1069180b9552ab7cbfa3649`
- **Mode:** independent, single-axis S2.5 round 1; core read-only
- **Rows graded:** L-COA-7, L-ORD-2, L-DEP-1, L-PRF-1, L-PRF-2,
  L-ENV-1..4 only
- **Finding count:** 0 P0 / 5 P1 / 1 P2 / 0 P3

Authority and instructions were read completely before review action, in the
mandated order: workspace `AGENTS_GWZ.md` and applicable repository
`AGENTS.md`; `GwzLogPlan.md`; `GwzLogRequirements.md` (including the executed
S0.2 L-ENV-1..14 section); `GwzLogAmbiguityRezo.md`;
`GwzLog-S0.1-Review.md`; `AgentQuickStart.md`; `GwzCommitMarker.md`; then the
landed `GwzLog-S2.1-Review.md`, `GwzLog-S2.2-B-Review.md`, and
`GwzLog-S2.4-Review.md`. No stale core copy was used as the canonical log
requirements source.

The simple cases are good: the candidate streams cursor events, drives S2.4's
private assembler, uses an inclusive 60-second window, repeats valid marker
provenance on the checked-in 61-second and emitted-frontier splits, orders by
absolute `i64` committer instants, ignores offsets for ordering, applies the
least-sibling member/hash tie key, and implements the global post-coalescing
depth-policy values.

The state machine does not survive the harder envelope it owns. A time-closed
but output-blocked group remains mutable; adding a repository to an older group
can reverse that repository's cursor order; one timestamp inversion can retain
the whole no-limit input despite only one or two entries occupying any W-sized
time interval; path-backed readers start outside the jobs budget; and the cap
reads and reports failures beyond the requested depth before it stops. The
checked-in tests miss all five defects and the jobs/determinism test survives
deletion of the budget itself.

## Findings

### [P1 F1] A time-closed group remains mutable while output-blocked, so a frontier-late sibling is incorrectly absorbed

`take_emittable_groups` at `src/operation/commit_log/merge.rs:363-380`
removes a group only when it is both time-closed and unblocked by an earlier
same-repository group. A time-closed group that is not yet legal to emit remains
in `pending`. `admit_entry` at `merge.rs:303-315` does not distinguish that
sealed group from an open group and can mutate it later.

A disposable exact-candidate probe used:

```text
cursor A: unmarked a-blocker@0, marker M a-marker@100
cursor B: unmarked b-blocker@0, marker M b-marker@100
W=60, coalescing enabled, no cap
```

After `a-marker@100` is admitted, A is exhausted and B's current head is
`b-blocker@0`, strictly past the marker group's threshold 40. The marker group
is therefore time-closed, but it cannot emit because `a-blocker` precedes it in
A. The implementation leaves the group mutable, advances B through its
non-monotone inversion, and joins `b-marker@100`.

Actual: one two-member marker group. Required frontier behavior: two singleton
fragments, each retaining `marker:M`. Closure and output readiness are separate
states; a closed group must reject later membership even while waiting behind a
native-order predecessor.

This violates L-COA-7 and L-ENV-2's explicit frontier escape and repeated-key
rule.

### [P1 F2] Late group membership invalidates the fixed-vector precedence model and reverses a repository cursor

`admit_entry` mutates the first compatible group in place at `merge.rs:303-315`.
Per-repository precedence, however, is inferred solely from group creation
position: `blocked_by_earlier_group` checks only `groups[..index]` at
`merge.rs:395-399`. A group created first by repository B can later acquire an
A entry even though A already has a native predecessor stored at a later vector
position.

Confirmed probe:

```text
cursor A native order: unmarked a-prior@90, marker M a-late@100
cursor B:              marker M b@100
```

Actual group/hash order:

```text
[["b", "a-late"], ["a-prior"]]
```

The native-preserving order is:

```text
[["a-prior"], ["b", "a-late"]]
```

No emission frontier had yet been crossed and both marker siblings are within
W, so the group should remain coalesced; it must instead inherit A's predecessor
constraint. The current result reverses A's cursor. That is an S2.5 merge-state
failure under L-ENV-2's non-monotone envelope and regresses the already-landed
within-repository order invariant. The permitted L-ENV-2 exception is the
reason this cursor may depart from timestamp order; the candidate fails to
apply it and instead reverses A. The merge needs per-repository
sequence/predecessor state, or an equivalent precedence graph/repositioning
rule; group creation index is not sufficient.

### [P1 F3] One timestamp inversion can retain the whole no-limit history outside W

The newest-head selector at `merge.rs:294-301`, timestamp-only closure at
`merge.rs:351-360`, and fixed-index blockers combine to starve an old frontier
group while buffering arbitrarily many entries far outside its window.

Confirmed probe:

```text
cursor A: frontier@0, then 100 unique entries at
          10000, 9939, 9878, ... 3961 (61 seconds apart)
cursor B: one unique other-frontier@0
coalescing enabled, no cap, jobs=1
```

Only the two `@0` records ever share a 60-second interval; every later record is
more than W from its neighbor. Actual:

```text
max_buffered_entries = 102
```

That is the complete input. Increasing the post-inversion tail increases the
buffer linearly, so the implementation is O(history), not O(selected repos x
entries within W). The checked-in high-water test uses monotone cursors and
cannot expose this.

Ignoring a cursor already represented in a group is grouping-safe because S2.4
forbids same-repository siblings, but the two-cursor probe shows that change
alone is insufficient: the merge must also advance a closure blocker or define
an equivalent bounded frontier escape. If the literal phrase "every live
cursor" is intended to forbid both measures, L-COA-7/L-ENV-2 and L-PRF-1 are
not simultaneously implementable for the accepted non-monotone envelope and
the lane owner must clarify the contract before remediation.

This violates L-COA-7, L-PRF-1, and L-ENV-2.

### [P1 F4] `--jobs` does not bound production path-history readers

The budget is created from `options.jobs` at `merge.rs:143`, but
`stream_sources` creates one worker per selected repository at lines 145-157.
For production histories, each worker calls `history.messages()` before
entering `serve_cursor` at lines 108-110.

A path-backed `history.messages()` immediately spawns `git rev-list` in
`src/operation/commit_log/mod.rs:213-253`. The permit is acquired only later,
around the parent iterator's `next()`, at `merge.rs:175-179`. The child process
continues traversing history asynchronously while its worker is outside the
permit and blocked on pull/reply channels.

Thus `jobs=1` with N path-routed repositories can run N worker threads and N
repository-reading `git rev-list` children. `max_concurrent_reads` measures
only the small parent-side `next()` region and reports a compliant value while
the actual readers exceed the ceiling. A four-source disposable setup-barrier
probe confirmed all four pre-permit setups overlapped with `jobs=1` while the
stat remained `<= 1`.

`jobs > repositories` naturally bottoms out at the repository count. The
internal test constructor clamps zero to one; the production request path keeps
the existing `resolve_jobs` handling for an invalid wire-level zero. Neither
case cures the jobs=1 defect. L-PRF-2 fails.

### [P1 F5] The depth cap reads, emits failures from, and can block on history beyond the cap

After taking a known cursor head at `merge.rs:245-248`, the controller
unconditionally calls `advance_cursor` at line 249 before assembly, emission,
or a cap check. `receive_head` at lines 213-228 emits every encountered
degradation and keeps pulling until it obtains another entry or EOF. Only then
does the no-coalesce path emit the already-known entry and test the limit at
lines 251-261.

Confirmed probe:

```text
options: coalesce=false, max_entries=1, jobs=1
cursor:  Entry("visible", t=100), Degradation(HistoryUnreadable), EOF
```

Actual:

```text
pulls  = 3
events = [Degradation, Group("visible")]
```

A real cursor can produce exactly this shape when its first commit is readable
and its successor fails. A blocking second `next()` prevents the already-known
`-n 1` result from being emitted; channel drop cannot cancel the read because
the controller is synchronously waiting for its reply. Future strict
aggregation could also promote the beyond-cap degradation into a false command
failure.

Normal post-return cleanup is otherwise sound: rendezvous channels are bounded,
cursor drop disconnects blocked workers, scoped threads join, and the permit is
released by RAII. A worker panic disconnects the reply and is ultimately
re-propagated rather than channel-deadlocking. Those facts do not cure the
pre-cap synchronous read.

L-ENV-3 says reaching the cap terminates the walk. This implementation reads
and reports past it, so the row fails.

### [P2 F6] The jobs/determinism acceptance test is not mutation-tight or byte-complete

The named L-PRF-2/L-ENV-4 test at
`src/operation/commit_log/merge_tests.rs:267-284` asserts only
`max_concurrent_reads <= jobs`; it never proves overlap and never proves that
production path readers obey the budget. Deleting the permit acquisition
entirely leaves `max_concurrent_reads == 0`, and the named test still passes.

Its `fingerprint` at lines 333-347 retains only ordering timestamp, member id,
and commit hash. The helper at lines 304-311 discards all degradations. It does
not compare provenance, paths, parents, identities, offsets, messages,
encodings, or degradation order, so it is not evidence for byte-identical
results across jobs values.

The suite also lacks the sealed-but-blocked, late-membership precedence,
no-limit inversion high-water, path-backed jobs=1, and beyond-cap degradation
probes above. Empty input and ordinary single-repository input are structurally
safe; exact `i64::MAX`, jobs zero, worker drop/panic, marker-invalid, and
heuristic merge paths were inspected but are not pinned at the S2.5 layer.
Each P1 remedy needs its exact regression, and L-ENV-4 needs a complete event
comparison across 1, intermediate, and greater-than-repository job values.

## Owned-row matrix

| Row | Result | Evidence |
|---|---|---|
| L-COA-7 | **FAIL** | The simple `W=60` inclusive / `W=61` split passes, current-head closure uses `all`, and marker fragments repeat the UUID. F1 leaves a closed blocked group mutable; F3 defeats the bounded window. |
| L-ORD-2 | **PASS in isolation** | `compare_entries`/`compare_groups` order by absolute committer seconds and `group_tiebreak` uses the least sibling `(member_id, hash)`. F2 is scored under L-ENV-2 because it fails to preserve the cursor when exercising the only authorized non-monotone exception; no additional intentional ordering relaxation was found. |
| L-DEP-1 | **PASS** | Default 50, explicit positive N, zero/no-limit, post-coalescing counting, explicit-range lift, and `since`/`until` lift are correctly represented. Explicit N overrides a lift. F5 is termination behavior, not an output-count error. |
| L-PRF-1 | **FAIL** | There is no explicit history `collect`, channels are rendezvous-bounded, and ordinary monotone inputs stream. F3 retains the complete no-limit input despite O(1) entries in each W-sized time interval. |
| L-PRF-2 | **FAIL** | F4 starts path-backed readers outside the jobs permit. Results are structurally scheduling-independent for the checked simple cursors, but the production ceiling is false. |
| L-ENV-1 | **PASS** | Ordering uses signed `i64` epoch seconds; offsets remain carried but are ignored by comparisons. `abs_diff` and `saturating_sub` avoid overflow at the extremes, with no clamp or warning. The checked-in test covers `i64::MIN` and `i64::MAX - 1`; exact `i64::MAX` remains an acceptance-test gap, not a code defect. |
| L-ENV-2 | **FAIL** | The checked emitted-frontier marker case passes, but F1 fails when a closed group is output-blocked, F2 loses per-cursor predecessor state on a late join, and F3 is unbounded under an inversion. |
| L-ENV-3 | **FAIL** | Cap counting and seen-head sibling absorption work, and group records claim only members actually observed. F5 reads/emits/block-waits beyond the cap instead of terminating there. |
| L-ENV-4 | **FAIL acceptance** | No jobs-dependent semantic branch was found, but F6's partial fingerprint and surviving no-budget mutation do not establish byte-identical complete events. |

## S2.4 integration, state ownership, and edge audit

- `join_group` first enforces the inclusive 60-second span with overflow-safe
  `abs_diff`, then reuses `assemble_commit_log_groups`; no second marker or
  heuristic parser was created.
- Valid marker fragments outside W retain `marker:<uuid>`. Invalid claims
  remain singleton `marker-invalid`; marked commits do not enter heuristic
  groups; heuristic max/min checks remain S2.4-owned.
- The merge owns the cursor heads, pending groups, closure, ordering, cap, and
  workers. S2.4 remains finite and stateless.
- Offsets never enter an ordering/window comparison. Equal-time ordinary groups
  and coalesced groups use the required least-member/hash key.
- Degraded and empty cursors terminate independently. Ordinary cap return drops
  channels and joins workers; the inherited path-cursor failed-child reaping
  concern predates this delta and is not scored here.
- There is no hidden Rust-side whole-history collection. F3 is nevertheless a
  real pending-buffer whole-history path, and F4 permits asynchronous Git
  children to traverse outside the jobs budget.

## Scope, visibility, and LOC

The exact base-to-candidate diff is:

```text
A src/operation/commit_log/merge.rs        +512/-0
A src/operation/commit_log/merge_tests.rs  +449/-0
M src/operation/commit_log/mod.rs             +3/-0
M src/operation/commit_log/request.rs         +13/-1
M src/operation/commit_log/tests.rs           +30/-0
```

Production is `+526/-1`; test-only/module-test wiring is `+481/-0`; total
conservative churn is 1,008 lines against the plan's approximately 350-line
aspirational target. The overrun is not an automatic blocker. Inspection of
every added seam found no accidental feature scope: the request boolean is the
depth-lift input; group/cap/order logic is S2.5; and worker/channel/budget/stats
logic is the jobs/streaming machinery. The excess is material because the
bespoke state/concurrency seams contain F1-F5 and their tests do not constrain
them, not because the number itself crosses a hard limit.

Scope and visibility otherwise pass:

- no handler, output, filter implementation, protocol/schema/generated file,
  CLI, gwz-py, Cargo dependency/lockfile, checked-artifact, lifecycle,
  inventory, pin, `lib.rs`, or `operation/mod.rs` change;
- `merge` is a private child of private `operation::commit_log`;
- merge API/types are `pub(super)` and do not widen the operation seam;
- `CommitLogHistories::has_explicit_range` is effectively bounded by its
  `pub(super)` containing type;
- no public renderer, S2.6 behavior, or request handler was wired early.

## Identity and verification

Identity is exact and clean:

```text
base:           2214eace46b72915f76ab28e03e16716ce9d1a60
base tree:      f6ba0d2a30fdb8508e6f91fd4b1affa847617b39
candidate:      ba525e91c8b0363fcd59c33410635e7cc2781b55
candidate tree: d9d1ab2bc2450e3d3e5bb04ab39544a328f8fbd4
sole parent:    2214eace46b72915f76ab28e03e16716ce9d1a60
binary diff:    bfa409d29afa1d58995c0b0f5e5d0f98700a9edc46a50ac378bff91264d8fc66
stable patch:   7331a8b4ea92bc9c915b87bd618aafbfa7275f4d
```

The repository is non-shallow, has no replacements or grafts, and the core
worktree/index remained clean. Frozen-surface allowlist checks and `git diff
--check` passed.

Reviewer-run proportional evidence:

- `cargo test --locked --lib operation::commit_log::merge_tests -- --nocapture`
  — 8/8 passed.
- `cargo test --locked --lib operation::commit_log:: -- --nocapture` — 75/75
  passed.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --locked --all-targets` — exit 0.
- `CLIPPY_CONF_DIR="$PWD" cargo clippy --locked --all-targets --all-features --
  -D warnings` — exit 0.
- checked-artifact boundary — exit 0; 15 visible entries, 5 classified modules.
- release-boundary unit suite — 6/6 passed.
- per-commit lane gate for exact base to candidate — exit 0.
- Five disposable exact-source/state probes reproduced F1-F5; a disposable
  removal-of-budget mutation reproduced F6. The candidate worktree was never
  edited.

Builder broad evidence is useful but must not be called a green broad run:

```text
cargo test --locked, 597.6s:
  1,769 passed / 2 failed / 1 ignored
```

All 75 commit-log tests were green. The two failures were the unrelated,
interference-sensitive preservation matrices `root_ambiguity_matrix` and
`root_fault_matrix`, observed while another heavy checked-artifact compiler
matrix was also running. The builder then reran each exact test serialized:

- `root_ambiguity_matrix::every_root_phase_rejects_fresh_ambiguous_work_without_physical_execution`
  — 1/1 passed in 549.58 s.
- `root_fault_matrix::every_root_physical_and_successor_boundary_recovers_without_repeating_mutation`
  — 1/1 passed in 1,110.77 s.

The combined evidence supports the builder's interference diagnosis for those
two matrices; it does not turn the original broad run into a pass and does not
exercise the failing S2.5 states above. The reviewer did not repeat the
approximately 20-minute suite.

## Final decision and remediation gates

**NO-GO — round 1. Do not land or push `ba525e91c8b0363fcd59c33410635e7cc2781b55`.**

Round-2 remediation must, at minimum:

1. separate membership sealing from output readiness and retain repeated
   marker provenance for a sibling arriving after a sealed frontier;
2. track per-repository predecessor/sequence constraints when group membership
   grows;
3. prove the non-monotone no-limit high-water bound, or return the closure and
   performance contract to the lane owner if literal every-live-cursor closure
   makes it impossible;
4. place all production repository readers, including path-backed child
   lifetime/work, under the standard jobs ceiling;
5. stop before advancing beyond a satisfied cap and prove no beyond-cap
   degradation, block, panic, or strict-status pollution; and
6. add mutation-tight, complete-event determinism tests across jobs values and
   exact regressions for every finding above.

No core remediation belongs in this review lane.

---

## Round 2 — terminal final review

**Verdict: NO-GO — TERMINAL. S2.5 freezes.**

- **Integrated core base:** `20d7c4bea41d51983fa4a136b983bedb9ec017a6`
- **Rebased S2.5 commit:** `85525681048228bf4e7c95695c6d21742bdfd9db`
- **Remediation / reviewed HEAD:** `f165040207fbfa9d8ae7ab990b7bdf5df81a388a`
- **Normative CLI authority:** `64e159064a1d0a050fb4a63414e3d0a62fe67aa9`
- **Mode:** final round-2 review of the five round-1 P1 cures, the P2
  acceptance cure, rebase integrity, S2.5 envelope preservation, and the
  disclosed path-history tradeoff only; core read-only
- **Round-2 finding count:** 0 P0 / 1 P1 / 1 P2 / 0 P3

The complete 373-line round-1 report above was reread from exact CLI authority
`64e1590` before round-2 review action. The round-1 authority chain and its
executed S0.2 L-ENV-1..14 requirements remain controlling; no stale core copy
was substituted. This section is the terminal decision for the exact integrated
train. It does not open a wider third discovery round.

Four production cures are sound, and the jobs/determinism acceptance cure is
materially stronger. Sealed groups are no longer mutable, late group membership
inherits repository-local predecessors, all path-reader process lifetime is
inside the jobs budget, and both cap branches stop before their successor read.
The complete-event jobs test now forces real overlap and compares full event
values.

The non-monotone memory cure is incomplete. It bounds the checked one-sided
inversion only when the other cursor ends at its old frontier. Two simultaneously
live inverted cursors still make pending memory grow linearly with history at
constant W-density. Separately, the default-coalescing cap regression admits a
mutation that restores a beyond-cap successor read, although the reviewed
production code is correct.

### Round-1 cure matrix

| Round-1 finding | Round-2 result | Evidence |
|---|---|---|
| F1 sealed-but-output-blocked membership | **CURED** | `PendingGroup.sealed` is independent of readiness; `try_join` rejects sealed groups. The exact two-fragment marker regression passes with repeated provenance. |
| F2 late-membership repository precedence | **CURED** | Per-cursor last-group state and an acyclic predecessor graph carry native precedence through late joins. The exact probe and held-group variant pass. |
| F3 non-monotone no-limit high water | **NOT CURED** | The checked one-sided 20/200-tail regression is bounded, but the exact same-finding two-live-cursor extension grows from 23 to 103 buffered entries as tail depth grows from 20 to 100. See R2-F1. |
| F4 path readers outside `--jobs` | **CURED** | Path cursor construction is inert; each synchronous child is started, waited, and reaped inside a read permit. The production `jobs=1` path-reader test reports and observes one active reader. |
| F5 beyond-cap successor work | **CURED in production; acceptance incomplete** | Both merge branches check/return before advancing. The exact no-coalesce entry/degradation regression passes, but the coalescing test admits a branch-local reversion. See R2-F2. |
| F6 weak jobs/determinism acceptance | **CURED** | Jobs 1, 2, and 8 over four cursors force exact overlap 1, 2, and 4; reported concurrency matches, and full `Vec<CommitLogMergedEvent>` values, including degradation and byte-carrying entry fields, compare equal. |

## Round-2 findings

### [P1 R2-F1] Two live non-monotone cursors still retain history outside W

The F3 remediation adds `frontier_blocker_cursor` at
`src/operation/commit_log/merge.rs:431-462`. It advances a cursor that prevents
an unsealed predecessor from closing when a sealed descendant waits behind that
predecessor. This is sufficient for the checked regression at
`merge_tests.rs:357-381`: cursor A has the inverted tail, while cursor B ends
after its `@0` frontier.

It is not sufficient when both cursors remain live. A disposable exact-source
probe used unique, unmarked entries:

```text
cursor A: a-frontier@0, a-tail-0@10000, a-tail-1@9939, ...
cursor B: b-frontier@0, b-tail-0@10000, b-tail-1@9939, ...
coalesce=true, no cap, jobs=1
```

Every adjacent tail instant differs by 61 seconds. At each instant there are
only the A/B pair; time zero has only the two frontier entries. Messages and
hashes are member-qualified, so heuristic coalescing is impossible. Thus every
inclusive 60-second interval contains at most two entries regardless of tail
depth.

Exact results on `f165040` were:

```text
tail length 20:   max_buffered_entries = 23
tail length 100:  max_buffered_entries = 103
```

The 100-tail run emitted all 202 singleton groups, and each repository retained
its exact native sequence `frontier, tail-0, ..., tail-99`; correctness was not
traded for the high-water result. The scheduler instead chooses one old
unsealed root and drains the other live cursor until it passes that root's
threshold. Each drained tail group remains blocked behind that other cursor's
own `@0` predecessor, so the pending set grows with the tail. Switching roots
does not recover the already accumulated bound.

The probe's high-water assertion (`<= 6`) failed, as expected, with `short=23`
and `long=103`; 0/1 passed, 1 failed. A byte comparison confirmed the probe's
`merge.rs` was identical to the candidate; its only source delta was the added
test. This is the same round-1 F3 envelope, now exercised with two still-live
cursors, not a new third-round feature case.

The implementation remains O(history) for this accepted non-monotone input,
not O(selected repositories × entries within W). It violates L-COA-7,
L-PRF-1, and L-ENV-2.

### [P2 R2-F2] Default-coalescing cap termination is correct but not mutation-tight

The reviewed production code cures F5 in both paths. No-coalesce emits and
checks the limit before `advance_cursor` at `merge.rs:260-273`. Coalescing
admits, absorbs already-seen siblings, and force-emits at the satisfied cap
before the later advance at `merge.rs:276-293`.

The exact round-1 entry/degradation probe at `merge_tests.rs:184-206` is strong
but sets `coalesce=false`. The default-coalescing acceptance test at lines
143-181 permits `pulls <= 4`. A disposable branch-local mutation moved only the
coalescing `advance_cursor` from after its cap block to before it. All 12 merge
tests still passed.

In the existing three-cursor cap-one fixture, correct execution performs
exactly three prime pulls. The mutant adds one successor pull from the selected
cursor, totaling four, which the assertion explicitly admits. If that successor
is a degradation, the mutated reader emits it and pulls EOF; the helper also
reduces raw events to groups before checking membership. The code is presently
correct, so this is an acceptance P2 rather than a production P1. A protecting
test would assert exactly three pulls and inspect raw events with a
beyond-cap degradation or panic/block sentinel.

## Final owned-row matrix

| Row | Round-2 result | Evidence |
|---|---|---|
| L-COA-7 | **FAIL** | Inclusive W=60 joining, W=61 splitting, sealing, and repeated marker provenance pass. R2-F1 still retains an unbounded pending tail for two live inversions at constant W-density. |
| L-ORD-2 | **PASS** | Absolute committer seconds and least-sibling `(member_id, hash)` ties remain deterministic. The two-live probe preserves both native cursors; its timestamp escape is confined to L-ENV-2's explicit exception. No wider ordering relaxation was found. |
| L-DEP-1 | **PASS** | Global post-coalescing default 50, explicit positive N, zero/no-limit, explicit range/since/until lift, and explicit-N override are unchanged and pass. |
| L-PRF-1 | **FAIL** | Ordinary monotone and checked one-sided inputs stream without whole-history collection. R2-F1 makes pending memory linear in tail depth while entries within W remain constant. |
| L-PRF-2 | **PASS** | Every production reader setup/read and complete path-child lifetime is under the standard jobs budget; jobs greater than repository count bottom out naturally and results remain complete-event identical. |
| L-ENV-1 | **PASS** | Signed `i64` absolute instants, saturation at extremes, preserved offsets, and offset-independent ordering are unchanged. |
| L-ENV-2 | **FAIL** | F1 sealing and F2 predecessor cures pass, but the bounded frontier escape is incomplete for two simultaneously inverted live cursors. |
| L-ENV-3 | **FAIL acceptance** | The production cap implementation closes only observed siblings and performs no successor work after satisfaction. R2-F2 shows the default-coalescing regression is not mutation-tight. |
| L-ENV-4 | **PASS** | The forced-overlap test reaches exact ceilings for 1/intermediate/>repositories and compares complete event values across schedules. No jobs-dependent semantic branch was found. |

## Path-history process/CPU tradeoff

F4's cure deliberately replaces one persistent path-history `git rev-list`
child with a synchronous command for each pull:

```text
git rev-list --max-count=1 --skip=N ... -- <pathspecs>
```

The constructor now stores only the resolved OID plan, pathspecs, and skip
counter. `serve_cursor` holds the read permit across `next()`, and
`Command::output` waits for and reaps the child before releasing it. This makes
the jobs ceiling honest and removes persistent-child cleanup ambiguity.

The cost is real and explicitly accepted for this review axis: exhausting a
D-entry path cursor requires D+1 process starts, and repeated `--skip=N` may do
roughly quadratic traversal work for a deep unlimited path history. The
ordinary default cap bounds the requested result depth; explicit
range/no-limit path requests can be materially slower. Bare no-path deep
histories remain on the streaming libgit2 revwalk, so this tradeoff does not
cure R2-F1 and does not itself fail L-PRF-1 or L-PRF-2. It is recorded as
non-blocking technical debt, not hidden as free performance.

## Rebase, scope, and identity

The train is exact and linear:

```text
f165040207fbfa9d8ae7ab990b7bdf5df81a388a
  parent 85525681048228bf4e7c95695c6d21742bdfd9db
85525681048228bf4e7c95695c6d21742bdfd9db
  parent 20d7c4bea41d51983fa4a136b983bedb9ec017a6
20d7c4bea41d51983fa4a136b983bedb9ec017a6
  parent 2214eace46b72915f76ab28e03e16716ce9d1a60
```

Trees are respectively:

```text
20d7c4b  e9c81c4ad6393d1f0761d21e0d8480a6d20cb9ce
8552568  caf09fd573cf39ba24d9781547f112c66e5c343e
f165040  a33438c9e816481279e9ae9f9775c289af2e58c0
```

`range-diff` from original `2214eac..ba525e9` to integrated
`20d7c4b..8552568` shows only the expected S2.3 conflict resolutions: the
landed lock argument remains in `open_history(..., lock.as_ref())`, and the
test import context retains S2.3's removal of now-unused snapshot symbols while
adding the S2.5 merge imports. No S2.5 hunk is silently lost.

The remediation delta `8552568..f165040` is confined to four private
commit-log implementation/test files:

```text
src/operation/commit_log/merge.rs        +203/-101
src/operation/commit_log/merge_tests.rs  +251/-38
src/operation/commit_log/mod.rs          +114/-68
src/operation/commit_log/tests.rs         +42/-0
```

The larger state-machine rewrite is within the five required cures. No handler,
output, filter implementation, request protocol/schema/generated artifact,
CLI, S2.6 wiring, dependency, or lockfile changed. The merge remains a private
child; its API stays `pub(super)`; path-reader probes are test-only. Public
handler/output/filter/protocol surfaces remain untouched.

The base-to-final binary diff SHA-256 is
`ed6d318d3778b60c1a6d35a983385e28cc2f6041546bc7bd29c051f7a56a1431`;
the remediation-only binary diff SHA-256 is
`c1bed1c6f88d6b7fa02400e9805dc97583600f25302928517e66b5421fcbcef8`.
The repository is non-shallow, has no replacements or grafts, and `git fsck`,
`git diff --check`, worktree, and index checks pass. The core worktree remained
clean and read-only throughout both rounds.

## Round-2 verification

Reviewer-run proportional gates on exact `f165040`:

- `cargo test --locked --lib operation::commit_log::merge_tests -- --nocapture`
  — 12/12 passed.
- `cargo test --locked --lib operation::commit_log:: -- --nocapture` — 82/82
  passed.
- Exact disposable two-live F3 high-water probe — expected RED, 0/1 passed;
  `short=23`, `long=103`.
- Disposable coalescing-only pre-cap-advance mutation — unexpectedly 12/12
  merge tests passed, proving R2-F2.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --locked --all-targets` — exit 0.
- `CLIPPY_CONF_DIR="$PWD" cargo clippy --locked --all-targets --all-features --
  -D warnings` — exit 0.
- `protocol/.regen-venv/bin/python protocol/regen.py --check` — exit 0.
- checked-artifact boundary — exit 0; 15 visible entries, 5 classified
  modules.
- release-boundary unit suite — 6/6 passed.
- per-commit lane gates — exit 0 at both `8552568` and `f165040`.

The builder's exact complete pinned broad run is valid evidence, but it does
not contain the missing two-live probe:

```text
TAUT_PYTHON=$PWD/protocol/.regen-venv/bin/python cargo test --locked
exit 0 on f165040:
  lib:         1,778 passed / 0 failed / 1 ignored
  diff-render: 10/10
  protocol:    33/33
  publish:      9/9
  rename:       2/2
  doctests:     0
```

A preceding unpinned run was not green: it exited 101 after the lib
1,778/0/1 and diff-render 10/10 portions because ambient Homebrew Python lacked
`taut`. The exact pinned protocol rerun was 33/33 before the complete pinned
full run above. The builder also reported merge 12/12, commit-log 82/82,
fmt/check/clippy, protocol regeneration, checked boundary, the privacy/call-
graph aggregate including its 558.7-second compiler suite, release 6/6,
no-ff 7/7, and both per-commit lane gates all at exit 0. The reviewer did not
repeat the full approximately 20-minute suite.

## Terminal decision

**NO-GO. Do not land or push `85525681048228bf4e7c95695c6d21742bdfd9db`
or `f165040207fbfa9d8ae7ab990b7bdf5df81a388a`.**

R2-F1 leaves the original P1 memory envelope unsatisfied; R2-F2 leaves a
mandated cap acceptance seam unprotected. Under the final-review charter this
decision is terminal: S2.5 freezes, no round-3 remediation is authorized, and
the core remains unchanged. No landing or push was performed by the reviewer.
