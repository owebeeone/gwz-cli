# `gwz log` S4.1 final independent settle review

- Date: 2026-08-31 (Australia/Sydney)
- Review mode: final independent settle review; product candidates read-only
- Exact `gwz-cli` candidate:
  `59da08a230c1c7f8cf80d16c0b50013fcfc80f2f`
- CLI sole parent: `098e1b7f3219fa4f1e23540c40e069caacf512c4`
- CLI tree: `670870d79f720bc7ed167d8442b0231e1f450bf5`
- Exact `gwz-core` candidate:
  `eb3a37c3d657b28c9fb3c85054056aa9192ee353`
- Core product-gated sole parent:
  `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Core tree: `fda06e172b75a4f85a7cbda4fbc2397e2a6fad18`
- Exact `gwz-py` candidate:
  `4c5ad072d3c191a6e1b6b34c62037f9e715d5b2d`
- Python sole parent: `ec1b01f1801c930da930acefbf8d48f7e612ce96`
- Python tree: `7be7eaff1a82a5c3f6b158e8bca8b6089da7e753`

## Verdict

# GO

All implemented v0 requirements are satisfied and graded: **55/55 pass**,
including **L-ENV-1..14**. The two other labeled rows, **L-COA-8** and
**L-OUT-3**, are explicitly and correctly deferred to v2. The complete
inventory is therefore **57/57 accounted for** with no missing, extra, or
ungraded row.

The retained S3.4 `:^` pathspec gap is closed through a checked-in real
Rust/Python/native-Git parity case. The previously omitted S1.2 documentation
duty is now complete and agrees with the landed marker serde contract, root
trailer authority, and terminal L-COA-8 disposition. The exhaustive gate
ledger is internally consistent and its two non-green events are classified
honestly: an interrupted ENOSPC Clippy attempt was rerun successfully, while
standalone `cargo test` is not a valid macOS gate for this PyO3
`extension-module` cdylib and is replaced by the repository's actual
Maturin-plus-pytest release/CI path.

Final finding count:

| Priority | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 0 |

No condition, ungraded gate, or open implemented-v0 requirement remains.

## Supersession and review independence

The initially supplied CLI settle commit `5df33799...` was not verdicted. The
audit found three documentation defects before filing: a wildcard where the
traceability preamble promised exact test names, an overbroad statement about
the stale umbrella core checkout, and the genuinely unexecuted S1.2 marker
document reconciliation. The builder replaced the candidate rather than
asking this report to waive them.

The final CLI candidate names all eight L-COA-2 negative tests, narrows the
umbrella statement to the byte-identical J-7/G23 source subset while naming
both core identities, records the docs-only S1.2 child, and changes the S1.2
charter from a nonexistent retry fix to the actual terminal disposition. The
final core child performs the missing reconciliation. Intermediate
`5df33799...`, `eb65c399...`, and `b99c8dbf...` are not the approved CLI
identity.

## Exact scope, topology, and integrity

All three final candidates are clean, non-shallow, replacement-free,
single-parent commits on their required bases. `git diff --check` is green for
every range.

| Repository | Exact final delta | Result |
| --- | --- | --- |
| gwz-cli | Three documentation files, `+351/-3`: new gate ledger, new traceability table, Plan settlement/disposition edits | Docs-only; all source, tests, Cargo files, generated CLI docs, pins, and public surfaces equal the parent |
| gwz-core | Only `dev-docs/GwzCommitMarker.md`, `+59/-18` | Docs-only child of the exact product-gated core; `src/`, tests, protocol, manifests, lock, generated artifacts, scripts, and public surfaces equal `bdb398c3...` |
| gwz-py | Only `src/tests/test_log_real_workspace.py`, `+14/-5` | Test-only `:^` matrix/sentinel change; Python production, native code, protocol, renderer, manifests, locks, and dependencies equal the parent |

Independent patch identities:

| Repository | Binary-diff SHA-256 | Stable patch id |
| --- | --- | --- |
| gwz-cli | `2a5b61abd6b703811cafb5ff0a8b494124db67da758afded229d8ce67936ece2` | `16dfe089d3b24839033aaea3b2073da19048e86c` |
| gwz-core | `bee2a6b31b0b9ae69d8dfd7f2b0c9a3ca165c89eeda3d56bd32a7e0b4c993c14` | `1be0e2a35c696a91bf95ff18eef954b591641092` |
| gwz-py | `588d9ce4fab0e959d1e5832a977873205c44eaaf9eb94ba1d975e4261d78061e` | `62500fc6f80334b97e07eab32a5d0c1a9fea0243` |

The CLI lineage contains the terminal accepted S2/S3 reports and preserves
the accepted CLI `src` tree. Core product history is the exact accepted train
through `bdb398c3...`; the final child changes no product byte. Python is the
terminal accepted S3.4 package plus the one chartered test-only row. No
release tag, push, version, schema version, dependency pin, root manifest,
source-loading inventory, or public API change is part of S4.1.

## Traceability inventory

The requirements and traceability table each contain exactly 57 unique
`L-*` identifiers. Automated and manual reference checks found no broken path
or test symbol in the final table. The 55 implemented rows have checked-in
acceptance and terminal GO lineage; the two deferred rows have explicit
dispositions rather than false PASS labels.

### Selection, operands, and history

| Row | Result | Representative checked-in evidence |
| --- | --- | --- |
| L-SEL-1 | PASS | CLI and Python parser/lowering tests for operands, post-`--` pathspecs, and selectors |
| L-SEL-2 | PASS | Core default root-plus-active-members selection fixture |
| L-SEL-3 | PASS | Exact local-tag intersection/narrowing and `+`-operand refusal fixtures |
| L-RNG-1 | PASS | Shared diff grammar, two/three-dot ranges, exact Git-magic routing, and real native-Git oracle including `:!` and `:^` |
| L-RNG-2 | PASS | Per-repository no-operand HEAD histories |
| L-RNG-3 | PASS | Per-member snapshot and snapshot-range resolution |
| L-RNG-4 | PASS | Per-member lock pins, open-right range, root/missing/unborn degradation |
| L-RNG-5 | PASS | Local-only/no-mutation test and real promisor-clone no-lazy-fetch test |
| L-RNG-6 | PASS | Internal dots, standalone legacy ids, and complete open-boundary teaching-refusal matrix |

### Coalescing, tolerance, ordering, filters, and performance

| Row | Result | Representative checked-in evidence |
| --- | --- | --- |
| L-COA-1 | PASS | Real trailer siblings, strict canonical UUIDv7, and wrong-variant marker-invalid cases |
| L-COA-2 | PASS | Positive fan-out plus all eight exact negative functions: author, email, both windows, distinct markers, marked/unmarked, same-repository twins, and rebase restamps |
| L-COA-3 | PASS | `--no-coalesce` singleton groups |
| L-COA-4 | PASS | Latest group time and provenance matrix |
| L-COA-5 | PASS | Filtered survivor narrowing and empty success |
| L-COA-6 | PASS | All provenance values and exact machine tokens |
| L-COA-7 | PASS | Inclusive W=60/W=61 split, exact frontier eligibility/K=64, immutable fragments, and flat non-monotone high water |
| L-COA-8 | DEFERRED v2 | S1.1-B terminal fallback; v0 promises safe split, not retry identity; S1.2 records it |
| L-COA-9 | PASS | Mangled/identical invalid markers remain independent marker-invalid groups |
| L-TOL-1 | PASS | Unreadable member degrades without stopping peers |
| L-TOL-2 | PASS | Independent revision degradation and strict promotion |
| L-TOL-3 | PASS | Unborn repository contributes degradation and no entry |
| L-TOL-4 | PASS | Detached HEAD logs normally |
| L-TOL-5 | PASS | Every locally available shallow commit is retained |
| L-TOL-6 | PASS | Conf-integrity mismatch does not gate reads |
| L-ORD-1 | PASS | Repository cursor matches native Git default order |
| L-ORD-2 | PASS | Absolute-time global merge with least-member/hash tie break |
| L-DEP-1 | PASS | Global default 50, explicit N/zero/no-limit, and range/since/until lift with exact >50 histories |
| L-FIL-1 | PASS | Raw message/author filters and native first-parent/no-merges/range parity |
| L-PRF-1 | PASS | Window-bounded streaming high water and immediate cap stop |
| L-PRF-2 | PASS | Actual jobs overlap ceiling, complete-event equality, and real path-reader lifetime bound |

### Output, protocol, clients, and lifecycle

| Row | Result | Representative checked-in evidence |
| --- | --- | --- |
| L-OUT-1 | PASS | Compact/full date, member-set, identity, subject, body, and table tests |
| L-OUT-2 | PASS | Default compact/full switch and real-runner spool release |
| L-OUT-3 | DEFERRED v2 | Grouped per-repository rendering deliberately outside v0 |
| L-OUT-4 | PASS | Human degradation summary remains stderr-safe and complete |
| L-OUT-5 | PASS | Exact color policy, help, generated docs, and no-pager contract |
| L-JSN-1 | PASS | One ordered schema document with uniform complete member arrays and exact fields |
| L-JSN-2 | PASS | Stable degradation reasons and optional context |
| L-PRO-1 | PASS | Additive Rust wire values/projection and Python pre-log shape preservation |
| L-INT-1 | PASS | Dispatch, cursor/Data/EOF/release, post-registration cleanup, and strict aggregate |
| L-PY-1 | PASS | Python flag/default/operand parity and complete real-seam request lowering |
| L-PY-2 | PASS | Structured API request/records, multi-page delivery, early close/cancel/error release |
| L-PY-3 | PASS | Captured and live compact/full/machine cross-language byte oracles |
| L-EXIT-1 | PASS | Complete degradation truth table and real 0/1/2 runner mappings |

### Executable-environment rows

| Row | Result | Representative checked-in evidence |
| --- | --- | --- |
| L-ENV-1 | PASS | Signed-i64 absolute instants, preserved offsets, exact seconds, and overflow-safe legacy slot |
| L-ENV-2 | PASS | Inclusive window and non-monotone frontier fragments with repeated provenance |
| L-ENV-3 | PASS | Honest seen-sibling cap closure and exact zero-beyond-cap-yield sentinel |
| L-ENV-4 | PASS | Constant-density non-monotone tails have flat high water; jobs outputs are byte-complete identical |
| L-ENV-5 | PASS | Raw author/committer separation, regex semantics, and pre-workspace invalid-regex refusal |
| L-ENV-6 | PASS | Exact RFC3339/local/date/epoch grammar, inclusive bounds, i64 extremes, DST gap/overlap refusal |
| L-ENV-7 | PASS | Filtering precedes cap/order for every jobs value and preserves group survivors |
| L-ENV-8 | PASS | Omitted/zero/no-limit lowering, conflicts/negatives, every behavior flag, and parser parity |
| L-ENV-9 | PASS | Human/machine EPIPE stops immediately, emits no spray, and releases without hidden reads |
| L-ENV-10 | PASS | Lossy text plus C0 sanitization, member boundaries, color, and no width truncation |
| L-ENV-11 | PASS | Commit-own-offset dates, TTY-only auto color, and exact zero-entry behavior |
| L-ENV-12 | PASS | Exact i64 seconds/offset object, full hashes/parents, lossy bit, and UTF-8/escaping edge |
| L-ENV-13 | PASS | Exact JSONL header/one-line records/EOF and canonical empty/one-record bytes |
| L-ENV-14 | PASS | Actual Python machine bytes match Rust, including the lossy edge and anti-collapse oracle |

Counts reconcile as 41 implemented non-ENV rows + 14 implemented ENV rows +
2 deferred rows = 57 total labels.

## Deferred and prior-finding disposition

- **L-COA-8:** S1.1 and S1.1-B terminal NO-GO candidates did not land. The
  pre-authorized fallback moves retry identity to v2 artifact-assisted
  association. v0 may mint a fresh marker after a partial commit and never
  heuristically fuses distinct valid marker identities.
- **L-OUT-3:** grouped/per-repository rendering and graphing remain deliberate
  v2 work; compact/full/machine v0 output is complete.
- S2.2's open legacy-boundary defect is cured by S2.2-B.
- S2.5's non-monotone-memory and cap-acceptance defects are cured by S2.5-B.
- S3.5's repeated-global parser residue is cured by S3.5-B.
- S3.4's retained `:^` P3 is cured by the final Python test-only candidate.
- The path-history implementation's accepted CPU/process-startup tradeoff
  remains explicit: an unlimited path walk may start one synchronous
  `git rev-list --skip=N` process per pull. It does not violate the memory,
  cleanup, or jobs-ceiling contracts.

No terminal report leaves an implemented-v0 row open.

## S1.2 marker-document reconciliation

The final core document no longer calls the schema proposed, omits no landed
top-level field, predicts no `gwz log --merged` marker-file lookup, or promises
a retry-identity fix that did not land.

The documented marker shape matches the actual serde model:

- `MarkerArtifact` carries the existing identity, actor, root, target, and
  member fields plus optional additive `merge`;
- `MarkerMergeArtifact` carries `merge_id`, `operation_id`, `source_ref`,
  selected targets, participants, and optional `root_merge_commit`;
- participant `target_kind` is the serde snake-case `root`/`member` enum;
- outer and merge target vectors must match, every selected target has the
  correctly typed participant, and root evidence is present exactly when
  `@root` participates and equals its resulting commit; and
- absent optional fields are omitted while old `gwz.marker/v0` artifacts
  without `merge` remain readable.

The shipped log authority description is also exact: Git histories and valid
canonical `GWZ-Commit-ID` trailers identify root/member entries; marker files
remain workspace evidence, not the v0 root lookup or coalescing authority.

Reviewer-focused checks on exact `eb3a37c...` passed:

- marker round trip: 1/1;
- additive merge round trip and old-marker compatibility: 1/1;
- merge target/root-evidence validation: 1/1; and
- shipped marker/history coalescing: 1/1.

The stale strings are absent. A direct product-path diff against gated parent
`bdb398c3...` is empty, so carrying the exhaustive product evidence forward
to this docs-only child is valid and a broad rerun would add no product
coverage.

## `:^` closure

Python candidate `4c5ad072...` factors the native pathspec cases into one
shared table and adds `(".", ":^side-only.txt")` beside long exclusion,
`:!`, and `:(top)`. The behavior test executes each case through distinct
Rust and Python subprocesses, compares their complete process results, and
then compares exact ordered hashes with native `git rev-list`. A separate
sentinel requires both short aliases, so deleting only `:^` is RED.

The reviewer reran the alias sentinel and real behavior node together: **2/2
passed in 19.52 seconds**. Collection independently confirms the amended real
battery has 36 tests and the complete Python suite has 571 tests.

## Exhaustive evidence classification

The definitive builder gate ran serially and is accepted rather than repeated:

| Partition | Exact result |
| --- | --- |
| Core format/check/strict Clippy | exit 0 after the disclosed Clippy environment retry |
| Core full test | lib 1,799 passed / 0 failed / 1 ignored (1,800 total), integrations 10/10, 33/33, 9/9, 2/2, docs 0 |
| CLI format/check/strict Clippy/docs | exit 0 |
| CLI full test | lib 122, integrations 26/26, 25/25, 4/4, 2/2, 2/2, docs 0 |
| Python native build/format/check/strict Clippy | exit 0 through exact ABI3 rebuild and compiler gates |
| Python full test | 571 passed |
| Real Rust/Python workspace battery | 36 passed |
| Core protocol | additive fingerprint `d0c205c8767f8d54d32ead2f676a05077d849f6a12278d9de52b3c132c3c9372` |
| Python protocol | drift fingerprint `46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277`; regeneration current |
| Compiler boundary matrix | 69 passed in 554.986 seconds |
| Checked boundary / release boundary | 15 visible, 5 modules / 6 passed |
| Scenario/compatibility/docs | 39 rows + 43 tests + 22 registry rows; 7 rules + 7 bindings + 10 shapes; 27 tests; 12 sources + 155 assertions; 3 tests |
| G23 | 124 passed, 1,676 filtered (total 1,800) |

### ENOSPC retry

The first exact-core Clippy dispatch exited 101 before semantic analysis when
only 117 MiB remained. Four ignored `target/` caches in completed historical
log worktrees were removed with `cargo clean`, recovering 15.2 GiB. The exact
partition then ran once to exit 0. Worktree histories, source, indices, and
committed data remained clean. Timestamps and absent ignored targets
corroborate the disclosed sequence; the size figures themselves remain the
builder's direct command record. The failed dispatch is an environment event,
not a green test and not concealed as one.

### PyO3 standalone `cargo test`

The diagnostic exit 101 is correctly excluded. This crate is a `cdylib` with
PyO3 `abi3-py310` plus `extension-module`; on macOS the extension leaves `_Py*`
symbols for the loading interpreter, so a standalone Rust test binary is not a
loadable extension and cannot link. The retained Cargo diagnostic has exactly
those undefined symbols.

The actual repository gates are stronger for the shipped artifact:
`run_tests.py` performs `maturin develop`, protocol regeneration, and pytest;
release reconciliation performs Cargo check plus that runner and package
smoke; publish/package-smoke workflows build wheels with Maturin and run
pytest/smoke tests. The exact release ABI3 module was rebuilt and loaded by the
571-case suite. There is no missing CI/release gate disguised as an exclusion.

### Reviewer spot checks

Without repeating the broad suites or 69-case matrix, the reviewer observed:

- three-repository `cargo fmt --all -- --check`: exit 0;
- CLI generated reference freshness: exit 0;
- core protocol regeneration/additivity: exit 0 with the recorded fingerprint;
- Python protocol drift/regeneration: exit 0 with the recorded fingerprint;
- exact core per-commit lane gate `834275d...bdb398c3`: exit 0;
- exact `:^` behavior and sentinel: 2/2 passed;
- exact marker/merge/coalescing checks: 4/4 passed;
- Python collect-only totals: 571 complete and 36 real-workspace tests; and
- final candidate status/diff/topology/scope checks: clean.

The gate ledger's J-7 wording is now precise. The umbrella core checkout used
for host-root document checks remained at `834275d...`; only the named
J-7/G23 source subset matched exact `bdb398c3...`. All compilation, the full
core suite, and G23 ran in the exact `bdb398c3...` worktree. The final docs-only
core child does not alter that product evidence.

## Final landing disposition

S4.1 is settled. Land only this exact tuple:

- gwz-cli `59da08a230c1c7f8cf80d16c0b50013fcfc80f2f`;
- gwz-core `eb3a37c3d657b28c9fb3c85054056aa9192ee353`; and
- gwz-py `4c5ad072d3c191a6e1b6b34c62037f9e715d5b2d`.

Do not substitute an intermediate settle tree. This review authorizes no tag,
push, version bump, release-branch operation, or dependency/schema update.
Those remain a later operator decision under `GwzLogPlan.md` section 1.6.
