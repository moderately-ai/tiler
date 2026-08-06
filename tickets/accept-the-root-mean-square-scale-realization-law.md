---
id: accept-the-root-mean-square-scale-realization-law
title: Accept the root-mean-square scale realization law
status: awaiting-decision
priority: p2
dependencies: []
related: [widen-the-staged-realization-law-to-the-registered-elementary-families, accept-the-multi-region-index-realization-surface, accept-the-governed-reciprocal-square-root-scalar-key]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
---
## What is being accepted

One further variant of the public `#[non_exhaustive]` `IndexRealizationLaw`, landed as a labelled draft by [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

New in `tiler_ir::index`:

```rust
pub enum IndexRealizationLaw {
    // ... nine existing variants, none changed ...
    StagedRootMeanSquareScaleF32 {
        axes_attribute: AttributeFieldId,
        eps_attribute: AttributeFieldId,
    },
}

impl IndexRealizationLaw {
    pub const fn staged_root_mean_square_scale_f32() -> Self;
}
```

Nothing else in the public surface moves. No existing variant's shape, payload, or encoding tag changes; the constructor is `const` because both fields are compile-time attribute identifiers.

One registration moves with it: the standard semantic provider now registers this law for `tiler::rms-norm-f32@1`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for that family.

## The choices worth objecting to

- **A new variant rather than a widened `StagedStrictSerialSumThenPointwiseF32`.** Widening the existing form would mutate an already-accepted variant's payload and therefore its tag-9 encoding, and it would admit field combinations that denote no program. The cost of a new variant is a tenth tag and a tenth arm in five matches.
- **The chain is fixed; only the two attribute identifiers are law data.** The prologue multiply, the mean division, the bias addition, the reciprocal square root, and the two scale multiplies are named by the template rather than carried in it. Carrying them would need a scalar-program language inside a law, which is the universal IR `law.rs`'s header refuses, or five independently settable keys whose combinations this module could no longer claim to interpret. The counter-argument, and the honest one: a second width's normalization would need a second variant rather than a second row, which is the opposite of the `constant_f32`/`constant_bf16` relationship. That trade is deliberate and is the thing to object to if it is wrong. It is currently unreachable in either direction, because there is no `bf16` division, reciprocal square root, or NaN canonicalization key.
- **The law refuses a folded extent no binary32 value equals** (`rms-scale-extent-not-exact`), rather than dividing by the nearest one. The reference divides by the extent; a rounded divisor is a different function. The cost is that a normalization over an axis whose extent is above 2^24 and not exactly representable has no realization at all rather than an approximate one.
- **The law refuses an empty fold** (`rms-scale-empty-fold`). A fold seeded at the first contributor has no first contributor over an empty axis, so the reference's own fold is undefined before the division by zero is reached.
- **The occurrence's attribute record must be exactly the two fields the law names.** This is stricter than every other variant, and deliberately so: `reduction_axes` tolerates a record carrying more than it reads, and the family's `eps` payload is part of its identity, so tolerance here is the silent-wrongness path. The cost is that an extension registering this law for an operation carrying a third attribute is refused rather than served.

## Evidence

The deriving ticket's Outcome carries: the elimination that produced this shape, the step-for-step realization evidence read off the verified regions, the tag-10 injectivity reasoning, the four watched refusals of the attribute check, three watched-failing perturbations of the step-for-step test, and the pin survey.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the variant is in use inside `tiler-ir` and labelled a draft at its definition.
