---
id: qualify-the-model-level-claims-per-apple-device-and-toolchain-row
title: Qualify the model-level claims per Apple device and toolchain row
status: todo
priority: p3
dependencies: [build-the-model-level-measurement-harness, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-level-qualification-and-optimization, measure-apple-numerics-on-physical-ios-device, define-the-model-level-regression-policy]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, apple-targets, metal, qualification, measurement, language-model, class-performance-study]
---
## User-visible outcome

Which of the four model-level claims holds on which Apple device family and toolchain row is a maintained table with `Unknown` where a row is unmeasured, rather than a reader's inference from a `macOS` label.

## Why this exists, and why it is per claim rather than per row

**Fact.** L8's four named claims — Correctness, Feasibility, Estimated cost, Measured performance — have different device and toolchain dependencies, and the maintained matrix carries the documented half-rows rather than collapsing each claim to one cell. The matrix rows are: reference-side bound (no device; CPU and pinned reference); correctness, Tiler side (live Apple9 device); feasibility, compile side (offline toolchain, no device); feasibility, device side (live device, pipeline creation and dispatchability); **Estimated cost** (no device; toolchain identity is the target-profile identity; current state 8 of 9 analytical components `Unknown`); measured performance (bench host specifically). A single "supported on macOS" cell would assert the strongest of those for all rows.

**Fact — the precedent for splitting a matrix this way is in this scope already.** [`docs/research/apple-targets/numerical-behaviour.md`](../docs/research/apple-targets/numerical-behaviour.md) opens with *Which artifact family each finding covers*, which separates the compile side (three families) from the device side (two GPUs, and one of them is the other under a different name).

## Required work

- Build the per-claim table with the named profile `apple9-f32-unified-msl4-macos26` as the row key, plus host, OS build, Xcode build, and offline compiler build. **Not `registryID`** — it is a within-run correlation handle whose measured lifetime is bounded from below, it changed at least once between retained records for the same named Apple M4 Max, and [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) eliminates it by name as an applicability predicate.
- Record the native translator identity as `Unknown` for every AOT row, and record the source-JIT compiler build only against the rows it actually qualifies. Substituting one for the other certifies a relationship no measurement established.
- State the family coverage exactly: macOS measured; physical iOS device unmeasured because none is attached; the iOS Simulator dispatching on the host Mac GPU, so a simulator result is a measurement about the simulator and not about iOS hardware.
- **Carry the compile-then-refuse case as a first-class outcome.** The same record measures a family whose toolchain compiles and links a module its own device then declines to run. So "it compiled for this target" is not a dispatchability claim, and a matrix that inferred one would already be wrong on a measured row.
- Every unmeasured cell reads `Unknown` and is never predicted from a neighbour. The rule has been refuted empirically once inside this record — the subnormal flush was inferred dtype-independent from a module-level declaration and measured to be false — so it is recorded as a rule with its counterexample rather than as a convention.
- **Reference-side State must not copy the understated transferred L8 string.** Cell values for the reference-side bound must reflect the 2026-08-01 joint measurement in [`docs/research/program-planning/model-level-qualification.md`](../docs/research/program-planning/model-level-qualification.md) (post-transfer correction): all three of P-reorder, P-flush, and P-elem are measured jointly on the correctness host with no Metal compilation, no device, and no Tiler execution — not "P-reorder measured; P-flush and P-elem measurable today".

## Explicit non-goals

No new device measurement, no iOS work, and no target-profile contract change. If the matrix shows a cell that a stated target fact cannot express, file that separately rather than widening a type here.

## Closes when

The table exists in the Apple-targets scope with the L8 matrix claim-axis (six rows: reference-side bound; correctness Tiler side; feasibility compile side; feasibility device side; Estimated cost; measured performance — or an equivalent nested presentation of the four named claims that still carries those halves and Estimated cost), a row key that is the named profile rather than a device handle, `Unknown` in every unmeasured cell, reference-side State reflecting the joint 2026-08-01 measurement rather than the understated transferred string, and the compile-then-refuse case represented as an outcome the matrix can express.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Filing prose said "four claims" with a device-dependency list that split Feasibility and Correctness but omitted Estimated cost, and Closes when said "one row per claim". L8's matrix is six claim-rows under four named claims (Correctness and Feasibility each have a documented half-row; Estimated cost and Measured performance are full rows). Estimated cost has device requirement none, target-profile identity as toolchain key, and state "8 of 9 components `Unknown`". The reference-side State string still printed in the transferred L8 matrix body is understated: post-transfer correction 2026-08-01 records all three of P-reorder, P-flush, and P-elem measured jointly. Required work and Closes when above align the row schema and reference-side cell values with that record. Environment key wording (named profile + host + OS/Xcode/offline compiler builds; native translator `Unknown` on AOT; not `registryID`) was already correct and is unchanged.
