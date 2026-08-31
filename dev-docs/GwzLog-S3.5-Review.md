# GWZ Log S3.5 independent review

- Date: 2026-08-31
- Review round: 1
- Exact `gwz-py` base / sole parent:
  `10aec7fd69c2f94d90d4aead2bf125f76267b01a`
- Exact `gwz-py` candidate:
  `c34f09fdbe8484b547dfb2d12cf7053da10f9d3e`
- Exact sibling `gwz-cli` authority:
  `e783b7390c7fe5e7d1f0df9c982fdf59dadd6940`
- Exact sibling `gwz-core`:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Scope: S3.5 only — L-PY-1, L-PY-2, CLI/API lowering parity,
  generated-protocol native dispatch, output cursor/release lifecycle, exact
  aggregate and process exits, frozen invalid-marker provenance, complete
  structured-record preservation for S3.6, and no-renderer scope

## Verdict: NO-GO

The production transport is thin and structurally sound. It dispatches the
generated `LogRequest` through the public core operation, returns the generated
`LogResponse`, reads bounded generated `LogOutputRecord` batches in encounter
order, follows the opaque cursor through explicit EOF, and releases in an
unconditional `finally`. Partial and Failed outputs remain available to API
callers. The frozen invalid-marker pair survives unchanged, and no record
renderer or shadow protocol model entered S3.5.

The candidate nevertheless has two user-observable parity defects. Python
accepts a depth outside the protocol's signed-i64 domain and then exits 1 with
an uncaught encoding traceback instead of refusing the invocation with 2. Its
machine-error stdout path also lets EPIPE escape, producing BrokenPipe spray
instead of clean exit 0. The parser additionally admits repeated boolean flags
and abbreviated long options that the exact Rust CLI rejects.

Three acceptance areas are not mutation-tight despite the 693 added test
lines: the real CLI-to-client option seam and exact Failed status, complete
multi-batch record fidelity/order, and task-cancellation release. The reviewed
production code is correct on those axes, but narrow contract-violating
mutants remain green.

Finding count: **0 P0 / 2 P1 / 3 P2 / 0 P3**.

## Findings

### S3.5-F1 — P1 — Out-of-i64 `-n` is accepted and exits 1 with a traceback

`configure_log` uses the shared unbounded Python `int` parser for `-n` at
`src/gwz/cli_log.py:53-57`. The protocol field is signed i64, and the exact
Rust S3.1 authority rejects values outside that domain during argument parsing.

On the exact candidates:

```text
argv: gwz-py log -n 9223372036854775808
Python parser: accepted, max_entries=9223372036854775808
Python process: exit 1, uncaught taut.wire.cbor.EncodeError: IntOutOfSubset

argv: gwz log -n 9223372036854775808
Rust process: exit 2
```

The Python failure occurs during request encoding, before native dispatch, and
prints a traceback. This violates L-PY-1's flag and exit parity and turns an
invalid invocation into an execution-class failure.

Required remediation:

1. parse CLI `-n` as an exact non-negative signed i64;
2. assert `i64::MAX` is accepted by the parser and `i64::MAX + 1` is rejected
   before bridge access with process exit 2 and no traceback; and
3. retain the existing negative-value and `-n`/`--no-limit` refusals.

### S3.5-F2 — P1 — Log machine-error EPIPE is not clean early termination

The log-specific error classification is reached only after `cli.main` writes
the error with unguarded `print` at `src/gwz/cli.py:124-129`. With `--json` or
`--jsonl`, that write targets stdout. A closed consumer raises
`BrokenPipeError`, bypasses the intended return code, and produces interpreter
shutdown spray.

A real closed-pipe probe produced Python producer status 120 and a
`BrokenPipeError` diagnostic. The exact Rust S3.1 authority returned 0 on the
same rejected log request and emitted no error spray. An injected broken
stdout writer likewise raised rather than returning 0.

This violates L-PY-1's inherited L-ENV-9 process contract.

Required remediation:

1. route log output/error writes and the relevant flush through an
   EPIPE-aware boundary;
2. stop after the first BrokenPipe, return 0, and perform no fallback write;
3. retain non-EPIPE output failures as exit 1; and
4. add both an injected broken-writer regression and a real subprocess pipe
   regression. This is lifecycle plumbing, not S3.6 record rendering.

### S3.5-F3 — P2 — The Python log parser accepts spellings Clap rejects

`_StoreOnce` protects value-taking log options, but the boolean options at
`src/gwz/cli_log.py:61-130` use ordinary `store_true`. Argparse long-option
abbreviation also remains enabled. The exact comparison is:

| Invocation | `gwz-py` | exact `gwz` |
|---|---:|---:|
| `log --strict --strict` | accepts | exit 2 |
| `log --no-limit --no-limit` | accepts | exit 2 |
| `log --body --body` | accepts | exit 2 |
| `log --stric` | accepts as `--strict` | exit 2 |
| `log --no-lim` | accepts as `--no-limit` | exit 2 |

The same repeat issue applies to the remaining log booleans. These are extra
public flag spellings, contrary to L-PY-1 and L-ENV-8.

Required remediation: disable long-option abbreviation for the log-visible
surface and give every non-repeatable log option Clap-equivalent one-shot
behavior, while retaining repetition for the global selector options that are
intentionally repeatable. Pin both before-command and after-command global
placement where applicable.

### S3.5-F4 — P2 — CLI lowering and exact Failed preservation are not mutation-tight

The production lowering at `src/gwz/cli_log.py:150-165` is correct, and
`Client.log` returns the generated response without applying the ordinary
unsuccessful-response raiser. The tests split their evidence, however: the
parser test sees active filters/behaviors, the client test calls `Client.log`
directly, and the only handler spy leaves most of those flags absent.

Each of these wrong implementations left all 31 focused S3.5 tests green:

- swap `since` and `until` at the handler seam;
- drop handler forwarding of `strict`, `tagged`, `no_merges`, or
  `first_parent`; and
- rewrite an exact API `Failed` response to `Partial`.

The native strict CLI test cannot distinguish the last mutation because both
Partial and Failed map to process exit 1.

Required remediation:

1. drive one all-active parsed command through `handle_log` into a spy client
   and assert every operand, pathspec, selector/policy field, filter, behavior,
   cap, cwd, and tri-state exactly;
2. retain a separate all-absent assertion; and
3. assert through the programmatic API that a real or generated strict Failed
   response remains exactly Failed and its degradation/entry output remains
   consumable.

### S3.5-F5 — P2 — Multi-batch record fidelity and cancellation cleanup are not pinned

The implementation is correct: native reads at most the requested batch,
encodes generated records in vector order, the Python bridge decodes every
record, `Client.log_output` yields every Data record before advancing to the
next page, stops only at explicit EOF, and releases in `finally`.

The checked-in fixtures do not distinguish several narrow violations. Each of
the following exact mutants left all 31 focused log tests green:

- reverse every decoded native batch;
- drop every `@root` degradation;
- return when a second Data page is received;
- erase every entry body immediately before `yield`; and
- skip release only when the consumer task receives `CancelledError`.

The body-erasure mutant is especially direct: every checked-in entry fixture
has `body=None`, so an S3.6-visible field can be discarded without detection.
The same fixtures do not jointly pin non-empty members/parent order, offsets,
exact seconds, lossy, and body.

Direct probes confirm this is acceptance debt, not a production failure. The
unmodified candidate delivered a real 132-record history as Data[128],
Data[4], EOF with exact newest-first subjects, and a task cancelled while
blocked in a read released its log id.

Required remediation:

1. exercise at least two Data pages and explicit EOF through
   `Client.log_output`;
2. assert the complete exact interleaved sequence, including entry and root/
   member degradation positions;
3. use rich generated records to pin full hashes, parent order, members,
   provenance, subject/body, identities, record-own offsets, exact seconds,
   lossy, and all degradation fields without reconstruction or normalization;
4. assert exactly one release after EOF and after a task is cancelled while
   blocked in `log_output_read`; and
5. make the reverse/drop/early-return/body-erasure/cancellation-release mutants
   all RED.

## Requirement and charter matrix

| Surface | Result | Evidence |
|---|---|---|
| L-PY-1 CLI surface/lowering | **FAIL** | Core fields are currently lowered correctly, but F1 and F3 break exact accepted-vocabulary parity and F4 leaves the real handler seam under-pinned. Operands and post-`--` pathspecs remain separate; omitted tri-states remain wire `None`; `--no-limit` lowers to zero. |
| L-PY-1 exit semantics | **FAIL** | Partial/Failed return 1 and typed rejections return 2 in ordinary operation, but F1 misclassifies an invalid cap as 1 and F2 breaks clean EPIPE exit 0. |
| L-PY-2 programmatic API | **PASS implementation / FAIL acceptance** | `Client.log` and `log_output` expose generated responses and structured records. F4 does not pin exact Failed, and F5 does not pin complete record fidelity/order across pages. |
| Generated-protocol-only native dispatch | **PASS** | `native/src/dispatch/log.rs` validates and decodes `LogRequest`, calls public `operation::handle_log`, and encodes `LogResponse`; output records are generated CBOR. No Python/Rust shadow log model exists. |
| Cursor and bounded delivery | **PASS implementation / FAIL acceptance** | Cursor starts absent, advances from every answer, batches are requested at 128, Data pages are yielded, and only explicit EOF terminates. F5's second-page and ordering mutants survive. |
| Release lifecycle | **PASS implementation / FAIL acceptance** | EOF, read failure, and explicit `aclose()` release; a direct cancellation probe also releases. Cancellation-specific mutation coverage is missing per F5. |
| Partial/Failed output availability | **PASS implementation / FAIL acceptance** | The API bypasses the generic unsuccessful-response raiser so records remain consumable. Exact Failed identity is not pinned per F4. |
| Frozen `marker-invalid` pair | **PASS** | A real invalid marker remains `LogMergeKind.none` plus `gwz_commit_id="marker-invalid"`; no downstream reinterpretation occurs. |
| Complete S3.6 record preservation | **PASS implementation / FAIL acceptance** | Generated decoding/yielding is lossless in production. F5's field/order/page mutants show the tests do not yet prove it. |
| No renderer creep | **PASS** | S3.5 drains records and returns the aggregate exit only. It adds no human or JSON record renderer and does not modify `cli_render`. |
| Help/docs | **PASS** | Runtime help exposes the complete S3.1 log-specific surface and exact grammar/strict teaching text. No generated Python CLI-reference convention exists. |
| Core/CLI ownership | **PASS** | The exact sibling core and CLI are byte-untouched and clean. No Git/artifact logic was duplicated in Python. |

## Mutation audit

| Mutant or wrong behavior | Focused result |
|---|---|
| Repeated boolean flag accepted | **Production failure — accepted** |
| Unique long-option abbreviation accepted | **Production failure — accepted** |
| `i64::MAX + 1` cap accepted | **Production failure — accepted, later exit 1** |
| Closed machine-error stdout | **Production failure — status 120 / BrokenPipe spray** |
| Swap handler `since`/`until` | **GREEN — F4** |
| Drop handler strict/tagged/filter forwarding | **GREEN — F4** |
| Collapse API Failed to Partial | **GREEN — F4** |
| Reverse each native output batch | **GREEN — F5** |
| Drop root degradation | **GREEN — F5** |
| Stop on the second Data page | **GREEN — F5** |
| Erase entry body before yielding | **GREEN — F5** |
| Skip release only on task cancellation | **GREEN — F5** |
| Break generated request/response message names | **RED** |
| Lose the invalid-marker sentinel | **RED** |
| Omit ordinary EOF/read-error/explicit-close release | **RED** |

## Scope, size, and integrity

The candidate is one clean commit on the stated sole parent, is non-shallow,
has no replacement refs, and passes `git diff --check`. A temp-index replay of
the exact base-to-candidate binary patch reproduces the candidate tree.

```text
base tree:             1a8008d556065c34665c422e514c23a2733b9892
candidate tree:        e6b50b0109585196cc77edcca21204ac4a185aea
candidate binary diff: b915da833ccd92413fbd89f5da12c5b64e76a0ff9481dfa78444cc0e4cc65f12
stable patch id:       bca56526e97b094d2a2d296015216a369fedac3b
```

The delta changes 12 handwritten files by +1,194/-7:

- production: +501/-7 across native dispatch/spool plumbing and Python
  bridge/CLI/client plumbing;
- tests: +693/-0 across the three focused S3.5 modules.

That is about 2.65 times the plan's aspirational ~450 lines including tests;
production alone is near the estimate. Inspection of every changed seam found
no renderer, protocol schema/generated artifact, dependency, Cargo/lock,
human-output, Git, artifact, or unrelated command change. The overrun is mostly
the focused test matrix and is not a separate blocker, though F4/F5 show that
volume has not supplied the required mutation distinctions.

## Proportional verification

The fast-test instruction was followed. No broad Python suite, core suite, or
compiler mutation matrix was run.

Reviewer-run evidence on exact `c34f09f`:

- S3.5 CLI/client/native focused tests — 31/31 passed;
- log protocol, codec, and native-bridge focused tests — 13/13 passed;
- direct two-page real-history probe — 132 records in exact order, then EOF;
- direct blocked-read cancellation probe — release observed;
- parser parity probes — repeated boolean, abbreviation, and out-of-i64 cap
  accepted by Python and rejected with 2 by exact Rust;
- real closed-pipe comparison — Python producer 120 with BrokenPipe spray;
  Rust producer 0 without spray;
- protocol drift check — exit 0, fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`;
- protocol regeneration check — exit 0;
- Python compileall — exit 0;
- `cargo fmt --all -- --check` — exit 0;
- `cargo check --locked --all-targets` — exit 0;
- strict `cargo clippy --locked --all-targets --all-features -- -D warnings`
  — exit 0; and
- candidate/core/CLI worktrees and indexes — clean at their exact identities.

All source mutants ran only in disposable exact-candidate worktrees. The
reviewer's body-erasure worktree was restored byte-exact; the original
candidate remained read-only.

## Round-2 acceptance gate

Do not land or push `c34f09f`. Restrict round 2 to S3.5-F1 through S3.5-F5 and
integrity:

1. exact non-negative-i64 cap parsing and clean max+1 rejection/2;
2. real and injected machine-output EPIPE termination/0 with no spray;
3. Clap-equivalent no-abbreviation and repeated-singleton behavior;
4. complete active/absent CLI-to-client lowering plus exact API Failed
   preservation; and
5. rich ordered multi-page record fidelity, explicit EOF, and exact release on
   EOF/error/early close/task cancellation.

Every named mutant must be RED. Preserve generated protocol bytes, the native
dispatch/registry implementation unless a minimal test seam is necessary,
the frozen marker-invalid pair, and the no-renderer boundary. Do not add S3.6
human or machine record rendering during remediation.

## Terminal round-2 review

- Date: 2026-08-31
- Reviewer: same independent reviewer as round 1
- Round-1 report commit:
  `4a90881ce6401f9ea29d59dda4a92aa7c6320c2b`
- Exact final `gwz-py` candidate:
  `f827e30fd28c7322c9d70883b8a6d9a873bbfd0a`
- Exact base / sole parent:
  `10aec7fd69c2f94d90d4aead2bf125f76267b01a`
- Round-1 comparison candidate:
  `c34f09fdbe8484b547dfb2d12cf7053da10f9d3e`
- Exact sibling `gwz-cli` authority:
  `e783b7390c7fe5e7d1f0df9c982fdf59dadd6940`
- Exact sibling `gwz-core`:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Scope: the five round-1 findings and final integrity only

### Terminal verdict: NO-GO — S3.5 freezes

F1, F2, F4, and F5 are cured. The final candidate rejects a cap above signed
i64 before dispatch, handles real and injected log machine-error EPIPE as
clean exit 0, pins every active and absent handler field plus exact Failed API
status, and proves rich ordered multi-page record delivery and cancellation
release. Every prescribed mutant for those four findings is red.

F3 is not fully cured. Abbreviated options and repeated log-specific switches
are now rejected, but global non-repeatable options on the log-visible surface
remain repeatable before, after, and split around `log`. This is the exact
global-one-shot clause of round-1 F3, not wider discovery. The terminal review
therefore has **0 P0 / 0 P1 / 1 P2 / 0 P3**. Under the two-round cap there is
no third remediation review; S3.5 freezes at NO-GO.

### Residual finding

#### S3.5-R2-F3 — P2 — Global non-repeatable log options still accept repetition

The amendment adds a one-shot action for log-local booleans and disables
argparse long-option abbreviation. The global actions in
`src/gwz/cli_shared.py`, however, remain ordinary `store_true` or scalar
actions. Their root and command-local values are then ORed or resolved by last
value during normalization.

The exact final Python parser accepts all 13 global options that should be
singletons when each is repeated before, after, or split around `log`:
`--root`, `--all`, `--dry-run`, `--partial`, `--force`, `--sync`, `--remote`,
`--jobs`, `--max-per-host`, `--progress-interval`, `--json`, `--jsonl`, and
`--ssh-timeout`. Representative direct results are:

| Invocation | Final `gwz-py` | Exact Rust `gwz` |
|---|---|---|
| `--all --all log` | accepted | exit 2, cannot be used multiple times |
| `log --all --all` | accepted | exit 2, cannot be used multiple times |
| `--json --json log` | accepted | exit 2, cannot be used multiple times |
| `log --jobs 2 --jobs 3` | accepted, `jobs=3` | exit 2, cannot be used multiple times |
| `--jobs 1 log --jobs 2` | accepted, `jobs=2` | exit 2, cannot be used multiple times |

Intentionally repeatable selectors remain correct: `--target a --target b`
is accepted and preserves both values. The defect is therefore specifically
failure to distinguish repeatable selectors from global singleton options.
The checked-in round-2 tests cover global abbreviations but contain no repeated
global-singleton matrix, so this wrong behavior ships in the final candidate.

### Round-1 finding regrade

| Finding | Terminal result | Evidence |
|---|---|---|
| F1 — signed-i64 cap | **CURED** | `0..=i64::MAX` is accepted and `i64::MAX + 1` is a parser rejection with process exit 2 and no traceback. Removing the upper bound makes both parser and real-process regressions red. |
| F2 — log machine-error EPIPE | **CURED** | JSON and JSONL injected BrokenPipe return 0 immediately; a real closed pipe exits 0 without stderr spray; non-EPIPE output failure stays 1. Restoring the unguarded write makes the injected and real-pipe tests red. |
| F3 — exact accepted vocabulary | **FAIL / residual P2** | Abbreviations and repeated log-local booleans are cured and mutation-tight. Repeated global singletons remain accepted as recorded above. |
| F4 — lowering and Failed identity | **CURED** | Active and absent real handler seams pin every field and tri-state. Exact programmatic Failed remains Failed with entry/degradation output consumable and released. Since/until swap, dropped strict/tagged/no-merges/first-parent, and Failed-to-Partial mutants are all red. |
| F5 — complete delivery and cancellation | **CURED** | Generated-CBOR Data[2], Data[2], EOF preserves exact four-record order and all rich fields, cursors `None -> 19 -> 41`, and one release. Blocked-read cancellation propagates and releases once. Reverse/drop/early-return/body-erasure/cancellation-release mutants are all red. |

The F5 fixture pins complete member vectors, 40-character hashes, parent order,
author and committer identities, offsets and exact seconds, subject/body,
lossy, marker-invalid, and full root/member degradation fields. The final
native dispatch, bridge, and client production blobs are byte-identical to
round 1; F5 is a test-only acceptance cure.

### Integrity and preservation

The final candidate is a clean one-commit rewrite, not an additive child of
the round-1 candidate. Both have the required base as sole parent, and the
round-1 and final candidates are one commit on each side of that base. A
temp-index replay of the exact base-to-final binary patch reproduces the final
tree.

```text
base tree:                    1a8008d556065c34665c422e514c23a2733b9892
round-1 tree:                 e6b50b0109585196cc77edcca21204ac4a185aea
final tree:                   69d96c9ddf355246087de4aeba4e55ad08f7319e
base-to-final binary diff:    6850d18b3180994fe274a47d3443cb8ceea012c649b4a66fe07c577ad4161ba9
round-1-to-final binary diff: 17011e9ddcd4db0ff24fbbe63d83f1603091b32b9c463e8fd56f4ddcbd73b1b0
final stable patch id:        5a91fca6c51044901292903d830f242ee269c956
```

The round-2 amendment changes only four files by +574/-12:

- production: `src/gwz/cli.py` and `src/gwz/cli_log.py`, +83/-11;
- tests: `src/tests/test_cli_log.py` and `src/tests/test_client_log.py`,
  +491/-1.

The cumulative base-to-final delta remains the original 12-file S3.5 surface,
now +1,756/-7: production +573/-7 and tests +1,183. No native dispatch,
registry, bridge, client production, renderer, generated protocol, schema,
dependency, Cargo/lock, documentation, core, or Rust CLI byte changed versus
round 1. The frozen `LogMergeKind.none` plus `gwz_commit_id="marker-invalid"`
pair and no-renderer boundary are preserved.

The repositories are non-shallow and have no replacement refs. The exact
candidate, sibling core, and sibling CLI worktrees are clean; the diff check
is green. No candidate source was modified during review.

### Proportional terminal evidence

The fast-test boundary was preserved. No broad Python suite, core suite, or
compiler mutation matrix was run.

- Focused CLI/client/native/protocol/codec/native-bridge set: 67/67 passed.
- Focused client plus real native log set: 14/14 passed.
- Log protocol shape/additivity set: 3/3 passed.
- Protocol drift check: exit 0, fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`.
- Protocol regeneration check: exit 0.
- Python compile-all check with bytecode redirected outside the candidate:
  exit 0.
- Direct Python/Rust repeated-global matrix: Python accepted; exact Rust
  rejected representative all/json/jobs cases with exit 2.
- All F1, F2, F4, and F5 mutants named above: red.
- Candidate diff, topology, replay, blob-preservation, and cleanliness checks:
  green.

This terminal NO-GO is solely the uncured round-1 F3 global-singleton parity
condition. No third-round gate is authorized.
