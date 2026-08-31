# GWZ Log S2.7 independent review

**Verdict: NO-GO.**

- **Core base / sole parent:**
  `5a4f9cbe033805d8c54d78cc93b84f949ec429b5`
- **Core candidate:** `77cb47171256db4c548117e1dbdcf586f6e3071a`
- **Python base / sole parent:**
  `381602ed177bd64ffdec0de72763d7e1e29a3621`
- **Python candidate:** `10aec7fd69c2f94d90d4aead2bf125f76267b01a`
- **Normative CLI authority:**
  `c541d0ee22442d4078ac1afda047c21b1f7e096b`
- **Mode:** independent, single-axis S2.7 round 1; core and Python
  candidates read-only
- **Rows graded:** L-INT-1, the 2026-08-31 L-PRO-1 rider, exact
  L-ENV-1/L-ENV-12 projection, and the L-COA-6 `marker-invalid` projection
- **Finding count:** 0 P0 / 0 P1 / 1 P2 / 0 P3

The production candidate replaces the intentional S2.0 refusal with the
completed engine, projects each callback directly into one file-backed record,
seals the output, and returns the engine's final S2.6 aggregate. The
caller-owned registry has
bounded record batches, boundary-validated opaque resume cursors, explicit EOF,
typed unknown/released/invalid refusals, idempotent release, and an anonymous
process-temp backing file. Error paths remove the registered authority before
returning. The deterministic representative, member order, exact seconds,
millisecond overflow behavior, and source-byte lossy fact all match authority.

The protocol change is exactly the approved still-unshipped `LogEntry` rider:
slot 7 becomes optional and slots 8 through 11 are author seconds, committer
seconds, ordering seconds, and lossy. Every pre-log shape is untouched. Core
and Python generated artifacts agree and both prescribed regeneration/drift
checks pass. No renderer, command/client surface, machine JSON, engine
semantics, `lib.rs`, checked-artifact inventory, or `diff::log_service` change
rode S2.7.

The round cannot pass because L-INT-1 acceptance is not mutation-tight. Four
narrow wrong lifecycle/response implementations still leave all four
checked-in `l_int_1` tests green. The code does not contain those violations,
so the finding is P2 acceptance rather than a production P1. The cure is
narrow test instrumentation and fixtures; protocol, engine, projection, and
Python generation do not need to reopen.

The workspace `AGENTS_GWZ.md`, applicable root/core/CLI instructions, the full
current `GwzLogPlan.md`, the full current `GwzLogRequirements.md`, and
`AgentQuickStart.md` were read before candidate inspection. The authority's
S2.7 charter and L-INT-1/L-PRO-1 rider, rather than a stale core copy, governed
the review.

## Finding

### [P2 F1] Fresh output authority, Partial propagation, and seal cleanup are not pinned

The implementation correctly mints a non-empty sequence id, preserves the
S2.6 aggregate without reinterpretation, and releases the registry entry on
every explicit spool error branch. The checked-in tests do not distinguish all
three obligations.

In a disposable exact-candidate worktree, each of these independent wrong
implementations left all four `l_int_1` tests green:

1. replace the minted id with the empty string;
2. replace it with the same non-empty `commitlog_fixed` value for every
   request (the external unknown/released registry test also stayed green);
3. map `AggregateStatus::Partial` to `Ok` at response projection; and
4. omit `output_registry.release(&log_id)` only from the seal-failure branch.

The first violates L-INT-1's explicit non-empty requirement. The second lets a
later successful request overwrite the first authority, so the first response
no longer identifies its own sealed result. The third stops returning the
final S2.6 aggregate. The fourth leaves a resolvable id after a spool failure.
These are contract-observable failures, not synthetic internal refactors.

The exact round-2 remediation gate is test-focused:

1. execute two successful requests against one registry; assert both ids are
   non-empty and distinct, and that each id still resolves its own expected
   first record after the second request;
2. drive the public handler through the S2.6 default `Partial` case (one
   contributing repository plus one read-failure repository) and assert the
   response aggregate is `Partial`; and
3. add a test-only next-seal-failure injection, assert a typed error and zero
   registered authorities.

The empty-id, fixed-id, Partial-to-Ok, and missing-seal-release mutants must all
be RED under the focused round-2 suite. Preserve the already-green production
bytes unless the minimum test hook requires a `cfg(test)` field/branch. Do not
reopen the protocol rider, engine, projection mapping, Python generated
artifacts, renderer/client work, or diff output service.

## Owned contract audit

| Surface | Result | Evidence |
|---|---|---|
| Public dispatch | **PASS implementation / FAIL acceptance** | Public `operation::handle_log` accepts the caller's registry and invokes the already-reviewed open/merge/filter/coalesce engine. A real workspace test reaches that exported path and observes member then root in engine order, but F1 shows non-empty/fresh ids are not pinned. |
| Exact projection | **PASS** | Each successful engine callback makes exactly one `LogOutputRecord` append; group and degradation discriminants have mutually exclusive payload arms. Any projection/append error suppresses later writes and fails the unary operation. |
| Final aggregate | **PASS implementation / FAIL acceptance** | The response uses `result.aggregate().status()` without reinterpretation. Public fixtures observe `Ok` and strict `Failed`, but F1's Partial-to-Ok mutant survives. No process-exit mapping was introduced. |
| Registry lifecycle | **PASS implementation / FAIL acceptance** | `tempfile::tempfile()` supplies an anonymous/automatically removed process-temp file; records are length-prefixed CBOR, not retained as a whole-history collection. Reads are capped at 1,024 records, validate cursor boundaries, return a stable next cursor and explicit EOF, reject zero/oversize batches, and release idempotently. F1 shows id freshness/non-emptiness is not pinned. |
| Failure cleanup | **PASS implementation / FAIL acceptance** | Creation failure registers nothing. A post-registration append failure returns typed `IoError` and removes the id. Seal failure has the same release-before-return discipline, but F1's missing-seal-release mutant survives because no test drives it. Dropping the final registry/spool handle closes and removes the anonymous file. |
| Determinism and time | **PASS** | Members sort by `(member_id, commit)` and the first is the text/identity representative. The group's independent latest admitted committer instant stays the ordering second. Exact signed-i64 seconds survive; all millisecond convenience fields become absent on checked-multiply overflow. |
| Lossy projection | **PASS** | Raw representative subject/body and author/committer identity bytes are checked before lossy UTF-8 conversion. The body participates only when requested. The additive protocol flag records the source-byte fact, rather than inferring from U+FFFD. |
| Protocol rider | **PASS** | Schema and generated diffs are confined to `LogEntry` slot 7 optionality and additive slots 8-11. Core corpus changes only the direct and nested `LogEntry` vectors. Python `api.py` and IR carry the identical fields and optionality. |
| Visibility and placement | **PASS** | The spool and projection remain private under `operation::commit_log`; only the registry/read request/read response/read state needed by callers are re-exported with the public handler. Existing `src/lib.rs` and `src/diff/log_service.rs` are byte-identical to base. |

## Frozen three-arm provenance decision

L-COA-6 requires four machine meanings while the S2.0 protocol's
`LogMergeKind` is frozen to `none`, `marker`, and `heuristic`, and the S2.7
L-PRO-1 rider permits no enum or message-shape change. The candidate represents
the additive fourth meaning as:

```text
kind = none
gwz_commit_id = "marker-invalid"
```

This is accepted. The pair is unambiguous against ordinary `none`
(`gwz_commit_id = null`), proven marker identity (`marker` plus UUID), and
heuristic (`heuristic` plus null), preserves the literal authoritative
`marker-invalid` token, and consumes the already-optional provenance text slot
without violating the narrow protocol rider. A focused regression pins both
halves of the pair. Downstream S3 projection must interpret the pair before the
ordinary-none arm so machine output remains exactly `marker-invalid`; no S3
renderer or client behavior is implemented in this step.

## Stream, spool, and failure analysis

The engine remains the only ordering and aggregate authority. On a successful
callback, `project_merged_event` immediately appends one record; there is no
intermediate event collection, sort, or post-merge filter. A first projection
or append failure is retained, subsequent callbacks perform no output work,
the registered id is removed, and the typed error is returned. The finite
engine still completes its bounded read so already-reviewed cursor ownership
is not changed by this adapter.

The spool's in-memory working set is a single encoded record while writing and
at most the requested bounded record batch while reading. Its 8-byte lengths
and file offsets use checked arithmetic. A cursor must be zero, the sealed end,
or an exact record boundary; offsets beyond the end, into a prefix, or into a
payload are typed `InvalidRequest`s. A drained read is explicit `Eof`; a final
data batch supplies its next cursor and the following read supplies EOF. The
public numeric representation follows the existing operation-log cursor
precedent but remains documented as an opaque resume value.

`tempfile::tempfile()` does not write the workspace or repository and has no
stable path to leak. The registry owns the file until release or registry drop.
The implementation performs no network access, lock acquisition, conf-integrity
gate, or diff-output service call.

## Projection and mutation-tightness analysis

The projection fixture deliberately presents `mem_z` before `mem_a`, gives the
least sibling different text/identity times, gives the other sibling the latest
ordering time, uses invalid UTF-8, and drives signed-i64 millisecond overflow.
It therefore distinguishes member sort, representative selection, independent
ordering, exact seconds, lossy conversion, and optional legacy milliseconds.
The public lifecycle fixture exercises two data batches, exact order, EOF,
invalid boundary, zero/oversize bounds, release, and released lookup. Separate
fixtures exercise spool creation, post-registration append cleanup, strict
aggregate projection, and invalid-marker provenance.

Three independent contract-violation mutants were applied in a disposable
exact-candidate worktree and each made its focused checked-in regression RED:

1. deleting the `(member_id, commit)` sort failed the exact member-order
   assertion;
2. replacing the `marker-invalid` sentinel with null failed the L-COA-6 pair
   assertion; and
3. deleting release from the post-registration failure path left one
   resolvable registry entry and failed the cleanup assertion.

The same mutation pass then found F1's four false-pass variants: empty and
fixed ids, Partial collapsed to Ok, and missing seal-failure release. This
separates the well-pinned projection/order/append path from the three narrow
acceptance gaps that block round 1.

## Scope, generated artifacts, and LOC

Core's exact base-to-candidate change is 11 files, +906/-48 including generated
artifacts and `Cargo.lock`. Excluding generated Rust/corpus/lock output,
handwritten core churn is +867/-41 = 908 lines: 567 production/schema/dependency
lines and 341 test lines. Python adds 13 handwritten test lines; its remaining
changes are generated protocol/IR and lock synchronization. The combined
handwritten churn is therefore about 1.8 times the plan's aspirational ~500
target, not a hard cap.

The overrun is justified by the owned lifecycle rather than unrelated scope:
the production bulk is the bounded file format, cursor validation, registry,
cleanup, and deterministic projection; the tests carry real repositories and
failure injection. No duplicate engine, renderer, protocol service, or generic
framework was added. `tempfile` is the sole direct core dependency addition;
its lock graph and gwz-py's refreshed sibling-core resolution account for the
generated lock movement.

Exact scope checks found:

- no `src/lib.rs`, diff output log, client handler, renderer, command registry,
  checked-artifact, source inventory, release protocol, or frozen-lifecycle
  edit;
- no operation public exports beyond the handler's four caller-required
  lifecycle types;
- no Python CLI/API surface; and
- no protocol movement outside the approved `LogEntry` rider and its generated
  consequences.

## Identity and verification

Identity is exact and linear:

```text
core base tree:       adb90bee92db924f0225c522c7c2289861b5fe8a
core candidate tree:  5eabc97d63c9f27313e56136f8aec4337315837d
core binary diff:     6eb6dc740d1b4380701671cf2a96519351f753d658c41c1ca3d34280688aa453

python base tree:     88269b8267020151827b64f40013c0dceb0bea49
python candidate tree: 1a8008d556065c34665c422e514c23a2733b9892
python binary diff:   5205dc3fdd802acd076cae2e2b947aec9a0ee0e40de59188a8e68da1306468ea
```

Each candidate is one commit with the stated sole parent, is non-shallow, and
has no replacement refs. `git diff --check` passes in both repositories.
`git fsck --full` reports only existing unreachable review/insurance objects,
including the named insurance commits, and no integrity error. The exact core
and Python candidate worktrees and indexes remained clean; mutations ran only
in a disposable worktree that was restored byte-exact and removed.

Reviewer-run proportional gates on the exact candidates:

- four L-INT-1 focused core tests — 4/4 passed;
- exact-time/lossy projection and marker-invalid projection — 2/2 passed;
- log protocol integration/additive tests — 4/4 passed;
- three exact contract-violation mutants — each RED for the intended
  distinction;
- four F1 contract-violation mutants — unexpectedly green under all four
  `l_int_1` tests;
- core `cargo fmt --check` — exit 0;
- locked all-target `cargo check` — exit 0;
- locked all-target/all-feature strict clippy with `-D warnings` — exit 0;
- pinned core protocol regeneration and additive check — exit 0;
- Python pinned regeneration and drift checks — exit 0; and
- Python focused log protocol tests — 3/3 passed.

Direct ambient-system-Python regeneration attempts lacked `taut-proto`; the
repository-prescribed pinned environments passed and are the authoritative
result. This is an environment distinction, not candidate drift.

The builder's exact broad evidence was inspected and accepted rather than
repeating the long suites:

```text
core 77cb47171256db4c548117e1dbdcf586f6e3071a
  focused commit-log: 98/98
  full pinned lib: 1,795 passed / 0 failed / 1 ignored (724.79s)
  diff-render 10/10; protocol 33/33; publish 9/9; rename 2/2; doctests green
  fmt/check/strict clippy; pinned regen; source boundary; release 6/6;
  checked-artifact compiler mutation matrix 69/69: all green

python 10aec7fd69c2f94d90d4aead2bf125f76267b01a
  native rebuilt; full pytest 333/333
  drift/regen/check/clippy: all green
```

## Round-2 disposition

Do not land or push either candidate. Preserve the reviewed production and
protocol bytes, apply only F1's minimum acceptance/test-hook cure, and return
the exact remediated core plus byte-identical Python candidate to this same
reviewer for the terminal round 2 under the fresh two-round cap. S3.1 and S3.5
remain blocked on S2.7 landing.
