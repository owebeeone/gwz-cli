# `gwz log` S4.1 integrated-gate evidence

Date: 2026-08-31 (Australia/Sydney)

Scope: S4.1 traceability/settle candidate; no self-review verdict

## Exact inputs

| Repository | Remote | Exact gate input |
| --- | --- | --- |
| gwz-cli | `git@github.com:owebeeone/gwz-cli.git` | `098e1b7f3219fa4f1e23540c40e069caacf512c4` plus this docs-only settle candidate |
| gwz-core | `git@github.com:owebeeone/gwz-core.git` | product gate input `bdb398c3fa8581531eb1a38674ef89f56fc192e2`; final S1.2 docs-only candidate `eb3a37c3d657b28c9fb3c85054056aa9192ee353` |
| gwz-py | `git@github.com:owebeeone/gwz-py.git` | base `ec1b01f1801c930da930acefbf8d48f7e612ce96`; test-only candidate `4c5ad072d3c191a6e1b6b34c62037f9e715d5b2d` |

Host: macOS 26.6.2 (25G83), Darwin 25.6.0, arm64. Tool identities:
Rust `rustc 1.95.0 (59807616e 2026-04-14)`, Cargo
`1.95.0 (f2d3ce0bd 2026-03-21)`, Git `2.52.0`, CPython `3.12.12`,
pytest `9.1.1`, and canonical `taut-proto 0.9.1`. Core protocol tests and
generation used
`/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python`;
Python acceptance used `/Users/owebeeone/limbo/gwz-dev/.venv/bin/python`.

The Rust oracle for Python and S3.4 was the binary built from the exact CLI
worktree. The Python ABI3 extension was rebuilt from the exact gwz-py
candidate and sibling exact core with `maturin develop --release`; the emitted
module was the current worktree's `src/gwz/_gwz_core.abi3.so`. Maturin selected
the host ABI3 build interpreter at `/Users/owebeeone/.venv/bin/python`
(CPython 3.10.17) and pinned that build environment from `taut-proto 0.8.1` to
the repository-required 0.9.1; the completed module was then loaded and tested
by the separately identified CPython 3.12.12 acceptance interpreter.

## TDD closure of the retained S3.4 P3

Before the product-independent table change:

```text
/Users/owebeeone/limbo/gwz-dev/.venv/bin/python -m pytest -q \
  src/tests/test_log_real_workspace.py::test_native_git_magic_matrix_carries_both_short_exclusion_aliases
exit 1 — assertion RED: (".", ":^side-only.txt") absent
```

After adding that tuple to the same real native-parity matrix as long exclude,
`:!`, and `:(top)`:

```text
same focused command
exit 0 — 1 passed
```

The final S3.4 battery below then executes the tuple through both clients and
compares its exact hash sequence with `git rev-list`. No product source changed.

## One serialized exhaustive integrated gate

Commands below ran serially. Counts are direct command output, not inferred
from intent.

### gwz-core

```text
cargo fmt --all -- --check
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo check --locked --all-targets
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python CLIPPY_CONF_DIR="$PWD" \
  cargo clippy --locked --all-targets --all-features -- -D warnings
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo test --locked
exit 0
lib: 1799 passed, 0 failed, 1 ignored (1800 total), 826.11 s
diff_render_spike: 10 passed
protocol: 33 passed
publish_workflow: 9 passed
rename: 2 passed
doc tests: 0
```

The first clippy dispatch failed before semantic analysis completed because the
volume had only 117 MiB available and rustc could not write its query cache
(`No space left on device`, exit 101). Only exact ignored Cargo caches from
four completed historical log worktrees were removed with `cargo clean`
(15.2 GiB reproducible cache). The same interrupted clippy partition then ran
once to exit 0. No source, index, or committed data was removed or changed.

### gwz-cli

```text
cargo fmt --all -- --check
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo check --locked --all-targets
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python CLIPPY_CONF_DIR="$PWD" \
  cargo clippy --locked --all-targets --all-features -- -D warnings
exit 0

python3 scripts/generate_cli_reference.py --check
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo test --locked
exit 0
lib: 122 passed; diff_workflows: 26; local_workflows: 25;
publish_workflow: 4; release_script: 2; rename: 2; doc tests: 0
```

### gwz-py and real workspace

```text
/Users/owebeeone/limbo/gwz-dev/.venv/bin/python -m maturin develop --release
exit 0 — exact ABI3 native rebuilt

cargo fmt --all -- --check
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo check --locked --all-targets
exit 0

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python CLIPPY_CONF_DIR="$PWD" \
  cargo clippy --locked --all-targets --all-features -- -D warnings
exit 0

PYTHONPATH="$PWD/src" GWZ_RUST_BIN=/Users/owebeeone/limbo/gwz-log-worktrees/s4.1/gwz-cli/target/debug/gwz \
  /Users/owebeeone/limbo/gwz-dev/.venv/bin/python -m pytest -q
exit 0 — 571 passed in 68.74 s

PYTHONPATH="$PWD/src" GWZ_RUST_BIN=/Users/owebeeone/limbo/gwz-log-worktrees/s4.1/gwz-cli/target/debug/gwz \
  /Users/owebeeone/limbo/gwz-dev/.venv/bin/python -m pytest -q \
  src/tests/test_log_real_workspace.py
exit 0 — 36 passed in 23.15 s
```

A direct `cargo test --locked` diagnostic for the PyO3 `cdylib` was not a
gwz-py gate: on macOS the `extension-module` feature intentionally leaves
Python symbols for the loading interpreter, so a standalone Rust test binary
cannot link them. It exited 101 with undefined `_Py*` symbols. The repository's
release/CI replacement evidence is the successful ABI3 `maturin` build plus
the full 571-case Python suite against that rebuilt module; the publish and
package-smoke workflows likewise build through maturin and run pytest, not
`cargo test`. Per operator direction this non-applicable diagnostic was not
rerun.

### Protocol generation and drift

```text
python3 protocol/regen.py --check --venv /Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv
exit 0 — additive fingerprint
sha256:d0c205c8767f8d54d32ead2f676a05077d849f6a12278d9de52b3c132c3c9372;
Rust API/runtime and both corpora current

PYTHONPATH="$PWD/src" /Users/owebeeone/limbo/gwz-dev/.venv/bin/python scripts/check_protocol_drift.py
exit 0 — sha256:46055287954f4035d07bb1bb88cf79f758a764cbadb1223d4944bf1848f7d277

PYTHONPATH="$PWD/src" /Users/owebeeone/limbo/gwz-dev/.venv/bin/python scripts/regen_protocol.py --check
exit 0 — generated Python API and exported IR current
```

### Structural, release, and real-workspace compatibility gates

```text
python3 scripts/checks/check_checked_artifact_boundaries.py
exit 0 — 15 visible entries, 5 classified modules

python3 -m unittest -v scripts/checks/test_check_checked_artifact_boundaries.py
exit 0 — 69 tests in 554.986 s

python3 -m unittest -v scripts/checks/test_release_boundary.py
exit 0 — 6 tests

python3 scripts/checks/check_m4_scenario_map.py \
  --doc /Users/owebeeone/limbo/gwz-dev/dev-docs/GwzM5-8R4bG-Evidence.md
exit 0 — 39 scenario rows, 43 named tests, 22 registry rows all claimed

python3 scripts/checks/check_merge_compatibility_predicates.py \
  dev-docs/GwzM5-8I2CompatibilityPredicates.json --core .
exit 0 — 7 migration rules, 7 runtime bindings, 10 archive shapes

python3 -m unittest -v scripts/checks/test_merge_compatibility_predicates.py
exit 0 — 27 tests

python3 scripts/checks/check_merge_docs.py \
  --workspace-root /Users/owebeeone/limbo/gwz-dev
exit 0 — 12 sources, 155 assertions

python3 -c 'from pathlib import Path; import unittest; import scripts.checks.test_check_merge_docs as suite; suite.WORKSPACE_ROOT = Path("/Users/owebeeone/limbo/gwz-dev"); result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromModule(suite)); raise SystemExit(0 if result.wasSuccessful() else 1)'
exit 0 — 3 tests

TAUT_PYTHON=/Users/owebeeone/limbo/gwz-dev/gwz-core/protocol/.regen-venv/bin/python cargo test --locked --lib \
  workspace_ops::tests::g23::
exit 0 — 124 passed, 1676 filtered out, 35.82 s
```

The explicit real-workspace arguments discharge the documented J-7 host
boundary without changing or copying the root-manifest/source-loading
inventories. The umbrella `/Users/owebeeone/limbo/gwz-dev/gwz-core` checkout
was stale at `834275d6633ccba0755859e9c6437b69ba52d05a` (tree
`ec8e1dba9b918f626c346aeccff95492eeef61f2`). Only the J-7/G23 gate-source
subset was byte-identical: a direct diff against exact core was empty for
`src/workspace_ops/tests/g23.rs`, `src/workspace_ops/tests/g23`, and
`scripts/checks`. All compilation, the full core suite, and G23 execution used
the exact worktree at `bdb398c3fa8581531eb1a38674ef89f56fc192e2`
(tree `20b52eb0b425e8482f4bd853fe4a6a580deb28e3`).

After the exhaustive product gate, S1.2 reconciled only
`dev-docs/GwzCommitMarker.md` in core candidate
`eb3a37c3d657b28c9fb3c85054056aa9192ee353` (tree
`fda06e172b75a4f85a7cbda4fbc2397e2a6fad18`), whose sole parent is the exact
gated core commit above. Its product source, manifests, locks, protocol,
generated artifacts, and test sources are byte-identical to the gated parent;
therefore the docs-only reconciliation requires no product-gate rerun.

## Release and scope disposition

`GwzLogPlan.md` §1.6 assigns no release to this plan. S4.1 therefore performs
no tag, push, version bump, protocol-version bump, dependency pin refresh, or
release-branch reconciliation. The package versions and all pins remain
byte-identical to the exact bases. Core has one docs-only S1.2 reconciliation
on top of the byte-identical gated product source; the only gwz-py tracked
change is the `:^` acceptance row; and the CLI candidate changes settle
documentation only. There is no `Cargo.toml`, `Cargo.lock`, `lib.rs`,
protocol/generated artifact, command implementation, renderer, or public seam
change.
