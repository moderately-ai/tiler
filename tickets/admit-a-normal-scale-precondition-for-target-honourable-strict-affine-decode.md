---
id: admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode
title: Admit a normal-scale precondition so strict-affine decode is target honourable
status: done
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile]
related: [implement-first-runtime-semantic-value-precondition-enforcement, produce-typed-strict-affine-quantize-semantic-preconditions, implement-first-quantized-backend-profile, enforce-resolved-encoded-value-binding-conformance, resolve-the-provisional-normal-scale-discharge-public-surface]
scopes: [implementation/ir, implementation/reference, implementation/metal, implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, numerics, metal]
---
## User-visible outcome

A strict-affine value whose scale is a normal `f32` decodes on the measured Apple profile instead of being refused, because the obligation the contract declares is narrowed to one the target can actually honour — and a subnormal scale still refuses, by name, at the earliest layer that can see it.

## Why the current refusal is right and still wrong to leave

**Fact.** `tiler-metal` emits the structured strict-affine dequantization vocabulary and then refuses with `MetalNumericalGap::SubnormalFlushInArithmetic`, because the registered decode contract declares `preserve-subnormals` unconditionally while the qualified `apple9-f32-unified-msl4-macos26` row flushes `f32` input and result subnormals. That is fail-closed behaviour working, and weakening the contract to remove it would be exactly the substitution [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids.

**Inference — the obligation is stronger than the operation needs, and the derivation is exhaustive over the code domain.** [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) derives it in full: the i32 subtraction of two codes in `[0, 255]` is exact and cannot overflow; converting a value of magnitude at most 255 to `f32` is exact, so the converted operand is `+0.0` or has magnitude at least `1.0` and is never subnormal; the product with the scale is `+0.0` when the codes are equal, and otherwise has magnitude at least the scale, so it is subnormal only if the scale is. Therefore **a normal scale makes the decode bit-identical under `FlushSubnormalsToZeroF32` and under a subnormal-preserving `f32`**, and the flush has nothing to act on.

**Fact — the seam already exists and this strengthens it.** `QuantizeStrictAffine` declares `positive_finite_scalar_predicate` on operand 1 with the typed invalid-input code `tiler::strict-affine-quantize-scale-not-positive-finite@1`. A positive *normal* predicate is strictly stronger: it admits nothing the current predicate rejects, so tightening it cannot make a currently valid program invalid in a way that surprises a caller — it narrows a valid domain in order to discharge an obligation.

## Implementation keys

- The predicate is a new named semantic value predicate, not a flag on the existing one, and it carries its own invalid-input code so a diagnostic distinguishes "scale is zero, negative, infinite, or NaN" from "scale is subnormal". Two different causes with two different fixes must not share one code.
- `DequantizeStrictAffine` currently declares no semantic preconditions. Decide explicitly whether the normal-scale obligation attaches to the assembled value's type contract, to `Assemble`, or to `Dequantize`, and state the reason at the site: a decode that receives an already-assembled value cannot re-derive where its scale came from.
- The honourability decision in `tiler-metal` must consult the discharged obligation rather than the contract's unconditional declaration. Whatever carries that — a narrowed realization requirement on the schedule, or an obligation the verifier discharges — must be a checked lowering, not a comment.
- Constant producer operands prove their operation predicates statically through the existing `StandardConstantF32BitsV1` proof basis. The selected workload instead supplies a direct encoded input whose positive-normal scale domain is enforced through [`enforce-resolved-encoded-value-binding-conformance`](enforce-resolved-encoded-value-binding-conformance.md) and its runtime integration. This ticket enforces neither path.
- Nothing about U4 packing, per-axis maps, or a contraction belongs in this ticket. It moves one predicate and the refusal that depends on it.

## Closes when

The strict-affine decode passes the Metal honourability boundary for a normal scale and is refused for a subnormal one, both demonstrated — the subnormal case watched failing before the change is relied on; the two scale-domain diagnostics are distinct and each was observed firing; the static proof path and the residual-obligation path are both exercised; the derivation above is recorded at the site it governs rather than only in the research record; targeted package tests and Clippy pass; `tkt lint` and `git diff --check` pass; and one `make full` passes.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) from [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md). It is the first ticket in that record's delivery order because every later one assumes the target can honour the decode.
- Advancing this does **not** make a quantized program executable. This bullet used to say integer arithmetic had never been measured on an Apple GPU here; that is stale as of 2026-07-31, when [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) ran E-1 and landed finding 32 — the complete decode chain agreed bit-for-bit with the exact rational reference over 1,310,720 normal-scale cells on the qualified row. The selected U8 route still needs its per-axis map, fused consumer, static direct-binding enforcement plan, and post-`RoutingCommit` conformance scan. Packed-tail validation and unmeasured packed extraction belong only to the separate U4 representation.
- Move no cell of [the dtype support ledger](../docs/dtype-support.md) for the U4 profile beyond what this ticket actually tests. **Done — no cell moved.** The decode passing a *compile-side* honourability boundary is not dispatchability evidence, and the ledger's own rule against reading recognition as support applies exactly here.

## Outcome

**Where the obligation attaches, and why.** Three carriers, one decision, stated at `strict_affine_type` in `crates/tiler-ir/src/semantic/quantization.rs`.

- **The type contract carries the guarantee.** A new static-contract field `ENCODED_NUMERIC_SCALE_DOMAIN` (`AttributeFieldId(11)`) states `positive-normal-f32` on both admitted strict-affine contracts. A value's admissible component domain is part of what its encoded-numeric type *is*, and the decode consumes the type, so this is the only carrier a consumer can rely on without inspecting provenance it cannot reach.
- **The producers carry their own obligations.** `QuantizeStrictAffine` and `AssembleStrictAffine` each declare both scale predicates on operand 1, where the scale is still a typed rank-zero `f32` whose producer is visible. That supports proof composition for those operations, but neither is present in the selected workload: its encoded weight arrives as a direct interface input and is governed by resolved-value binding conformance.
- **`Dequantize` carries neither, and the reason is structural rather than an omission.** Its sole operand is the complete resolved compound value. `SemanticLogicalView` has only `WholeValue`, and `StandardConstantF32BitsV1` reads the exact bits of an `f32` *scalar* constant, so a declaration there could never be proven statically — every decode, including one over a governed constant scale, would carry a residual, and the honourability question would still be unanswered where it is asked.

**The two diagnostics, verbatim.** `tiler::strict-affine-quantize-scale-not-positive-finite@1` and `tiler::strict-affine-quantize-scale-subnormal@1`; on the assemble side, `tiler::strict-affine-assemble-scale-not-positive-finite@1` and `tiler::strict-affine-assemble-scale-subnormal@1`. Each was observed firing. The names are load-bearing: static disproof priority is `(invalid-input code, declaration ordinal)`, and `…-not-positive-finite` orders before `…-subnormal`, so a value failing both — a negative subnormal — reports the general cause rather than the narrower magnitude complaint. Perturbing one code's name to reverse that order was watched failing.

**The honourability transition, in three observations.** Before: `strict_affine_u4_dequantization_is_refused_on_the_measured_apple_profile` passed, asserting `UnrealizableNumericalObligation { gap: SubnormalFlushInArithmetic }`. Mid-change: that same assertion failed with `called Result::unwrap_err() on an Ok value: ()` — the refusal was gone and the old test caught it. After: the renamed `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile` asserts an empty gap set and a successful `require_declared_realization`, *and* asserts in the same test that `pointwise_kernel` — declaring the identical `Preserve`/`Preserve` realization on the identical target row — is still refused by name. A subnormal scale never reaches this boundary: it is refused earlier, at the semantic precondition, which is the "earliest layer that can see it" the outcome asks for.

**What carries the discharge, as a checked lowering.** `tiler_ir::schedule::SubnormalFreedom` (`Unproven` | `StrictAffineNormalScaleDecode`), with `discharges(ArithmeticType) -> bool`. It is **derived, never declared**: `VerifiedScheduledRegion::subnormal_freedom` and `KernelBuilder::build` both call one `subnormal_freedom_of` over the region's scalar program, and `KernelBuilder::new` — the only path reaching `build` — takes a `VerifiedScheduledRegion`, so the program classified has already passed `verify_strict_affine_u4_dequantize`, which proves the three component roles are exactly the governed strict-affine constants of one input tensor and that the realization is the strict one. There is no setter, so a producer cannot assert a freedom its values lack. `tiler-metal`'s `record_subnormal_obligation` consults it before resolving any target fact, through a private exhaustive `MetalFloatArithmeticType → ArithmeticType` map.

**Why the freedom is typed rather than boolean.** The derivation rests on `f32`'s exponent range and on integers up to 255 being exactly representable in `f32`; neither premise transfers to a narrower format, so the decode's freedom covers `F32` alone and a region emitting `f16` arithmetic under it would still record its gap.

**Static and residual paths, both exercised.** Static: `exact_governed_constants_prove_each_predicate_without_an_obligation` proves all three predicates through `StandardConstantF32BitsV1` at `f32::MIN_POSITIVE`, `0.5`, and `f32::MAX`; `each_exact_constant_removes_only_its_own_residual` shows a constant scale proving both scale predicates while the expressed-value predicate stays residual. Residual: `runtime_unknown_u4_and_u8_quantize_inputs_retain_three_ordered_residuals` pins three residuals in declaration order with three *distinct* obligation identities — the two scale predicates share an operand and must not collapse into one runtime check — and `obligation_identity_is_occurrence_exact_and_topological_order_independent` now pins six distinct identities over two occurrences. No tensor value is enforced here. The selected runtime ticket owns direct bound-value conformance only; operation-residual enforcement awaits a selected producer.

**Where the derivation is recorded, with finding 32 cited at each.** `positive_normal_scalar_predicate` in `crates/tiler-ir/src/semantic/precondition.rs`; `SubnormalFreedom::StrictAffineNormalScaleDecode` in `crates/tiler-ir/src/schedule/numerics.rs`; and the strict-affine profile section of `docs/numerical-semantics.md`. Each states the exhaustive argument and then the measurement with its boundary — one GPU family, one toolchain and flag row, `u8` codes, one non-overflowing subtraction, no packed extraction, no timing.

**Public surface added** (`implementation/ir`; for the provisional-acceptance packet, not self-accepted):
- `tiler_ir::semantic::positive_normal_scalar_predicate() -> SemanticPredicateIdentity`
- `tiler_ir::semantic::ENCODED_NUMERIC_SCALE_DOMAIN: AttributeFieldId`
- `tiler_ir::schedule::SubnormalFreedom` (enum, two variants, not `#[non_exhaustive]`, matching the module's stated convention) and `SubnormalFreedom::discharges`
- `tiler_ir::schedule::VerifiedScheduledRegion::subnormal_freedom`
- `tiler_ir::kernel::VerifiedKernel::subnormal_freedom`

No public item was removed or renamed. `SubnormalFreedom` is deliberately *not* encoded into `CanonicalKernelIdentity`: it is a total function of the scheduled program whose canonical identity the kernel identity already folds in, and keeping it out of `KernelData` also keeps it out of the refinement gate, where it would have compared a derived value against itself.

**Digest movement — one pin rebaselined, one scope added.** The explain request qualifier moved `0b7759de2d9b5756` → `bae4788d2fc79631` at `crates/tiler-compiler/src/explain.rs`, because the request subject covers the frozen semantic registry snapshot and two things inside it moved: the strict-affine value contract gained its scale-domain field, and two operations gained precondition declarations. This is the assertion working, not collateral damage — a request whose semantic authority admits a different set of scale values is a different request. No encoding version advanced. `implementation/compiler` was added to this ticket's scopes for that one-line rebaseline plus its comment, following the precedent in [`admit-multi-input-tensors-in-the-scheduled-region-vocabulary`](admit-multi-input-tensors-in-the-scheduled-region-vocabulary.md). No other pin, golden, or fixture moved.

**Perturbations watched failing** (each applied, run, reverted):
1. `subnormal_freedom_of` returns the decode freedom for every scalar program → 3 metal tests fail, including `every_arithmetic_kernel_records_the_subnormal_gap`.
2. `SubnormalFreedom::discharges` returns true for every arithmetic type → `a_decode_freedom_discharges_f32_alone` fails on `F16`.
3. The new predicate's static assessment uses `is_finite` instead of `is_normal` → `a_subnormal_scale_is_disproved_under_its_own_code_not_the_finiteness_one` fails.
4. The reference scale validator uses `is_finite` instead of `is_normal` → 2 reference tests fail.
5. `record_subnormal_obligation` stops consulting the discharge → the decode is refused again, and the unstated-fact test fails too.
6. The `…-not-positive-finite` code renamed to order after `…-subnormal` → the negative-subnormal case reports the wrong cause.

**Contract corrections in `docs/numerical-semantics.md`.** The scale domain, both producers' declarations and `Dequantize`'s deliberate absence, the two codes and their ordering, the discharge as a general mechanism in the subnormal section, the reference boundary's new cases, the identity sentence, the Metal measurement paragraph (refusal → honoured, with the non-vacuity witness), and the proposal's remaining blockers.

**Not done here, deliberately.** No U4 packing, per-axis map, or contraction work. No tensor-payload enforcement. No dtype-ledger cell moved. The `f16`/`bf16`/`f64` arms of `discharges` are reservations, not implemented support — nothing emits arithmetic in them.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the five public items — `positive_normal_scalar_predicate`, `ENCODED_NUMERIC_SCALE_DOMAIN`, `SubnormalFreedom` with `discharges`, and the two derived accessors — with the deliberate identity exclusions (derived value never folded twice) noted. Recorded for Tom's morning review.

## Later public-boundary correction — 2026-08-09

The paragraph above is the latest acceptance provenance in this record; it is
not Tom's acceptance of the exact public surface. A full current-source audit
found six exported symbols in the five groupings it names:
`positive_normal_scalar_predicate`, `ENCODED_NUMERIC_SCALE_DOMAIN`, the
`SubnormalFreedom` enum, `SubnormalFreedom::discharges`,
`VerifiedScheduledRegion::subnormal_freedom`, and
`VerifiedKernel::subnormal_freedom`. Their current rustdoc carries no draft
marker. [`resolve-the-provisional-normal-scale-discharge-public-surface`](resolve-the-provisional-normal-scale-discharge-public-surface.md)
now owns the missing exact-boundary decision and, absent Tom's acceptance, the
restoration of draft labels. The numerical implementation and this ticket's
completed honourability outcome are unchanged.
