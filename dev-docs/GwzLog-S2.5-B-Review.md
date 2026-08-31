# GWZ Log S2.5-B round-1 independent review

**Verdict: NO-GO.**

- **Required S2.5-B base / sole parent:**
  `f165040207fbfa9d8ae7ab990b7bdf5df81a388a`
- **Core candidate:** `fee91150baee6161f003edb13efcc5022a02e40e`
- **Normative CLI authority:** `d25e66053013997fef42ee1480632066a5ca4f2d`
- **Mode:** independent S2.5-B round 1; exact delta only; core read-only
- **Finding count:** 0 P0 / 0 P1 / 1 P2 / 0 P3

The amended closure implementation is correct on inspection and under the
reviewer's behavioral probes. The two-live non-monotone high water is bounded
at 66, cap one stops after the three initial cursor yields, the delta is 145
changed handwritten LOC, and replay from the required base reproduces the
candidate tree exactly. The five retained S2.5 cures remain intact by bounded
diff.

The round cannot pass because the amended L-COA-7/L-ENV-4 acceptance is not
mutation-tight. Four direct regressions of the new per-cursor closure contract
still pass all 12 checked-in merge tests. The production code does not contain
those defects; this is one P2 acceptance finding with a compact test-only cure
available for the final round.

Authority and instructions were read completely before review action, in the
mandated order: workspace `AGENTS_GWZ.md`; applicable CLI and core `AGENTS.md`;
the full amended `GwzLogPlan.md`; the full amended `GwzLogRequirements.md`;
the complete terminal `GwzLog-S2.5-Review.md`; then S2.5-B's section 6 charter
again in its amended context. The terminal S2.5 candidate was treated only as
the explicitly preserved base, not subjected to a fourth broad review.

## Finding

### [P2 F1] The amended frontier regression is neither contract-complete nor mutation-tight

The production state machine implements the clarified L-COA-7 eligibility
rule:

- `GROUP_CLOSURE_PATIENCE_ENTRIES` is the private constant 64 at
  `src/operation/commit_log/merge.rs:11`.
- Each cursor retains the order-independent minimum entry instant ever seen at
  `merge.rs:237-247`; a newer head cannot erase evidence that the cursor has
  already crossed below a group's boundary.
- A newly pending group records its exact global yield serial at
  `merge.rs:351-369`. A successful later join changes membership and
  predecessor state, but does not reset that serial, at `merge.rs:372-397`.
- `cursor_closes_group` at `merge.rs:457-471` accepts exactly the four amended
  alternatives: the cursor is represented; exhausted; has ever seen an entry
  strictly below `newest - W`; or its retained 64-yield suffix is wholly newer
  than the group's opening serial.
- `seal_groups` requires that predicate for every cursor at `merge.rs:444-455`,
  and `closure_progress_cursor` reuses the same predicate at
  `merge.rs:473-500`.
- Sealing is monotonic. `try_join` rejects a sealed group at
  `merge.rs:372-383`, so a compatible entry arriving after closure forms a new
  fragment through the S2.4 assembler and retains the same valid marker
  provenance.

However, disposable exact-candidate mutations established that all of the
following contract violations still pass the complete checked-in merge suite,
12/12:

1. Replace the cumulative minimum with only the current entry's seconds.
2. Initialize `opened_at_yield` one yield late with `saturating_add(1)`.
3. Reset `opened_at_yield` whenever a compatible sibling joins.
4. Exempt marker-provenance groups from the K=64 closure alternative.

These are not equivalent formulations. Reviewer probes produced observable,
contract-relevant differences:

- **Monotone seen-below state.** With three same-marker cursors at times
  `[0, 0, 100]`, `[40, 0, 99]`, and `[39, 40, 100]`, production emits member
  groups
  `[[1-0,2-0,0-0],[2-1,0-1,1-1],[2-2],[0-2],[1-2]]`. The current-head-only
  mutant incorrectly reunites the final two entries as `[[0-2,1-2]]` after
  forgetting an earlier boundary crossing.
- **Exact K boundary and immutable provenance fragment.** A blocker primed
  before the target group opens joins at post-open yield 63, while the same
  marker arriving at post-open yield 64 must be a separate fragment. Production
  gives `[[a,b-late]]` for 63 and `[[a],[b-late]]` for 64, with both split
  groups carrying the same marker provenance. This kills the one-late opening
  serial and marker-only K exemption.
- **Opening epoch does not reset on join.** After one marker sibling joins a
  pending group late, production still closes on the original group's 64th
  post-open blocker yield. An epoch-reset mutant waits another patience period
  and absorbs a second late sibling. A pull-count form records production
  closure at 65 total blocker pulls (one pre-open prime plus 64 post-open),
  versus 128 for the reset mutant.

The existing high-water regression at
`src/operation/commit_log/merge_tests.rs:360-386` is useful but not sufficient.
It kills K `64 -> 63` (`65` rather than `66` buffered), K `64 -> 65` (`67`
rather than `66`), removal of patience (`103`), and removal of the represented
arm (`128`). Those coarser mutations do not exercise the opening epoch,
monotone historical crossing, or provenance-independent late fragmentation.

The long checked fixture also crosses its old frontier: at tail indices 163
and 164, `10_000 - index * 61` is 57 and -4 around the frontier at zero. Its
maximum W-local timestamp density is therefore four rather than the short
fixture's two. That does not hide a production defect: a disposable shifted
version preserving exact density two for tail lengths 100 and 1,000 emitted
all 202 and 2,002 groups and reported the same high water, 66. It does mean the
checked long case is not literally the terminal review's constant-density
probe shape and should be corrected while adding the missing mutations.

Because the implementation is correct and the defect is confined to required
acceptance evidence, this is P2 rather than P1. It nevertheless blocks round 1:
the S2.5-B charter expressly requires the amended frontier and high-water
regressions, not merely a code inspection of them.

## S2.5-B checklist matrix

| Chartered item | Result | Evidence |
|---|---|---|
| Amended L-COA-7: monotone seen-below-boundary | **PASS implementation / FAIL acceptance** | Production stores the cumulative minimum and compares it strictly below the saturated boundary. The current-head-only mutant passes all 12 checked tests; the three-cursor probe kills it. |
| Amended L-COA-7: exhausted cursor | **PASS** | `cursor.done` is an explicit closure arm; EOF and disconnected reply terminate the cursor. Removing the surrounding eligibility structure is covered by focused behavior. |
| Amended L-COA-7: represented cursor | **PASS** | Repository membership is an explicit closure arm. Removing it fails two checked tests, including high water 128. |
| Amended L-COA-7: K=64 yields since closure pending | **PASS implementation / FAIL acceptance** | Constant changes and total removal are killed, but an opening serial one late and resetting the serial on join both survive 12/12. |
| Amended L-COA-7: immutable closure / late fragment | **PASS implementation / FAIL acceptance** | Sealed groups reject joins and behavioral probes repeat marker provenance. Marker-only removal of the K arm survives the checked suite and incorrectly absorbs the late marker. |
| Amended L-ENV-4 high-water shape | **PASS behavior / FAIL acceptance** | Candidate high water is 66 for checked tails 100/200 and exact-density tails 100/1,000, with all groups emitted. The checked long fixture crosses the frontier, and the four direct frontier mutants survive. |
| Amended L-ENV-3 zero beyond-cap yield | **PASS** | Default-coalescing cap one yields exactly the three initial heads, emits no later degradation, and returns before `advance_cursor` at `merge.rs:309-321`. The exact pre-cap-advance mutant fails the raw-event sentinel and records five pulls. |
| Delta hard cap | **PASS** | `merge.rs` is +65/-27 and `merge_tests.rs` is +29/-24: 94 additions + 51 deletions = **145 changed handwritten LOC**, 55 below the hard cap of 200. |
| Required-base integrity | **PASS** | Candidate is a direct one-parent child of exact `f165040`; binary-patch replay from that tree reproduces the candidate tree byte-for-byte. |

## Preservation of the terminal S2.5 base

This matrix is a bounded diff-preservation audit, as chartered. It is not a
new review of the base surfaces.

| Preserved cure | Result | Delta evidence |
|---|---|---|
| F1 sealed groups | **PRESERVED** | `PendingGroup.sealed`, monotonic sealing, sealed-join rejection, and the exact regression remain. |
| F2 repository-native precedence | **PRESERVED** | Per-cursor predecessor creation, late-join inheritance, cycle guard, blocker relation, and the exact regressions are unchanged apart from passing the new admission serial. |
| F4 honest path-reader jobs ceiling | **PRESERVED** | `mod.rs` and its path-reader integration tests are blob-identical; constructor/read permit boundaries are outside the delta. |
| F5 cap-before-successor-read | **PRESERVED** | Both cap branches still return before `advance_cursor`; the no-coalesce regression is unchanged and the default-coalescing sentinel is stronger. |
| F6 actual overlap and byte-complete determinism | **PRESERVED** | The overlap gate, jobs 1/2/8 assertions, stats equality, and complete merged-event equality are unchanged. |

The F4 path-history tradeoff remains explicit and unchanged. A path cursor is
inert at construction; each requested entry starts and synchronously reaps one
`git rev-list --max-count=1 --skip=N` child while holding the read permit. This
makes the jobs ceiling and cleanup honest, but exhausting D path entries costs
D+1 process starts and repeated skip traversal can approach quadratic work for
a deep explicit no-limit history. Bare no-path history retains the libgit2
stream. S2.5-B neither worsens nor conceals this accepted base tradeoff.

## Scope, identity, and replay

The candidate is exact and linear:

```text
fee91150baee6161f003edb13efcc5022a02e40e
  parent f165040207fbfa9d8ae7ab990b7bdf5df81a388a
f165040207fbfa9d8ae7ab990b7bdf5df81a388a
  base tree a33438c9e816481279e9ae9f9775c289af2e58c0
fee91150baee6161f003edb13efcc5022a02e40e
  tree 98b310634bc190e97919fe7975e99d16b2241738
```

Core main `20d7c4bea41d51983fa4a136b983bedb9ec017a6` is the merge base and an
ancestor. The repository is non-shallow and has no replacements or grafts.
Replaying the candidate's binary patch against a temporary index loaded from
exact `f165040` produced tree
`98b310634bc190e97919fe7975e99d16b2241738`, exactly matching the candidate.

- **Stable patch ID:** `025c92b065d4fb5a78a592b16f0aea2599b82def`
- **Binary-diff SHA-256:**
  `3c37cddbb9d514072ee79bff45b48f8de38054196558370aedfdf25b1648a540`

Only the private merge state machine and its tests change. There are no
renames, mode changes, generated artifacts, dependencies, configuration,
protocol, CLI, public visibility, handler, output, filter, request, or S2.6
changes. `git diff --check` is clean; `git fsck --full` reports no errors beyond
unreachable dangling objects. The base and candidate core worktrees remained
clean and read-only.

## Verification

Reviewer-run proportional gates on exact `fee91150`:

- focused merge suite — 12/12 passed;
- complete commit-log suite — 82/82 passed;
- exact high-water probes — checked tails 100/200 and shifted exact-density
  tails 100/1,000 all emitted every group with high water 66;
- exact cap sentinel and its pre-cap-advance mutation — candidate passed; the
  mutant failed on the beyond-cap degradation and exact pull count;
- exact frontier mutations — all four unexpectedly passed the checked 12-test
  suite; the compact reviewer probes above killed them;
- `cargo fmt --all -- --check` — exit 0;
- `cargo check --locked --all-targets` — exit 0;
- strict all-target/all-feature clippy with `-D warnings` — exit 0;
- protocol regeneration check — exit 0;
- checked-artifact boundary — exit 0, 15 visible entries and 5 classified
  modules;
- release-boundary suite — 6/6 passed;
- exact per-commit lane gate from `f165040` through `fee91150` — exit 0.

The builder's exact pinned complete-suite evidence is accepted and was not
repeated:

```text
TAUT_PYTHON=$PWD/protocol/.regen-venv/bin/python cargo test --locked
exit 0 on fee91150baee6161f003edb13efcc5022a02e40e:
  lib:         1,778 passed / 0 failed / 1 ignored (889.61s)
  diff-render: 10/10
  protocol:    33/33
  publish:      9/9
  rename:       2/2
  doctests:     0
```

The builder also reported exit 0 for format, merge 12/12, commit-log 82/82,
check, strict clippy, protocol regeneration, checked boundaries, release 6/6,
no-FF wire 7/7, privacy aggregate (157.5s), call-graph/boundary compiler
mutation suite (766.0s), and lane gates at `8552568`, `f165040`, and
`fee9115`. This broad evidence is genuinely green on the exact candidate, but
it cannot detect tests that accept the four targeted mutations.

## Final-round remediation gates

Round 2 is the final S2.5-B review. A cure must remain within the same narrow
charter and satisfy all of these gates:

1. Add a compact seen-below-before-open case that fails when cumulative
   `oldest_seen` is replaced by current-head-only state. Pin the exact group
   sequence, not only high water.
2. Pin the exact K boundary relative to the original group-opening epoch: 63
   post-open yields remain joinable, 64 close the group, and a sibling joining
   in between does not restart the clock. Assert the distinguishing pull count
   or exact group sequence so both one-late initialization and reset-on-join
   mutations fail.
3. Exercise that K-closed late-fragment boundary with valid marker provenance.
   Assert two immutable fragments carrying the same marker, so a marker-only K
   exemption fails.
4. Shift or otherwise construct the long high-water tail so both compared
   lengths retain the terminal probe's exact constant W-density. Assert all
   groups emitted and an identical exact high water.
5. Run each of the four exact mutations and record RED against the checked-in
   tests, then run the focused/formal gates and exact pinned broad evidence on
   the final candidate.
6. Keep the cumulative `f165040..final` delta at or below 200 changed
   handwritten LOC, including tests. Reuse or compact the existing amended
   fixture rather than exceeding the remaining 55-line allowance. Preserve
   F1/F2/F4/F5/F6 by diff; do not widen production or public surfaces unless a
   newly demonstrated production defect requires it.

## Decision

**NO-GO for S2.5-B round 1. Do not land or push
`fee91150baee6161f003edb13efcc5022a02e40e`.**

Production behavior is consistent with the amended contract, the cap sentinel
is strong, and integrity/scope pass. F1 leaves the exact clarified frontier
semantics insufficiently protected and the checked long high-water fixture
slightly misses the terminal probe's constant-density shape. One final,
test-focused round is authorized. No core edit, landing, or push was performed
by the reviewer.

## Round 2 — terminal final review

**Verdict: GO.**

- **Required S2.5-B base / sole parent:**
  `f165040207fbfa9d8ae7ab990b7bdf5df81a388a`
- **Round-1 candidate:** `fee91150baee6161f003edb13efcc5022a02e40e`
- **Final core candidate:** `638cdcdeabc1dc272b28ce7387c4fbec1333edaa`
- **Round-2 CLI authority:** `ace5d30a26ca8b8bcafca3ac74ad0d1b75e40d3c`
- **Mode:** terminal round 2; round-1 P2 cure and final integrity only; core
  read-only
- **Round-2 finding count:** 0 P0 / 0 P1 / 0 P2 / 0 P3

The round-1 P2 is cured. The final candidate leaves the reviewed production
state machine byte-identical and replaces only its frontier regression. The
checked-in test now kills all four prescribed contract mutations, exercises
the exact historical boundary state and K epoch, proves same-marker immutable
fragmentation, and uses genuinely constant-density tails of 100 and 1,000
entries. The beyond-cap sentinel and all five preserved S2.5 cures remain
unchanged.

This terminal review did not reopen any preserved base surface and found no
new issue. S2.5-B is eligible to land.

### Round-1 cure matrix

| Round-1 remediation gate | Terminal result | Evidence |
|---|---|---|
| Monotone seen-below-before-open state | **CURED** | The exact three-cursor marker sequence is checked. Replacing the cumulative minimum with the current head incorrectly combines the final `0-2` and `1-2` entries and fails the assertion. |
| Exact K epoch: 63 joins, 64 closes | **CURED** | The checked fixture admits `b-late` on the 63rd post-open yield and fragments `c-late` on the 64th. Initializing the opening serial one yield late fails. |
| Joining a sibling does not reset K | **CURED** | `b-join` joins the pending marker group before the boundary; `c-late` still fragments at the original group's 64th post-open yield. Resetting the epoch on join fails. |
| Valid-marker late fragment retains provenance | **CURED** | The closed result is `[[a,b-join],[c-late]]`; both groups are marker groups built from the same valid `MARKER_A`. Exempting marker groups from patience incorrectly absorbs `c-late` and fails. The retained exact provenance regression also passes independently. |
| Exact-density high water | **CURED** | The two inverted cursors use frontier zero and tails beginning at 1,000,000 with 61-second spacing. Lengths 100 and 1,000 emit all 202 and 2,002 groups with identical exact high water 66 and maximum W-density two. |
| Four exact mutations recorded RED | **CURED** | Current-head-only state fails the exact seen sequence. Opening-serial `+1`, reset-on-join, and marker-only patience disable each produce the forbidden `[[a,b-join,c-late]]` merge and fail the checked test. |

The consolidated acceptance test is
`l_coa_7_frontier_eligibility_is_exact_and_bounded` at
`src/operation/commit_log/merge_tests.rs:360-404`. Its compact construction
keeps the cumulative base-to-final delta below the hard cap without weakening
the exact outputs.

### Final S2.5-B matrix

| Chartered item | Final result | Evidence |
|---|---|---|
| Amended L-COA-7: seen-below, exhausted, represented, K=64 | **PASS** | Production remains the round-1-correct implementation. The final fixture now distinguishes historical minimum from current head and pins the original opening epoch at both sides of K. |
| Amended L-COA-7: immutable closure and repeated provenance | **PASS** | Yield 64 seals before the late compatible marker is admitted; it becomes a second marker fragment. The marker-only K mutation is killed. |
| Amended L-ENV-4 high-water probe | **PASS** | Exact density-two tails 100/1,000 emit every group and remain flat at 66. |
| Amended L-ENV-4 mutation tightness | **PASS** | All four exact round-1 mutations fail the checked-in test independently. |
| Amended L-ENV-3 zero beyond-cap yield | **PASS** | The byte-unchanged cap regression still observes exactly three prime pulls and no beyond-cap degradation; the production return remains before successor advancement. |
| Cumulative hard cap | **PASS** | Base-to-final delta is 115 additions plus 54 deletions = **169 changed handwritten LOC**, 31 below 200. |
| Base/replay integrity | **PASS** | The final candidate is a direct one-parent child of exact `f165040`; temp-index replay produces its tree exactly. |
| F1/F2/F4/F5/F6 preservation | **PASS** | Production is byte-identical to round 1, and those five regressions plus the cap sentinel are unchanged. |

### Mutation evidence

The reviewer independently applied each mutation to an isolated disposable
copy of exact `638cdcde` and ran the focused merge suite:

1. **Current-head-only boundary state:** replace the cumulative `min` with
   `Some(current_seconds)`. The exact sequence fails because the final two
   singleton marker groups become one `['0-2', '1-2']` group.
2. **Opening epoch one late:** store
   `yield_serial.saturating_add(1)`. The K fixture fails because `c-late` is
   absorbed into `['a', 'b-join', 'c-late']`.
3. **Reset epoch on join:** update `opened_at_yield` whenever a sibling joins.
   The same forbidden three-member group appears and the fixture fails.
4. **Marker-only patience exemption:** disable the K arm for
   `CommitLogProvenance::Marker`. The same forbidden merge appears and the
   fixture fails.

After each isolated mutation the source was restored byte-for-byte. The
disposable worktree was removed. The reviewed core worktree remained clean.

### Final integrity and scope

The final candidate is a sibling amendment of round 1, not an additive child:

```text
f165040207fbfa9d8ae7ab990b7bdf5df81a388a
├─ fee91150baee6161f003edb13efcc5022a02e40e  round 1
└─ 638cdcdeabc1dc272b28ce7387c4fbec1333edaa  final
```

That topology is valid: `638cdcde` is exactly one commit ahead of the required
base, and core main `20d7c4bea41d51983fa4a136b983bedb9ec017a6` remains its
merge base and ancestor. The production `merge.rs` blob at both `fee91150` and
`638cdcde` is
`f56f7c950dd2b0ca5b85844f52d95a0646d52fbc`. Relative to round 1, only
`merge_tests.rs` changes, +33/-15.

Cumulative scope from exact `f165040` is:

```text
src/operation/commit_log/merge.rs        +65/-27
src/operation/commit_log/merge_tests.rs  +50/-27
total                                    +115/-54 = 169 changed LOC
```

All changed lines are handwritten. There is no generated, dependency,
protocol, public or `pub(super)` visibility, mode, rename, handler, output,
filter, request, CLI, or S2.6 change. The path-history CPU/process-startup
tradeoff recorded in round 1 is unchanged and outside this test-only cure.

Final identity:

- **Tree:** `cc20f810fd38390c8fc62dd680c2fa85b75241e7`
- **Cumulative stable patch ID:**
  `f00b456bc8ac1346864927388503c69544971a60`
- **Cumulative binary-diff SHA-256:**
  `443650c3afbb614ce0a1cd6098d22c847ff2144e0f872504726224054eddf37c`

Temp-index replay from exact `f165040` reproduced the final tree byte-for-byte.
The repository is non-shallow with no replacements. `git diff --check` is
clean. Both base and final core worktrees finished clean and read-only.

### Round-2 verification

Reviewer-run gates on exact `638cdcde`:

- consolidated L-COA-7 acceptance — 1/1 passed;
- focused merge suite — 12/12 passed;
- complete commit-log suite — 82/82 passed;
- four exact disposable contract mutations — each failed the consolidated
  acceptance as required;
- exact cap sentinel — 1/1 passed with three prime pulls and no later event;
- `cargo fmt --all -- --check` — exit 0;
- `cargo check --locked --all-targets` — exit 0;
- strict all-target/all-feature clippy with `-D warnings` — exit 0;
- protocol regeneration check — exit 0;
- checked-artifact boundary — exit 0, 15 visible entries and 5 classified
  modules;
- release-boundary suite — 6/6 passed.

The builder's exact pinned complete run is accepted as broad evidence and was
not repeated:

```text
TAUT_PYTHON=$PWD/protocol/.regen-venv/bin/python cargo test --locked
exit 0 on 638cdcdeabc1dc272b28ce7387c4fbec1333edaa:
  lib:         1,778 passed / 0 failed / 1 ignored (742.78s)
  diff-render: 10/10
  protocol:    33/33
  publish:      9/9
  rename:       2/2
  doctests:     0
```

The builder also reported exit 0 for the exact acceptance, merge 12/12,
commit-log 82/82, format, locked all-target check, strict locked clippy,
protocol regeneration, checked boundary, release 6/6, no-FF wire 7/7, privacy
aggregate (95.1s), call-graph/boundary compiler mutation suite (555.5s),
post-commit format/merge checks, and lane gates at `8552568`, `f165040`, and
`638cdcd`. The builder independently recorded the same four mutation failures,
constant-density group counts and high water, production blob identity, LOC,
replay tree, and binary-diff digest.

### Terminal decision

**GO. S2.5-B final candidate
`638cdcdeabc1dc272b28ce7387c4fbec1333edaa` is approved to land.**

The sole round-1 finding is cured, all terminal gates pass, and there are no
round-2 findings or conditions. This closes the two-round S2.5-B review with a
successful verdict; no third round is needed or authorized. The reviewer did
not modify, land, or push core.
