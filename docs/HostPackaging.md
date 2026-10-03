# SSPI self-execution packaging

`gwz::run` inspects the private worker marker before build-info, parser, Git and
runtime initialization. A malformed invocation or absent/malformed compile-time
fingerprint refuses silently with exit 2. Ordinary arguments continue normally.
Runtime environment cannot provision this entry. `sspi_worker_executable()` is a
callable descriptor for the actual current executable with compiled expected
bytes; it creates no Supervisor or child. HTTP composition remains future work.

Normal Cargo builds are Python-free and unprovisioned. An explicit local build
uses the Cargo-resolved SSPI producer:

```sh
python3 scripts/build_sspi.py --target aarch64-apple-darwin --profile dev \
  --target-dir /absolute/external/cache
```

Release CI provisions every actual cargo-dist matrix target before cargo-dist
builds. The build script selects the fingerprint for Cargo TARGET from that
build-only table; each identifier records the target, dist profile, exact matrix
options, compiler, Cargo graph/lock, source/contracts and effective workspace
configuration. A diagnostic receipt accompanies release artifacts. It is not
runtime authority or a finished binary hash. No credentials belong in metadata.

Cargo archives include scripts/build_sspi.py; the producer is discovered through
Cargo metadata from the SSPI dependency, never a sibling-source guess. Registry
reconciliation pins SSPI =0.1.0 before the existing no-path release check. SSPI is
currently publish=false and unavailable as a published registry prerequisite;
local provisioned builds do not bypass that release block. Extracted consumers
must resolve the pinned dependency through Cargo.

The actual matching worker Hello still must agree with the descriptor before
Begin. Portable early-dispatch/descriptor tests do not qualify Windows installed
mismatch or native operation. Digest remains refused, the Unix candidate endpoint
gate is unchanged, and full Windows transport/release remains NO-GO.

Local binaries appear under TARGET_DIR/TARGET/debug (dev) or the requested
profile directory. Release bundles/installer retain their ordinary executable
layout, with receipts under target/distrib during packaging. Install or upgrade
the whole trusted CLI artifact; self-execution needs no companion worker file.
Removing the CLI executable removes its worker entry as well. Unprovisioned
portable builds remain usable for ordinary CLI commands, but their descriptor
refuses; a portable descriptor is not usable Windows HTTP authentication.

## Explicit wrapper options

| Option | Local invocation | Matrix invocation |
|---|---|---|
| `--target TRIPLE` | Required, no default; Rust target triple | Not accepted |
| `--target-dir PATH` | Required absolute output directory, no default | Not accepted; uses build-time CARGO_TARGET_DIR or `target` |
| `--profile NAME` | Optional, defaults to `release`; `dev`, `release`, `dist` or another configured Cargo profile. Cargo refuses undefined profiles | Not accepted; matrix uses `dist` |
| `--targets JSON` | Not accepted | Required nonempty distinct target-triple array from the actual cargo-dist matrix |
| `--dist-args TEXT` | Not accepted | Required exact matrix options, supplied as `--dist-args='...'` |
| `--help` | Prints usage and exits | Same |

Targets are triples, not custom JSON target paths. Cross builds need the selected
Rust target and platform build prerequisites. Local output is
TARGET_DIR/TRIPLE/debug for `dev`, or TARGET_DIR/TRIPLE/PROFILE otherwise, plus
TARGET_DIR/distrib/sspi-artifact-sets.json. Matrix receipts have target-set-qualified
`sspi-artifact-sets-<digest>.json` names; flattened aggregation retains each matrix
set. These receipts remain diagnostic, not worker authority.

For a host-native local release-profile build, derive the compiler's actual host
triple rather than copying the Darwin example (run from the CLI checkout):

```sh
python3 -c 'import subprocess,sys;host=next(line[6:] for line in subprocess.check_output(["rustc","-vV"],text=True).splitlines() if line.startswith("host: "));subprocess.run([sys.executable,"scripts/build_sspi.py","--target",host,"--target-dir","/absolute/external/cache"],check=True)'
```

This omits `--profile`, selecting release. Local provisioning builds one explicit
Cargo target; release provisioning consumes the actual cargo-dist matrix and
passes its per-target metadata to subsequent dist builds. An ordinary Cargo
build supplies neither handoff and remains unprovisioned.
