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
