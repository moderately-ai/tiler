---
id: admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode
title: Admit a normal-scale precondition so strict-affine decode is target honourable
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile]
related: [implement-first-runtime-semantic-value-precondition-enforcement, produce-typed-strict-affine-quantize-semantic-preconditions, implement-first-quantized-backend-profile, admit-strict-affine-quantize-physical-candidate]
scopes: [implementation/ir, implementation/reference, implementation/metal, contracts/numerics]
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
- Constant scales prove statically through the existing `StandardConstantF32BitsV1` proof basis; runtime scales become residual obligations that [`implement-first-runtime-semantic-value-precondition-enforcement`](implement-first-runtime-semantic-value-precondition-enforcement.md) enforces. Do not enforce a tensor value here.
- Nothing about U4 packing, per-axis maps, or a contraction belongs in this ticket. It moves one predicate and the refusal that depends on it.

## Closes when

The strict-affine decode passes the Metal honourability boundary for a normal scale and is refused for a subnormal one, both demonstrated — the subnormal case watched failing before the change is relied on; the two scale-domain diagnostics are distinct and each was observed firing; the static proof path and the residual-obligation path are both exercised; the derivation above is recorded at the site it governs rather than only in the research record; targeted package tests and Clippy pass; `tkt lint` and `git diff --check` pass; and one `make full` passes.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) from [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md). It is the first ticket in that record's delivery order because every later one assumes the target can honour the decode.
- Advancing this does **not** make a quantized program executable. Integer arithmetic has never been measured on an Apple GPU in this repository; [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) owns that and is a separate prerequisite of execution.
- Move no cell of [the dtype support ledger](../docs/dtype-support.md) for the U4 profile beyond what this ticket actually tests.
