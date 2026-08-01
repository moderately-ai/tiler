---
id: measure-code-domain-integer-arithmetic-on-the-qualified-apple-row
title: Measure code-domain integer arithmetic on the qualified Apple row
status: todo
priority: p2
dependencies: [scope-first-quantized-lm-profile]
related: [broaden-the-apple-numerical-probe-matrix, implement-first-quantized-backend-profile, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, quantization, measurement]
---
## User-visible outcome

The integer machinery a quantized decode actually executes — a `u8` buffer read, an `int` subtraction, an `int`-to-`float` conversion, and the multiply that follows — is measured on the qualified Apple row instead of assumed from a derivation. Today no integer arithmetic of any kind has ever been measured on an Apple GPU in this repository.

## What is unmeasured, stated exactly so it can be refuted

**Fact.** The retained Apple numerical probe's dtype axis is exactly `f32`, `f16`, and `bf16`; its generated kernels use `uint`/`ushort` only as bit-pattern carriers inside `as_type` immediates and the NaN-canonicalization helper, and contain no integer arithmetic operation. [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) says so itself: "every integer and quantized format remain entirely unmeasured, and an unmeasured dtype must not be generalized from a neighbour". The one sub-byte construct in the repository, the U4 extraction expression in `crates/tiler-metal/src/emit.rs`, is checked at the string level by a test whose name ends in `_is_refused_on_the_measured_apple_profile`, is absent from the compiled golden fixtures, and has never been dispatched.

**Inference — the residual risk is narrow, which is what makes the experiment small rather than what makes it unnecessary.** [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) derives that the decode's values are exact by construction over the finite code domain: the subtraction cannot overflow, the conversion is exact for magnitudes at most 255, and no operand or result is subnormal when the scale is normal. So this is not a numerical-behaviour sweep asking what the hardware rounds to — it is a compile-and-dispatch check asking whether the emitted MSL computes what the contract says.

## The bounded experiment

- **Inputs.** Kernels on the `apple9-f32-unified-msl4-macos26` row under the governed flags (`-target air64-apple-macos26.0 -std=metal4.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`) that read a `u8` buffer, widen to `int`, subtract an `int` zero point, convert to `float`, and multiply by a `float` scale. The complete 256 × 256 code and zero-point grid, against a scale corpus spanning the measured workload range (`1.358e-5` to `1.536e-1`), the `f32` minimum normal `2^-126`, and at least one deliberately subnormal scale.
- **Outputs.** The returned bits against exact rational evaluation rounded once to binary32, per case; and the delivered behaviour at the subnormal scale, recorded separately.
- **Stop condition.** Either every cell of the grid matches the reference and the subnormal case is observed flushing exactly where the derivation predicts, or a divergence is found and named with its exact inputs.
- **What it must not do.** It must not generalize to another Apple family, another dtype, a packed sub-byte extraction it did not run, or integer arithmetic outside the code domain it measured. A `u4` extraction is a *different* measurement and belongs in its own row if a `u4` profile is ever selected.

## Closes when

The measurement is run and retained beside the existing Apple records under its own dated result directory with its own schema row, its environment and toolchain identity recorded exactly, its population named and counted so a check that did not run is distinguishable from a check that found nothing, the subnormal case demonstrated, and [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) updated with the finding and its boundary — or the measurement is blocked and the exact blocker is recorded in its place.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) as experiment E-1 of [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md).
- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) depends on this before claiming device executability; an unmeasured `(target family, dtype)` pair is `Unknown` and cannot produce an executable artifact.
- This measures correctness, not speed. The bandwidth comparison is experiment E-2 and belongs with cost calibration.
