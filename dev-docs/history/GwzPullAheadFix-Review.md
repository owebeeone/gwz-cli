# `gwz pull` ahead-only misclassification fix — single-axis peer-blind review

- Date: 2026-09-01 (Australia/Sydney)
- Review mode: single-axis peer-blind review of an UNCOMMITTED working-tree
  change; product read-only, probes transient and reverted
- Repository: `/Users/owebeeone/limbo/gwz-dev/gwz-core`
- Base commit: `845956c59a8727572553fde78967e872209a429f` (`main`,
  "test: update v0.12 fault battery counts")
- Base tree: `f5498078abc5984461e1c8eaa5b1427d11545aea`
- Reviewed candidate: the working-tree diff vs that commit,
  `git diff | shasum` = `1fa8358fc91c5cdead3fc9fd47c16e0f53069d78`
- Diff shape: 4 files, +308 / -85
  - `src/workspace_ops/pull_head_member_preflight.rs` (+199/-85 raw; **+30/-0
    ignoring whitespace**)
  - `src/workspace_ops/pull_head_merge_preflight.rs` (+28)
  - `src/workspace_ops/tests/g25.rs` (+149)
  - `scripts/checks/run_r4bg_aggregate_gates.py` (+17/-2)

## Verdict

# GO-WITH-CONDITIONS

The product change is correct at all three sites and I found no defect in it.
The classification `ahead = !behind && is_ancestor(remote, local)` is sound —
strict, because commit equality is excluded ahead of every one of the three
guards — and the `Reset` carve-out is real, not merely intended: I measured
that an ahead-only root and an ahead-only member under `--sync reset` still
snap HEAD back to the remote, with byte-identical probe output before and
after the fix. `FetchOnly` is unchanged at both sites. The merge-preflight's
new branch is a faithful mirror of the equal-commit arm, and its
current-not-projected manifest/lock choice is the only correct one for a
no-op. The member match body is provably untouched: `git diff -w` reduces the
199-line member hunk to a pure `+30/-0` insertion.

The conditions are about the evidence the change ships with, not the fix.
Two of them are measured, not asserted:

1. The `Rebase` rows of both new regressions **pass against the pre-fix
   sources**. I ran replicas of those rows, with identical assertions, on a
   tree with both source files reverted to `845956c`: both passed, aggregate
   `Ok`. One third of the shipped test matrix therefore discriminates
   nothing, and the pin docstring's claim that the two tests pin the
   behaviour "under every non-destructive sync mode" over-states what they
   prove.
2. The `Reset` carve-out — the single guard whose silent loss would break a
   documented destructive mode rather than merely re-break the reported bug —
   is pinned by no test at all. `pull_head_reset_discards_local_divergence`
   (g01) covers **true divergence**, not ahead-only.

Both remedies fit inside the two test functions the change already adds, and
neither moves the remainder pin. See **Conditions** for the exact form.

| Priority | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 2 |
| P3 | 5 |

## 1. Semantics

### 1.1 The classification is right, and it is strict

`GitBackend::is_ancestor(path, ancestor, descendant)` — argument order
confirmed at `src/git/gitbackend.rs:145`. So `is_ancestor(root, local,
remote)` is *behind* and `is_ancestor(root, remote, local)` is *ahead*, as
used. At all three sites the `local == remote` case has already returned
before the new guard runs:

- `pull_head_member_preflight.rs:245` (root fn) → `:248` behind → `:249` new
  guard.
- `pull_head_member_preflight.rs:746` (member fn) equality branch → `:766`
  behind → `:767` new `ahead`.
- `pull_head_merge_preflight.rs:92` equality → `:118` behind → `:148` new
  branch.

so "ahead" is strictly ahead in every case. The `!behind` conjunct is
logically redundant (two distinct commits cannot each be the other's
ancestor) but is a correct short-circuit that saves one `is_ancestor` call on
the common behind path, and it makes the mutual exclusion explicit at the
call site. Keep it.

These are the only pull classification sites in the crate: a sweep of
`is_ancestor(` outside tests returns exactly the six lines above plus
`merge/status/observe.rs`, `merge/preserve/plan.rs` and
`merge_prepared.rs:448` (post-rebase self-verify) — none of which classify a
pull. Nothing equivalent was missed.

### 1.2 Reset is excluded at both sites — measured, not inferred

Root site: `!matches!(sync, crate::SyncBehavior::Reset)`
(`pull_head_member_preflight.rs:250`). Member site: `!matches!(sync,
FetchOnly | Reset)` (`:768-771`). Both exclude `Reset`, so the guard cannot
swallow it.

Measured (probe, ahead-only fixture, `--sync reset`):

- Root: HEAD moves to the remote commit; aggregate `Ok`.
- Member: HEAD moves to the base commit; member `Ok`, aggregate `Ok`.

I ran the same two probes against the **pre-fix** sources: identical output
(`Ok`, HEAD snapped back). The carve-out demonstrably changes nothing about
`Reset`.

### 1.3 FetchOnly is unchanged

Member: `FetchOnly` is excluded from `ahead`, so it still falls into
`PullHeadAction::FetchOnly` (measured: member `Ok`, aggregate `Ok`, before
and after). Root: `FetchOnly` is *not* excluded from the new guard, but the
arm it bypasses (`:258`) returns `Ok(false)` too, so the outcome is
identical (measured: aggregate `Noop`, before and after). The only cost is
one extra `is_ancestor` call for an ahead-only fetch-only root — see [P3-4].

### 1.4 The merge-preflight branch mirrors the equal arm faithfully

`pull_head_merge_preflight.rs:148-175` against `:92-117`: same
`prepare_merge_upstream_checked` call with the same five arguments, same
`!= GitPreparedMerge::Unchanged` drift refusal with the **same** message
("root up-to-date result changed during pull preparation") and the same
`ErrorCode::MergeRecoveryRequired`, same `RootMergePullAction::UpToDate` with
the same five fields, same `manifest` / `lock` (the values passed in, not
projected). Correct: `read_manifest_at(remote_commit)` is used by the
FastForward arm precisely because the root is about to *move* to the remote
commit; on the ahead path the remote commit is an *ancestor*, so projecting
would substitute an older manifest/lock for the live one. Current is the only
right answer.

The whole path is consistent end to end. `classify_merge`
(`merge_support.rs:4-29`) returns `UpToDate` when the target descends from
the source, so `prepare_merge_upstream_checked` returns `Unchanged`;
`validate_prepared_merge_upstream_in_repo` accepts `(UpToDate, Unchanged)`;
`execute_prepared_merge_upstream_checked` takes the `kind == UpToDate` early
return, calls `verify_merge_result(..., expected_text)` and returns
`clean(expected)` — HEAD is verified *unmoved*, not moved. `apply_root_merge_pull`
then returns `Ok(false)`, i.e. `root_changed = false`. That is exactly the
live-fire `status: Noop` the operator observed.

### 1.5 The member match body is untouched

`git diff -w -- src/workspace_ops/pull_head_member_preflight.rs` reduces the
199-line hunk to `+30/-0`: the `let ahead = …` binding, the `if ahead { … }`
arm, and the `} else {` / `}` wrapper. Every arm of the wrapped `match sync`
is byte-identical modulo leading whitespace. No `-` line survives the
whitespace-insensitive diff. This is the strongest possible answer to the
"diff the match body" question and it is clean.

The `ahead` arm's mode set is also exactly right relative to the equal-commit
arm: the equal arm computes `prepared: Some(…)` for `FfOnly | DriverSelected
| Merge | Rebase` and `None` for `FetchOnly | Reset`; the `ahead` arm is
reachable for precisely `FfOnly | DriverSelected | Merge | Rebase` and always
carries `Some(…)`. Same set, same shape. This also means the ahead arm cannot
hit a new `DirtyMember` surface: all four of those modes are already refused
by `pull_dirty_guard` (`:717`) before the arm is reached.

## 2. Regression hunt

### 2.1 The behaviour matrix, measured on both trees

Fixtures: the change's own `seed_root_ahead` / `seed_ahead_member`. "pre-fix"
= both source files replaced by `git show HEAD:…`, test file unchanged;
"post-fix" = the candidate working tree. Probes were appended to `g25.rs`,
executed, and removed (§5).

| sync | root, pre-fix | root, post-fix | member, pre-fix | member, post-fix |
| --- | --- | --- | --- | --- |
| `FfOnly` | ERROR `DivergedMember` | `Noop` | ERROR `DivergedMember` | `Noop` (member `Noop`) |
| `DriverSelected` | ERROR `DivergedMember` | `Noop` | ERROR `DivergedMember` | `Noop` (member `Noop`) |
| `Merge` | ERROR `MergeRecoveryRequired` | `Noop` | ERROR `MergeRecoveryRequired` | `Noop` (member `Noop`) |
| `Rebase` | `Ok`, HEAD unmoved | `Noop` | `Ok`, HEAD unmoved | `Noop` (member `Noop`) |
| `FetchOnly` | `Noop` | `Noop` | `Ok` (member `Ok`) | `Ok` (member `Ok`) |
| `Reset` | `Ok`, HEAD → remote | `Ok`, HEAD → remote | `Ok`, HEAD → base | `Ok`, HEAD → base |
| dry-run `FfOnly` | — | — | ERROR `DivergedMember` | `Noop`, planned `Noop` |
| dry-run `Merge` | — | — | ERROR `MergeRecoveryRequired` | `Noop`, planned `Noop` |

`lock_match` is `Matches` on every post-fix member row. HEAD is unmoved on
every post-fix non-`Reset` row.

Three things fall out of this table that the change's own narrative does not
state:

- `DriverSelected` was broken pre-fix and is fixed. It shares the `FfOnly`
  arm, so this follows, but no test drives it — [P3-2].
- The **dry-run** path was broken pre-fix and is fixed. No test drives it —
  [P3-3]. (The ahead arm's `prepare_pull_unchanged` is non-mutating: for
  `UpToDate`, `prepare_merge_upstream_mode_checked` returns before any tree
  write. Measured: HEAD unmoved after both dry runs.)
- `Rebase` did **not** error pre-fix; it silently no-opped and reported `Ok`.
  That is the basis of [P2-1] and of [P3-5].

### 2.2 `root_remote_changes_are_auto_repairable` — nothing is lost

The mandate's question resolves to a proof, not a judgement. For an ahead-only
root, `merge_base(local, remote) == remote`, so the function's
`changed_paths_between(root, base, remote_commit)` diffs the remote tree
against itself (`comparison.rs:20-38`, `diff_tree_to_tree` of one commit's
tree with itself) and is necessarily **empty**; the function's very next
expression is `!remote_paths.is_empty() && …`, so it returns `false`
unconditionally on that input. The pre-fix ahead-only `FfOnly` path could
therefore only ever reach the `else` branch — `DivergedMember`. That is
precisely what I measured. There is no ahead-only lock-only-remote-change
case to lose, because "ahead-only" means the remote holds nothing the local
lacks; the premise is self-contradictory. Auto-repair remains reachable for
genuine divergence, which the guard does not touch.

### 2.3 Barrier, apply, and response paths

- `validate_pull_barrier` → `validate_member_plan` already has an
  `UpToDate { prepared: Some(_) }` arm (`pull_head_barrier.rs:68-74`) that
  runs `validate_source_ref` + `validate_prepared`. With `local != remote`
  and `prepared = Unchanged`, `classify_merge` still says `UpToDate`, so the
  `(UpToDate, Unchanged)` pair validates. No new barrier surface, and the
  late-drift protection still applies to the ahead case (an ahead-only member
  whose remote ref moves between preflight and barrier is still refused with
  `MergeDrift`).
- Root plan `UpToDate` was already a `validate_root_merge_pull` arm; nothing
  new there either.
- `apply_pull_action`'s `UpToDate { prepared: Some(_) }` arm re-executes the
  checked merge, which for `UpToDate` verifies HEAD is still at `expected`
  and returns clean. The ahead case therefore keeps the same
  execution-time recheck the equal-commit case has.
- `PullHeadAction::is_noop()` already includes `UpToDate`, so both the dry-run
  aggregate (`pull_aggregate_status`) and the live aggregate
  (`pull_response_aggregate` via `MemberStatus::Noop`) report `Noop` without
  any vocabulary change. Measured.
- `--target` selection: the root only enters either path when
  `pull_root_selected` (`:66-68`, `CommandDefaultTargets::All` +
  `RootSelectionPolicy::Allow`). A member-only `--target` leaves an ahead-only
  root entirely untouched, before and after. No interaction.

### 2.4 Existing refusals still refuse

- g07 `pull_head_divergence_blocks_all_selected_members_before_branch_mutation`
  (true divergence → `DivergedMember`, no branch mutation): passes. The guard
  cannot reach it — for true divergence both `is_ancestor` calls are false.
- g21's `DivergedMember` assertion is `handle_branch`, not pull. Unaffected.
- g01 `pull_head_reset_discards_local_divergence`: passes.
- Behind-only fast-forward (root and member) and the g25 barrier/drift suite:
  pass.

All of the above are inside the 1097-test remainder partition, re-executed
green on this tree (§4).

### 2.5 One behaviour change beyond the reported bug — [P3-5]

Pre-fix, an ahead-only root under `--sync rebase` did *not* error: it called
`rebase_onto`, hit `analysis.is_up_to_date()`, left HEAD alone, then ran
`rewrite_root_lock_from_live_members` and returned `Ok(true)`, so the
aggregate was `Ok`. Post-fix it returns `Ok(false)` before that, so the
aggregate is `Noop` and the root lock is not rebuilt from live members.

I judge the new behaviour correct — reporting a change for a no-op was itself
wrong, and the main flow writes the lock at
`pull_head_member_preflight.rs:185` regardless. The one real difference is
that `rewrite_root_lock_from_live_members` rebuilds entries for **all active**
members while the main flow updates only **selected** ones, so a narrow
`--target` plus an ahead-only root under `--sync rebase` no longer refreshes
unselected members' lock rows as a side effect. That is more faithful to the
selection, not less — but it is a silent behavioural delta outside the stated
bug and belongs in the landing note.

## 3. Tests

### 3.1 What the two regressions do prove

Re-executed on this tree: **g25 7 passed, 0 failed** (0.73s), including both
new tests.

Negative probe, independently re-executed: with both source files reverted to
`845956c` and the test file untouched, the two new tests **fail** with exactly
the operator-reported classifications —

```
ahead-only root refused under FfOnly: workspace root has diverged from remote; rerun with --sync merge, rebase, or reset
ahead-only member refused under FfOnly: member 'mem_app' has diverged from remote
```

— and the other five g25 tests stay green (11 passed / 2 failed). Each source
file is also *individually* pinned: reverting only `pull_head_merge_preflight.rs`
still fails the root test's `Merge` row; reverting only
`pull_head_member_preflight.rs` fails the root test's `FfOnly` row and the
member test. The claimed evidence holds.

### 3.2 What they do not prove — [P2-1], measured

The assertion in both tests is

```rust
assert!(matches!(
    response.response.meta.aggregate_status,
    crate::AggregateStatus::Ok | crate::AggregateStatus::Noop
), …);
```

plus "HEAD did not move". For the `Rebase` row, pre-fix behaviour is aggregate
`Ok` with HEAD unmoved (§2.1) — which satisfies both assertions. I confirmed
this directly rather than reasoning about it: replicas of each test's `Rebase`
row, carrying the *identical* assertions, run against the pre-fix sources:

```
test …::zprobe2_rebase_leg_only_member … PROBE2 member rebase aggregate Ok … ok
test …::zprobe2_rebase_leg_only_root   … PROBE2 root rebase aggregate Ok   … ok
```

Both pass. The `Rebase` rows are decorative: they would survive a full revert
of the member-site `Rebase` handling. And because the disjunction admits `Ok`,
the tests would also survive a future regression that re-classified an
ahead-only member as a planned `Rebase`/`Merge` action — reporting a change
for a no-op is exactly the failure mode the fix exists to remove, and the
tests do not catch it.

The measured exact aggregate is `Noop` for `FfOnly`, `Merge`, `Rebase` **and**
`DriverSelected`, at both root and member. The disjunction is therefore not
protecting against any real variation; it is pure slack.

### 3.3 Reset is unpinned — [P2-2]

No test in the tree drives `--sync reset` against an ahead-only target. g01's
`pull_head_reset_discards_local_divergence` seeds a genuinely diverged member
(local `B` and remote `C` both on base `A`). If a later refactor folded
`Reset` into the ahead branch — the obvious "simplify these two nearly
identical guards" move, since the root guard's exclusion list and the member
guard's differ — `gwz pull --sync reset` on an ahead-only target would
silently stop discarding local commits, and the whole suite would stay green.
That is a user-visible loss of a documented mode with no tripwire.

### 3.4 Smaller gaps

- `source.clone()` — [P3-1]. It compiles and is correct: the struct
  expression evaluates `source: source.clone()` first (a clone, so nothing is
  moved), then borrows `source` for `prepare_pull_unchanged`. The clone is
  nevertheless avoidable by writing `prepared` first and `source` last, which
  is how the equal-commit arm two screens up avoids it. Cosmetic.
- The member test asserts HEAD and the aggregate but not the per-member
  `status` / `lock_match`; the root test does not assert the workspace lock is
  unchanged. Both are measured-correct (`Noop`, `Matches`); asserting them is
  optional.
- `--target` defaults are exercised only implicitly (default `All`). Fine —
  the root/member split is already covered by having one root-only and one
  member-only fixture.

## 4. Gate re-execution on this tree (all direct exits)

| Gate | Command | Result |
| --- | --- | --- |
| g25 suite | `cargo test --lib g25` | 7 passed, 0 failed |
| lib remainder | `cargo test --lib -p gwz-core -- --skip checked_artifact:: --skip workspace_ops::merge::v1_lifecycle::` | **1097 passed, 0 failed, 1 ignored**, 704 filtered |
| checked-artifact census | `cargo test --lib -p gwz-core checked_artifact::` | **447 passed** |
| v1 lifecycle | `cargo test --lib -p gwz-core workspace_ops::merge::v1_lifecycle:: -- --skip root_fault_matrix` | **256 passed** |
| format | `cargo fmt --check` | exit 0 |
| lint | `cargo clippy --all-targets -p gwz-core -- -D warnings` | exit 0 |
| whitespace | `git diff --check` | clean |
| driver syntax | `python3 -m py_compile scripts/checks/run_r4bg_aggregate_gates.py` | ok |

Partition arithmetic cross-check: the remainder run reports 704 filtered out,
and `447 + 256 + 1` (`root_fault_matrix`, release profile) `= 704`. The four
partitions are disjoint and complete over the 1802-entry lib binary, which is
the driver's own stated invariant.

## 5. Pin move

Convention followed. The new block in `_fault_count`'s docstring is dated
(2026-09-01), states which partition moves and by how much ("the lib remainder
and only it, by two"), names both new tests, marks darwin **MEASURED** and
linux **DERIVED / FIRST-DISPATCH-EXPECTED** with the "a measured number wins"
clause, and explains why the derivation is cfg-independent. It matches the
shape of the preceding "gwz log settlement (2026-09-01)" block, and the
`1095 → 1097` / `1096 → 1098` edit at the `BATTERIES` row is the only
functional change to the file.

Same-commit duty discharged, and independently verified: darwin 1097 + 1
ignored is what this tree actually produces, and the block's side claim that
`checked_artifact::` and `v1_lifecycle::` are unmoved at 447 / 256 is true —
I measured all three rather than trusting the arithmetic.

One wording defect, [P3-6]: the block says the two tests pin the behaviour
"under every non-destructive sync mode". They drive three of six modes; the
`Rebase` row of each is non-discriminating (§3.2); `DriverSelected` is not
driven; and `FetchOnly`, which is non-destructive, is deliberately *not*
treated as up-to-date (it reports `Ok`/fetched). Discharging [P2-1] and
[P2-2] makes most of that sentence true; the `FetchOnly` clause should be
narrowed either way.

## 6. Findings

**[P2-1] The `Rebase` rows of both new regressions are non-discriminating,
and the aggregate assertion is looser than the measured truth.**
Evidence: replicas of those rows with identical assertions pass against
sources reverted to `845956c` (aggregate `Ok`, HEAD unmoved). The exact
post-fix aggregate is `Noop` for `FfOnly` / `Merge` / `Rebase` /
`DriverSelected` at both sites, so `Ok | Noop` admits a regression class the
fix exists to prevent (a no-op reported as a change). Files:
`src/workspace_ops/tests/g25.rs`, both `an_ahead_only_*` tests.

**[P2-2] The `Reset` carve-out is pinned by no test.**
Evidence: no ahead-only `--sync reset` case exists in the tree; g01's
`pull_head_reset_discards_local_divergence` covers true divergence only. I
measured that `Reset` still snaps HEAD back at both sites, identically before
and after the fix — so the code is right today and nothing keeps it right.
This is the one guard whose silent removal breaks a mode rather than
re-breaking the bug.

**[P3-1]** `source.clone()` in the member `ahead` arm is avoidable by ordering
`prepared` before `source` in the struct expression, matching the
equal-commit arm's style. `pull_head_member_preflight.rs:778`.

**[P3-2]** `DriverSelected` was equally broken pre-fix (measured
`DivergedMember`) and is fixed, but no test drives it. It shares the `FfOnly`
match arm, so risk is low; one extra loop row closes it for free.

**[P3-3]** The dry-run path was equally broken pre-fix (measured
`DivergedMember` / `MergeRecoveryRequired`) and is fixed, but no test drives
it. `--dry-run` is the flag an operator hitting this bug would reach for
first.

**[P3-4]** Guard asymmetry: the member guard excludes `FetchOnly`, the root
guard does not (it relies on the bypassed arm returning `Ok(false)` too).
Outcome is identical — measured — but the two guards now read differently
for no reason, and the divergence is exactly the kind of thing a later
"unify these" pass gets wrong. Consider excluding `FetchOnly` at the root
site too, purely for symmetry.

**[P3-5]** Behaviour change outside the reported bug: ahead-only root under
`--sync rebase` moves from `Ok` + `rewrite_root_lock_from_live_members` to
`Noop` with no root-lock rebuild (§2.5). Correct, but it should be named in
the landing note rather than discovered later.

**[P3-6]** The pin docstring's "under every non-destructive sync mode"
over-states what the two named tests prove (§5).

## 7. Conditions

Both are confined to the two test functions the change already adds, and
**neither moves the remainder pin** — adding rows inside the existing loops
leaves the test-function count, and therefore 1097 / 1098, unchanged.

- **C1** — replace the `matches!(… Ok | Noop)` assertion in both
  `an_ahead_only_*` tests with `assert_eq!(…, crate::AggregateStatus::Noop)`.
  Measured to hold for `FfOnly`, `Merge`, `Rebase` and `DriverSelected` at
  both sites. Adding `DriverSelected` to both loops at the same time
  discharges [P3-2] for free.
- **C2** — add an ahead-only `--sync reset` case to each test asserting that
  HEAD **does** move to the remote/base commit, so the carve-out is pinned in
  the direction that matters. Keep it inside the existing test functions.

If C2 is instead landed as a new `#[test] fn`, the remainder pin must move
again in the same commit — darwin re-measured to 1098, linux derived to 1099 —
with its own dated block. Adding rows avoids that entirely, which is why I
name the form.

Discharging C1 and C2 also makes [P3-6]'s sentence true apart from the
`FetchOnly` clause, which should be narrowed in the same edit.

The source diff itself needs no change.

## 8. Probe hygiene

All probes were appended to `src/workspace_ops/tests/g25.rs` and removed; the
two source files were transiently replaced with `git show HEAD:…` for the
negative and discrimination probes and restored from byte-exact backups. No
commit, stash, tag, branch or index operation was performed at any point.

Post-review state, verified: `git status --porcelain --untracked-files=all`
lists exactly the four candidate files and nothing else; the restored files'
SHA-1s match their pre-probe backups
(`f3ffa24e…` member preflight, `2d04635c…` merge preflight, `c85d81ed…`
g25.rs); `git diff --stat` is again `17 / 199 / 28 / 149`, +308 / -85; and
`git diff | shasum` is again `1fa8358fc91c5cdead3fc9fd47c16e0f53069d78` —
identical to the candidate hash recorded in this document's header. A
`__pycache__` directory created by the driver syntax check was removed.
