# `gwz log` S3.6 single-axis review

- Date: 2026-08-31
- Reviewer: independent S3.6 reviewer (the S3.2 builder; not the S3.6 builder)
- Exact `gwz-py` base / sole parent:
  `dc6915545b8c65d01cebc02ba1c3c7f1df9a5f8b`
- Exact `gwz-py` candidate:
  `0bc58a887a9548dbe8b2d2608c8d0ba15915b0ee`
- Candidate tree: `3653d0f51a1f521f9c46070617515c1ecab40cbc`
- Exact Rust CLI authority:
  `39184ec325167a695d1a1a6cee1eac7894a6fe81`
- Rust CLI tree: `c54c0740f8f03159a53d3eefde5ee3280d89e36f`
- Exact sibling `gwz-core`:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Core tree: `20b52eb0b425e8482f4bd853fe4a6a580deb28e3`
- Scope: S3.6 only — L-PY-3 and L-ENV-14; semantic and byte parity with
  terminal S3.2/S3.3; human channels/color/no-pager; machine schema,
  ordering, provenance, body and lossy behavior; EPIPE/read/cancel/release;
  help/docs; mutation tightness; and no protocol/core/Rust creep

## Verdict: GO

The candidate is contract-correct and the acceptance package is
mutation-tight on every reviewed axis. Python compact, full, degradation,
JSON, and JSONL output match the exact Rust authority for the same rich
records. The machine forms are byte-identical, including raw UTF-8 U+FFFD,
the core-owned `lossy` bit, control escaping, member and parent order, exact
signed-i64 times and recorded offsets, optional body, all provenance tokens,
schema framing, and final newlines. Human rendering preserves Rust's recorded
date arithmetic over the full i64 domain, member-set boundaries, C0
sanitization, stdout-only TTY color policy, stderr degradation channel, and
standing no-pager decision.

The CLI streams records without collecting history, stops immediately and
cleanly on stdout EPIPE, distinguishes stderr EPIPE and other I/O failures,
and releases the caller-owned output on prefix failure, normal EOF, early
consumer closure, read failure, invalid records, and cancellation. The
private release helper added for pre-read machine EPIPE does not widen the
public client API. No production or acceptance defect survived the focused
review.

No P0, P1, P2, or P3 finding was found.

## Independent cross-language oracle

The checked-in tests contain captured Rust constants, but the review did not
trust those constants alone. A disposable reviewer-only Rust test constructed
the exact rich S3.3 record set in the Rust authority, invoked the candidate's
live Python production renderers in a separate process, and compared the
actual UTF-8 bytes for all five outputs:

1. compact human entry;
2. full human entry with body/member table;
3. human degradation summary;
4. complete JSON document; and
5. complete JSONL stream.

The live comparison passed byte-for-byte. Its record set included two members,
parent order, negative and positive offsets, exact seconds, C0 in the subject,
U+FFFD plus `lossy=true`, quoted/backslashed body data, a degradation between
entries, and `marker-invalid`. The temporary probe was removed after the
result; neither authority worktree was changed.

## Requirement and behavior audit

| Surface | Result | Evidence |
|---|---|---|
| L-PY-3 compact parity | PASS | Date, member-set forms, 12-character representative hash, complete sanitized subject, spacing, and colors follow `log_render.rs`. The actual CLI path is pinned, not only the direct formatter. |
| L-PY-3 full parity | PASS | Git-style `commit` block, complete ID/path/hash table, character-count padding, author identity/date, body indentation, blank lines, and the runner's extra record separator match Rust. Ignoring `args.full` is red. |
| Human degradation channel | PASS | All seven reason labels are exact; optional operand/message handling and empty-path fallback match Rust; stdout remains record-only and human degradations go to stderr. |
| L-ENV-10 C0/lossy human text | PASS | Tab becomes one space, every other C0 including ESC becomes U+FFFD, body newlines alone are preserved, and no width truncation occurs. ESC pass-through is red. |
| L-ENV-11 time/color | PASS | Euclidean civil-date arithmetic matches Rust at i64 MIN/MAX and uses each record's own offset. Compact uses committer time; full uses author time. `auto` depends only on stdout TTY, while `always`/`never` override it. |
| L-PY-3 machine byte parity | PASS | `json.dumps(... ensure_ascii=False, separators=(",", ":"), sort_keys=True)` matches the Rust `serde_json::Value` ordering/escaping. JSON and JSONL framing, record order, exact bytes, and final LF are pinned through the real Python CLI path and the live oracle. |
| L-ENV-14 lossy edge | PASS | U+FFFD is emitted as UTF-8 and `"lossy":true` follows only the protocol bit; a genuine U+FFFD with `lossy=false` stays unflagged. ASCII-escaping and lossy-bit mutants are red. |
| Time/member/provenance/body | PASS | Exact seconds and offsets, full hashes, parent order, complete member arrays, optional body, `none`, `heuristic`, `marker:<uuid>`, and `marker-invalid` are exact. Invalid offsets or provenance return typed `InternalError`. |
| Streaming/order | PASS | Records are emitted as received; JSON uses comma state rather than retention, JSONL emits one escaped record per line, and zero records have exact canonical bytes. S3.5-B's rich two-page/EOF transport evidence remains intact. |
| EPIPE and output errors | PASS | Prefix write/flush EPIPE stops before the first read; record EPIPE stops before later records; JSON suffix EPIPE is clean; all return 0 and release. Non-EPIPE stdout failure and stderr EPIPE remain execution failures. Swallow-and-continue is red. |
| Read/cancel/release | PASS | The inherited client generator releases on EOF, read failure, explicit close, and blocked-read cancellation. S3.6 additionally closes the live generator before returning from early render termination and directly releases if machine framing fails before generator creation. All omission mutants are red. |
| Help/docs/no pager | PASS | Runtime help and README describe compact/full forms, complete member attribution, human degradation stderr, machine schemas, color policy, and no pager. Python has no pager module or pager invocation; record writes go directly to stdout/stderr. |
| Thin-client boundary | PASS | Candidate consumes generated protocol objects and calls only `Client.log` / `log_output`; it adds no Git, artifact, workspace, core-semantic, or shadow-protocol logic. |

## Mutation audit

Every reviewer mutation was applied only in a disposable exact-candidate
worktree and restored byte-exact before the next probe.

| Exact wrong implementation | Result |
|---|---|
| Change machine JSON to `ensure_ascii=True` | RED on direct and actual-CLI Rust-byte oracles. |
| Invert the protocol lossy-bit condition | RED on the positive lossy and genuine-U+FFFD negative sentinels. |
| Encode `marker-invalid` as `none` | RED on the captured oracle and four-token provenance matrix. |
| Use author time/offset in compact mode | RED on exact compact bytes and both extreme-i64 cases. |
| Pass ESC through human sanitization | RED on path, subject, and terminal-control assertions. |
| Make `--color=auto` ignore a true stdout TTY | RED on the actual CLI TTY matrix. |
| Ignore `--full` at the CLI-to-renderer call | RED on the actual full human runner. |
| Omit `aclose()` after early renderer termination | RED before returning to the still-live event loop. |
| Omit the direct release when machine prefix EPIPE occurs before first read | RED on write- and flush-side prefix sentinels. |
| Swallow BrokenPipe inside the writer and continue reading/emitting | RED in JSON/JSONL and human modes; later writes/reads and degradation spray are observed. |
| Omit client release after read failure, early close, and cancellation | RED on all three inherited lifecycle cases. |

The tests also directly pin degradation tokens, empty machine streams,
invalid payload-arm handling, missing identity offsets, exact body bytes,
member-set count boundaries, and all color modes. No output-only test was
credited where a lifecycle sentinel was available.

## Scope, size, and integrity

The candidate is one clean commit with the required sole parent, is
non-shallow, has no replacement refs, carries no trailers, and passes
`git diff --check`. Its stable patch id is
`d95dabf04e0f12bc61e2be3785a16f954f5f89a1`.

The seven-file delta is `+1239/-10`:

- Python production: `+454/-8` across the private log renderer, CLI driver,
  private render exports, and private release helper;
- focused tests: `+770/-2`; and
- README documentation: `+15/-0`.

Total handwritten churn is about 3.5 times the plan's aspirational ~350-line
target; production alone is about 1.3 times it. This is not a finding. The
renderer is a direct port of the reviewed Rust compact/full/machine behavior,
and the test volume supplies the required exact-byte, i64, reason,
provenance, EPIPE, cancellation, and mutation matrix. No needless public
abstraction or alternate model was found.

There is no native Rust, Cargo manifest/lock, generated protocol, schema,
core, Rust CLI, dependency, output-registry, artifact, pin, inventory, or
unrelated-command change. The candidate's packaged protocol tree is
byte-identical to its terminal S3.5-B parent.

## Proportional evidence

The operator's fast-test policy was followed: no broad Python suite, Rust
suite, core suite, or compiler mutation matrix was run.

- Focused Python S3.6 plus adjacent log/client/native/protocol set:
  **196 passed / 5 skipped**.
- Exact Rust authority `g10`: **10/10 passed**.
- Exact Rust authority `g11`: **11/11 passed**.
- Reviewer live cross-language oracle: **5/5 byte comparisons passed**.
- Eleven exact reviewer mutants: all RED as recorded above.
- Protocol drift under the repository-pinned `taut-proto 0.9.1`: exit 0,
  fingerprint
  `sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`.
- Protocol regeneration check under `taut-proto 0.9.1`: exit 0.
- Python compile-all with bytecode redirected to a disposable temp tree:
  exit 0.
- Native `cargo fmt --all -- --check`: exit 0.
- Candidate and full base-to-candidate `git diff --check`: exit 0.
- Exact candidate, Rust authority, core, and report worktrees were clean.

An initial protocol-drift invocation used the workspace venv's stale
`taut-proto 0.8.1` and reported a different exported-IR fingerprint. The
repository explicitly pins 0.9.1; rerunning both protocol gates in an
isolated 0.9.1 environment produced the green results above. No candidate
bytes changed, and the 0.8.1 result is classified as an environment/tool
version mismatch rather than product evidence.

## Landing identity

Land exactly `0bc58a887a9548dbe8b2d2608c8d0ba15915b0ee` as the one-commit
child of `dc6915545b8c65d01cebc02ba1c3c7f1df9a5f8b`. No substitute tree or
additional amendment was reviewed.
