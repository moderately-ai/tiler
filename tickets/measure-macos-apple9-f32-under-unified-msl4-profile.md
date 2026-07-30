---
id: measure-macos-apple9-f32-under-unified-msl4-profile
title: Measure macOS Apple9 F32 under the unified MSL 4 profile
status: done
priority: p0
dependencies: []
related: [restore-replayable-apple-compatibility-evidence, record-metal-runtime-compiler-provenance-gap, carry-the-dtype-on-the-metal-subnormal-flush-fact, spike-bf16-through-the-second-dtype-seams]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [metal, numerics, target-profiles, evidence]
---
## User-visible outcome

The first production Metal profile has one retained, replayable Apple9/macOS measurement produced under the exact unified MSL 4.0 and macOS 26 target it will name, covering F32 dispatch and every numerical claim used by the offline and runtime compilation paths.

## Facts and measurement boundary

**Fact:** the retained numerical record currently fixes MSL 3.1 and requests an older target that the compiler raises to a macOS 14 emitted triple. It is excellent evidence for that exact row and is not evidence for a new MSL 4.0/macOS 26 profile merely because the same host and compiler accept both.

**Fact:** one macOS host resolves the offline `metal` frontend/code generator, `metallib` linker, and an OS-resident runtime/pipeline compiler as distinct provenance components. The two compilation paths can differ and must be recorded separately even where observations agree.

**Measurement boundary:** qualify Apple9 on the exact measured device, OS version/build, architecture, Xcode build, macOS SDK version/build, requested and emitted target, compiler/linker/runtime-compiler builds, optimization selections, flags, source bytes, and harness revision. Do not generalize to another Apple family, Intel Mac, physical iOS, F16, BF16, or an unmeasured toolchain row.

## Experiment keys

Extend the checked-in Apple numerical harness so one producer constructs the exact MSL 4.0/macOS 26 source and options consumed by both offline and runtime cases. Run both the covering matrix and the exhaustive matrix, include a witnessed F32 dispatchability case, retain raw records and the exact input manifest, and document the invocation and stop conditions. Keep runtime compilation provenance separate from artifact AOT provenance.

## Required evidence

Retain covering and exhaustive records with complete environment and compiler identities, source/harness/manifest digests, terminal command-buffer success, execution witnesses, exact F32 bit patterns, and comparisons between offline and runtime compilation. Add a negative validator mutation that changes a producer-defining input or manifest byte and must fail. Re-run the validator against the restored record and record both commands.

## Closes when

A clean replay produces the retained MSL 4.0/macOS 26 covering and exhaustive rows, the validator demonstrably rejects a mutated producer input, all portable harness tests pass, and the research memo states exactly which F32 dispatch and numerical claims the row supports and which remain `Unknown`.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile` and `validate-macos-metal-profile-host-applicability`. Keep `restore-replayable-apple-compatibility-evidence` related because compatibility replay is adjacent but not a substitute for numerical execution, keep `record-metal-runtime-compiler-provenance-gap` related, and keep the dtype tickets related without widening this F32 experiment.
