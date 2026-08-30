# GWZ Log S2.2 — Round 1 Peer-Blind Review

**Verdict: NO-GO**

- Baseline: `14bd5acf01485a6a72922ff7527d9275f0877869`
- Candidate: `e541be4dc24fb117b17b535aa39e447dd33186e3`
- Candidate parent and merge-base: `14bd5acf01485a6a72922ff7527d9275f0877869`
- Worktree: `/Users/owebeeone/limbo/gwz-log-worktrees/s2.2/gwz-core`
- Review axis: S2.2 only — L-RNG-1 including pathspecs, L-RNG-3, L-SEL-3, L-TOL-2, exact diff, and inherited frozen/read-only constraints
- Finding count: **0 P0 / 2 P1 / 5 P2 / 2 P3**

The normal revision/range implementation, exact-tag narrowing, shared diff grammar, and ordinary per-member tolerance are substantially correct. The candidate nevertheless has two blocking safety failures and several material semantic gaps. Green focused tests do not cover those failures.

## Findings

### F1 — [P1] Path-limited history can fetch from a promisor remote and mutate the repository

`src/operation/commit_log/mod.rs:216-227` starts:

```text
git --git-dir <repo> rev-list <pushes> <hides> -- <pathspecs>
```

It sets `GIT_OPTIONAL_LOCKS=0`, but not `GIT_NO_LAZY_FETCH=1`.

An exact-form probe against a fresh `--filter=tree:0` partial clone produced:

- `git fetch origin`
- `git-upload-pack`
- a new promisor pack
- `git maintenance run --auto`
- packed-object count increasing from 1 to 2

`GIT_OPTIONAL_LOCKS=0` did not prevent any of this. This violates the inherited S2.1 guarantees of no network and no mutation, plus L-RNG-5.

The existing `l_rng_5_local_read_is_network_free_and_does_not_take_the_mutation_lock` test only exercises the no-pathspec libgit2 cursor over fully local objects. Its `FETCH_HEAD` check is also insufficient because lazy fetch uses `--no-write-fetch-head`.

**Remedy:** enforce `GIT_NO_LAZY_FETCH=1` or an equivalent guaranteed-offline mechanism on the subprocess. Add a promisor-clone regression that proves no transport helper runs, no object pack/ref changes, and an unavailable tree becomes a structured per-member degradation.

---

### F2 — [P1] Snapshot operand paths and identities are unbound, permitting directory escape and a process panic

The chain is:

- `src/diff/operands.rs:35-39` accepts any suffix following `+`.
- `src/diff/handle_diff.rs:560-575` passes that suffix to the snapshot reader.
- `src/artifact/mod.rs:389-390,534-535` constructs `<snapshot-dir>/<requested-id>.yaml` without first validating the requested ID.
- An absolute or parent-containing suffix can therefore make filesystem access escape the snapshot directory.
- The loaded artifact is not checked to ensure its embedded `snapshot_id` equals the requested filename.
- `src/operation/commit_log/request.rs:360-364` searches by the requested ID and calls `expect`.

Thus a valid snapshot-shaped file under a mismatched requested name loads, then a retained member causes the read-only log process to panic. Diff handles the corresponding lookup fallibly; log does not.

**Remedy:** before path construction, validate operand snapshot IDs with the canonical snapshot-ID grammar; bind the loaded artifact’s embedded ID to the requested ID; replace the `expect` with a typed error/degradation path. Add malformed, absolute/parent-containing, and filename/body-ID mismatch tests that prove no panic and no out-of-directory access.

---

### F3 — [P2] Foreign-workspace snapshots are accepted as current-workspace state

`open_request_histories` validates the request against the current manifest, but loaded `SnapshotArtifact.workspace_id` values are never compared with `manifest.workspace.id`. Resolution later matches only snapshot ID and member ID.

A copied snapshot from another workspace can therefore resolve silently when it has the same snapshot/member IDs and its recorded OID happens to exist locally. That is false snapshot identity under L-RNG-3.

**Remedy:** make the shared snapshot-loading seam accept and verify the expected workspace ID. Reject a foreign artifact before per-member resolution. Add a fixture with matching snapshot/member IDs and a locally present OID but a different `workspace_id`.

---

### F4 — [P2] Path routing silently removes mandatory snapshot degradation records

`src/operation/commit_log/request.rs:101-102` routes pathspecs before snapshot validation. `route_pathspecs` at lines 215-226 drops every target untouched by the pathspec. Only surviving plans reach `open_history` and `validate_snapshots` at lines 114-117 and 298-300.

Consequently, a default request shaped like:

```text
+snapshot -- member/file
```

silently removes `@root` before its required `SnapshotEntryMissing` record is created. Selected members absent from the snapshot but outside the routed path disappear the same way.

This contradicts L-RNG-3’s explicit rule that root/member snapshot absence is visible, not silent. Diff’s existing order reinforces the intended behavior: snapshot exclusions are recorded at `src/diff/plan.rs:209-275`, then pathspec intersection occurs at lines 292-295.

**Remedy:** validate snapshots across the selector-selected set first, preserve degradation plans/records independently, and path-route only ready histories. Add a combined snapshot-plus-member-pathspec test asserting member history and the root degradation record together.

---

### F5 — [P2] Repository-root `.` pathspecs lose native Git history semantics

`src/operation/commit_log/request.rs:229-235` converts any routed `"."` into an empty pathspec list. `src/operation/commit_log/mod.rs:155-157` interprets that empty list as “use the unfiltered libgit2 walk.”

For history, `git log -- .` is not equivalent to no pathspec. A probe with one empty commit produced:

```text
rev-list HEAD count:      1
rev-list HEAD -- . count: 0
```

This also loses companion exclusion pathspecs: if `"."` appears with an exclusion pattern, `normalize_pathspecs` discards the entire set. The normalization is valid for diff’s whole-repository file comparison, not native Git history simplification.

**Remedy:** reserve an empty vector exclusively for a request with no pathspec. Preserve routed `"."` and all companion pathspecs so they use `git rev-list -- ...`. Add root/member `"."`, empty-commit, merge-simplification, and `"."` plus exclusion tests.

---

### F6 — [P2] Valid dotted snapshot IDs collide with range syntax

`SnapshotArtifact::validate` permits adjacent dots because `require_slug` at `src/artifact/mod.rs:665-675` allows every ASCII dot. A snapshot such as `snap..one` is therefore valid.

`parse_revision_arg` at `src/diff/operands.rs:65-73`, however, applies `split_range` before endpoint classification. The operand `+snap..one` becomes a range from snapshot `snap` to revision `one`, making the valid snapshot `snap..one` unaddressable. Three-dot-containing IDs have the same contract collision.

**Remedy:** define one unambiguous contract shared by snapshot creation, diff, and log. The simplest option is to reserve `..` in snapshot IDs, with a compatibility audit; otherwise define an explicit escaping/spelling rule. Add creation/read/parser tests for range-looking IDs.

---

### F7 — [P2] Owned-row tests contain material false-pass seams

The new tests are useful but can still accept several incorrect implementations:

- The strict test at `src/operation/commit_log/tests.rs:619-648` passes literal `true` to `strictness_status`. It does not prove strict plus no degradation remains `Ok`, nor that an observed event supplies the degradation bit.
- `entry_ids` and `degradations` filter opposite event kinds. Tests for a degraded history would still pass if that history emitted both commits and a degradation.
- There is no log-level bare pre-`--` path test or revision/path ambiguity test; all log path-history tests supply `explicit_pathspecs`.
- L-SEL-3 lacks negative log tests for a tag absent everywhere and tags distributed across repositories with no all-tags intersection.
- The three-dot test has one merge base. It does not guard the requirement that all best merge bases are hidden; the current implementation correctly iterates `repository.merge_bases`, but a single-base substitution would pass.
- There is no ordinary-ref two-dot test with one endpoint missing in only one selected member.
- The F1-F6 combinations are all absent, explaining why the focused suite remains green.

Full event-derived aggregation remains S2.6’s responsibility; the S2.2 test should still cover the strictness overlay truth table and use the complete observed event sequence.

**Remedy:** add exact-sequence degradation assertions, strict/no-degradation coverage, bare classifier wiring tests, tag-negative tests, a criss-cross two-merge-base fixture compared with Git, and the focused regressions specified in F1-F6.

---

### F8 — [P3] Log operand errors tell users to invoke `gwz diff`

`src/operation/commit_log/request.rs:78` propagates shared-classifier errors whose suggestions at `src/diff/classify.rs:152-181` are hard-coded as:

```text
gwz diff [<revision>...] -- [<file>...]
```

An ambiguous or unknown `gwz log` operand therefore recommends the wrong command.

**Remedy:** retain one shared classifier but parameterize its diagnostic command/context so existing diff wording remains unchanged and log says `gwz log`. Add ambiguity and unknown-operand log tests.

---

### F9 — [P3] The step materially exceeds its aspirational LOC budget

The exact diff is:

```text
7 files changed, 1318 insertions(+), 118 deletions(-)
```

That is net +1,200 handwritten LOC, approximately:

- +698 implementation
- +502 tests

The adopted S2.2 target is approximately 350 LOC including tests, with a general aspirational target below 500. The budget is explicitly not a hard limit, and the changed files remain functionally related to S2.2, so this is not independently blocking. It does materially enlarge the review and regression surface; the duplicated request routing/process machinery contributed to multiple findings above.

One additional visibility nit is `RepositoryHistory::pathspecs`, a `pub` accessor used only by one test, although the containing module remains private.

**Remedy:** after correctness fixes, reduce or centralize planning/routing machinery where practical, narrow test-only accessors, or record an explicit re-budget justification.

## Requirement-row matrix

| Row / constraint | Result | Evidence |
|---|---|---|
| **L-RNG-1** | **FAIL** | Genuine shared classifier/range reuse exists, and normal A..B/A...B lowering is correct. Native `"."` history semantics fail (F5), valid dotted snapshot IDs collide with ranges (F6), and log-level classifier wiring lacks guards (F7/F8). |
| **L-RNG-3** | **FAIL** | Per-member snapshot, snapshot/snapshot, and snapshot/HEAD happy paths work. Unbound IDs can panic/escape (F2), workspace identity is unchecked (F3), path routing can erase required degradation records (F4), and valid dotted IDs are unaddressable (F6). |
| **L-SEL-3** | **PASS in implementation; test guard incomplete** | Each retained repository is checked against its exact local tag set; same-named branches do not satisfy it. `qualify_tag` centralizes the shared `+`-operand refusal and exact wording. Negative aggregate tests remain missing under F7. |
| **L-TOL-2** | **PARTIAL / FAIL for acceptance** | Ordinary refs resolve independently and emit structured `RevisionUnresolved` records; the strict engine parameter maps observed degradation to `Failed` without owning CLI exit mapping. Snapshot degradations can disappear under F4, F2 can panic instead of degrade, and the strict/degraded-event tests can false-pass under F7. |
| Shared `gwz diff` behavior | **PASS** | Log calls the real `classify_operands`; range splitting/default endpoints and tagged refusal are diff-owned shared primitives. Refactoring changes are `pub(crate)` and focused diff suites remain green. |
| A..B semantics | **PASS in code** | Right endpoint is pushed and left hidden at `request.rs:437-464`; snapshot two-dot tests pass. Add an ordinary-ref negative-member guard per F7. |
| A...B semantics | **PASS in code; test guard incomplete** | Both endpoints are pushed and every OID returned by `merge_bases` is hidden. Current test has only one merge base. |
| Leading `+` after `--` | **PASS** | Explicit pathspecs bypass operand classification and the existing literal `+notes` history test passes. |
| Exact local tags / `+` refusal | **PASS** | Uses shared `missing_exact_local_tags`, shared aggregate validation, exact `refs/tags/...` qualification, and centralized refusal wording. |
| Inherited no-network/read-only invariant | **FAIL** | Path-limited subprocess can lazy-fetch, write packs, and launch maintenance (F1). No conf-integrity gate or workspace mutation lock was introduced. |
| S2.1 default selection/cursors | **PASS except F1 path route** | Root plus active members, no-operand HEAD, detached, unborn, shallow, unreadable, and damaged-conf tests remain green. |
| Strict ownership boundary | **PASS structurally** | Core exposes a strictness parameter/overlay only. No clap flag, process exit mapping, handler production, or renderer was added. |
| Frozen inventories/pins/protocol | **PASS** | No diff under `src/checked_artifact/` or `workspace_ops/merge/v1_lifecycle/`; no `lib.rs`, Cargo, schema, generated protocol, census, or pin change. Boundary and protocol checks are green. |
| S2.3/S2.5/later scope | **PASS** | No `+lock`, k-way merge, depth, jobs, coalescing window, handler output, CLI, or renderer implementation. |
| Visibility | **PASS externally** | `operation::commit_log` remains private; shared reuse points are `pub(crate)` only. No public crate/API surface widened. |
| LOC/scope | **CONDITIONAL** | Files are relevant, but net +1,200 handwritten LOC is about 3.4× the adopted target (F9). |

## Shared-reuse and diff-regression assessment

The operand syntax is genuinely shared rather than copied:

- `request.rs:71-89` calls `classify_operands`.
- `parse_revision_arg` owns the common endpoint/range grammar.
- `parse_tagged_revision_args` and diff’s tagged parser share `qualify_tag`, including the required exact refusal string.
- Exact-local-tag inspection and aggregate intersection validation are shared from `src/diff/tagged.rs`.
- Snapshot reading and cwd resolution are shared from diff with `pub(crate)` visibility.

Existing `gwz diff` behavior did not regress in the focused suites. The shared snapshot reader itself needs the stronger ID/workspace binding identified above, but the candidate did not semantically duplicate the classifier.

## Scope and visibility assessment

Changed files:

```text
src/diff/handle_diff.rs
src/diff/mod.rs
src/diff/operands.rs
src/diff/tagged.rs
src/operation/commit_log/mod.rs
src/operation/commit_log/request.rs
src/operation/commit_log/tests.rs
```

Positive scope results:

- No `gwz-cli` or gwz-py work.
- `src/operation/commit_log/handler.rs` is unchanged and remains the S2.0 refusal.
- No machine/human output production.
- No `+lock`, merge, depth, jobs, or coalescing implementation.
- No new dependency or protocol change.
- No forbidden checked-artifact/lifecycle, inventory, pin, or crate-root diff.
- New cross-module helpers are `pub(crate)`; the commit-log module remains private.

The exact diff is therefore topically contained, but materially oversized and contains the correctness/safety issues above.

## Commands and direct exits

### Identity and diff

- `git rev-parse HEAD` — exit 0; `e541be4dc24fb117b17b535aa39e447dd33186e3`
- `git merge-base 14bd5acf01485a6a72922ff7527d9275f0877869 e541be4dc24fb117b17b535aa39e447dd33186e3` — exit 0; exact baseline
- `git diff --stat 14bd5acf... e541be4d...` — exit 0; 7 files, 1,318 insertions, 118 deletions
- `git diff --check 14bd5acf... e541be4d...` — exit 0
- Final `git status --short` — exit 0; empty

### Focused tests and compilation

- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" cargo test --locked operation::commit_log` — exit 0; 25 passed, 0 failed
- `cargo test diff::tests -- --nocapture` — exit 0; 88 passed, 0 failed
- Independent broader `cargo test --locked diff:: -- --nocapture` — exit 0; 105 passed
- `cargo fmt --all -- --check` — exit 0
- `cargo check --all-targets` — exit 0
- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" CLIPPY_CONF_DIR="$PWD" cargo clippy --all-targets --all-features -- -D warnings` — exit 0
- `cargo metadata --format-version 1 --locked --no-deps` — exit 0

### Boundary, frozen surface, and protocol

- `bash scripts/checks/check_lane_commits.sh 14bd5acf01485a6a72922ff7527d9275f0877869 e541be4dc24fb117b17b535aa39e447dd33186e3` — exit 0; boundary green at the exact candidate
- `PYTHONDONTWRITEBYTECODE=1 python3 scripts/checks/check_checked_artifact_boundaries.py --source src` — exit 0; `15 visible entries, 5 classified modules`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts/checks/test_release_boundary.py -v` — exit 0; 6 passed
- `PYTHONDONTWRITEBYTECODE=1 python3 protocol/regen.py --check` — exit 0
- `protocol/.regen-venv/bin/python protocol/check_log_additive.py` — exit 0; expected additive fingerprint
- Frozen-path/`lib.rs`/Cargo/protocol `git diff --quiet` checks — exit 0

### Semantic probes

Fresh repository containing one empty commit:

- `git rev-list --count HEAD` — exit 0; `1`
- `git rev-list --count HEAD -- .` — exit 0; `0`

Fresh `--filter=tree:0` promisor clone, using the candidate subprocess environment:

- `GIT_TRACE=1 GIT_OPTIONAL_LOCKS=0 git --git-dir <partial>/.git rev-list <commit> -- p` — exit 0
- Packed objects changed `1 -> 2`
- Trace showed `git fetch origin`, `git-upload-pack`, and `git maintenance run --auto`

Same shape with lazy fetch prohibited:

- `GIT_NO_LAZY_FETCH=1 git --git-dir <fresh-partial>/.git rev-list <commit> -- p` — exit 128 on the locally missing tree
- No fetch helper ran and packed-object count remained 1

### Non-evidence

A duplicate full-suite run was intentionally interrupted with exit 130 after the lane requested prompt consolidation. No full-suite result is claimed by this review, and it is not needed to establish or cure the findings above.

## Final decision

**NO-GO.**

S2.2 must not land at `e541be4dc24fb117b17b535aa39e447dd33186e3`. F1 and F2 are blocking safety/tolerance failures; F3-F7 are material owned-row or acceptance gaps. The P1/P2 findings trigger the plan’s required second-axis escalation. Round 2 should review the exact remediated candidate, with focused regressions for every finding and all standard gates rerun.

## Round 2 final peer-blind re-review

**Verdict: NO-GO — terminal under the fixed review cap**

- Baseline/parent: `dd31d54439e9244cba876d159383a5fc5e9584b2`
- Candidate: `8ec04f1d66f15ab21a99e04b73823b2845602bed`
- Merge-base: `dd31d54439e9244cba876d159383a5fc5e9584b2`
- Worktree: `/Users/owebeeone/limbo/gwz-log-worktrees/s2.2/gwz-core`
- Uncured finding count: **0 P0 / 0 P1 / 3 P2 / 0 P3**
- The three P2 dispositions comprise two independent semantic defects, F5 and F6, plus F7’s overlapping false-pass test guards.

F1–F4 and F8 are cured on the assigned S2.2 axis. F9 is accepted with the explicit scope/LOC justification below. F5 and F6 remain materially non-compliant, and their regressions expose an uncured F7 test-quality finding. Under the fixed cap, any uncured P0/P1/P2 is terminal.

### F1 — CURED [former P1]

Path-limited history now sets both:

```text
GIT_OPTIONAL_LOCKS=0
GIT_NO_LAZY_FETCH=1
```

at `src/operation/commit_log/mod.rs:218-230`. A failed offline `rev-list` becomes a target-scoped `HistoryUnreadable` degradation.

Independent real-promisor evidence:

- Fresh `--filter=tree:0` clone had a locally missing tree, one packed object, and one pack.
- Exact candidate subprocess shape with `GIT_NO_LAZY_FETCH=1` exited 128.
- Trace contained no fetch, upload-pack, index-pack, transport helper, or maintenance invocation.
- Full `.git` content digest, packs, objects, refs, `FETCH_HEAD`, commit graph, and multi-pack-index state were unchanged.
- A fresh negative-control clone omitting only `GIT_NO_LAZY_FETCH` exited 0, invoked fetch/upload-pack/index-pack/maintenance, and changed objects and packs from 1 to 2.
- The candidate regression emitted exactly one structured `HistoryUnreadable` event and byte-compared every regular `.git` file before and after.

No residual F1 issue was found.

### F2 — CURED [former P1]

The S2.2 log/diff paths now confine and bind snapshot operands:

- `artifact::snapshot_path` validates the requested ID before joining it to the snapshot directory.
- Empty, absolute, slash/backslash, parent-containing, and other non-portable IDs fail before filesystem access.
- `read_snapshot` verifies that the embedded `snapshot_id` equals the requested filename ID.
- Diff and log share the same loader.
- Log’s former snapshot lookup `expect` is gone; the fallback is a typed degradation rather than a panic.
- Malformed and mismatched artifacts produce typed errors, not raw I/O failures.

The implementation is cured. The “before access” regression itself remains a false-pass and is recorded under F7.

### F3 — CURED [former P2]

The shared diff/log snapshot loader now accepts the current manifest workspace ID and rejects an artifact whose `SnapshotArtifact.workspace_id` differs.

The regression uses the adversarial case required by round 1: matching snapshot and member IDs with a locally present OID but a foreign workspace ID. It is rejected before member resolution. Both diff and log pass `manifest.workspace.id` to the shared loader.

### F4 — CURED [former P2]

Snapshot validation now runs over the selector-selected plans before path routing at `src/operation/commit_log/request.rs:102-104`. `route_pathspecs` preserves any already-degraded plan at lines 230-244.

The combined regression proves that:

- `@root` retains its required `SnapshotEntryMissing` record;
- the path-routed ready member contributes history;
- an off-path selected member absent from the snapshot also retains its degradation record;
- target order is `@root`, routed member, missing member;
- degraded histories emit exactly one degradation and no entries.

### F5 — UNCURED [P2]

The original `.` normalization defect is fixed: root and member-root `.` now retain native empty-commit and merge-simplification behavior, and companion exclusions work when invoked at a repository root.

However, valid Git-magic exclusions are still corrupted when invoked from a subdirectory.

`request.rs:206-218` passes the complete raw token through filesystem-oriented `route_pathspec`. From `src/`, this native request:

```text
-- . :(exclude)artifact
```

is lowered to:

```text
rev-list <head> -- src src/:(exclude)artifact
```

The second token no longer begins with Git’s pathspec-magic signature, so it becomes an ordinary path and the exclusion silently disappears.

Independent exact-candidate probe:

```text
git -C src rev-list HEAD -- . ':(exclude)artifact'
```

returned 242 commits.

```text
git --git-dir="$(git rev-parse --absolute-git-dir)" \
  rev-list "$(git rev-parse HEAD)" -- src 'src/:(exclude)artifact'
```

returned 244 commits. Comparing the sequences exited 1. The two incorrectly retained commits were:

```text
8b9953bb36e92cd2234726c7ca7dfdc758ec7535  src/artifact/mod.rs
e1413a6d6665236055dd4244231e118f0af363f0  src/artifact/mod.rs
```

The correctly rerooted spelling, `:(exclude)src/artifact`, matched native Git.

**Required remedy:** make routing pathspec-magic-aware, preserving the magic prefix while rerooting only its pattern payload. Cover long and short exclusion forms, top semantics, root/member subdirectories, and workspace-root fan-out into member exclusions.

### F6 — UNCURED [P2]

The candidate reserves adjacent dots in snapshot IDs and preserves internal single-dot IDs such as `release.one`. That does not make the grammar disjoint.

`validate_snapshot_id` rejects only `value.contains("..")`; trailing single dots remain valid. `split_range` checks `...` before `..`. Therefore a valid snapshot ID `release.` cannot safely participate in L-RNG-3 ranges:

- Intended two-dot range `+release.` + `..` + `+tip` spells `+release...+tip`.
- The parser silently interprets that as the three-dot range `+release...+tip`, drops the trailing dot, and selects snapshot `release` instead.
- If both `release.` and `release` exist, this silently selects the wrong snapshot and wrong range semantics.
- Intended three-dot range from `release.` spells `+release....+tip`; the four-dot guards reject it as a range, after which it becomes one invalid snapshot endpoint.

Both diff and log share this defect.

The compatibility response is also incomplete:

- Released `v0.11.1` at `be693bdebbecd8208ffc61f3343f8185c06f7184` accepted adjacent-dot IDs through unrestricted portable-slug validation.
- The candidate retains schema `gwz.snapshot/v0`.
- There is no migration, schema change, or compatibility note.
- Existing adjacent-dot artifacts would now make `list_snapshots` fail.
- An independent scan found no adjacent-dot IDs among 24 discoverable local artifacts, but that cannot establish compatibility for already-deployed workspaces.

**Required remedy:** make endpoint and range spellings truly disjoint—at minimum reject trailing-dot IDs while retaining internal dots, or define an explicit escaping scheme. Add create/read/diff/log coverage for internal dotted IDs on both sides of `..` and `...`, trailing-dot rejection or escaping, and a documented compatibility path for previously valid adjacent-dot `gwz.snapshot/v0` artifacts.

### F7 — UNCURED [P2]

Most named round-one guards are now strong:

- entry/degradation helpers panic on the opposite event type;
- strictness covers all four strict/non-strict × degraded/clean cases using observed events;
- bare pre-`--` path and real path/revision ambiguity are covered;
- absent-everywhere and distributed/no-intersection tag cases are covered;
- the criss-cross fixture proves two best merge bases and compares the complete history with Git;
- ordinary two-dot mixed-member resolution asserts one exact degradation.

The overall finding remains uncured because required F1–F6 regressions still false-pass:

- The F2 “before access” test creates readable valid YAML at the escaped destinations and checks only the eventual error. A join/read-first implementation followed by validation would pass it.
- F5 tests invoke exclusions only at repository roots, missing the subdirectory corruption above.
- F6 tests cover internal `release.one`, not leading/trailing dots, range-boundary collisions, or end-to-end dotted snapshot ranges.

These gaps directly allowed both remaining P2 defects through a green focused suite.

**Required remedy:** add an access-order sentinel or injected read counter for F2, subdirectory/native-parity magic tests for F5, and boundary-dot plus end-to-end diff/log range tests for F6.

### F8 — CURED [former P3]

One contextual classifier is shared:

- `classify_operands` retains established `gwz diff` wording.
- Log invokes the same implementation with `gwz log`.
- Existing diff tests assert byte-exact ambiguous and unknown messages.
- Log tests require the `gwz log` synopsis and reject any `gwz diff` occurrence.

Focused diff behavior remained green.

### F9 — CURED for acceptance [former P3]

Exact diff:

```text
13 files changed, 2038 insertions(+), 173 deletions(-)
```

Independent handwritten-LOC split:

- Production: +902/-139, net +763
- Tests: +1136/-34, net +1102
- Total: net +1865

This remains about 5.3× the approximately 350-line S2.2 target. The plan explicitly makes that budget aspirational rather than hard. The overage is recorded and accepted here because it comprises the four-row request planner, shared classifier/snapshot safety seams, streaming native path-history cursor, and the adversarial remediation matrix.

The former accessor concern is cured: `RepositoryHistory::pathspecs` is now private and `#[cfg(test)]`. Cross-module helpers remain `pub(crate)`, `operation::commit_log` remains private, and no external crate surface was widened.

## Requirement-row matrix

| Row / constraint | Result | Evidence |
|---|---|---|
| **L-RNG-1** | **FAIL** | Grammar is genuinely shared; normal A..B/A...B, bare classifier wiring, and post-`--` leading `+` work. Subdirectory rerouting destroys exclusion magic (F5), and the snapshot/range language remains ambiguous at trailing-dot boundaries (F6). |
| **L-RNG-3** | **FAIL** | Per-member snapshot resolution, root/member degradation, snapshot/snapshot, snapshot/HEAD, identity binding, and path-routing visibility are cured. A valid trailing-dot snapshot cannot safely participate in required ranges (F6). |
| **L-SEL-3** | **PASS** | Exact local tags, all-tag intersection, same-named branch rejection, absent/distributed negative cases, and the shared exact `+snapshot` refusal are implemented and tested. |
| **L-TOL-2** | **PASS in implementation** | Ordinary mixed resolution degrades per member; exact events and the observed-event strict truth table are correct. Core exposes only the strict overlay, not the later CLI flag/exit mapping. |
| A..B semantics | **PASS except F6 boundary grammar** | Normal ranges push B and hide A; one-member endpoint failure degrades only that member. |
| A...B semantics | **PASS except F6 boundary grammar** | Normal ranges push both endpoints and hide every best merge base; a two-base criss-cross fixture matches Git. |
| Leading `+` after `--` | **PASS** | It bypasses operand classification and remains a literal path. |
| Native path history | **FAIL** | Root/member `.` and merge simplification are fixed, but valid subdirectory exclusion magic is lost (F5). |
| Snapshot confinement/body/workspace identity | **PASS** | Requested IDs are validated before joining, body IDs are bound, foreign workspaces are rejected, and no log panic remains. |
| Inherited no-network/read-only invariant | **PASS** | Real-promisor positive and negative controls establish that lazy fetch is disabled and repository bytes remain unchanged. |
| S2.1 selection/cursors/tolerance | **PASS** | Root plus active members, HEAD/detached/unborn/shallow/unreadable behavior, and no-conf-gate behavior remain green. |
| Shared `gwz diff` behavior | **PASS with shared F6 defect noted** | Classifier/tag/refusal implementations are genuinely shared and 106 focused diff tests pass; F6 affects the common grammar rather than representing duplicated log behavior. |
| Strict ownership boundary | **PASS** | No CLI flag, exit mapping, renderer, handler production, or output implementation was added. |
| S2.4 declarations after rebase | **PASS** | `coalesce.rs` is byte-unchanged; the 60-second admission declaration and assembly seam remain. The sole coalescing-test change supplies the new private `source_kind` field. |
| S2.3/S2.5/later scope | **PASS** | No `+lock`, k-way merge, jobs, depth, coalescing consumer, handler, or output creep. |
| Frozen inventories/pins/protocol | **PASS** | No checked-artifact/lifecycle, `lib.rs`, Cargo, protocol, inventory, census, or pin diff. |
| Visibility | **PASS** | Commit-log remains private; shared seams are no wider than `pub(crate)`; the test accessor is private. |
| LOC/scope | **ACCEPTED DEVIATION** | Net +1865 is materially above the aspirational target, with the production/test split and justification recorded above. |

## Commands and direct exits

### Identity and exact diff

- `git rev-parse HEAD` — exit 0; `8ec04f1d66f15ab21a99e04b73823b2845602bed`
- `git rev-parse HEAD^` — exit 0; `dd31d54439e9244cba876d159383a5fc5e9584b2`
- `git merge-base dd31d544... 8ec04f1d...` — exit 0; exact parent
- `git diff --stat dd31d544...8ec04f1d` — exit 0; 13 files, 2,038 insertions, 173 deletions
- `git diff --check dd31d544...8ec04f1d` — exit 0
- Final `git status --short` — exit 0; empty

### Focused tests

- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" cargo test --locked operation::commit_log -- --nocapture` — exit 0; 60 passed
- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" cargo test --locked diff:: -- --nocapture` — exit 0; 106 passed
- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" cargo test --locked artifact::tests:: -- --nocapture` — exit 0; 56 passed
- Exact F1 promisor regression — exit 0; 1 passed
- F7-focused commit-log guards — exit 0; 5 passed
- F8-focused diagnostic guard — exit 0; 1 passed
- Strict-overlay truth table — exit 0; 1 passed
- Exact tag and two-tree focused groups — exit 0

### Adversarial probes

- F1 exact offline promisor `rev-list` — exit 128 as required; no transport or mutation
- F1 negative control without `GIT_NO_LAZY_FETCH` — exit 0; fetch and pack mutation reproduced
- F5 native subdirectory `.` plus exclusion — exit 0; 242 commits
- F5 candidate-shaped subprocess — exit 0; 244 commits
- F5 sequence comparison — exit 1, proving loss of exclusion semantics
- F5 corrected magic-preserving spelling comparison — exit 0
- F6 split-grammar probe — exit 0:
  - `+release.one` → one snapshot endpoint
  - `+release.one..+tip.one` → two-dot range
  - `+release...+tip` → three-dot range from `release`, proving trailing-dot loss
  - `+release....+tip` → no range
- Released-contract audit — exit 0; `v0.11.1` used unrestricted portable-slug validation
- Local artifact scan — exit 0; 24 artifacts, no adjacent-dot IDs found

### Formal gates

- `cargo fmt --all -- --check` — exit 0
- `cargo check --all-targets` — exit 0
- `TAUT_PYTHON="$PWD/protocol/.regen-venv/bin/python" CLIPPY_CONF_DIR="$PWD" cargo clippy --all-targets --all-features -- -D warnings` — exit 0
- `cargo metadata --format-version 1 --locked --no-deps` — exit 0
- Exact lane-commit gate — exit 0
- Checked-artifact boundary — exit 0; 15 visible entries, 5 classified modules
- Release-boundary unit tests — exit 0; 6 passed
- `protocol/regen.py --check` — exit 0
- Additive protocol fingerprint — exit 0; expected fingerprint
- Frozen-path, handler, coalescer, Cargo, `lib.rs`, and protocol quiet checks — exit 0

The long full suite was deliberately not started; landing owns it.

## Final decision

**NO-GO — terminal.**

Do not land or push `8ec04f1d66f15ab21a99e04b73823b2845602bed`. F5, F6, and F7 remain uncured P2 findings, and the fixed round-two cap makes that terminal for this review charter. Further work requires lane-owner re-chartering or disposition, not another review round under this S2.2 cap.
