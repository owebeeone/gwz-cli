# `gwz log` S3.1 Single-Axis Review

- Date: 2026-08-31
- Reviewer: independent S3.1 reviewer
- Authority / sole base: `0c9629f6ed311a70ddbe80e7974a675ec45aeb3c`
- Exact `gwz-cli` candidate: `3b39e19600110966482656ad2aa89e25e1a0c3b1`
- Candidate tree: `65d21ece037fd1abe210da2e5bd19d14d61b6dc6`
- Exact sibling `gwz-core`: `834275d6633ccba0755859e9c6437b69ba52d05a`
- Scope: S3.1 only — L-SEL-1, L-EXIT-1; the CLI flag surfaces of
  L-DEP-1/L-FIL-1/L-TOL-2/L-COA-3/L-JSN-1; L-ENV-8/L-ENV-9; thin-client,
  no-pager, lifecycle/release, generated docs/lock, and scope/LOC

## Verdict: NO-GO

The command and complete flag surface are wired through a thin, private CLI
layer, request lowering leaves range/filter/coalescing semantics in core, the
caller-owned spool is released on every post-dispatch output disposition, and
the EPIPE behavior is sound. The package nevertheless violates L-EXIT-1 on
real rejected requests, and its large acceptance suite does not prove that the
real runner consumes the core aggregate status. The checked-in `--strict` help
also narrows the promised behavior incorrectly.

No P0 finding was found. There is one P1 and two P2 findings.

## Findings

### S3.1-F1 — P1 — Real rejected requests are emitted as process exit 1

[`exit_code_for_log_error`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-cli/src/log_exec.rs:88)
uses a short ErrorCode allow-list for exit 2 and maps every other core error to
1. That loses the request phase and contradicts L-EXIT-1's exact distinction:
invalid invocation or unreadable workspace is rejected/2; execution failure is
1.

Two real public/core paths reproduce the defect:

1. `gwz --root <existing-empty-directory> log` accepts the explicit root in
   core, then reaches
   [`read_manifest`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-core/src/operation/commit_log/request.rs:65).
   The shared artifact reader maps the missing `gwz.conf/gwz.yml` filesystem
   error to generic `IoError` at
   [`artifact/mod.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-core/src/artifact/mod.rs:575).
   A reviewer end-to-end fixture asserted the observed core code was
   `IoError`, then expected L-EXIT-1 exit 2. It failed exactly with `left: 1,
   right: 2`.
2. On a valid created workspace, `gwz ... log +missing` returns the typed
   `SnapshotNotFound` rejection from core and the same classifier returns 1.
   This is not an inferred enum case: the reviewer executed the real
   `run_log_with_registry` path. Core's accepted S2.2 tests themselves name
   malformed/foreign snapshot results “rejected” while fixing the code as
   `SnapshotNotFound`; the analogous `--tagged absent` path is explicitly a
   `TagNotFound` rejection.

The pure enum table at
[`g09.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-cli/src/tests/g09.rs:369)
cannot detect either problem. It even fixes every `IoError` at 1, although a
real unreadable-workspace path currently arrives with that code. Conversely,
blindly changing every `IoError` to 2 would misclassify genuine execution and
stdout failures. The rejection/execution distinction must be preserved at a
typed seam, not guessed from an error message.

Required remediation:

1. Preserve enough typed core context for an explicit unreadable workspace to
   be classified as rejected without making all `IoError` values exit 2. A
   narrow core correction that surfaces the missing root manifest as
   `ManifestNotFound`, or an equally typed phase/class signal, is acceptable;
   CLI artifact inspection or message matching is not.
2. Map actual log invocation rejections including `SnapshotNotFound` and
   `TagNotFound` to 2, while retaining genuine execution/output failure as 1.
3. Add real-runner fixtures for an existing explicit root without a manifest,
   missing/foreign snapshot, and missing `--tagged` tag. Assert exact core code,
   process class, channel, and no stdout payload. Retain an actual execution-I/O
   fixture at 1. Mutants that remove each named rejection class must be red.

### S3.1-F2 — P2 — The aggregate exit test bypasses the runner it claims to protect

The production runner correctly calls `exit_code_for_response` at
[`log_exec.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-cli/src/log_exec.rs:49),
but the aggregate test at
[`g09.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-cli/src/tests/g09.rs:275)
calls the helper directly. Every real `run_log_with_registry` test currently
produces status `Ok` or takes EPIPE, whose required override is 0.

The exact reviewer mutant replaced the production line with `let code = 0;`.
All 17 S3.1 tests remained green. This is the false-pass shape the S3.1
mutation-audit requirement is meant to reject: a future disconnect between
the core aggregate and process exit could ship despite more than 500 test LOC.

Required remediation: drive actual core fixtures through
`run_log_with_registry` and assert the returned exit for at least an ordinary
success (0), contributing-plus-read-failure `Partial` (1), strict degradation
`Failed` (1), and the rejected request path (2). Keep the direct shared-seam
vocabulary table, but add integration assertions that kill the literal-0
mutant and a mutant that substitutes any fixed aggregate class.

### S3.1-F3 — P2 — `--strict` help promises only unreadable-history failure

The new public help says “Fail if any selected repository history is
unreadable” at
[`logargs.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.1/gwz-cli/src/logargs.rs:67),
and the same text is checked into generated `docs/CLI.md`. L-TOL-2 and
L-EXIT-1 are broader: `--strict` promotes **any degradation** to failure,
including otherwise benign missing operand/snapshot/lock rows and unborn
repositories. Core implements the broad behavior; the S3.1-owned flag surface
teaches a narrower one.

The help test only searches for the `--strict` token, so replacing the text
with the current narrower statement is not observable.

Required remediation: describe the actual contract (for example, “Promote any
selected-repository degradation to failure”), regenerate `docs/CLI.md`, and
assert the semantic phrase in the help regression. A mutant restoring
“history is unreadable” must be red.

## Requirement and lifecycle audit

| Surface | Result | Evidence |
|---|---|---|
| L-SEL-1 command + standard selectors | PASS | `log` is a first-class Clap subcommand; root, include/exclude selectors, aliases, `--all`, `--jobs`, request metadata, and workspace-relative cwd are preserved. |
| L-DEP-1 cap flag | PASS | Omitted `-n` stays wire `None`; `-n N` is raw `Some(N)`; `-n 0` and `--no-limit` lower to `Some(0)`; negatives and the conflict are Clap refusals. Core retains cap/default/lift semantics. |
| L-FIL-1 flag surface | PASS | All six raw filter fields lower without parsing time, regex, range, path, or Git semantics in the client. Help names Rust regex and accepted time families. |
| L-TOL-2 `--strict` behavior | PASS implementation / FAIL help | Absent is wire `None`, present is `Some(true)` and core owns semantics; S3.1-F3 records the inaccurate public help. |
| L-COA-3 / L-JSN-1 flags | PASS | `--no-coalesce` lowers only `Some(false)` and `--body` lowers only `Some(true)`; absent values stay `None`. |
| L-ENV-8 Clap vocabulary | PASS | Cap conflict, exact color vocabulary, repeated single-value denial, and global `--jobs` inheritance are covered without bespoke repetition handling. |
| L-EXIT-1 | FAIL | Aggregate helper vocabulary is correct, but S3.1-F1 breaks real rejected errors and S3.1-F2 leaves runner integration unpinned. |
| L-ENV-9 EPIPE | PASS | Both successful output and structured machine-error output stop at the first BrokenPipe, return 0, and emit no fallback spray. Non-EPIPE write errors remain typed failures. |
| Caller-owned registry lifecycle | PASS | The runner copies the id, renders, releases before interpreting the write result, and tests released lookup on success and EPIPE. Accepted S2.7 core owns cleanup before any successful response exists. |
| No pager | PASS | Log writes directly to its provided/stdout sink and does not call the existing pager module. This matches the standing L-OUT-5 decision. |
| Core owns semantics | PASS | CLI performs only Clap surface validation and lossless request lowering. It does not call Git or read/write GWZ artifacts. No core semantic parser is duplicated. |
| Visibility/public API | PASS | New modules, request variant, invocation, runner, color type, and re-exports are `pub(crate)`; the crate's existing public surface is not widened. Core is byte-untouched by this candidate. |

## Mutation audit

| Mutant / failure mode | Result |
|---|---|
| Omitted behavior flags lowered as explicit false | RED by the absent-wire-`None` test |
| Present `--no-coalesce`/`--body`/filters lowered to the wrong wire slot | RED by exact field assertions |
| Operands and post-`--` pathspecs combined or swapped | RED by exact two-vector assertions |
| `-n` omitted/zero/no-limit tri-state collapsed | RED by exact tri-state assertions |
| Remove `-n`/`--no-limit` conflict or widen color vocabulary | RED by parser tests |
| Replace aggregate runner result with literal 0 | **GREEN — S3.1-F2** |
| Collapse every core error to 1 | RED for the small allow-list, but real snapshot/tag/unreadable-root rejection omissions are **GREEN — S3.1-F1** |
| Omit registry release on success or BrokenPipe | RED by released-id lookup |
| Return aggregate 1/2 rather than 0 on BrokenPipe | RED by explicit precedence tests |
| Fall back to an unguarded machine-error print after BrokenPipe | RED by one-write sentinel |
| Replace broad strict contract in help with unreadable-history-only text | **GREEN — S3.1-F3** |

## Scope, generated artifacts, and integrity

Candidate identity is exact: one commit, sole parent `0c9629f...`, clean tree,
no rename or mode change, and `git diff --check` passes. The candidate changes
12 files by `+1615/-8`:

- handwritten production: approximately `+297`, essentially the aspirational
  `~300` S3.1 implementation target;
- handwritten tests/module registration: `+533`;
- generated CLI reference: `+138`;
- generated dependency lock refresh: `+647/-8`.

The plan counts handwritten tests, so approximately 830 handwritten lines are
about 2.8x the aspirational target. This is not a separate blocker: the
production slice is compact, the test volume is predominantly the mandated
surface/lifecycle matrix, and the budget is explicitly non-hard. The two
mutation gaps above do mean that volume cannot substitute for the missing
acceptance properties.

`Cargo.toml` and every pin are unchanged. The lock refresh resolves the already
configured path sibling at `gwz-core 0.11.1` and its existing dependency graph;
`cargo check --locked` succeeds. `docs/CLI.md` is generator-clean. There is no
protocol/schema/generated-code edit, no core source edit, no artifact or
source-loading inventory edit, and no handler/renderer/public-seam creep beyond
the S3.1 private runner and plumbing response.

## Proportional evidence

The operator's fast-test policy was followed: no 1,700+ suite and no 69-case
compiler mutation matrix were run.

- `cargo test --locked tests::g09` — 17/17 S3.1 tests passed.
- Reviewer explicit-root-without-manifest fixture — RED exactly: observed
  `IoError`, exit `1`, required `2`.
- Reviewer valid-workspace `+missing` fixture — RED exactly: observed
  `SnapshotNotFound`, exit `1`, required `2`.
- Reviewer literal-0 aggregate-runner mutant — 17/17 S3.1 tests still passed.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --locked --all-targets` — exit 0.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` — exit 0.
- `python3 scripts/generate_cli_reference.py --check` — exit 0.
- Direct checked-artifact source boundary on exact sibling core — exit 0,
  15 visible entries / 5 classified modules.
- Release-boundary unit suite — 6/6 passed.
- Exact candidate and sibling worktrees remained clean after review.

## Round-2 acceptance gate

Round 2 should be restricted to S3.1-F1 through S3.1-F3 plus candidate
integrity. Accept only if all three real rejection paths return 2 through the
public runner, genuine execution/output I/O remains 1, actual core aggregate
0/1 behavior is integrated and mutation-tight with rejected request 2, the
strict help states the broad degradation rule, generated docs/lock are clean,
and the rest of the reviewed candidate is byte-identical modulo those narrow
corrections. No rendering, protocol, core filter/coalescing behavior, pager, or
new public surface belongs in the remediation.
