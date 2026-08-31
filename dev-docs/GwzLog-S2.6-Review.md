# GWZ Log S2.6 round-1 independent review

**Verdict: NO-GO.**

- **Core base / sole parent:**
  `638cdcdeabc1dc272b28ce7387c4fbec1333edaa`
- **Core candidate:** `3efef9c6b9b70c264c80054feb6d1d4173fa0da8`
- **Normative CLI authority:**
  `b5e6a16044de02f0670887b04dfbcd71ecc63e02`
- **Mode:** independent, single-axis S2.6 round 1; core read-only
- **Rows graded:** L-FIL-1, L-COA-5, L-ENV-5, L-ENV-6, L-ENV-7, and
  the core per-member aggregate half of L-EXIT-1
- **Finding count:** 0 P0 / 0 P1 / 1 P2 / 0 P3

The production implementation is sound on inspection and under the reviewer's
adversarial probes. All six filters execute inside each repository cursor
before the cursor can feed the merge; raw-byte regex matching, inclusive
absolute-time bounds, survivor-only groups, empty success, native first-parent
and no-merge behavior, and the aggregate status implementation give the
required results. No output, handler, protocol, generated, public-operation,
or process-exit surface moved.

The round cannot pass because the required acceptance is not mutation-tight.
Four small, independent violations of the S2.6 contract still pass all 89
checked-in commit-log tests. The code does not contain those violations, so the
finding is P2 acceptance rather than a production P1. The cure is test-only and
fits the fresh round-2 charter without reopening S2.5-B or expanding S2.6.

Authority and instructions were read completely before review action, in the
mandated order: workspace `AGENTS_GWZ.md`; the applicable core and CLI
`AGENTS.md`; the full current `GwzLogPlan.md`; the full current
`GwzLogRequirements.md`; `GwzLogAmbiguityRezo.md`; the complete terminal
`GwzLog-S2.5-Review.md`; the complete successful `GwzLog-S2.5-B-Review.md`;
the current core `GwzCommitMarker.md`; and `AgentQuickStart.md`. No stale core
copy was substituted for the canonical log requirements.

## Finding

### [P2 F1] The S2.6 suite accepts wrong filter surfaces, ancestry behavior, and degradation classes

The implementation makes the required distinctions:

- bounds read `entry.committer.time.seconds` at
  `src/operation/commit_log/filter.rs:46`;
- `--author` constructs raw `entry.author` name plus ` <email>` at
  `filter.rs:57-63`;
- no-path histories invoke libgit2's first-parent simplification at
  `src/operation/commit_log/mod.rs:224-239` while path histories pass native
  Git `--first-parent` at lines 324-326; and
- `is_read_failure` at
  `src/operation/commit_log/aggregate.rs:120-127` classifies exactly
  unsupported-source, repository-unreadable, and history-unreadable as
  failures. Unborn, missing revision, missing snapshot entry, and missing lock
  entry remain benign degradations.

However, every checked-in S2.6 identity/time fixture creates the same author
and committer with the same instant. The native filter test at
`src/operation/commit_log/tests.rs:1566-1606` always supplies pathspec `.` and
HEAD, so it exercises only the external Git path reader, never the no-path
libgit2 branch or a range. The aggregate test at lines 1669-1738 distinguishes
one benign revision failure from one repository read failure, but does not
enumerate the seven degradation kinds. Earlier converted strictness tests add
lock-missing as benign; they still do not constrain snapshot-missing,
unborn-head, unsupported-source, or history-unreadable aggregate classes.

In a disposable exact-candidate worktree, each of these independent mutations
left the complete checked-in commit-log suite green, **89/89**:

1. compare `--since`/`--until` against the author instant instead of the
   committer instant;
2. match `--author` against the committer name/email instead of the author;
3. classify `SnapshotEntryMissing` as a read failure; and
4. delete no-path `walk.simplify_first_parent()` while leaving the path-backed
   Git flag intact.

All four mutations change contract-observable behavior. The first violates
L-ENV-6; the second violates L-FIL-1/L-ENV-5; the third turns a benign
per-member absence into aggregate failure under L-EXIT-1; and the fourth
silently drops `--first-parent` for the ordinary no-path cursor. This is not a
request for broad mutation infrastructure: four compact, direct fixtures kill
the demonstrated holes.

The same acceptance cluster leaves two mandated edge dispositions unpinned.
The checked time grammar covers one negative epoch, ordinary RFC3339/basic
offset form, local date/time conversion, inclusivity, invalid calendar input,
approxidate refusal, and locale/TZ independence. It does not cover `i64::MIN`,
`i64::MAX`, epoch overflow, or an offsetless local time in a DST gap/overlap.
The candidate correctly accepts both i64 endpoints, refuses overflow, and
refuses both ambiguous and nonexistent local times rather than guessing; an
exact process-pinned reviewer probe passed. Those results need a named
regression because the review checklist expressly includes overflow and DST
behavior.

The production code is correct under all of these probes. A disposable raw
commit with non-UTF-8 author bytes, a different committer identity, author time
10, and committer time 100 survived an exact `--author` raw-byte pattern plus
`--since=@100 --until=@100`. A table probe over every degradation enum produced
the intended four benign and three failed classes. Native Git parity passed for
both `--first-parent` and `--no-merges`, with and without pathspec `.`, over
`HEAD~2..HEAD`. These probes establish that the remedy is acceptance evidence,
not a production rewrite.

## Owned-row matrix

| Row | Result | Evidence |
|---|---|---|
| L-FIL-1 | **PASS implementation / FAIL acceptance** | `--since`, `--until`, `--author`, `--grep`, and `--no-merges` are entry filters inside each repository iterator; `--first-parent` configures that repository's revwalk before merge. Full-message/body and no-merge path parity pass. F1 shows author/committer identity and no-path/range first-parent are not pinned. |
| L-COA-5 | **PASS** | Filtering precedes `CommitLogMergedEvent`; the exact merge fixture removes only the root merge sibling and emits the surviving member singleton. Selection and filter narrowing therefore share the survivor rule. |
| L-ENV-5 | **PASS implementation / FAIL acceptance** | `regex::bytes::Regex` consumes raw message and combined `Name <email>` bytes, is case-sensitive, and returns an `InvalidRequest` naming the flag, Rust regex, pattern, and parser error before workspace access. F1's committer-identity mutation survives because fixtures do not distinguish the identities. |
| L-ENV-6 | **PASS implementation / FAIL acceptance** | RFC3339, extended/basic offset forms, local date-only midnight, local offsetless time, `@i64`, inclusive bounds, numeric/locale-independent parsing, and explicit approxidate teaching are implemented. F1's author-time mutation survives; i64 boundaries and DST disposition are not checked in. |
| L-ENV-7 | **PASS** | Filters are applied before merge; partially removed groups retain survivors; zero survivors emit no groups and aggregate `Ok`. Degradation events bypass filters. |
| L-EXIT-1 core aggregate half | **PASS implementation / FAIL acceptance** | Outcomes preserve every selected target as Empty/Contributed/Degraded/Failed. Default benign degradation is `Ok`; a read failure plus contribution is `Partial`; failure-only is `Failed`; strict promotes any degradation to `Failed`. F1 demonstrates incomplete degradation-class coverage. No process exit mapping was added. |

## Filter, merge, cap, ordering, and jobs audit

`RepositoryMessages::next` loops over rejected entries before returning an
event to its repository worker. Consequently rejected commits cannot enter the
S2.5-B assembler, predecessor graph, ordering comparison, group count, cap, or
aggregate collector. This is genuinely per-repository and pre-merge, not a
post-render filter disguised by fixtures.

The inherited global cap remains post-coalescing and still lifts only for an
explicit range or `since`/`until`; author/grep/ancestry filters do not silently
change that depth policy. A filtered cursor can read past rejected entries to
find its next survivor, but once the output cap is satisfied the preserved
S2.5-B sentinel still prevents any further cursor yield. Jobs permits surround
the complete `messages.next()` call, including all filtering work and every
path-reader child it may spawn, so filtering does not create an outside-budget
reader. No jobs-dependent filter branch exists, and the preserved complete-
event determinism test remains byte-equal across jobs values.

Marker provenance and grouping are untouched. Filtering a marked merge sibling
before assembly yields a singleton survivor with the valid marker provenance;
it cannot mutate or reopen an S2.5-B group. Empty filtered histories end
normally and are not reported as degradation.

## Regex and time audit

- Regex matching is over `Vec<u8>` without lossy conversion. A non-UTF-8 body
  is matchable using the regex crate's byte-mode syntax; `--grep` sees the full
  raw message regardless of the later `--body` renderer flag.
- The author surface is exactly raw name, ASCII ` <`, raw email, and `>`.
  Matching remains case-sensitive and locale-independent.
- Invalid patterns are invocation refusals before workspace discovery, so no
  repository or artifact access precedes the error.
- RFC3339 timestamps compare their absolute epoch seconds; their written
  offset does not enter the comparison. Date-only and offsetless forms use the
  process local timezone. `LocalResult::single()` deliberately refuses DST
  ambiguity and nonexistence, which avoids guessing.
- Time bounds are inclusive. Parsing and comparison use `i64`, accept
  pre-epoch/far-future values without clamping, and reject numeric overflow.
- The teaching error names RFC3339/ISO-8601, local/date-only availability,
  `@<epoch-seconds>`, and the explicit approxidate exclusion.

## Aggregate audit

The collector is initialized from the complete selected target list before any
streaming begins, so filtered-empty and naturally empty repositories remain
represented. A group marks only its surviving member entries as contributed.
Any degradation marks that target degraded or failed; failure dominates a
prior outcome. Default status is:

```text
no read failures                         -> Ok
read failure + another contributing repo -> Partial
read failure with no contributing repo   -> Failed
strict + any degradation                 -> Failed
```

That is the core-owned truth table S3.1 can consume. The candidate does not map
it to a process exit, does not construct a response, and does not change the
still-intentional handler stub. Empty filtered output remains `Ok`, not `Noop`,
as L-ENV-7 requires.

## Scope, dependencies, visibility, and LOC

The exact base-to-candidate diff is:

```text
M Cargo.toml                               +2/-0
M Cargo.lock                             +268/-0  generated dependency resolution
A src/operation/commit_log/aggregate.rs  +127/-0
A src/operation/commit_log/filter.rs     +113/-0
M src/operation/commit_log/merge.rs       +42/-2
M src/operation/commit_log/mod.rs         +69/-12
M src/operation/commit_log/request.rs     +17/-14
M src/operation/commit_log/tests.rs      +380/-29
```

Excluding generated `Cargo.lock`, handwritten churn is **+750/-57 = 807**:
production plus `Cargo.toml` is +370/-28 = 398, and tests are +380/-29 = 409.
This is about 3.2 times the plan's aspirational ~250 target, but it is not a
hard limit and is justified. The two direct dependencies are the named
contract choices (`regex` grammar and local/RFC3339 time handling); the private
filter is 113 lines, the complete aggregate is 127, and the remaining
production integration is narrow. No unnecessary abstraction, duplicate
parser, whole-history collection, or client behavior was found. F1 concerns
test discriminating power, not production bulk.

Scope and visibility otherwise pass:

- no handler, renderer, output registry, protocol/schema/generated artifact,
  command surface, gwz-py, checked-artifact, lifecycle, source inventory,
  `lib.rs`, or `operation/mod.rs` change;
- `filter` and `aggregate` are private children of the already-private
  `operation::commit_log` seam;
- the stream result, aggregate, outcomes, and collector are `pub(super)` only;
- no existing operation public surface widened; and
- dependency resolution contains only the expected chrono/regex transitive
  graph, with no unrelated package version churn.

## Identity and verification

Identity is exact and linear:

```text
base:           638cdcdeabc1dc272b28ce7387c4fbec1333edaa
base tree:      cc20f810fd38390c8fc62dd680c2fa85b75241e7
candidate:      3efef9c6b9b70c264c80054feb6d1d4173fa0da8
candidate tree: 32c5195f67ec61b28d5cdebc7a6381eae668878a
sole parent:    638cdcdeabc1dc272b28ce7387c4fbec1333edaa
binary diff:    67e40e82a4191df3760c4b11e56a0cf0732bae224b752555fafaf6bc892a8721
stable patch:   b01fec244fa69972114247c5675a39c718e62bfe
```

The repository is non-shallow, has no replacement refs or grafts, and
`git diff --check` passes. `git fsck --full` reports only existing unreachable
dangling objects, no integrity error. The candidate core worktree and index
remained clean and read-only throughout review. All review mutations and probes
ran in a separate disposable exact-candidate worktree and were restored or
removed there.

Reviewer-run proportional gates on exact `3efef9c6`:

- complete commit-log unit suite — 89/89 passed;
- four exact contract mutations above — each unexpectedly passed the same
  checked-in 89-test suite;
- raw distinct-author/committer + all-degradation-class probes — 2/2 passed;
- native range/path filter parity probe — all four first-parent/no-merge modes
  passed against `git rev-list`;
- i64-boundary/overflow and Sydney DST gap/overlap probe — passed;
- `cargo fmt --all -- --check` — exit 0;
- `cargo check --locked --all-targets` — exit 0;
- strict locked all-target/all-feature clippy with `-D warnings` — exit 0;
- pinned protocol regeneration/additive check — exit 0;
- checked-artifact source boundary — exit 0, 15 visible entries and 5
  classified modules; and
- release-boundary suite — 6/6 passed.

The builder's exact pinned complete run is accepted as broad evidence and was
not repeated:

```text
TAUT_PYTHON=$PWD/protocol/.regen-venv/bin/python cargo test --locked
exit 0 on 3efef9c6b9b70c264c80054feb6d1d4173fa0da8:
  lib:         1,785 passed / 0 failed / 1 ignored (790.33s)
  diff-render: 10/10
  protocol:    33/33
  publish:      9/9
  rename:       2/2
  doctests:     green
```

The builder also reported post-commit commit-log 89/89 and exit 0 for format,
locked all-target check, strict locked clippy, pinned protocol regeneration and
additive checks, source boundary 15/5, release 6/6, checked-artifact compiler
matrix 69/69, lifecycle privacy 8/8, and the exact final per-commit lane gate.
One direct system-Python additive invocation failed only because the ambient
Python lacked `taut`; the prescribed pinned-venv reruns and complete pinned
protocol target were green.

## Final-round remediation gates

Round 2 is final under the fresh S2.6 cap. A cure must remain test-focused and
satisfy all of these gates:

1. Add one raw commit whose author and committer have different names, emails,
   raw-byte content, and instants. Assert `--author` matches the combined raw
   **author** `Name <email>` and `--since`/`--until` compare the **committer**
   instant. The two exact identity/time substitutions above must turn RED.
2. Add native `git rev-list` sequence parity for both `--first-parent` and
   `--no-merges` over a nontrivial merge **range**, in both no-path and path
   history modes. Deleting the no-path `simplify_first_parent` call must turn
   RED.
3. Table-drive all seven degradation kinds through the aggregate collector,
   pinning the four benign versus three read-failure classes. Include default,
   strict, contributed+failed Partial, and failed-only truth-table assertions.
   Misclassifying `SnapshotEntryMissing` as failure and treating
   `HistoryUnreadable` as benign must both turn RED.
4. Pin `@i64::MIN`, `@i64::MAX`, both numeric overflows, and a process-TZ
   DST gap/overlap. The present fail-closed disposition for ambiguous and
   nonexistent local times must be explicit; no guessed instant is accepted.
5. Retain the survivor-only marker fixture and add a cap/order sentinel where
   enough leading commits are rejected that a pre-filter raw cap would lose a
   later survivor. Compare the filtered merged event sequence across jobs
   values so moving filters after merge/cap or adding a scheduling-dependent
   filter path turns RED.
6. Run the focused commit-log suite, each exact mutation, the proportional
   formal gates, and one exact pinned complete suite on the final candidate.
   Production may remain byte-identical if these acceptance cures suffice; no
   handler, output, protocol, public seam, or S3 behavior belongs in this
   round.

## Decision

**NO-GO for S2.6 round 1. Do not land or push
`3efef9c6b9b70c264c80054feb6d1d4173fa0da8`.**

The implementation itself satisfies the reviewed S2.6 contract, the LOC
overrun is justified, and scope/integrity/formal gates pass. F1 leaves several
normative distinctions vulnerable to regressions that the current green suite
cannot detect. One final, narrow test-remediation round is authorized; there is
no third round if the final review fails.

---

## Round 2 — terminal final review

**Verdict: GO.**

- **Required core base / sole parent:**
  `638cdcdeabc1dc272b28ce7387c4fbec1333edaa`
- **Round-1 candidate:** `3efef9c6b9b70c264c80054feb6d1d4173fa0da8`
- **Final core candidate:** `5a4f9cbe033805d8c54d78cc93b84f949ec429b5`
- **Round-2 CLI authority:**
  `733567708f872b36f49195ec051769440b7e92ad`
- **Mode:** terminal round 2; round-1 P2 acceptance cure and final integrity
  only; core read-only
- **Round-2 finding count:** 0 P0 / 0 P1 / 0 P2 / 0 P3

The round-1 P2 is cured. The final candidate leaves every reviewed production
and dependency blob byte-identical and changes only the S2.6 test module. The
new fixtures distinguish raw author from committer identity and time, exercise
native ancestry-filter parity over no-path and path-backed ranges, enumerate
all seven degradation classes and the aggregate truth table, pin epoch and DST
edges, retain valid marker survivor provenance, and prove filter-before-cap
ordering with complete events across jobs values. All eight prescribed
contract mutations now turn RED.

No in-scope production defect, scope expansion, integrity problem, or new
acceptance hole was found. This terminal review did not reopen S2.5-B or the
already accepted production implementation. S2.6 is eligible to land.

### Round-1 cure matrix

| Final-round gate | Terminal result | Evidence |
|---|---|---|
| Distinct raw author/committer surfaces | **CURED** | `l_env_5_6_filters_distinct_raw_author_and_committer_surfaces` writes one raw commit whose names and emails contain different non-UTF-8 bytes and whose author/committer instants are 10/100. It asserts decoded bytes first, then author-only, time-only, and combined filters. Author-time and committer-identity substitutions both fail. |
| Native ancestry-filter range parity | **CURED** | `l_fil_1_ancestry_filters_match_native_merge_range_with_and_without_paths` compares complete sequences against `git rev-list` for `--first-parent` and `--no-merges` over `HEAD~2..HEAD`, with no path and with `.`. Deleting libgit2 first-parent simplification adds the feature-parent commit and fails while the path branch remains native. |
| Seven degradation classes and truth table | **CURED** | `l_exit_1_aggregate_tables_every_degradation_kind_and_truth_class` pins Empty/Ok; all four benign classes as Degraded/Ok; all three read failures as Failed/Failed; every class under strict as Failed; contribution plus a failed peer as Partial; and failure-only through each failed-class row. Both named class mutations fail. |
| Epoch extremes and DST | **CURED** | `l_env_6_epoch_extremes_and_dst_gap_overlap_fail_closed` accepts exact i64 endpoints, rejects both overflows, and runs a fresh `TZ=Australia/Sydney` child that refuses the 2026 fall overlap and spring gap. Earliest-instant guessing and overflow clamping fail. |
| Valid-marker survivor provenance | **CURED** | The survivor fixture now uses a complete terminal trailer block with the workspace companion and asserts that filtering the root merge sibling leaves only `mem_member` while retaining `Marker(<uuid>)` provenance. |
| Filter before cap/order/jobs | **CURED** | `l_env_7_filtering_precedes_cap_and_order_for_every_jobs_value` puts rejected commits ahead of three survivors, caps at two post-filter entries, checks the exact survivor order, and compares complete merged events for jobs 1, 2, and 8. A raw/pre-filter cap or post-merge filter loses the later survivors and fails. |
| Eight exact mutations | **CURED** | Author-time, committer-identity, no-path first-parent removal, snapshot-as-failure, history-failure-as-benign, actual after-merge/cap filtering, DST earliest guessing, and overflow clamping each fail a named checked-in test. |
| Production and scope preservation | **CURED** | Cargo, dependency lock, aggregate, filter, merge, cursor, and request blobs are byte-identical to round 1. Final-minus-round-1 is only `tests.rs` +378/-2. |

### Mutation evidence

The builder applied each mutation independently to exact final source,
recorded the distinguishing failure, and restored the source byte-for-byte:

1. **Committer bounds → author time:** the distinct-time fixture loses its
   `@100` commit and fails.
2. **Author regex → committer identity:** the raw `Auth\xffor
   <author-\xfe@...>` pattern no longer matches and fails.
3. **Delete no-path first-parent simplification:** native no-path range parity
   returns three commits instead of two and fails; path parity remains intact.
4. **Snapshot-missing → failed:** the four-benign/three-failed table reports
   Failed where Degraded/Ok is required and fails.
5. **History-unreadable → benign:** the same table reports Degraded/Ok where
   Failed is required and fails.
6. **Move filtering after merge/cap:** the cap consumes rejected heads, loses
   required survivors, and fails the exact complete-event assertion across
   jobs values.
7. **DST `earliest()` guessing:** the Sydney overlap resolves to an instant
   instead of refusing and the child test fails.
8. **Overflow clamping:** an out-of-range epoch returns an i64 endpoint instead
   of a typed refusal and the boundary test fails.

The reviewer independently re-applied mutations 1-5, 7, and 8 and observed the
same RED distinctions. A direct bypass of the per-repository filter also failed
the cap/order/jobs test on the forbidden raw head hashes; the builder's stricter
actual after-merge/cap move produced the same lost-survivor class and is
accepted as the exact mutation-6 evidence. The disposable reviewer worktree
was restored and removed; the reviewed core worktree was never edited.

### Final behavior and fixture audit

The raw-identity fixture validates both the object bytes and filter result, so
it cannot false-pass merely because Git normalized the invalid UTF-8. The regex
uses byte-mode escapes for both name and email. Separate author-only and
time-only arms identify which surface failed; the combined arm pins their
composition.

The ancestry range is nontrivial: first-parent and no-merges each return two
commits but not by relying on the same filtering rule, and the complete hashes
are compared to native Git in both cursor implementations. The explicit
expected length prevents an accidentally empty native comparison from passing.

The aggregate table covers every current enum variant explicitly. Adding a new
degradation kind will force a deliberate test update rather than silently
inheriting a class. Strict status is tested for benign and failed classes;
default failed-only and contributed-plus-failed statuses distinguish Failed
from Partial. No process exit mapping is introduced.

The DST probe starts a new process with `TZ` set before `chrono::Local` is
used, avoiding process-global timezone cache ambiguity. It covers both local
failure modes: two possible instants and no possible instant. Epoch extrema use
the literal i64 parser boundary, and the immediately adjacent values prove
overflow is refused rather than saturated.

The filter/cap fixture's rejected prefix is longer than the requested output
depth across the participating cursor heads. Its expected hashes are the older
survivors, so a raw-entry cap cannot false-pass. Full
`Vec<CommitLogMergedEvent>` equality carries entries, identities, messages,
provenance, and degradation order across jobs 1/intermediate/greater-than-
repository values; the separate hash assertion pins output order.

The valid-marker survivor test now proves that per-repository filtering narrows
membership without downgrading proven marker identity. Empty filtering remains
`Ok`, and the existing exact S2.5-B cap-yield sentinel remains unchanged.

### Final integrity, scope, and LOC

The final candidate is an amended sibling of round 1, with exactly one parent:

```text
638cdcdeabc1dc272b28ce7387c4fbec1333edaa
├─ 3efef9c6b9b70c264c80054feb6d1d4173fa0da8  round 1
└─ 5a4f9cbe033805d8c54d78cc93b84f949ec429b5  final
```

Final identity:

```text
tree:             adb90bee92db924f0225c522c7c2289861b5fe8a
binary diff:      794845e13d150d31416beb8f6c3159d51f27f3fd63b92d03fd262c750272b6d2
stable patch:     98b458a865ff8cdb7a6324498e157bd1b96bde83
sole parent/base: 638cdcdeabc1dc272b28ce7387c4fbec1333edaa
```

A temporary-index replay of the complete binary patch from exact base wrote
tree `adb90bee92db924f0225c522c7c2289861b5fe8a`, exactly matching the final
commit. The repository is non-shallow with no replacements or grafts;
`git diff --check` is clean. The final core worktree and index remained clean.

Relative to round 1, only the test module changes:

```text
src/operation/commit_log/tests.rs  +378/-2
```

Every non-test blob in the round-1 candidate was compared by object id and is
identical in the final candidate, including `Cargo.toml`, `Cargo.lock`,
`aggregate.rs`, `filter.rs`, `merge.rs`, `mod.rs`, and `request.rs`. There is
therefore no handler, renderer, output registry, operation public seam,
protocol/schema/generated artifact, dependency, checked-artifact, lifecycle,
inventory, frozen-surface, or S3 behavior change.

The complete base-to-final handwritten delta, excluding generated
`Cargo.lock`, is production +370/-28 and tests +756/-29: **+1,126/-57 = 1,183
changed LOC**. This is larger than round 1's already justified 807 and the
aspirational ~250 plan target, but the round-2 increment is wholly the exact
acceptance matrix requested by the terminal gates. The helpers and fixtures
are direct, contain no product abstraction, and kill each named mutation. The
overrun remains non-blocking.

### Terminal verification

Reviewer-run evidence on exact `5a4f9cbe`:

- complete commit-log suite — 94/94 passed;
- mutations 1-5, 7, and 8 — independently RED with the expected distinction;
- direct per-repository filter bypass — RED on exact survivor hashes;
- `cargo fmt --all -- --check` — exit 0;
- `cargo check --locked --all-targets` — exit 0;
- strict locked all-target/all-feature clippy with `-D warnings` — exit 0;
- pinned protocol regeneration/additive check — exit 0;
- checked-artifact source boundary — exit 0, 15 visible entries and 5
  classified modules; and
- release-boundary suite — 6/6 passed.

The builder's exact pinned complete run is accepted as broad final evidence:

```text
TAUT_PYTHON=$PWD/protocol/.regen-venv/bin/python cargo test --locked
exit 0 on 5a4f9cbe033805d8c54d78cc93b84f949ec429b5:
  lib:         1,790 passed / 0 failed / 1 ignored (734.44s)
  diff-render: 10/10
  protocol:    33/33
  publish:      9/9
  rename:       2/2
  doctests:     green
```

The builder additionally reports focused 94/94 and exit 0 for format, locked
all-target check, strict locked all-target/all-feature clippy, pinned protocol
regeneration/additive checks, checked source boundary 15/5, release 6/6, and
the exact-base per-commit lane gate. All eight exact builder mutations were RED
with byte-exact restoration between runs.

### Terminal decision

**GO. S2.6 final candidate
`5a4f9cbe033805d8c54d78cc93b84f949ec429b5` is approved to land.**

The sole round-1 finding is cured, every final remediation gate passes,
production and scope integrity are exact, and there are no terminal-round
findings or conditions. This closes the S2.6 two-round review successfully;
no third round is needed or authorized. The reviewer did not modify, land, or
push core.
