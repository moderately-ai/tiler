---
id: lower-bf16-to-metal
title: Lower a BF16 kernel to Metal and dispatch it on the measured macOS row
status: todo
priority: p1
dependencies: [admit-bf16-into-the-schedule-and-kernel-vocabulary, declare-the-bf16-rows-on-the-authoritative-metal-profile]
related: [spike-bf16-through-the-second-dtype-seams, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation]
scopes: [implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, metal, lowering, apple-targets]
---
## User-visible outcome

A verified BF16 kernel emits `bfloat` MSL, compiles, dispatches on the measured macOS row, and returns results that agree with `tiler-reference` within the target's declared numerical realization. The same kernel is refused before submission on the iOS Simulator.

## What the target already fixes

**Fact.** `MetalFloatArithmeticType::Bf16` exists in `crates/tiler-metal/src/target.rs` and already carries the measured BF16 flush in its own slot, inheriting nothing from `f32`.

**Fact.** `msl_type` maps `KernelType::F32` to `"float"`; the BF16 spelling is `bfloat`. `KernelConstant::F32Bits` emits `as_type<float>(0x...u)`; BF16 needs its own reinterpretation, and MSL's `ushort` is the carrier the Apple probe harness uses for it.

**Measurement — the flush is not optional and the agreement must account for it.** Finding 24 records that BF16 arithmetic on the macOS row **flushes** subnormal operands and results, sign-preserving. A reference that preserves subnormals will therefore disagree with the device on exactly the subnormal cases, and that disagreement is *correct behaviour on both sides*. The comparison must apply the declared `SubnormalMode` rather than expecting bit equality everywhere, and `ReferenceNumericalConformance` is the existing mechanism for that.

**Measurement — no fused form exists.** Finding 29 records that `metal` rejects `bfloat v6 = fma(v3, v4, v5)` with "cannot initialize a variable of type 'bfloat' with an rvalue of type 'float'". There is no `bfloat` overload of `fma`, so a BF16 contraction cannot lower to one.

**Measurement — contraction defence differs from `f16`.** Finding 28 records that under `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not. Do not carry an `f16` contraction conclusion across to BF16.

## Implementation keys

- `msl_type` gains `bfloat`, and BF16 constants emit through the `ushort` reinterpretation rather than `float`. **It will already have a BF16 arm when this ticket starts, and that arm is a refusal.** `admit-the-bf16-type-and-carrier-into-every-total-map` makes `msl_type` fallible and rejects BF16 by name, because `KernelType` is not `#[non_exhaustive]` and `crates/tiler-metal/src/emit.rs:812` stops compiling the moment the variant exists — and spelling `bfloat` there would have published an unmeasured capability while this ticket's profile dependency was still blocked. Replacing that refusal with the spelling is this ticket's job, and doing so is only admissible once the measured MSL 4.0 row is declared.
- BF16 binary operations map to the operator and to `MetalFloatArithmeticType::Bf16`, so the subnormal obligation is recorded against the right dtype. The existing machinery already refuses to answer an unstated dtype from a neighbour's fact; do not weaken it.
- A BF16 NaN canonicalization helper, distinct from `tiler_canonicalize_nan_f32_7fc00000`. The Apple harness's mangled name for the BF16 helper is in its recognizer and is the shape to match.
- Emission refuses when the target states no BF16 subnormal fact — the `Unknown` path, which is what the iOS device gets.
- `-ffp-contract=off` remains the contraction defence, measured at BF16 as well as `f32`.

## Required evidence

- A BF16 kernel emits, compiles offline, dispatches on macOS, and agrees with `tiler-reference` on every element **after** the declared flush is applied to the reference — with the subnormal elements shown to be the ones the flush moves, not silently excluded.
- An execution witness on a non-subnormal operand reports `executed`. Without it, "flushed" and "the arithmetic was optimized away" are the same observation.
- The same program is refused for the iOS-Simulator profile before any submission, by the dispatchability fact rather than by a pipeline failure.
- A target stating no BF16 subnormal fact refuses emission with the unstated-fact diagnostic, observed failing.
- A strict subnormal-preserving contract is refused on the macOS row with a named numerical gap.
- The F32 golden compilation is unchanged.

## Closes when

A BF16 kernel dispatches on the measured macOS row and agrees with the reference under the declared realization, the simulator refusal happens before submission, every refusal above is observed failing, the execution witness is present, and the `Backend lowering` and `Backend execution` cells for BF16 move with their host/toolchain boundary stated.

## Graph maintenance

- Depends on the kernel vocabulary and on the profile carrying the BF16 rows; emission consults the target fact and would fail closed without it.
- Does not depend on the artifact ticket: offline emission and dispatch do not need the artifact round trip, and keeping them independent lets the two land in parallel.
- Nothing here may claim an iOS-device BF16 result. That family is `Unknown` and only `measure-apple-numerics-on-physical-ios-device` can close it.
- Contraction, reassociation, and FMA are out of scope, and finding 29 makes the last one unimplementable at the source level. `design-the-bf16-computation-and-accumulator-contract` owns that question.
