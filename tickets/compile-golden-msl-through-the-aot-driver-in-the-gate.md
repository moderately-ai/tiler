---
id: compile-golden-msl-through-the-aot-driver-in-the-gate
title: Compile the golden MSL through the AOT driver as part of the gate
status: todo
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
