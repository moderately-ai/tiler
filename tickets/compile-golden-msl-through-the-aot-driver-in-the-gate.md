---
id: compile-golden-msl-through-the-aot-driver-in-the-gate
title: Compile the golden MSL through the AOT driver as part of the gate
status: done
priority: p1
dependencies: []
related: [prototype-metal-kir-lowering, prototype-apple-aot-driver]
scopes: [implementation/metal, implementation/metal-aot, implementation/workspace, implementation/cargo-lock]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, testing, verification]
---
`tiler-metal`'s golden fixtures prove **byte-stability and structure only**. They
do not prove the emitted MSL compiles. That distinction is currently load-bearing
and invisible: a change that keeps the goldens byte-identical while making them
uncompilable would pass the entire repository gate.

The gap is real but the evidence is good. Both the authoring agent and the
coordinator compiled all four goldens by hand with

```
xcrun --sdk macosx metal -Wall -Wextra -target air64-apple-macos13.0 \
  -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
  -ffp-contract=off
```

on Metal 32023.883 — **exit 0, zero diagnostics, all four** — and linked them into
a valid 14,620-byte `MTLB` library, the same byte count reached independently
twice. That is a real measurement, and it is exactly the kind of measurement that
silently stops being true when nobody re-runs it. Nothing in
`scripts/check_repository.py` invokes `xcrun`.

Close it by compiling the goldens through the already-merged `tiler-metal-aot`
driver — which exists precisely to run `metal`/`metallib` fail-closed and capture
toolchain provenance — rather than by shelling out to `xcrun` from a test.

Why this needs four scopes: `tiler-metal` currently has **no dependencies**, so
using the driver adds a `tiler-metal` → `tiler-metal-aot` edge, which touches the
crate manifest, the workspace lockfile, and `scripts/check_workspace.py`'s
`EXPECTED_DEPENDENCIES`. That is why the authoring ticket could not do it: it held
`implementation/metal` alone. Consider whether the dependency belongs on
`tiler-metal` itself or only on its test target, since a lowering crate depending
on a toolchain driver is a heavier edge than it first appears — a dev-dependency
may be the honest shape.

Follow the driver's existing pattern: **self-skip when no qualified toolchain
resolves**, so the gate stays green on machines and CI runners without Xcode,
exactly as `tiler-metal-aot`'s own toolchain-dependent tests already do. A test
that fails on a toolchain-less host would be worse than the gap it closes.

Also relevant: `scripts/check_workspace.py` pins `EXPECTED_TESTS` per package and
has no `tiler-metal` entry, which is why that crate's tests live in `src/` today.
Adding an integration-test target needs the same `implementation/workspace` scope.

## Outcome

**Fact — the gap is closed inside the gate.** `crates/tiler-metal/src/golden_compilation.rs` compiles every fixture in `goldens/` through `tiler_metal_aot::driver::Toolchain`, under the target and numerical realization the fixtures declare, and asserts real output: the linked library begins with the `MTLB` magic and contains the fixture's `tiler_kernel_*` symbol verbatim, and its provenance records the exact flags that reached `metal`. `cargo test --workspace` runs it, so `scripts/check_repository.py` now exercises the offline Apple toolchain where one is present.

**Fact — a development dependency, not a normal one.** The edge is `[dev-dependencies] tiler-metal-aot` on `tiler-metal`. Three reasons, in order of weight. Ownership: `tiler-metal` is documented as pure source emission that owns no Apple tool discovery, and a normal dependency would put a process-spawning toolchain driver into every consumer's build graph to serve tests alone. Direction: the natural production edge is `tiler-metal-aot` → `tiler-metal`, since the driver consumes emitted MSL; Cargo permits a cycle through a development dependency but rejects one through normal dependencies, so the development-only edge keeps that direction available and a normal edge would foreclose it. Honesty: only the tests need it, and the manifest should say so. The ticket's premise that `tiler-metal` has **no dependencies** did not survive contact with the source — at base `6555119` it already depends on `tiler-artifact` and `tiler-ir`; what it had no edge to was `tiler-metal-aot`, which was also absent from `[workspace.dependencies]` entirely.

**Fact — the workspace boundary stays exact.** `Cargo.toml` gains `tiler-metal-aot = { path = "crates/tiler-metal-aot" }` under `[workspace.dependencies]`; `crates/tiler-metal/Cargo.toml` gains the `[dev-dependencies]` table; `Cargo.lock` gains one line. `scripts/check_workspace.py` is updated in both places that pin this: `EXPECTED_WORKSPACE_DEPENDENCIES` and `EXPECTED_DEPENDENCIES["tiler-metal"]`, the latter with `kind="dev"`, which also drives the authored-manifest comparison in `expected_member_manifest`. `EXPECTED_TESTS` is **unchanged**: the tests are a `#[cfg(test)] mod` inside `src/`, not an integration-test target. That was a deliberate choice, not an avoided edit. Inside the defining crate a `match` over `#[non_exhaustive] MetalNumericalRequirement` is exhaustive, so `realization_honours` fails to compile if a requirement variant is added without naming the driver selection that delivers it; from `tests/` the same match would need a wildcard arm and that compile-time guard would be lost. Living in `src/` also lets the module reuse `crate::tests`'s kernel fixtures — five functions widened to `pub(crate)` — instead of duplicating roughly 200 lines of builder code.

**Fact — the self-skip is narrow and cannot pass silently.** `resolved_toolchain` treats only `DriverError::{ToolchainUnavailable, SdkUnavailable}` as an absent toolchain. Every other variant panics, and the classifying match is exhaustive over `DriverError`, which is not `#[non_exhaustive]`, so a new variant must be classified deliberately rather than defaulting to a skip. Each branch announces itself on standard error, visible under `cargo test -p tiler-metal --lib -- --nocapture golden_compilation`; the four conditional tests carry a `when_a_toolchain_resolves` suffix so the default output still says they are conditional. Setting `TILER_REQUIRE_METAL_TOOLCHAIN` turns a skip into a failure, so a runner that is supposed to have Xcode proves it did. It is the only ambient input here and it can only make the tests stricter.

**Measurement — it runs, and it has teeth.** On an Apple M4 Max under macOS 27.0 (build 26A5388g), Metal 32023.883, macOS SDK 26.5 (build 25F70), `cargo test -p tiler-metal --locked -- golden_compilation`: 7 passed in 0.50 s, linking 3,683/3,715/3,747/3,859 bytes for the four goldens and 14,716 bytes for the portfolio unit. Running the same test binary with `xcrun` off `PATH` gives 7 passed with four `skipped, no qualified Apple Metal toolchain resolved` announcements; adding `TILER_REQUIRE_METAL_TOOLCHAIN=1` to that environment fails those four. Injecting `v_undefined_symbol` into `reduction_multi_axis.metal` fails `every_golden_compiles_and_links_when_a_toolchain_resolves` with the compiler's own `use of undeclared identifier` diagnostic, naming the fixture — the failure the byte-stability goldens could not produce.

**Measurement — the ticket's 14,620-byte figure had already stopped being true.** Linking the four goldens at commit `59060b5` reproduces 14,620 bytes exactly; the same command at `6555119` yields 14,716, because `e24f4c5` changed the emitted source 47 minutes after the measurement was recorded. The measurement was correct and became false with nothing noticing, which is precisely the thesis of this ticket. `tickets/prototype-metal-kir-lowering.md` is amended to qualify that figure by commit rather than leave it reading as a durable fact. This module therefore asserts no `metallib` size. It does assert that compiling one golden twice yields identical bytes, which is the reproducibility claim worth holding: the driver's scratch directory name differs on every call, and a path-length difference of 29 characters changes the intermediate `.air` by 80 bytes while leaving the linked library byte-identical.

**Fact — what this now proves that the goldens did not.** Four things. The checked-in MSL is accepted by `metal` and linked by `metallib`, not merely stable. The symbols the emitter reports really exist in the compiled library, so the binding table describes something the compiler produced. The flag strings the two crates own agree: every `MetalNumericalRequirement::flag()` recorded by a live emission appears verbatim in `CompileRequest::compile_flags()`, and the typed selection behind it actually delivers the obligation. And the target the goldens declare in their provenance header is the target they are compiled for, checked by building the header strings from the driver's own `MetalTarget` and matching them against the fixture bytes. Two guards keep the coverage from decaying: `every_checked_in_golden_is_compiled_by_this_module` reads `goldens/` at run time and fails if a fixture is not in the compiled list, and the portfolio test compiles the multi-entry-point shared-helper form that no single golden pins, restoring in the gate the second half of the original hand measurement.

**Measurement — an incidental toolchain fact.** `metal` on 32023.883 accepts a translation unit with `#include <metal_stdlib>` removed but `using namespace metal;` retained (exit 0), so the emitted include is not load-bearing for the `metal` namespace on this toolchain. Recorded because it invalidated a first attempt at a breakage fixture, and because a future change should not assume removing the include is a detectable error.

**Measurement boundary.** Everything above is one host and one toolchain row. Compilation acceptance is not device execution: no value is computed here, and the numerical behaviour of these kernels remains the separately recorded device measurement. Nothing here proves the fixtures compile on another SDK, family, or Metal version; the driver's `MetalTarget` makes widening that a per-target compilation rather than an assumption. On a host without a qualified toolchain the four conditional tests contribute nothing, which is the intended trade and the reason `TILER_REQUIRE_METAL_TOOLCHAIN` exists.

**Verification.** `uv run --locked python scripts/check_repository.py` passed, `git diff --check` was clean, and `ticketsplease guard tkt/compile-golden-msl-through-the-aot-driver-in-the-gate` reported no scope escape.
