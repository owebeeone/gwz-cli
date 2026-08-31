# `gwz log` S3.3 Single-Axis Review

- Date: 2026-08-31
- Reviewer: independent S3.3 reviewer (the S3.2 builder; not the S3.3 builder)
- Authority / sole base: `e783b7390c7fe5e7d1f0df9c982fdf59dadd6940`
- Exact `gwz-cli` candidate: `85090d5ff35dc526d087e0ac6dc6b78305909c07`
- Candidate tree: `a71ed4de4edf61b0bfd2e52287477a97bbd15742`
- Exact sibling `gwz-core`: `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Core tree: `20b52eb0b425e8482f4bd853fe4a6a580deb28e3`
- Scope: S3.3 only — L-JSN-1/L-JSN-2, L-ENV-12/L-ENV-13,
  marker-invalid provenance, deterministic exact bytes, mixed record order and
  complete EOF consumption, caller-owned spool release/EPIPE, and scope

## Verdict: NO-GO

The production renderer is compact and correct over the reviewed success
surface. It emits one deterministic schema-tagged JSON document or a
schema-tagged JSONL stream; preserves core record, member, and parent order;
uses exact signed-i64 seconds and recorded offsets; consumes the core `lossy`
fact rather than searching for U+FFFD; renders all four provenance tokens
including `marker-invalid`; drains to explicit EOF; stops before a hidden read
when the consumer is already closed; and releases the caller-owned spool after
every disposition in the current implementation.

The checked-in acceptance suite nevertheless permits two wrong runner
implementations to ship. An exact mutant that discards `include_body`
immediately before public core dispatch leaves all 29 focused S3.1+S3.3 tests
green. A second exact mutant that omits release only for spool-read and
invalid-record failures also leaves all 29 green. The review brief makes a
surviving wrong implementation a finding, so the round cannot be GO despite
the current production lines being correct.

No P0 or P1 finding was found. There are two P2 findings.

## Findings

### S3.3-F1 — P2 — `--body` is not pinned through the real machine runner

The renderer correctly includes `LogEntry.body` when core supplies it at
[`src/log_machine.rs`](../src/log_machine.rs), and S3.1 separately proves
that Clap lowers `--body` to `LogOptions.include_body`. The S3.3 record fixture,
however, constructs a protocol `LogEntry` with `body: Some(...)` directly. The
only real machine-runner fixture invokes plain `log`, so core never receives an
`include_body` request in that path.

The exact reviewer mutant cloned the real request in
`run_log_with_registry`, set `request.options.include_body = None`, and then
called `operation::handle_log`. Both focused suites stayed green:

- `cargo test --locked --lib tests::g11` — 8/8 passed under the mutant.
- `cargo test --locked --lib tests::g09` — 21/21 passed under the mutant.

An independent reviewer fixture using a real initialized workspace and the
real runner proved the missing observation. It asserted no `body` key by
default and the exact post-subject message under `include_body = Some(true)`;
the production candidate passed, while the dispatch mutant failed with
`left: None`, `right: Some("body line\n")`.

Required remediation: check in the compact real-runner fixture for both JSON
and JSONL (one mode may share the same request fixture if byte equivalence is
already pinned), asserting subject-only/default and exact full-message body
under `--body`. It must kill a runner-boundary mutant that discards or forces
the wire flag; a direct constructed `LogEntry` assertion alone is not
acceptance for this row.

### S3.3-F2 — P2 — Release is unobserved on spool-read and record-validation failures

The current production sequence is correct: `run_log_with_registry` calls the
machine renderer, unconditionally releases `log_id`, then classifies
`Read`, `Write`, and `InvalidRecord`. Existing tests observe release after
successful JSON/JSONL output and an immediate BrokenPipe. They do not exercise
the two machine-only error variants introduced by S3.3.

The exact reviewer mutant made release conditional, skipping it only when
`output_result` was `LogMachineOutputError::Read` or `InvalidRecord`. All 8
S3.3 tests and all 21 adjacent S3.1 lifecycle tests remained green. This is a
real lifecycle leak shape: the runner owns the registry authority after
dispatch, and the S3.1 terminal review requires release on every post-dispatch
output disposition.

Required remediation: add an injectable/read-failing runner seam or an
equivalent narrowly scoped test hook, then prove that a typed spool read error
and an inconsistent record both return their existing error classes and leave
the log id unresolvable. The test must kill the conditional-release mutant.
Keep the public/core protocol and registry surface unchanged unless the test
cannot be expressed through an existing private seam; no message parsing or
artifact access belongs in CLI.

## Requirement, lifecycle, and byte audit

| Surface | Result | Evidence |
|---|---|---|
| L-JSN-1 entry shape | PASS implementation / FAIL acceptance | Uniform `members[]`, provenance, author/committer identities and exact times, subject, and optional body are emitted. F1 records the surviving real-dispatch body mutant. |
| L-JSN-2 degradation shape | PASS | All seven core reason variants map to stable snake-case tokens; id, path, operand, and message are preserved. |
| L-ENV-12 lossy behavior | PASS | `"lossy": true` appears only for core `Some(true)`; a literal genuine U+FFFD with `Some(false)` remains unflagged. C0/newline content is JSON-escaped, hashes remain full, parent order is untouched, and i64 MIN/MAX render exactly. |
| L-ENV-13 schema and record kinds | PASS | JSON is `{"records":[...],"schema":"gwz.log/v0"}`; JSONL starts with the exact header and every payload is an entry or degradation record on one line. Empty outputs are exact and deterministic. |
| L-COA-6 / L-COA-9 provenance | PASS | `none`, `heuristic`, `marker:<uuid>`, and `marker-invalid` are exact; inconsistent kind/id arms refuse rather than guess. |
| Complete mixed order | PASS | Multi-page `entry, degradation, entry` stays in core order through both machine modes; the real core fixture preserves `degradation, entry`; the renderer reads until explicit EOF and rejects empty Data/nonempty EOF/nonadvancing cursors. |
| Deterministic bytes | PASS | Repeated rendering is byte-identical; exact single-record JSON and JSONL strings pin key ordering, escaping, hashes, parents, offsets, schema, and final newline. |
| EPIPE / no hidden read | PASS | The JSON/JSONL prefix is written and flushed before the first spool read; write- and flush-side BrokenPipe stop immediately, map to exit 0, and the runner releases the unread spool without stderr fallback. |
| Release on every disposition | PASS implementation / FAIL acceptance | Success and EPIPE are observed. F2 records the surviving read/invalid conditional-release mutant. |
| Thin CLI / no pager | PASS | Production calls only the public core log registry/read seam and `Write`; it performs no Git/artifact/workspace read, adds no pager, and owns no semantic parsing. |

## Mutation audit

| Exact mutant / failure mode | Result |
|---|---|
| Encode `marker-invalid` as `none` | RED on the four-token provenance matrix. |
| Infer lossy from rendered U+FFFD instead of consuming the protocol bit | RED on both the positive bit and genuine-U+FFFD negative sentinel. |
| Stop after the first Data page instead of explicit EOF | RED on the explicit page queue / remaining-response sentinel. |
| Reverse records inside each read page | RED on exact mixed `entry, degradation, entry` order. |
| Reverse the protocol member vector | RED on the exact first-entry object and member hashes/parent arrays. |
| Omit optional body serialization | RED on the constructed-entry JSON contract. |
| Remove registry release entirely | RED on the real machine runner's released-id lookup. |
| Remove the prefix flush and read a buffered spool before discovering EPIPE | RED before the first spool read. |
| Drop `include_body` only at the real runner/core boundary | **GREEN — S3.3-F1.** Reviewer real-runner probe kills it. |
| Skip release only for `Read`/`InvalidRecord` results | **GREEN — S3.3-F2.** |

## Scope and integrity

Candidate identity is exact: one commit with sole parent `e783b739...`, clean
tree, no rename or mode change, no trailers, and green `git diff --check`. The
candidate changes 5 CLI files by `+921/-10`:

- private production/module wiring: `+251/-10` (`src/log_machine.rs`, the
  private `log_exec` integration, and two private module lines);
- focused tests/module registration: `+670`.

The production slice is effectively the plan's aspirational ~250 lines. Total
handwritten scope including tests is about 3.7x that aspiration; the budget is
explicitly non-hard, and most of the overrun is the required exact-byte and
lifecycle matrix. The two acceptance holes, rather than raw size, are the
blockers.

`Cargo.toml`, `Cargo.lock`, generated protocol, docs, pins, and sibling core
are byte-untouched. The CLI crate's external public surface is not widened;
the new module and re-export remain `pub(crate)`. No human renderer, pager,
gwz-py, protocol/schema generation, core semantics, source-loading inventory,
or unrelated output registry rides the candidate.

## Proportional evidence

The operator's fast-test policy was followed: no 1,700+ full suite and no
69-case compiler matrix were run.

- Exact candidate `cargo test --locked tests::g11` — 8/8 passed.
- Adjacent terminal S3.1 `cargo test --locked --lib tests::g09` — 21/21 passed.
- Reviewer real-runner default/`--body` probe — GREEN on production; RED on
  the runner-boundary body mutant.
- Eight exact semantic/order/lifecycle mutants — RED as recorded above.
- Body-discard runner mutant — existing 29/29 focused tests GREEN.
- Read/invalid conditional-release mutant — existing 29/29 focused tests GREEN.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --locked --all-targets` — exit 0.
- strict Clippy over locked all targets/features — exit 0.
- generated CLI-reference freshness check — exit 0.
- exact sibling checked-artifact boundary — exit 0, 15 visible entries / 5
  classified modules.
- Candidate, sibling, and disposable mutation worktrees restored clean at
  their exact SHAs.

The first focused compile initially stopped for environmental ENOSPC. Only
reproducible Cargo `target/` directories from completed review/probe
worktrees were cleaned; no source or user artifact was removed. The same gate
then completed green.

## Round-2 acceptance gate

Round 2 should be test-focused and restricted to S3.3-F1/F2 plus preservation
and candidate integrity. Accept only if:

1. a real runner fixture proves default subject-only and `--body` full-message
   behavior and kills dispatch-time loss/forcing of `include_body`;
2. typed spool-read and invalid-record failures prove idempotent release and
   kill release omission on either error branch;
3. all current exact-byte, ordering, marker-invalid, lossy, schema, EOF, EPIPE,
   and real mixed-order behavior stays byte-preserved; and
4. the amendment remains private/test-only unless a minimal injectable reader
   seam is necessary for F2.

No schema change, machine-error redesign, human rendering, pager, core engine,
protocol, gwz-py, or broad CLI lifecycle refactor belongs in remediation.
