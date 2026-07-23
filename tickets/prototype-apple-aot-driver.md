---
id: prototype-apple-aot-driver
title: Implement the Apple offline compiler driver
status: done
priority: p0
dependencies: [repair-apple-target-experiment-integrity, prototype-target-feasibility-authority, enforce-repository-validation-gate-integrity]
related: []
scopes: [implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, metal, aot, toolchain]
---
Implement a bounded driver with explicit SDK, platform family, deployment minimum, MSL version, output-affecting flags, metal/metallib invocation, diagnostics, fingerprint and provenance. Use one selected SDK and never inherit output-affecting defaults silently; exclude cache and proc-macro concerns.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Outcome

**Status:** implemented as a bounded, fail-closed offline compiler driver in the
new `tiler-metal-aot` crate. Delivered on branch `tkt/prototype-apple-aot-driver`
from base `c5c72efcc6683ebd12c3d928383ee89cd42ec099`.

### Driver contract (Fact)

- `crates/tiler-metal-aot/src/input.rs` models the request. A `CompileRequest`
  carries the MSL `source` plus a `MetalTarget` (one selected `AppleSdk`, a
  `DeploymentMinimum`, and an `MslVersion`), an `OptimizationLevel`, and a
  `NumericalRealization`. `NumericalRealization` bundles the three independent
  numerical permissions (`MathMode`, `Fp32Functions`, `FpContract`) and has **no
  `Default`**; the strict `safe`/`precise`/`off` baseline is the named
  `strict_baseline()` constructor, not an inherited default. `compile_flags()`
  returns the exact ordered `metal` flags (`-target <triple> -std=metal<v> -O<n>`
  then the three numerical flags); `link_flags()` is the reserved empty
  `metallib` seam.
- `crates/tiler-metal-aot/src/driver.rs` is the driver. `Toolchain::compile`
  takes MSL + explicit target/flags as input and returns `metallib` bytes plus
  provenance as output. It threads the single selected SDK to both the `metal`
  and `metallib` invocations, runs them with `ZERO_AR_DATE=1`, and validates the
  linker output begins with the `MTLB` Metal-library magic. It does not emit MSL
  and does not assemble the artifact bundle.

### Provenance and fingerprint (Fact)

- `Toolchain::resolve(sdk)` is the preflight: it resolves `metal`/`metallib`
  paths (`xcrun --find`), reads both tool versions, and reads the SDK identity
  (`--show-sdk-path`/`--show-sdk-version`/`--show-sdk-build-version`) before any
  compilation. `record.rs` captures this into `ArtifactProvenance`: platform
  family, normalized triple, deployment minimum, MSL version, optimization,
  numerical realization, `SdkIdentity`, resolved `metal`/`metallib`
  `ResolvedTool`s, the `CompilerFingerprint` (the two component version
  strings), and the exact `compile_flags`/`link_flags`.
- **Inference / deferred:** the portable fingerprint is the version strings; the
  resolved tool paths are local provenance that on this host encode the resolved
  Metal-toolchain component. A content digest of the tool binaries and SDK would
  strengthen cross-host identity (two component builds can share the `32023.883`
  front-end version); it is deferred to the expansion-cache/identity ticket and
  documented as a seam, keeping this driver dependency-free.

### Fail-closed behaviour (Fact)

Every failure returns a typed `DriverError` and no artifact:
`ToolchainUnavailable` when `xcrun`/`metal`/`metallib` cannot be run or report
no version (this is the expected path on a non-macOS host, since spawning a
missing launcher fails), `SdkUnavailable` when SDK identity cannot be read,
`ToolFailure { stage, status, stderr }` when a tool exits nonzero (carrying the
captured stderr), `Host` for scratch filesystem/process failures, and
`EmptyArtifact` when the linker yields no `MTLB` library.

### Workspace admission (Fact)

- Created `crates/tiler-metal-aot/` with a manifest mirroring existing members
  (workspace-inherited version/edition/license/repository/publish/lints; no
  external dependencies; not dependent on Candle or live runtime APIs).
- Added `crates/tiler-metal-aot` to the root `Cargo.toml` members and to
  `scripts/check_workspace.py`'s `EXPECTED_MEMBERS`, `PACKAGE_DESCRIPTIONS`,
  `PACKAGE_DIRS`, and `EXPECTED_DEPENDENCIES` (empty deps). `check_workspace.py`
  reports `Rust workspace boundary passed`; the contract was satisfied, not
  relaxed. Updated `Cargo.lock` with the new zero-dependency package.
- Replaced the temporary `implementation/metal-aot` `[scope_crates]` owner
  `tiler-prototype-compile` with the real owner `tiler-metal-aot` in
  `ticketsplease.toml`, so reverse-dependency expansion is no longer attached to
  the prototype.

### Toolchain reality (Measurement)

`xcrun --find metal` resolved on this host to the MobileAsset
`MetalToolchain-v17.6.109` component (`metal`/`metallib` front-end version
`32023.883`), matching the qualified compatibility row; the host Xcode is
`26.6` build `17F113`. Toolchain-dependent tests self-skip when
`Toolchain::system().resolve(...)` fails, so the canonical gate passes with or
without a live toolchain. On this host all toolchain-gated tests ran real
`metal`/`metallib` compiles and passed.

### Verification (Measurement)

`uv run --locked python scripts/check_repository.py`, `git diff --check`, and
`tkt guard tkt/prototype-apple-aot-driver` all pass. Cache and proc-macro
concerns are excluded by design and remain later tickets.
