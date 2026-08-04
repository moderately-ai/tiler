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

**Fact.** The four claims have different device dependencies. The reference-side comparison bound needs no device at all; compile-side feasibility needs the offline toolchain and no device; device-side feasibility and every correctness observable need a live Apple9 device; measured performance needs the bench host specifically. A single "supported on macOS" cell would assert the strongest of those for all four.

**Fact — the precedent for splitting a matrix this way is in this scope already.** [`docs/research/apple-targets/numerical-behaviour.md`](../docs/research/apple-targets/numerical-behaviour.md) opens with *Which artifact family each finding covers*, which separates the compile side (three families) from the device side (two GPUs, and one of them is the other under a different name).

## Required work

- Build the per-claim table with the named profile `apple9-f32-unified-msl4-macos26` as the row key, plus host, OS build, Xcode build, and offline compiler build. **Not `registryID`** — it is a within-run correlation handle whose measured lifetime is bounded from below, it changed at least once between retained records for the same named Apple M4 Max, and [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) eliminates it by name as an applicability predicate.
- Record the native translator identity as `Unknown` for every AOT row, and record the source-JIT compiler build only against the rows it actually qualifies. Substituting one for the other certifies a relationship no measurement established.
- State the family coverage exactly: macOS measured; physical iOS device unmeasured because none is attached; the iOS Simulator dispatching on the host Mac GPU, so a simulator result is a measurement about the simulator and not about iOS hardware.
- **Carry the compile-then-refuse case as a first-class outcome.** The same record measures a family whose toolchain compiles and links a module its own device then declines to run. So "it compiled for this target" is not a dispatchability claim, and a matrix that inferred one would already be wrong on a measured row.
- Every unmeasured cell reads `Unknown` and is never predicted from a neighbour. The rule has been refuted empirically once inside this record — the subnormal flush was inferred dtype-independent from a module-level declaration and measured to be false — so it is recorded as a rule with its counterexample rather than as a convention.

## Explicit non-goals

No new device measurement, no iOS work, and no target-profile contract change. If the matrix shows a cell that a stated target fact cannot express, file that separately rather than widening a type here.

## Closes when

The table exists in the Apple-targets scope with one row per claim, a row key that is the named profile rather than a device handle, `Unknown` in every unmeasured cell, and the compile-then-refuse case represented as an outcome the matrix can express.
