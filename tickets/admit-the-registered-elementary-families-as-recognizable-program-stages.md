---
id: admit-the-registered-elementary-families-as-recognizable-program-stages
title: Admit the registered elementary families as recognizable program stages
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: []
paths: []
tags: []
---
## User-visible outcome

A program whose middle stage is a registered elementary family — a softmax, an RMS normalization, or any family the registry carries with per-family facts — compiles through the ordinary path when that stage reads a materialized intermediate and feeds a later stage, exactly as elementwise epilogues and strict folds already do. The capability is the general one: any registered family with a realization law becomes a recognizable stage; no family-specific recognizer arm and no chain-shaped special case.

## Why this exists, and the worked instance

**Fact.** The multi-region realization law (accepted 2026-08-06) provides the staged template vocabulary, and its non-goals record that "registering the normalization's or the softmax's own law belongs to the family tickets once this vocabulary exists" — and no such family tickets were ever filed. The elementwise-epilogue, fold-write, and subset-read landings admitted every other stage shape; the registered elementary families are the remaining unrecognizable middles.

**The worked instance is the attention chain** (contraction → softmax → contraction): its three-member region already derives fusion legality, but the softmax cannot be a program stage because it has no registered index-realization law and no `NormalizedOutput` classification. The same wall holds rms-norm-after-anything and every future example program with an elementary middle — which is why this is one general capability, not an attention feature. Per the worked-examples discipline recorded in AGENTS.md: the example exercises the machinery; the capability lands general.

## Sequencing and the boundary inside it

- **The scalar admissions gate the laws, and they are semantic surfaces.** The softmax's law needs an `exp` scalar operation and rms-norm's needs `rsqrt`; `crates/tiler-ir/src/index/scalar.rs` registers ten governed keys and neither. **Corrected 2026-08-06 by [`re-read-the-bf16-and-elementary-support-rows-against-source`](re-read-the-bf16-and-elementary-support-rows-against-source.md), and only half of that sentence survives.** `exp_f32_scalar_op` is `crates/tiler-ir/src/index/scalar.rs:65` — the activation's landing put it there beside `divide_f32_scalar_op` at `:52` — so the softmax's *exponential* needs no new key. The ten keys are `constant`, `multiply`, `add`, `divide`, `exp`, and `canonicalize-nan` at `f32`, the strict-affine U4 dequantize, and the three `bf16` rows. What is genuinely absent is `rsqrt` for the normalization and a **maximum** for the softmax's shifting fold — a second missing key the sentence did not name, and one the softmax's registered definition pins to the NaN-propagating family with `-0.0 < +0.0` rather than leaving open. So the two families need different keys rather than sharing one gap, and the sequencing below should be read against that. A new scalar operation key is a public semantic surface — implemented as a labelled draft, acceptance node parked for Tom, per the standing convention.
- The law registrations then use the accepted staged template (or a single-region law where the family is one region); recognition follows as a `NormalizedOutput` classification derived from the registered law rather than a per-family arm.
- The two-region occurrence tests in `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` are the harness precedent; the wall test asserting `MissingRealizationLaw` for the normalization flips with the registration.

## Closes when

At least one registered elementary family compiles as a middle stage through the ordinary path with reference bit-agreement, the recognition is law-derived rather than family-cased, the scalar-admission surfaces are parked for Tom, and the attention chain's remaining refusal (if any) names a wall outside this ticket with an owner.
