# GWZ Log — S2.3 independent review, round 1

- **Date:** 2026-08-30
- **Mode:** independent single-axis review; core candidate held read-only
- **CLI normative authority / review base:**
  `19f1e3d471c70385f1069180b9552ab7cbfa3649`
- **Core sole parent / reviewed S2.2-B base:**
  `2214eace46b72915f76ab28e03e16716ce9d1a60`
- **Core S2.3 candidate:**
  `20d7c4bea41d51983fa4a136b983bedb9ec017a6`
- **Core worktree:**
  `/Users/owebeeone/limbo/gwz-log-worktrees/s2.3/gwz-core`
- **Scope:** S2.3 only — L-RNG-4 and the `--tagged` wording rider, plus
  regression of inherited operand, selection, tolerance, read-only/local-only,
  compatibility, and frozen-boundary contracts

## Verdict

# GO

**Finding count: 0 P0 / 0 P1 / 0 P2 / 0 P3.**

The candidate implements the one chartered lock-relative range as a private
core resolution feature. For a valid workspace lock, `+lock..HEAD` and
`+lock..` resolve each selected Git member from that member's own lock row,
degrade `@root` with a structured `LockEntryMissing` record, preserve missing
and commitless member rows as per-member degradations, and continue every
usable member. Detached HEADs work, unborn/unresolved right sides degrade, and
the existing default/strict overlay remains intact.

The compatibility split is correct: only a range-position snapshot endpoint
whose exact id is `lock` is treated as the lock pseudo-endpoint. Bare `+lock`
is not an implicit range and remains exact standalone access to a snapshot
literally named `lock`. Exact whole-token legacy snapshot matching still runs
before range parsing. Post-`--` plus-prefixed values remain literal paths.

The S0.1 F27 rider is cured accurately. The shared refusal now says
`GWZ '+'-prefixed operands`, which truthfully covers both `+snapshot` and
`+lock` without misnaming the latter.

No schema, protocol, generated artifact, inventory pin, public operation
surface, lock writer, CLI surface, handler/output path, or S2.5 behavior moved.

## Core identity and patch integrity

Topology is exactly linear:

```text
7e2cd3caa57d18cffdf00bf85c046ed3aa96e905
  -> 2214eace46b72915f76ab28e03e16716ce9d1a60
  -> 20d7c4bea41d51983fa4a136b983bedb9ec017a6
```

`HEAD^` is the exact named S2.2-B base, and the merge base of the base and
candidate is the base itself. The candidate has exactly one parent.

Trees:

```text
2214eace tree: f6ba0d2a30fdb8508e6f91fd4b1affa847617b39
20d7c4be tree: e9c81c4ad6393d1f0761d21e0d8480a6d20cb9ce
```

Patch identity:

```text
binary-diff SHA-256: e52b3afa1ade74f78e2603b54a05cb1d6bb0c361f8ce878ccc0b23a4d6ebcab3
stable patch-id:      ec06a88f002aac6aa3aad8942c163002262956ec
```

The repository is non-shallow and has no replacement refs or grafts. The
candidate worktree and index were clean before review, remained unmodified
throughout the authoritative checks, and were clean at the final identity
check.

## Diff, budget, and scope

The candidate changes exactly four mode-`100644` files:

```text
M src/diff/operands.rs
M src/operation/commit_log/mod.rs
M src/operation/commit_log/request.rs
M src/operation/commit_log/tests.rs
```

| Classification | Add | Delete | Conservative churn |
|---|---:|---:|---:|
| Production | 107 | 21 | 128 |
| Tests | 143 | 12 | 155 |
| **Total** | **250** | **33** | **283** |

The plan's `~150 LOC` estimate is aspirational, not a hard cap. The candidate
remains one bounded goal and below the program's `<500` target. Most of the
over-estimate is the acceptance matrix and reuse plumbing needed to keep
snapshot and lock endpoint semantics distinct.

Scope audit:

| Surface | Result | Evidence |
|---|---|---|
| L-RNG-4 production | **PASS** | Lock reading, workspace binding, per-member row resolution, and degradation lowering are confined to `operation/commit_log/request.rs`; the internal degradation enum gains one already-protocolled reason. |
| F27 tagged wording rider | **PASS** | One shared diagnostic string changes in `diff/operands.rs`; it accurately names all GWZ `+`-prefixed operands. |
| Schema / protocol / generated IR | **PASS — untouched** | No diff under `protocol/` or `src/protocol/`; the pre-existing wire reason `LockEntryMissing = 5` is reused, not moved. |
| Checked-artifact / merge lifecycle | **PASS — untouched** | No diff under `src/checked_artifact/` or `src/workspace_ops/merge/v1_lifecycle/`; the boundary checker is green. |
| Inventory / census / pins / dependencies | **PASS — untouched** | No Cargo, lockfile, manifest, inventory, checker, or pin diff. |
| Crate and operation public surface | **PASS — unchanged** | No `lib.rs` or `operation/mod.rs` diff. `operation::commit_log` remains a private child; the new internal enum variant is unreachable through a new public path. |
| Lock writes / conf-integrity marker | **PASS — absent** | Production calls only `artifact::read_lock`; no writer, staging, refresh, or mutation-lock seam is reachable. |
| Handler / rendering / S2.5 | **PASS — untouched** | No request dispatch output, grouping window, k-way merge, depth, jobs, renderer, or client surface change. |

## L-RNG-4 acceptance matrix

| Case | Result | Evidence |
|---|---|---|
| `+lock..HEAD`, distinct member pins | **PASS** | Each selected member looks up `lock.members[member_id]`; the fixture uses two different pins and obtains each repository's exact native `pin..HEAD` sequence. |
| `+lock..` open right side | **PASS** | Shared range parsing defaults the empty side to `HEAD`; the same fixture executes both spellings and receives identical per-member results. |
| Detached member HEAD | **PASS** | The app member is detached at its post-pin commit and contributes normally. |
| `@root` | **PASS** | A structured `LockEntryMissing` record is retained with member id `@root` and operand `+lock`; it never silently vanishes after routing. |
| Missing lock member row | **PASS** | Produces member-scoped `LockEntryMissing`, operand `+lock`. |
| Commitless lock member row | **PASS** | Produces member-scoped `LockEntryMissing`, operand `+lock`; no attempt is made to invent a pin. |
| Non-Git lock member row | **PASS** | Production rejects it through the same `LockEntryMissing` branch; a review-only exact-source probe passed. |
| Lock pin absent locally | **PASS** | The shared local resolver produces `RevisionUnresolved`, operand `+lock`, without stopping peers; a review-only exact-source probe passed. |
| Lock pin resolves, repository HEAD unborn | **PASS** | The right endpoint resolves as ordinary `HEAD`; failure becomes the existing benign `RevisionUnresolved(HEAD)` record and contributes zero entries. |
| Default aggregate | **PASS** | Observed lock degradations remain benign (`AggregateStatus::Ok`) without strictness. |
| Strict aggregate | **PASS** | `strict=true` promotes any observed lock degradation to `AggregateStatus::Failed`; with default selection the mandatory root record therefore promotes. |
| Valid peer survives another row's failure | **PASS** | The per-target plan retains valid member cursors while root/missing/commitless rows emit independent records. |
| Member pathspec routing | **PASS** | Operand validation runs before routing and degraded plans are explicitly retained; the lock fixture proves root degradation survives an app-only path. |
| Foreign workspace lock | **PASS** | `lock.workspace_id != manifest.workspace.id` is a typed request-level `SourceIdentityMismatch` refusal before member resolution. |
| Lock schema / manifest schema | **PASS** | Existing `LockArtifact::from_yaml` validation binds `gwz.lock/v0` to `gwz.workspace/v0`; S2.3 additionally binds the parsed lock's workspace id to the selected manifest. |
| Missing or malformed global lock artifact | **PASS as request refusal** | Review probes produced typed `IoError` / `ManifestInvalid`, never a panic or partial member plan. These are global artifact failures, distinct from a valid lock's missing root/member rows; foreign-artifact refusal is explicitly required. |

## Operand, compatibility, and selection matrix

| Contract | Result | Evidence |
|---|---|---|
| L-RNG-1 shared classification | **PASS** | Every pre-`--` leading-plus token remains revision-only through the existing classifier; no new classifier branch was added. |
| L-RNG-2 default HEAD histories | **PASS — unchanged** | No-operand requests never satisfy the lock-range predicate and do not read the lock through S2.3; the inherited attached/detached HEAD test remains green. |
| L-RNG-3 snapshot resolution | **PASS — unchanged** | Existing per-member snapshot, snapshot-range, snapshot-to-HEAD, root/missing degradation, and path-routing tests all pass; the `lock` collision fixture additionally pins standalone snapshot compatibility. |
| Literal plus after `--` | **PASS** | Explicit pathspecs bypass classification. The inherited `+notes` test passes; a review-only exact `+lock..HEAD` path probe also passed without reading a lock. |
| No bare implicit lock range | **PASS** | The pseudo test is gated on `ParsedRevisionArg::Range`. Review probing without a snapshot returned `SnapshotNotFound`, not lock-relative history. |
| Standalone snapshot id `lock` | **PASS** | With both a lock artifact and a snapshot named `lock`, bare `+lock` returns the snapshot's recorded commit, deliberately different from the lock pin. |
| L-RNG-6 exact-before-range | **PASS** | Stored exact whole-token IDs are accepted before range interpretation. A legacy snapshot whose entire id contains range punctuation therefore remains an endpoint rather than being forced into `+lock` semantics. |
| Ambiguous legacy boundaries | **PASS** | Existing adjacent/leading/trailing-dot typed-teaching matrices remain green; the candidate does not change their parser. |
| L-SEL-1 client selector surface | **N/A — untouched** | S3.1 owns the client surface. S2.3 adds no clap, Python, protocol, or handler option. |
| L-SEL-2 default root + members | **PASS** | Unchanged selection path; lock validation adds a root record rather than narrowing root away. |
| L-SEL-3 tagged refusal | **PASS** | Both `+snapshot` and `+lock..HEAD` return exact `InvalidRequest` text: `--tagged does not accept GWZ '+'-prefixed operands`. |
| Local-only refs / L-RNG-5 | **PASS** | Pin and `HEAD` resolution use local libgit2 only. Path-limited history retains both `GIT_OPTIONAL_LOCKS=0` and `GIT_NO_LAZY_FETCH=1`. |

## Inherited tolerance and read-only row matrix

| Row | Result | S2.3 regression assessment |
|---|---|---|
| L-TOL-1 | **PASS** | Valid lock artifacts lower row-level failures into independent repository events; no whole-plan member rejection was introduced. |
| L-TOL-2 | **PASS** | Missing/commitless/unresolved member pins are benign by default and strict-promoted through the existing overlay. |
| L-TOL-3 | **PASS** | Unborn repositories remain zero-entry degradations; lock validation cannot turn them into a mutation or network attempt. |
| L-TOL-4 | **PASS** | Detached HEAD is exercised directly in the lock-range fixture. |
| L-TOL-5 | **PASS** | Shallow/local history semantics are unchanged; lock pins are resolved only from locally available objects. |
| L-TOL-6 | **PASS** | `read_manifest` / `read_lock` do not invoke the conf-integrity gate. The existing damaged-integrity history test remains green. |
| Read-only | **PASS** | Static call audit found no write, stage, lock, fetch, or transport path; exact candidate status remained clean after all tests. |
| Streaming / S2.1 cursor behavior | **PASS** | A lock range only changes `WalkPlan.pushes/hides`; it does not collect history or change cursor ownership/order. |

## Mutation-tightness and adversarial probes

The checked-in L-RNG-4 matrix kills the two highest-risk semantic mutants in
independent disposable-source runs:

1. **Stop excluding range-position `lock` from snapshot artifact reads.**
   Result: focused L-RNG-4 tests exited 101; the missing/commitless fixture
   failed with a whole-request `SnapshotNotFound` instead of per-member lock
   degradations.
2. **Use one lock row for every member instead of indexing by member id.**
   Result: focused per-member test exited 101; `mem_lib` received `mem_app`'s
   pin and degraded `RevisionUnresolved` rather than producing its own
   post-pin entry.

The fixture's snapshot pin, app lock pin, app HEAD, and lib lock pin are all
deliberately distinct, so snapshot/lock confusion, bare-lock widening,
per-member collapse, and HEAD-as-pin substitutions cannot false-pass. The
root kind/operand assertions, routed-root assertion, missing/commitless kind
assertions, exact tagged message, and strict status assertions likewise pin the
observable branches rather than merely checking success.

A disposable detached review worktree added four probe tests without touching
the reviewed candidate. They confirmed:

- absent local pin -> `RevisionUnresolved(+lock)` and non-Git row ->
  `LockEntryMissing(+lock)`;
- literal post-`--` `+lock..HEAD` remains a path and never reads the lock;
- bare `+lock` without a matching snapshot is snapshot lookup, not an implicit
  lock range;
- missing and malformed global locks return typed request errors without a
  panic.

All four probes passed. The disposable source worktree and its 2.3 GiB build
target were removed after the run. The authoritative candidate worktree was
never edited.

Two small direct-test omissions are not graded findings: the checked-in suite
does not name an unresolved lock OID, and it does not create a legacy snapshot
whose literal whole id is `lock..HEAD`. The first traverses the already-tested
shared local resolver and passed the exact-source probe; the second is fixed by
the exact-match-before-range control flow already covered by the complete
L-RNG-6 matrices. Neither omission conceals an observed defect or leaves the
primary L-RNG-4 mutations alive.

## Commands and direct exits

Identity, cleanliness, and scope:

- Exact `HEAD`, `HEAD^`, `HEAD^^`, parent, merge-base, and tree checks — exit 0.
- Binary-diff SHA-256 and stable patch-id computation — exit 0.
- `git diff --name-status`, `--numstat`, `--summary`, and `--check` — exit 0;
  four expected files, no mode/rename/copy issue, no whitespace error.
- Frozen/schema/protocol/generated/Cargo/crate-root/handler path allowlist
  checks — exit 0.
- Final worktree diff, index diff, and porcelain-v2 status — exit 0; empty.

Focused semantic evidence, rerun personally at exact candidate:

- `TAUT_PYTHON=... cargo test --locked --lib l_rng_4_ -- --nocapture` —
  exit 0; 2 passed.
- `TAUT_PYTHON=... cargo test --locked --lib operation::commit_log::tests
  -- --nocapture` — exit 0; 44 passed.
- `TAUT_PYTHON=... cargo test --locked --lib
  tagged_comparison_rejects_missing_snapshot_and_open_range_operands
  -- --nocapture` — exit 0; 1 passed.
- `TAUT_PYTHON=... cargo test --locked --lib diff::tests::t_classify
  -- --nocapture` — exit 0; 11 passed.
- Disposable exact-source review probes — exit 0; 4 passed.
- Disposable reserved-lock snapshot-read mutant — expected exit 101; killed by
  the checked-in L-RNG-4 matrix.
- Disposable per-member-row-collapse mutant — expected exit 101; killed by the
  checked-in per-member fixture.

Proportional formal and boundary gates:

- `cargo fmt --all -- --check` — exit 0.
- `TAUT_PYTHON=... cargo check --locked --all-targets` — exit 0.
- `TAUT_PYTHON=... CLIPPY_CONF_DIR="$PWD" cargo clippy --locked
  --all-targets --all-features -- -D warnings` — exit 0.
- `bash scripts/checks/check_lane_commits.sh 2214eace... 20d7c4be...` —
  exit 0.
- `python3 scripts/checks/check_checked_artifact_boundaries.py --source src`
  — exit 0; 15 visible entries, 5 classified modules.
- `python3 -m unittest scripts/checks/test_release_boundary.py -v` — exit 0;
  6 passed.
- `cargo metadata --format-version 1 --locked --no-deps` — exit 0.

The approximately 21-minute full suite was deliberately not run. Builder and
landing evidence own it; this review used focused semantic, compiler, lane,
and frozen-boundary gates proportionate to the S2.3 delta.

## Final decision

**GO.**

`20d7c4bea41d51983fa4a136b983bedb9ec017a6` is approved as the exact S2.3
core candidate over sole base
`2214eace46b72915f76ab28e03e16716ce9d1a60`. No remediation round is required.
