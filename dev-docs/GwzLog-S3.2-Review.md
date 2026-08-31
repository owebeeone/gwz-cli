# `gwz log` S3.2 Single-Axis Review

- Date: 2026-08-31
- Reviewer: independent S3.2 reviewer
- Authority / sole base: `e783b7390c7fe5e7d1f0df9c982fdf59dadd6940`
- Exact `gwz-cli` candidate: `803d9929397cf5f4aa3f148164f20b73ee9a8e7b`
- Candidate tree: `84a03e72593f05c57ab4853ce344b6d7da0b844a`
- Exact sibling `gwz-core`: `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
- Scope: S3.2 only — L-OUT-1/L-OUT-2/L-OUT-4/L-OUT-5,
  L-ENV-10/L-ENV-11, human output lifecycle, generated help/docs, no-pager,
  no-machine-output creep, mutation tightness, and scope/LOC

## Verdict: NO-GO

The production implementation is contract-aligned: compact and full human
rendering preserve the required member attribution and recorded offsets,
degradations go to stderr, arbitrary `i64` instants render without locale or
host-time conversion, C0 data is sanitized, auto color keys only on stdout
TTY state, zero-entry output is empty, and the caller-owned spool is released
on every terminal path. The standing Q-11 no-pager ruling remains unamended
and is followed. No production-code contract violation was found.

The acceptance package nevertheless false-passes two exact mutations on
mandatory review axes. Its EPIPE fixture cannot distinguish immediate stop
from continued spool consumption, and no success fixture protects the S3.2
ownership boundary against taking over JSON/JSONL rendering. Under the
single-axis review ritual these P2 acceptance defects require a narrow second
round.

No P0 or P1 finding was found. There are two P2 findings and no P3 finding.

## Findings

### S3.2-F1 — P2 — The EPIPE fixture does not prove immediate termination

The implementation correctly returns immediately on the first stdout
`BrokenPipe` at
[`log_exec.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.2-review/gwz-cli-candidate/src/log_exec.rs:146),
then releases the caller-owned spool at line 110. The test named
`broken_pipe_is_immediate_success_and_releases_unread_log` at
[`g09.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.2-review/gwz-cli-candidate/src/tests/g09.rs:638)
uses a history with only one output entry. Its one-write assertion therefore
does not observe whether the runner returns at the failed write or continues
reading the spool to EOF after it.

The exact reviewer mutant replaced the immediate return with a continuation
after the failed write. All 21 `g09` and all 8 `g10` tests remained green.
That mutant violates L-ENV-9's “stop emitting” rule and can consume later
batches or process later degradation records after the downstream consumer is
gone. Release-at-the-end is necessary but does not prove early termination.

Required remediation: use a real spool containing at least two independently
renderable entries, preferably crossing a batch/cursor boundary or followed by
a degradation record. Instrument the stdout writer and, where practical, the
registry/read seam. Break on the first stdout write and assert exit 0, exactly
one output attempt, no later stdout/stderr emission or record read, no error
spray, and released id. The exact continue-after-BrokenPipe mutant must be red.

### S3.2-F2 — P2 — No success fixture protects the machine-output ownership boundary

The production branch at
[`log_exec.rs`](/Users/owebeeone/limbo/gwz-log-worktrees/s3.2-review/gwz-cli-candidate/src/log_exec.rs:89)
correctly invokes the new human renderer only for `OutputMode::Human`; JSON and
JSONL retain S3.1's plumbing response for S3.3 to replace. This is the right
S3.2 boundary, but the focused suite never drives a successful log request in
JSON or JSONL mode. `g10` has no machine-output case, while `g09` covers only
machine **error** EPIPE.

The exact reviewer mutant widened the human-rendering branch from `Human` to
`Human | Json | Jsonl`. All 21 `g09` tests remained green, and static
inspection confirms no `g10` assertion can distinguish the mutation. It would
make S3.2 silently consume and render commit records in the two modes owned by
S3.3.

Required remediation: drive the same successful real workspace through
Human, JSON, and JSONL. Assert that only Human drains/renders the entry stream,
that JSON and JSONL retain their exact S3.1 plumbing-level output, that no
human subject/member/degradation bytes leak into either machine response, and
that every registry id is released. The exact widened-branch mutant must be
red. This is an acceptance-only correction; no machine renderer or S3.3 schema
work belongs in S3.2.

## Requirement and lifecycle audit

| Surface | Result | Evidence |
|---|---|---|
| L-OUT-1 member attribution | PASS | Singleton compact output is the plain workspace-relative path; sets of two or three list every path; larger root-containing sets use `[root+N]`, other sets use `[N members]`; full output iterates the complete member table. The `<=3` to `<=2` mutant is red. |
| L-OUT-2 compact/full shapes | PASS | Compact is exactly date, member set, 12-character representative hash, and complete subject. `--full` emits a git-style commit block, complete ID/path/hash table, author identity/date, and optional body. No width truncation is applied. |
| L-OUT-4 degradation channel | PASS | The real zero-entry/unborn run keeps stdout empty and writes the sanitized member/reason summary to stderr. All seven accepted core reason variants have explicit text. |
| L-OUT-5 color/no pager | PASS | `always`, `never`, and `auto` reduce solely from the flag and stdout TTY boolean; colors wrap human entry/degradation labels only. Log never calls the existing pager module. The Q-11 post-review resolution remains blank, so the standing “no pager” ruling still governs. |
| L-ENV-10 lossy/C0 | PASS | Accepted S2.7 core projects arbitrary Git bytes with `String::from_utf8_lossy` and a `lossy` bit. S3.2 preserves U+FFFD, maps tab to one space, maps every other C0 including ESC to U+FFFD, and preserves only full-body newlines. The ESC pass-through mutant is red. |
| L-ENV-11 dates/determinism | PASS | Compact dates use the representative commit's committer seconds and recorded offset. Manual `i128` civil arithmetic covers `i64::MIN`/`i64::MAX`; there is no chrono/locale/TZ dependence or local conversion. Swapping author time for committer time is red. Zero entries return 0 with empty stdout. |
| EPIPE / non-EPIPE output | PASS implementation / FAIL acceptance | Production exits 0 on first BrokenPipe and returns typed `IoError` for other output failures. F1 records the exact surviving continuation mutant. |
| Caller-owned lifecycle | PASS | The spool id is released after Human and machine dispositions, including success, empty output, BrokenPipe, non-EPIPE output failure, and invalid record/read failure. Removing the release is red. |
| Machine ownership boundary | PASS implementation / FAIL acceptance | Production preserves S3.1's non-Human plumbing branch and adds no S3.3 renderer/schema. F2 records the exact surviving branch-widening mutant. |
| Help/docs | PASS | `--full`, compact/full descriptions, member attribution, no-pager behavior, color policy, and examples are present; `docs/CLI.md` is generator-clean and the log command page is in MkDocs navigation. |

## Mutation audit

| Mutant / failure mode | Result |
|---|---|
| Compact small-set boundary changed from `<=3` to `<=2` | RED on the three-member exact-path assertion |
| Compact date changed from committer identity/offset to author identity/offset | RED on the `+0530` versus `-0100` assertion |
| ESC passed through rather than mapped to U+FFFD | RED on path/subject terminal-control assertions |
| Caller-owned spool release removed | RED on real-runner released-id lookup |
| Continue consuming after the first stdout BrokenPipe | **GREEN — S3.2-F1** |
| Widen human record draining/rendering to JSON and JSONL | **GREEN — S3.2-F2** |

The existing full-member-table, extreme-`i64`, no-truncation, exact color,
degradation, empty-output, and non-EPIPE assertions are proportional and
direct. The two green mutants are not requests for a wider matrix; they are
the missing properties on the reviewer's named lifecycle and ownership axes.

## Scope, generated artifacts, and integrity

Candidate identity is exact: one clean commit with sole parent
`e783b7390c7fe5e7d1f0df9c982fdf59dadd6940`, no rename or mode change, and a
green `git diff --check`. The candidate changes 14 files by `+908/-30`:

- handwritten production: `+427/-20` (447 lines of churn, about 1.5x the
  aspirational `~300` implementation target);
- focused tests/module registration: `+403/-8`;
- command/help docs and navigation: `+78/-2`.

The total handwritten churn is about 3.1x the aspirational target. The overrun
is not a separate finding: the production renderer/lifecycle split is direct,
most implementation size is the required deterministic date formatter and
complete renderers, docs are explicitly owned here, and tests carry the
specified boundary matrix. No unnecessary public abstraction or semantic
duplication was found. Test volume does not, however, substitute for the two
surviving mutants.

There is no `Cargo.toml`, `Cargo.lock`, dependency-pin, protocol/schema,
generated-wire, core, checked-artifact, source-loading inventory, pager,
public-seam, or frozen-registry change. All new CLI modules and functions are
crate-private. The existing diff pager remains byte-untouched. Machine success
rendering is byte-preserved in production; only its missing regression is F2.

## Proportional evidence

The operator's fast-test policy was followed: no 1,700+ suite and no 69-case
compiler mutation matrix were run.

- `cargo test --locked tests::g10` — 8/8 S3.2 tests passed.
- `cargo test --locked tests::g09` — 21/21 S3.1/lifecycle tests passed.
- Continue-after-BrokenPipe reviewer mutant — all 21 `g09` and all 8 `g10`
  tests remained green.
- Human-branch-widened-to-JSON/JSONL reviewer mutant — all 21 `g09` tests
  remained green; `g10` contains no machine-output case.
- Small-member-boundary reviewer mutant — RED.
- Author-for-committer date reviewer mutant — RED.
- ESC pass-through reviewer mutant — RED.
- Registry-release removal reviewer mutant — RED.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --locked --all-targets` — exit 0.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` — exit 0.
- `python3 scripts/generate_cli_reference.py --check` — exit 0.
- Direct checked-artifact source boundary on exact sibling core — exit 0,
  15 visible entries / 5 classified modules.
- Release-boundary unit suite — 6/6 passed.
- Exact candidate, sibling, report, and mutation worktrees were clean after
  every reviewer mutation was restored.

## Round-2 acceptance gate

Round 2 should be restricted to S3.2-F1 and S3.2-F2 plus candidate integrity.
Accept only when a multi-record/sentinel EPIPE fixture proves immediate stop
and release, successful Human/JSON/JSONL fixtures kill the ownership-branch
mutant without implementing S3.3, all production files are byte-identical to
the reviewed candidate unless a testability-only seam is strictly necessary,
the focused/formal gates remain green, and no protocol, machine schema,
renderer, public surface, pager, pin, inventory, or sibling-core change rides
the remediation.
