# GwzLog S2.0 Independent Review — L-PRO-1

## Review identity

- **Axis:** L-PRO-1 only
- **Mode:** independent, peer-blind
- **Date:** 2026-08-29
- **Workspace state:** both candidate worktrees clean before and after checks
- **Excluded:** no plan, requirements, ambiguity-resolution, or prior GwzLog review documents were read

## Exact review tuple

| Repository | Base | Candidate | Relationship |
|---|---|---|---|
| `gwz-core` | `2a3297da16a5d3cd814619cb2b3d7d15223640a7` | `eb7740efd151302f37a930b44979539142498d33` | Candidate has base as its sole parent |
| `gwz-py` | `5f6689a30741f35c943839a6ead36582e6452a4b` | `381602ed177bd64ffdec0de72763d7e1e29a3621` | Candidate has base as its sole parent |

## Verdict

**NO-GO**

The protocol design, additive-wire proof, generated parity, regeneration gates, dispatch behavior, and focused tests are green. One explicit scope constraint remains violated: the approved `src/operation/commit_log/` seam is exposed crate-wide rather than with minimum necessary visibility. That is a [P2] blocker.

## Finding counts

| Priority | Count | Blocks |
|---|---:|---|
| P0 | 0 | Yes |
| P1 | 0 | Yes |
| P2 | 1 | Yes |
| P3 | 0 | No |
| **Total** | **1** | **1 blocking** |

## Findings

### [P2] The commit-log engine seam is broader than its only caller requires

Locations:

- `gwz-core/src/operation/mod.rs:3`
- `gwz-core/src/operation/commit_log/mod.rs:3`
- `gwz-core/src/operation/commit_log/handler.rs:12`
- Only production caller: `gwz-core/src/operation/push_event.rs:713`

The new module is declared `pub(crate)`, its handler is wildcard-re-exported `pub(crate)`, and the handler itself is `pub(crate)`. This makes the unfinished engine seam callable throughout the crate. The only production caller is a sibling under `operation`, so crate-wide visibility is unnecessary and contradicts the stated minimum-visibility constraint. The wildcard also allows future handler symbols to leak crate-wide accidentally.

**Remedy:** keep the public dispatch function at `operation::handle_log`, but make `commit_log` private to `operation` and expose only a parent-scoped handler facade. For example:

- Change `pub(crate) mod commit_log;` to `mod commit_log;`.
- Replace the crate-wide wildcard re-export with one narrowly named `pub(super)` facade in `commit_log/mod.rs`.
- Keep `handler` and its implementation no more visible than needed by that facade.

No `lib.rs`, inventory, pin, generated artifact, or wire change should be necessary.

## L-PRO-1 assessment

### Message shape: pass

`protocol/gwz.taut.py:129-138` defines:

- Unary inbound `log(LogRequest) -> LogResponse`.
- Outbound `log.output(log_id)` with `shape="log"` and `LogOutputRecord` append values.
- `LogResponse` contains the response envelope and mandatory opaque log handle, not a page of entries.
- Cursor, tail, EOF, retention, close, and backpressure remain owned by the existing taut log-shape contract.

This is coherent with the existing `diff.output` precedent and avoids inventing a second paging protocol.

### Degradation form: pass

`LogOutputRecord` carries a stable `entry`/`degradation` discriminant. `LogDegradation` supplies repository identity, logical path, optional source kind, a machine-stable reason enum, optional operand, and optional human detail. Entries and degradations share one ordered stream, so partial-history diagnostics are not detached from the results they qualify.

The generated wire type permits mismatched optional payloads, as does the established `DiffOutputRecord` pattern. The schema explicitly assigns the exactly-one invariant to the future emitter; there is no log emitter in this step, so this is not an S2.0 defect.

### Additive-only wire growth: pass

Independent canonical-IR comparison established:

- Base: 59 enums, 125 messages, 28 service methods.
- Candidate: 62 enums, 134 messages, 30 service methods.
- Removing only the three new `Log*` enums, nine new `Log*` messages, the two new methods, and `ActionKind.log=26` makes the candidate IR structurally identical to the base IR.
- Both canonical projections hash to  
  `d0c205c8767f8d54d32ead2f676a05077d849f6a12278d9de52b3c132c3c9372`.
- All 125 pre-existing golden CBOR vectors are unchanged.
- Nine golden vectors were added; none were removed.
- `ActionKind.merge` remains 25 and `ActionKind.log` occupies the next slot, 26.

The additive guard is non-vacuous: an in-memory mutation of the legacy `WorkspaceRef.root` tag changed the projection hash to `892bdf4fd948ceb9f7306e926b0443dca7d2d1dd876a68307fd74b3aac380c46`.

### Generated parity: pass

- `gwz-core/src/protocol/generated.rs` contains the new enums and nine message codecs.
- `gwz-py/src/gwz/protocol/generated/api.py` contains corresponding enums/dataclasses.
- `gwz-py/src/gwz/protocol/generated/gwz.ir.json` exactly matches export of the candidate `gwz-core` schema.
- Rust and Python regeneration checks both report current artifacts.
- The Python drift gate checks both exact cross-repository IR equality and the same pre-log projection fingerprint.

### Dispatch stub: behavior pass, visibility fail

The public dispatch reaches the approved physical seam, constructs `OperationRequest::Log`, derives the operation context/action, and deliberately returns `UnsupportedOperation` rather than a misleading empty history. Both the unit-level and public-wrapper tests pass.

The only defect is the excessive internal visibility described in the [P2] finding.

### Scope and hygiene: otherwise pass

Diff scope:

- `gwz-core`: 11 files, `+804/-14`.
- `gwz-py`: 4 files, `+1045/-176`.

No `lib.rs`, inventory, dependency-pin, `Cargo.lock`, or `uv.lock` changes occurred. Diff whitespace checks and Rust formatting pass. Both worktrees remained clean.

## Commands and direct exits

### Tuple and scope

- `git rev-list --parents -n 1 eb7740efd151302f37a930b44979539142498d33`  
  **EXIT 0** — sole parent is the specified `gwz-core` base.

- `git rev-list --parents -n 1 381602ed177bd64ffdec0de72763d7e1e29a3621`  
  **EXIT 0** — sole parent is the resolved full `gwz-py` base.

- `git diff --name-status <base> <candidate>` in each repository  
  **EXIT 0** — only the 11 and 4 files summarized above.

### `gwz-core`

- `python3 protocol/regen.py --check`  
  **EXIT 0** — additive check reports the pinned pre-log hash; generated Rust and corpus artifacts are current.

- `TAUT_PYTHON=protocol/.regen-venv/bin/python cargo test --locked --test protocol`  
  **EXIT 0** — 33 passed, 0 failed.

- `cargo test --locked --lib operation::commit_log::handler::tests::log_dispatch_reaches_the_future_engine_stub -- --exact`  
  **EXIT 0** — 1 passed, 0 failed.

- `cargo clippy --locked --lib -- -D warnings`  
  **EXIT 0**.

- `cargo fmt --all -- --check`  
  **EXIT 0**.

- `git diff --check 2a3297da16a5d3cd814619cb2b3d7d15223640a7 eb7740efd151302f37a930b44979539142498d33`  
  **EXIT 0**.

- Independent legacy-tag mutation probe against `pre_log_projection()`  
  **EXIT 0** — confirmed the mutation does not retain the baseline fingerprint.

### `gwz-py`

- `.venv/bin/python scripts/check_protocol_drift.py`  
  **EXIT 0** — packaged IR exactly matches the core schema; candidate canonical fingerprint is  
  `39f6ff65ff89251cdce03203a212d57d0ec89f5cbf1ae029a80ad20baac13889`.

- `.venv/bin/python scripts/regen_protocol.py --check`  
  **EXIT 0** — generated Python API and IR are current.

- `.venv/bin/python -m pytest -q src/tests/test_log_protocol.py`  
  **EXIT 0** — 3 passed.

- `git diff --check 5f6689a30741f35c943839a6ead36582e6452a4b 381602ed177bd64ffdec0de72763d7e1e29a3621`  
  **EXIT 0**.

### Independent parity checks

- Canonical base-IR versus stripped candidate-IR comparison  
  **EXIT 0** — `structurally_equal=True`; both hashes equal the pinned pre-log hash.

- Base/candidate golden-corpus comparison  
  **EXIT 0** — 125 common vectors unchanged, nine `Log*` vectors added, zero removed.

## Re-review boundary

A focused re-review need only verify the visibility reduction, confirm the public `operation::handle_log` stub still compiles and returns `UnsupportedOperation`, and rerun the focused Rust formatting, clippy, and tests. No protocol regeneration should result from the remedy.

## Round 2 Re-review — P2 Visibility Cure

### Exact tuple

| Repository | Base | Round 2 candidate | Relationship |
|---|---|---|---|
| `gwz-core` | `2a3297da16a5d3cd814619cb2b3d7d15223640a7` | `affaa69a9cb9c61fd94febf80d9c6382f1648a93` | Candidate has base as its sole parent |
| `gwz-py` | `5f6689a30741f35c943839a6ead36582e6452a4b` | `381602ed177bd64ffdec0de72763d7e1e29a3621` | Unchanged from Round 1; candidate has base as its sole parent |

Amended core candidate reviewed against original candidate:

- Original: `eb7740efd151302f37a930b44979539142498d33`
- Amended: `affaa69a9cb9c61fd94febf80d9c6382f1648a93`

### Scope

This re-review was limited to:

- Cure of the original [P2] minimum-visibility finding.
- Continued public reachability of `gwz_core::operation::handle_log`.
- Absence of wire, schema, generated-artifact, corpus, or protocol-test changes.
- Focused formatting, clippy, protocol tests, regeneration, drift, and Python parity gates.

No plan, ambiguity-resolution, prior external review, or other excluded project documents were read.

### Disposition

**CURED**

The amendment contains exactly three visibility changes:

- `gwz-core/src/operation/mod.rs:3` now declares `mod commit_log;`, making the engine module private to `operation`.
- `gwz-core/src/operation/commit_log/mod.rs:3` now narrowly re-exports only `handler::handle_log` as `pub(super)`.
- `gwz-core/src/operation/commit_log/handler.rs:12` uses `pub(in crate::operation)`, the visibility required for Rust to permit the parent-scoped re-export without exposing the handler crate-wide.

There is no remaining `pub(crate)` commit-log module, handler, or wildcard re-export. The only production call remains `src/operation/push_event.rs:713`, through the parent-scoped facade.

The intentionally public dispatch function at `src/operation/push_event.rs:708` remains reachable as `gwz_core::operation::handle_log`. Its external integration test and internal handler test both pass and retain the deliberate `UnsupportedOperation` result.

### Final verdict

**GO**

### Remaining finding counts

| Priority | Count | Blocks |
|---|---:|---|
| P0 | 0 | Yes |
| P1 | 0 | Yes |
| P2 | 0 | Yes |
| P3 | 0 | No |
| **Total** | **0** | **0 blocking** |

No new findings were introduced within the Round 2 regression boundary.

### Regression assessment

#### Visibility boundary: pass

The module hierarchy now provides the minimum necessary path:

`operation::handle_log` → private `operation::commit_log` → named parent-scoped `handle_log` facade → private `handler`.

Unrelated crate modules cannot name the engine seam, while the operation wrapper can still dispatch to it.

#### Wire and generated artifacts: unchanged

`git diff eb7740e..affaa69a` contains only:

- `src/operation/mod.rs`
- `src/operation/commit_log/mod.rs`
- `src/operation/commit_log/handler.rs`

The amendment is `+3/-3`. Schema, additive guard, corpus, Rust generated code, and protocol tests are byte-identical to the original candidate. The Python candidate is unchanged.

The core additive projection remains:

`sha256:d0c205c8767f8d54d32ead2f676a05077d849f6a12278d9de52b3c132c3c9372`

The Python candidate IR fingerprint remains:

`sha256:39f6ff65ff89251cdce03203a212d57d0ec89f5cbf1ae029a80ad20baac13889`

#### Worktree hygiene: pass

Both worktrees were clean before and after all checks. Diff whitespace checks pass.

### Commands and direct exits

#### Tuple and amendment

- `git rev-list --parents -n 1 affaa69a9cb9c61fd94febf80d9c6382f1648a93`  
  **EXIT 0** — sole parent is `2a3297da16a5d3cd814619cb2b3d7d15223640a7`.

- `git rev-list --parents -n 1 381602ed177bd64ffdec0de72763d7e1e29a3621`  
  **EXIT 0** — sole parent is `5f6689a30741f35c943839a6ead36582e6452a4b`.

- `git diff --name-status eb7740efd151302f37a930b44979539142498d33 affaa69a9cb9c61fd94febf80d9c6382f1648a93`  
  **EXIT 0** — exactly the three operation visibility files listed above.

- `git diff --quiet eb7740e affaa69a -- protocol/gwz.taut.py protocol/check_log_additive.py protocol/regen.py protocol/corpus/golden.json protocol/corpus/rust/vectors.rs src/protocol/generated.rs tests/protocol.rs`  
  **EXIT 0** — no schema, guard, corpus, generated, or protocol-test change.

#### `gwz-core`

- `python3 protocol/regen.py --check`  
  **EXIT 0** — additive guard green; generated Rust and corpus artifacts current.

- `TAUT_PYTHON=protocol/.regen-venv/bin/python cargo test --locked --test protocol log_dispatch_reaches_the_future_engine_stub -- --exact`  
  **EXIT 0** — external/public dispatch test: 1 passed.

- `cargo test --locked --lib operation::commit_log::handler::tests::log_dispatch_reaches_the_future_engine_stub -- --exact`  
  **EXIT 0** — internal handler test: 1 passed.

- `TAUT_PYTHON=protocol/.regen-venv/bin/python cargo test --locked --test protocol`  
  **EXIT 0** — 33 passed, 0 failed.

- `cargo fmt --all -- --check`  
  **EXIT 0**.

- `cargo clippy --locked --lib -- -D warnings`  
  **EXIT 0**.

- `git diff --check eb7740efd151302f37a930b44979539142498d33 affaa69a9cb9c61fd94febf80d9c6382f1648a93`  
  **EXIT 0**.

- `git status --short`  
  **EXIT 0**, empty output.

#### `gwz-py`

- `.venv/bin/python scripts/check_protocol_drift.py`  
  **EXIT 0** — packaged IR still exactly matches the amended core schema.

- `.venv/bin/python scripts/regen_protocol.py --check`  
  **EXIT 0** — generated Python API and IR current.

- `.venv/bin/python -m pytest -q src/tests/test_log_protocol.py`  
  **EXIT 0** — 3 passed.

- `git diff --check 5f6689a30741f35c943839a6ead36582e6452a4b 381602ed177bd64ffdec0de72763d7e1e29a3621`  
  **EXIT 0**.

- `git status --short`  
  **EXIT 0**, empty output.
