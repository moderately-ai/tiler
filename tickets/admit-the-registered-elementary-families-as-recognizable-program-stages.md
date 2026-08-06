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

- **The scalar admissions gate the laws, and they are semantic surfaces.** The softmax's law needs an `exp` scalar operation and rms-norm's needs `rsqrt`; `crates/tiler-ir/src/index/scalar.rs` registers ten governed keys and neither. A new scalar operation key is a public semantic surface — implemented as a labelled draft, acceptance node parked for Tom, per the standing convention.
- The law registrations then use the accepted staged template (or a single-region law where the family is one region); recognition follows as a `NormalizedOutput` classification derived from the registered law rather than a per-family arm.
- The two-region occurrence tests in `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` are the harness precedent; the wall test asserting `MissingRealizationLaw` for the normalization flips with the registration.

## Closes when

At least one registered elementary family compiles as a middle stage through the ordinary path with reference bit-agreement, the recognition is law-derived rather than family-cased, the scalar-admission surfaces are parked for Tom, and the attention chain's remaining refusal (if any) names a wall outside this ticket with an owner.
