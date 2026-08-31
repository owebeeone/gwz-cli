# S2.1 peer-BLIND review — NO-GO

Core: `affaa69a…` → `e10387a7…`
CLI docs: `9fae0d4e…` → `5c374fbd…`

Round 1: CURED/N/A classifications are not applicable.

## Findings

- P0: none.
- P1: none.
- P2-1 — Raw commit metadata is altered or silently discarded at [mod.rs:308](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/mod.rs:308). `git2::Commit::message_bytes()` explicitly returns a prettified message with leading newlines removed, while lines 309–312 collapse a non-UTF-8 encoding header into `None`. A valid commit therefore cannot round-trip byte-exactly, and later coalescing/rendering cannot recover the original metadata. The ordinary ASCII assertions at [tests.rs:63](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/tests.rs:63) do not expose either loss mode.

  Remedy: use `message_raw_bytes()`, preserve the encoding header losslessly—such as `Option<Vec<u8>>` sourced from raw commit headers—or emit an explicit degradation instead of treating invalid encoding as absent. Add raw-object tests covering a leading-newline message and a non-UTF-8 encoding header.

- P3: none.

Because P2-1 survives, the required overall verdict is NO-GO.

## Canonical-row coverage

| Row | Verdict | Evidence |
|---|---|---|
| L-SEL-2 | Pass | `CommandDefaultTargets::All` with root allowed at mod.rs:213–241; root/active-member ordering tested at tests.rs:17–45. |
| L-RNG-2 | Pass | Each repository’s HEAD OID is captured at open and pushed lazily at mod.rs:132–160, 257–265. Test at tests.rs:47–71 verifies root/member HEAD histories, detached snapshot timing, parents and structured entry data. |
| L-RNG-5 | Pass | Static call audit found no transport/fetch path. The absent remote and mutation-artifact test is at tests.rs:73–94. |
| L-ORD-1 | Pass | The cursor deliberately retains libgit2’s default `GIT_SORT_NONE`; libgit2 documents this as Git-compatible reverse chronological order and non-precollecting. The topology-discriminating comparison with native `git log` is at tests.rs:96–132. |
| L-TOL-1 | Pass | Repository-open, HEAD, revwalk and commit-read failures become member-scoped events at mod.rs:244–286 and 181–203. Mixed good/unreadable/unsupported coverage is at tests.rs:134–164. |
| L-TOL-3 | Pass | `UnbornBranch` returns an `Ok` history containing zero entries and one benign degradation at mod.rs:266–271; tested at tests.rs:166–188. |
| L-TOL-4 | Pass | `head().peel_to_commit()` supports detached HEAD; tested at tests.rs:190–204. |
| L-TOL-5 | Pass | Native local revwalk respects the shallow boundary; tested at tests.rs:206–226. |
| L-TOL-6 | Pass | The path uses ungated `artifact::read_manifest` and never calls the conf-integrity gate; damaged-marker coverage is at tests.rs:228–242. |

Streaming is structurally sound: `RepositoryMessages` owns a live revwalk and yields one entry at a time. The only collections are O(selected repositories), current-commit parent IDs, and current-entry bytes—no history is precollected. Per-repository failures terminate only that cursor.

## Placement verdict

`src/operation/commit_log/` is the correct existing seam.

- `operation/mod.rs` already privately mounted `commit_log` before this candidate.
- Existing request dispatch already enters its `handler.rs`.
- The unrelated `diff/log_service.rs` remains untouched.
- Selection logic is reused from `workspace_ops`; moving orchestration there or into the low-level Git layer would be a worse fit.
- `commit_log` remains a private child of public `operation`. Although internal records use `pub`, they are unreachable outside the private module path. The only seam-crossing re-export remains the existing `pub(super) handle_log`; therefore `operation`’s externally reachable surface is unchanged.

## Scope and frozen-surface audit

Core candidate changes exactly:

- `M src/operation/commit_log/mod.rs`
- `A src/operation/commit_log/tests.rs`

There is no `lib.rs`, `operation/mod.rs`, root-manifest/source-inventory, protocol, dependency, lockfile, checked-artifact pin, or `diff/log_service` change.

The CLI amendment changes exactly the two requested documents. It contains:

- the dated `2026-08-29` operator ruling;
- both checker refusal phrases with exit 1;
- preserved F15 distinct-home/no-diff-log collision substance;
- preserved core-owns-semantics split;
- current module-home references changed to `src/operation/commit_log/`;
- the adoption-trail entry.

The sole remaining `gwz-core/src/commit_log/` occurrence is explicitly historical text describing the refused original home, not a current path instruction.

## Commands and results

- `cargo test commit_log -- --nocapture`: 10 passed.
- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0.
- Core library suite: 1,693 passed, 1 ignored.
- `diff_render_spike`: 10 passed.
- Protocol target initially had 30 passes and three generator failures because this isolated worktree lacks the expected sibling `taut/src`. Re-running the exact generation, corpus, byte-comparison and additive checks against the workspace’s Taut source passed.
- `publish_workflow`: 9 passed.
- `rename`: 2 passed.
- Doc tests: exit 0.
- Checked-artifact boundary: exit 0, 15 visible entries and 5 classified modules.
- Checked-artifact Python suite: 69 passed.
- Per-commit lane gate: exit 0.
- `git diff --check` for both candidate ranges: exit 0.
- Both worktrees remained clean.

# S2.1 round-2 review — GO

Core tuple: `affaa69a9cb9c61fd94febf80d9c6382f1648a93` → `14bd5acf01485a6a72922ff7527d9275f0877869`
Remediation comparison: `e10387a7…` → `14bd5acf…`
CLI docs candidate: `5c374fbda71350be17e2112163fd35b0d87743e9`

## Finding disposition

- P2-1: **CURED**
- New P0: none
- New P1: none
- New P2: none
- New P3: none

At [mod.rs:308](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/mod.rs:308), entry construction now uses `message_raw_bytes()`, preserving leading newlines and all raw message bytes.

At [mod.rs:309](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/mod.rs:309), message encoding is extracted directly from `raw_header_bytes()` into `Option<Vec<u8>>`. The previous fallible UTF-8 conversion and `unwrap_or(None)` are gone. Once libgit2 has produced a `Commit`, both raw accessors return byte slices; therefore `None` now means no exact `encoding ` header was found, not that an operational or UTF-8 error was suppressed.

The parser at lines 313–319:

- preserves arbitrary non-UTF-8 header values;
- preserves empty values as `Some(Vec::new())`;
- does not confuse indented multiline-header continuations with an encoding header;
- performs no fallible operation that could be collapsed into absence.

## Regression validity

The new tests at [tests.rs:73](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/tests.rs:73) and [tests.rs:88](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/tests.rs:88) verify:

- a message beginning with a literal newline is returned byte-for-byte;
- an `encoding ISO-8859-\xff` header is returned byte-for-byte.

The helper at [tests.rs:335](/Users/owebeeone/limbo/gwz-log-worktrees/s2.1/gwz-core/src/operation/commit_log/tests.rs:335) writes the complete raw commit body through Git’s object database as `ObjectType::Commit`, installs it as HEAD, and then exercises normal `head().peel_to_commit()` and `find_commit()` cursor paths. A malformed/unparseable object would produce a degradation and fail the entry pattern assertions.

I additionally constructed a native Git object containing both the non-UTF-8 encoding header and leading-newline message. `git cat-file -t` reported `commit`, and `git fsck --strict --no-dangling` exited 0.

## Regression and canonical-row check

The remediation changes only raw-message representation, encoding extraction, and their tests. Selection, HEAD capture, lazy revwalk, native default order, network/mutation absence, conf-gate absence, shallow handling, detached HEAD handling, and degradation isolation are unchanged.

All 12 focused `commit_log` tests passed, including every assigned canonical-row test and both remediation regressions.

## Placement and frozen surface

Placement remains correct behind the existing private `operation::commit_log` seam:

- no `src/operation/mod.rs` change;
- no `src/lib.rs` change;
- no public operation-surface widening;
- no collision with `diff/log_service.rs`;
- only the existing `pub(super) handle_log` crosses the operation seam.

The complete baseline→candidate core diff remains exactly:

- `M src/operation/commit_log/mod.rs`
- `A src/operation/commit_log/tests.rs`

There are no root-manifest, source-loading inventory, protocol, dependency, lockfile, checker, or pin changes.

The replacement candidate `14bd5acf…` is directly parented by `affaa69a…`; the logical `e10387a7…`→`14bd5acf…` diff contains only:

- 17 changed lines in `mod.rs`;
- 52 added/changed test lines.

CLI HEAD is `634394956abfb32807fd134548c0f4598c6ce7ed`, directly parented by `5c374fbd…`. Its only added path is `dev-docs/GwzLog-S2.1-Review.md`; both amended planning documents are byte-unchanged from the reviewed `5c374fbd…` candidate.

## Commands and results

- `cargo test commit_log -- --nocapture`: 12 passed, exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cargo check --all-targets --all-features`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0.
- Checked-artifact boundary: exit 0; 15 visible entries, 5 classified modules.
- Remediation lane gate: exit 0 at `14bd5acf…`.
- `git diff --check` for both remediation and complete candidate ranges: exit 0.
- Native raw-object `git fsck --strict --no-dangling`: exit 0.
- Core and CLI worktrees: clean.
- The 20-minute full suite was not repeated, per the round-2 cap; the focused and all-target compiler gates cover the remediation.

**Overall: GO.**
