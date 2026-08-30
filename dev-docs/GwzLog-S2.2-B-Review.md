## S2.2-B fresh round-1 independent review

**Verdict: NO-GO — one P2 test-matrix remediation required**

- Normative authority: gwz-cli `096a5eea34566da5b1bdc83be0aaa7cb04834172`
- Core main/baseline: `dd31d54439e9244cba876d159383a5fc5e9584b2`
- Reviewed S2.2 base: `7e2cd3caa57d18cffdf00bf85c046ed3aa96e905`
- S2.2-B package HEAD: `40ae6245f25f61569fea968d79a7d876e855b2bf`
- Worktree: `/Users/owebeeone/limbo/gwz-log-worktrees/s2.2-b/gwz-core`
- Scope: L-RNG-6 open-boundary delta plus base integrity only
- Finding count: **0 P0 / 0 P1 / 1 P2 / 0 P3**

The production correction is semantically sound: all adjacent-, leading-, and trailing-dot open-boundary forms return the exact typed teaching refusal; shorter stored IDs are not selected; exact standalone matches still win; and safe/closed forms are not overmatched.

Base integrity, scope, and the hard LOC cap pass. The sole blocker is that the checked-in mandatory open-boundary matrices omit `.leading` at all three required layers.

### S2.2-B-F1 — [P2] `.leading` open-boundary regressions are absent from all three mandated layers

The §6 S2.2-B charter makes the delta scope “exactly the terminal review’s recorded remedy.” Terminal F7 requires adjacent-, leading-, and trailing-dot legacy IDs on both sides of open `..` and `...`, with exact `InvalidRequest` and complete teaching-message assertions.

The new case arrays contain only `trailing.` and `adjacent..dots`:

- `src/diff/tests/t_plan.rs:153-162`
- `src/diff/tests/t_handle.rs:254-263`
- `src/operation/commit_log/tests.rs:814-823`

Existing `.leading` coverage is standalone or closed-range coverage; it does not exercise these four open forms:

```text
+.leading..
..+.leading
+.leading...
...+.leading
```

Consequently, four required cells are absent at each of parser, end-to-end diff, and end-to-end log: twelve missing acceptance cells total.

An exact-source parser probe confirmed that the current generic implementation handles all four forms correctly and emits the complete required message. This is therefore an acceptance/regression-evidence defect, not an observed production-behavior defect. It remains P2 because this exact matrix was mandated after the prior F7 false-pass.

Required remediation: add the four `.leading` cases, with exact code and full-message assertions, to each of the three existing open-boundary case tables. No production change is indicated by this review.

## Base, tree, and patch integrity

Topology is exactly linear:

```text
dd31d54439e9244cba876d159383a5fc5e9584b2
  → 7e2cd3caa57d18cffdf00bf85c046ed3aa96e905
  → 40ae6245f25f61569fea968d79a7d876e855b2bf
```

`HEAD^` is the exact reviewed base object, not merely a rebased equivalent. `HEAD^^` is the exact named baseline.

Trees:

```text
dd31d544 tree: 9c1e7b0c48e1ff1081ee0d5d58bf8e69a61c3473
7e2cd3c tree:  64c75ebb8fd50492e8cf27f3bc2881b14d7d72fd
40ae6245 tree: bcb213be7d75fed7cb6ff69111363487744e8dc8
```

Patch integrity:

```text
dd31d544..7e2cd3c:
  binary-diff SHA-256: fdd655b54dda374a169716c206f06da2aef4974f3fec1accbbd549d1f2d2bc13
  stable patch-id:      67784721da17f962200670d067a041daa907cce4

7e2cd3c..40ae6245:
  binary-diff SHA-256: 94573f1192683e10c0c63050dace1cfd78cf2d3701dcda63f6bffeaf78ac323e
  stable patch-id:      a8bc1d6a44d4d504d92a3ae2c67e5d7a41d2c97f
```

`git diff --exit-code 7e2cd3c 40ae6245^` is empty, and range-diff reports the reviewed base as exact equality. The repository is non-shallow, with no replacement refs or grafts. Worktree and index remained clean.

## L-RNG-6 delta matrix

| Cell | Result | Evidence |
|---|---|---|
| Exact whole-token legacy access | **PASS** | Adjacent, leading, and trailing standalone IDs pass parser, diff, and log tests before range interpretation. |
| Open `..`, legacy left | **Behavior PASS / coverage FAIL** | Exact-source probe passes all three ID shapes; checked-in parser/diff/log tables omit `.leading`. |
| Open `..`, legacy right | **Behavior PASS / coverage FAIL** | Same omission. |
| Open `...`, legacy left | **Behavior PASS / coverage FAIL** | Same omission. |
| Open `...`, legacy right | **Behavior PASS / coverage FAIL** | Same omission. |
| Shorter stored-ID collision | **PASS** | `trailing` and `adjacent` coexist with their legacy IDs in parser, diff, and log fixtures. |
| Exact `+adjacent..dots..` | **PASS** | Present at all three layers and returns the required refusal. |
| Typed code and full teaching message | **PASS for present cells** | New tests assert exact `InvalidRequest` and exact complete message; required `.leading` open assertions are absent. |
| Closed ambiguous ranges | **PASS** | Existing adjacent/leading/trailing two-dot and three-dot tests remain green. |
| Safe internal-dot ranges | **PASS** | Existing diff/log regressions remain green. |
| No standalone/closed overmatch | **PASS** | Exact matching still precedes rejection; safe ranges, unrelated prefixes/suffixes, and an exact longer stored ID passed the exact-source probe. |
| Shared diff/log parser seam | **PASS** | One production predicate serves parser, diff, and log; no client fork was introduced. |
| **L-RNG-6 addendum overall** | **FAIL** | Production semantics pass, but the mandatory `.leading` open-boundary regression matrix is incomplete. |

## LOC and scope

The plan’s stated LOC basis includes tests. Conservative add-plus-delete churn is therefore the strictest useful count:

| Classification | Add | Delete | Churn |
|---|---:|---:|---:|
| Production: `src/diff/operands.rs` | 1 | 2 | 3 |
| Test-only files | 112 | 0 | 112 |
| **Total** | **113** | **2** | **115** |

The hard cap passes at **115/150**, leaving 35 lines of headroom. Production-only churn is 3 lines.

The delta changes exactly four mode-`100644` files and contains no rename or mode change. Its only production edit removes the two strict-length conjuncts from the legacy-boundary predicate. Nothing outside the open-boundary residue rides: no Cargo/dependency, protocol/generated, schema, checked-artifact, lifecycle, inventory, pin, crate-root, module-root, visibility, handler, renderer, or output change exists.

The whole baseline-to-package diff remains the expected S2.2 base plus this residue:

```text
14 files changed, 2818 insertions(+), 198 deletions(-)
```

No cured F1-F5/F8 surface was re-reviewed.

## Commands and direct exits

### Identity and integrity

- `git rev-parse HEAD HEAD^ HEAD^^ main` — exit 0; exact refs above
- `git merge-base dd31d544... 40ae6245...` — exit 0; exact `dd31d544...`
- `git diff --exit-code 7e2cd3c... 40ae6245^` — exit 0; empty
- `git range-diff dd31d544..7e2cd3c dd31d544..40ae6245^` — exit 0; exact equality
- Tree, binary-diff SHA-256, and stable patch-ID commands — exit 0; values above
- `git diff --name-status/--numstat 7e2cd3c..40ae6245` — exit 0; four files, `+113/-2`
- `git diff --check 7e2cd3c..40ae6245` — exit 0
- Frozen-surface `git diff --quiet` checks — exit 0
- Final `git status --porcelain=v2`, worktree diff, and index diff — exit 0; clean

### Focused semantic evidence

- `TAUT_PYTHON=... cargo test --locked --lib l_rng_6_ -- --nocapture` — exit 0; 13 passed
- `cargo test --locked --lib open_legacy -- --nocapture` — exit 0; parser, diff, and log tests all passed
- Corrected exact-source Rust probe importing current `src/diff/operands.rs` — compile exit 0; run exit 0:
  - 12/12 adjacent/leading/trailing open-boundary forms taught exactly
  - three standalone legacy forms matched exactly
  - safe ranges and unrelated prefix/suffix cases did not overmatch
  - an exact longer stored legacy ID still won before range rejection

### Proportional formal and boundary gates

- `cargo fmt --all -- --check` — exit 0
- `cargo check --locked --all-targets` — exit 0
- `TAUT_PYTHON=... CLIPPY_CONF_DIR="$PWD" cargo clippy --locked --all-targets --all-features -- -D warnings` — exit 0
- `bash scripts/checks/check_lane_commits.sh 7e2cd3c... 40ae6245...` — exit 0
- `python3 scripts/checks/check_checked_artifact_boundaries.py --source src` — exit 0; 15 visible entries, 5 classified modules
- `python3 -m unittest scripts/checks/test_release_boundary.py -v` — exit 0; 6 passed
- `cargo metadata --format-version 1 --locked --no-deps` — exit 0

The long full suite was deliberately not run.

## Final decision

**NO-GO — fresh round 1.**

Do not land or push `40ae6245f25f61569fea968d79a7d876e855b2bf`.

Route the package through the fresh-round remediation path for S2.2-B-F1 only: add the four `.leading` open-boundary cases to each of the parser, end-to-end diff, and end-to-end log matrices while retaining the hard 150-LOC cap and allowing nothing else to ride.

## S2.2-B fresh round-2 final independent review

**Verdict: GO — final**

- Normative authority: gwz-cli `096a5eea34566da5b1bdc83be0aaa7cb04834172`
- Round-one report commit: `54155e96f1f17b8a8eb7d17cec91ba27b0af1092`
- Core main/baseline: `dd31d54439e9244cba876d159383a5fc5e9584b2`
- Reviewed S2.2 base: `7e2cd3caa57d18cffdf00bf85c046ed3aa96e905`
- Final S2.2-B package: `2214eace46b72915f76ab28e03e16716ce9d1a60`
- Core worktree: `/Users/owebeeone/limbo/gwz-log-worktrees/s2.2-b/gwz-core`
- Scope: round-one S2.2-B-F1 remediation plus base-integrity, cap, and scope regression only
- Finding count: **0 P0 / 0 P1 / 0 P2 / 0 P3**

Round-one S2.2-B-F1 is cured. Parser, end-to-end diff, and end-to-end log now each carry the complete 12-case matrix: three legacy ID shapes across both open sides of `..` and `...`, for 36 checked cells total. Every layer asserts the exact `InvalidRequest` code and complete teaching message.

The suite is mutation-tight against restoration of the former strict-length predicate. Production is byte-identical to round one, total conservative churn remains below the hard cap, the reviewed base is exact, and nothing unrelated rides.

### S2.2-B-F1 — CURED [former P2]

Each required `.leading` form is now present at every mandated layer:

```text
+.leading..
..+.leading
+.leading...
...+.leading
```

Locations:

- Parser: `src/diff/tests/t_plan.rs:160-163`
- End-to-end diff: `src/diff/tests/t_handle.rs:255-258`
- End-to-end log: `src/operation/commit_log/tests.rs:815-818`

Each table’s shared assertion checks:

```text
ErrorCode::InvalidRequest
```

and the complete message:

```text
snapshot id '<id>' is ambiguous as a revision-range endpoint; use '+<id>' standalone or create a range-safe snapshot id without adjacent, leading, or trailing dots
```

The assertions are at:

- Parser: `src/diff/tests/t_plan.rs:174-183`
- Diff: `src/diff/tests/t_handle.rs:269-285`
- Log: `src/operation/commit_log/tests.rs:829-840`

No production amendment was needed for the cure.

## Complete open-boundary matrix

| Legacy ID | Open `..` left | Open `..` right | Open `...` left | Open `...` right |
|---|---|---|---|---|
| `.leading` | `+.leading..` | `..+.leading` | `+.leading...` | `...+.leading` |
| `trailing.` | `+trailing...` | `..+trailing.` | `+trailing....` | `...+trailing.` |
| `adjacent..dots` | `+adjacent..dots..` | `..+adjacent..dots` | `+adjacent..dots...` | `...+adjacent..dots` |

| Layer | `.leading` | `trailing.` | `adjacent..dots` | Total | Result |
|---|---:|---:|---:|---:|---|
| Parser | 4/4 | 4/4 | 4/4 | 12/12 | **PASS** |
| End-to-end diff | 4/4 | 4/4 | 4/4 | 12/12 | **PASS** |
| End-to-end log | 4/4 | 4/4 | 4/4 | 12/12 | **PASS** |
| **Total** | **12/12** | **12/12** | **12/12** | **36/36** | **PASS** |

The specifically required `+adjacent..dots..` appears at all three layers.

Shorter stored IDs coexist with the ambiguous IDs where relevant:

```text
trailing     / trailing.
adjacent     / adjacent..dots
```

Diff and log persist both the shorter and legacy artifacts before executing their matrices; parser supplies both IDs to the exact shared parser.

### Mutation-tightness

Restoring the former predicate:

```rust
(token.starts_with(&left) && token.len() > left.len())
    || (token.ends_with(&right) && token.len() > right.len())
```

misses all six open two-dot cells per layer:

- three legacy shapes;
- left and right open boundaries.

It still catches the six open three-dot cells through the shorter `..` prefix/suffix overlap. Therefore the old predicate would fail 6 assertions at each layer, 18 of the 36 cells overall. In normal test execution each table stops at its first missed `.leading` two-dot case, so parser, diff, and log each independently kill the mutant.

An exact-expression probe reported:

```text
current cells:                  12/12 matched
old predicate misses/layer:      6
old-mutant failing cells total: 18
```

This is the correct mutation-tightness claim; it does not overstate the three-dot cells.

### Compatibility regression

The focused L-RNG-6 suite remains green for:

- exact standalone adjacent-, leading-, and trailing-dot legacy IDs;
- closed ambiguous two-dot and three-dot ranges;
- safe internal-dot snapshot ranges;
- creation-only refusal and permanent schema-v0 read compatibility;
- shorter-ID collision refusal;
- exact whole-token matching before range interpretation.

Exact standalone access still returns at `src/diff/operands.rs:80-82`, before the legacy-range rejection at lines 83-84. Safe IDs never enter the legacy predicate.

## Base, tree, and patch integrity

Topology is exactly:

```text
dd31d54439e9244cba876d159383a5fc5e9584b2
  → 7e2cd3caa57d18cffdf00bf85c046ed3aa96e905
  → 2214eace46b72915f76ab28e03e16716ce9d1a60
```

`HEAD^` is the exact reviewed base object and `HEAD^^` is the exact baseline. Main remains at the named baseline.

Trees:

```text
dd31d544 tree: 9c1e7b0c48e1ff1081ee0d5d58bf8e69a61c3473
7e2cd3c tree:  64c75ebb8fd50492e8cf27f3bc2881b14d7d72fd
2214eace tree: f6ba0d2a30fdb8508e6f91fd4b1affa847617b39
```

Patch integrity:

```text
dd31d544..7e2cd3c
binary-diff SHA-256: fdd655b54dda374a169716c206f06da2aef4974f3fec1accbbd549d1f2d2bc13
stable patch-id:      67784721da17f962200670d067a041daa907cce4

7e2cd3c..2214eace
binary-diff SHA-256: 07b97822a8ba97c7ea47628f8ee0d062640414701730fb43e5ef8675920f0e9f
stable patch-id:      59b04124245edea077123c66d36e9634dc1d79bf
```

`git diff --exit-code 7e2cd3c HEAD^` is empty, and range-diff reports exact equality for the reviewed base. The repository is non-shallow, with no replacement refs or grafts.

## LOC and scope

Base-to-package delta:

```text
src/diff/operands.rs               +1 / -2
src/diff/tests/t_handle.rs        +48 / -0
src/diff/tests/t_plan.rs          +39 / -0
src/operation/commit_log/tests.rs +43 / -0
```

| Classification | Add | Delete | Net | Conservative churn |
|---|---:|---:|---:|---:|
| Production | 1 | 2 | -1 | 3 |
| Tests | 130 | 0 | +130 | 130 |
| **Total** | **131** | **2** | **+129** | **133** |

The plan’s handwritten-LOC basis includes tests. The strictest add-plus-delete count is therefore **133/150**, leaving **17 LOC** of headroom.

The production blob is byte-identical to round one:

```text
src/diff/operands.rs
f95222abac2f19e088f57c271a5b796b20d6d70e
```

`40ae6245..2214eace` changes only the three test files. Its textual `+23/-5` reflects the `.leading` fixture/case additions plus formatting of the parser ID array; net growth is 18 lines.

All four delta files remain mode `100644`. There is no rename, copy, dependency, Cargo, protocol/generated, schema, checked-artifact, lifecycle, inventory, pin, crate/module-root, visibility, handler, renderer, or output change.

The whole baseline-to-package diff remains the reviewed S2.2 base plus this residue:

```text
14 files changed, 2836 insertions(+), 198 deletions(-)
```

No cured base surface was reopened.

## Commands and direct exits

### Identity and diff integrity

- `git rev-parse HEAD HEAD^ HEAD^^ main` — exit 0; exact chain above
- `git merge-base dd31d544... HEAD` — exit 0; exact baseline
- `git merge-base 7e2cd3c... HEAD` — exit 0; exact reviewed base
- `git diff --exit-code 7e2cd3c... HEAD^` — exit 0; empty
- `git range-diff dd31d544..7e2cd3c dd31d544..HEAD^` — exit 0; exact equality
- Tree, binary-diff SHA-256, stable patch-ID, and blob checks — exit 0; values above
- `git diff --name-status/--numstat 7e2cd3c..HEAD` — exit 0; four expected files, `+131/-2`
- `git diff --exit-code 40ae6245..HEAD -- src/diff/operands.rs` — exit 0
- `git diff --check 7e2cd3c..HEAD` — exit 0
- Frozen-surface `git diff --quiet` checks — exit 0
- Final worktree and index diffs — exit 0; clean

### Authoritative focused tests

All cited package test results were rerun personally from exact HEAD using the fresh isolated target:

```text
/tmp/gwz-s2-2-b-r2-clean.fZK6Ph
```

- `CARGO_TARGET_DIR=<isolated> TAUT_PYTHON=... cargo test --locked --lib open_legacy -- --nocapture` — exit 0; 3 passed
- `CARGO_TARGET_DIR=<isolated> TAUT_PYTHON=... cargo test --locked --lib l_rng_6_ -- --nocapture` — exit 0; 13 passed
- Exact old/current predicate probe — compile exit 0; run exit 0; 12 current matches, 6 old misses per layer, 18 mutant-sensitive cells total

### Proportional formal and boundary gates

- `cargo fmt --all -- --check` — exit 0
- Isolated-target `cargo check --locked --all-targets` — exit 0
- Isolated-target `cargo clippy --locked --all-targets --all-features -- -D warnings` — exit 0
- `bash scripts/checks/check_lane_commits.sh 7e2cd3c... 2214eace...` — exit 0
- `python3 scripts/checks/check_checked_artifact_boundaries.py --source src` — exit 0; 15 visible entries, 5 classified modules
- `python3 -m unittest scripts/checks/test_release_boundary.py -v` — exit 0; 6 passed
- `cargo metadata --format-version 1 --locked --no-deps` — exit 0

The long full suite was deliberately not run.

### Excluded non-evidence

A delegated isolated-source mutation experiment mistakenly reused the shared Cargo target directory. That produced a concurrent clean-source exit 101 from stale mutant build artifacts. It is explicitly excluded from package evidence and from the verdict.

Core source, index, and worktree bytes remained untouched. All authoritative focused tests and compiler gates above were subsequently rerun from exact `2214eace` with a new isolated target directory and passed.

## Final decision

**GO — final.**

S2.2-B-F1 is cured, the complete 36-cell matrix is present and mutation-tight, base integrity passes exactly, the hard cap passes, and nothing unrelated rides.

`2214eace46b72915f76ab28e03e16716ce9d1a60` is approved for landing as the complete S2.2-B package under the documented ritual.
