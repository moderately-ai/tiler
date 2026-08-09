---
id: accept-the-softmax-realization-law
title: Accept the softmax realization law
status: done
priority: p2
dependencies: []
related: [register-the-softmax-realization-law, accept-the-root-mean-square-scale-realization-law, accept-the-governed-maximum-scalar-key, accept-the-multi-reader-index-realization-retention]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing, numerics]
---
## What is being accepted

One further variant of the public `#[non_exhaustive]` `IndexRealizationLaw`, landed as a labelled draft by [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

New in `tiler_ir::index`:

```rust
pub enum IndexRealizationLaw {
    // ... ten existing variants, none changed ...
    StagedSoftmaxF32 {
        axes_attribute: AttributeFieldId,
    },
}

impl IndexRealizationLaw {
    pub const fn staged_softmax_f32() -> Self;
}
```

Nothing else in the public surface moves. No existing variant's shape, payload, or encoding tag changes; the constructor is `const` because its one field is a compile-time attribute identifier. The new encoding tag is **11**, appended.

One registration moves with it: the standard semantic provider now registers this law for `tiler::softmax-f32@1`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for that family and `family_realizes_region_sequence` answers `true` for it.

## What is *excluded* from this surface

The three general emitters this work landed — the combiner-and-seeding-parameterized fold, the fold-with-epilogue region emitter, and the row-broadcasting stage emitter — are **private** to `crates/tiler-ir/src/index/law.rs`. They are the reusable machinery the next staged family instantiates, and they are deliberately not public: nothing outside this module builds an expected region, and exposing them would publish a region-construction API separate from the law vocabulary that owns it.

## The choices worth objecting to

- **A new variant rather than a widened staged template.** This is the second family whose chain the template *names* rather than *carries*, which is exactly the trade [`accept-the-root-mean-square-scale-realization-law`](accept-the-root-mean-square-scale-realization-law.md) accepted with the honest counter-argument attached: a second width, or a variant softmax, would need a third variant rather than a third row. The measurement that trade asked for is now available and is in the deriving ticket's Outcome — 62% of the two variants' emission is shared machinery, and the *scalar chain proper*, the part a scalar-program language inside a law would carry as data, is 46 of 697 code lines (6.6%). **Reopening the refusal of a scalar-program language is not proposed here**; the number is recorded so a future reopening argues from it.
- **The subtraction is spelled as an exact sign flip and one rounded add.** There is no subtraction scalar key, so `s_i - m` is emitted as `add(s_i, multiply(m, -1.0))`. Negating a binary32 value is exact, so this rounds exactly where the reference does — and `SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED` already names this adjacency as the operation's only multiply-add pair and withholds contraction over it. The alternative would be admitting a `subtract-f32` scalar key, which is a semantic surface of its own.
- **A zero-length reduced axis is refused** (`fold-empty-domain-without-identity`) rather than realized. The operation is shape-preserving, so its own semantics evaluate no scalar softmax there; this staged shape would still have to commit one row maximum per kept coordinate, and the pinned extrema family has no identity to commit. The cost is that a softmax over a zero-length axis has no realization at all, which is the fail-closed direction — it is the *shape rule*, not the fold, that makes the case unreachable in the operation's own semantics, and the law refuses rather than pretending otherwise.
- **The denominator's sum is also given no empty-domain identity**, though the governed addition has one. Both folds are seeded at the first contributor, which is what the reference pins for each; carrying `0.0` for the second would make the two folds disagree about the same axis.
- **The occurrence's attribute record must be exactly the one field the law names**, as the normalization's must be exactly two. Same reason: `reduction_axes` tolerates a record carrying more than it reads, and tolerance here is the silent-wrongness path.
- **One refusal has no watched perturbation.** `softmax-reduced-axis-rank` — a multi-axis reduced-axes sequence — is unreachable from a verified occurrence, because the family's own inferencer refuses an absent, duplicated, or second axis before a subject exists. It is stated anyway because a law is interpreted against a *subject* rather than against the inferencer that produced it. If a reviewer would rather have no unreachable check than an untested one, this is the thing to say so about.

## The identity consequence, and where it landed

Registering the law moves the count-prefixed law sidecar and therefore `FrozenIndexRealizationLawRegistry`'s identity, which moves the compiler's request-digest pin in `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`. That is the pin working rather than collateral damage: two requests built against different realization authorities are different requests. The semantic snapshot identity is computed without the sidecar and does not move, so every artifact and kernel-program identity derived from it is byte-identical.

## Evidence

The deriving ticket's Outcome carries: the four-stage chain and its sources, the three general emitters and the byte-for-byte-unchanged normalization identities that prove the refactor onto them was identity-neutral, the tag-11 injectivity reasoning, the step-for-step realization evidence read off the four verified regions, the watched-failing perturbations, the new softmax chain identity pins, and the shared-versus-per-family ratio.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the variant is in use inside `tiler-ir` and labelled a draft at its definition.

## Accepted — 2026-08-07

**Tom accepted this law on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, without exclusion — together with [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md) and the principle both nodes raised.

### What is accepted

`IndexRealizationLaw::StagedSoftmaxF32 { axes_attribute }` and its `const` constructor, at appended encoding **tag 11**. No existing variant's shape, payload, or tag moves. The standard semantic provider registers it for `tiler::softmax-f32@1`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` and `family_realizes_region_sequence` answers `true`.

The three general emitters stay **private** to `crates/tiler-ir/src/index/law.rs`. Nothing outside that module builds an expected region, and publishing them would mint a region-construction API separate from the law vocabulary that owns it.

**A new variant rather than a widened staged template** is accepted as the trade, with the measurement that trade asked for now on record: 62% of the two variants' emission is shared machinery, and the scalar chain proper — the part a scalar-program language inside a law would carry as data — is 46 of 697 code lines. **This does not reopen the refusal of a scalar-program language**; the number is recorded so a future reopening argues from it rather than from impression.

The subtraction spelled as an exact sign flip and one rounded add, the refusal of a zero-length reduced axis, and the denominator's fold seeded at the first contributor rather than at `0.0` are accepted with their stated grounds.

### The identity consequence is accepted as the pin working

Registering the law moves the count-prefixed law sidecar, hence `FrozenIndexRealizationLawRegistry`'s identity, hence the compiler's request-digest pin. The **semantic snapshot identity is computed without the sidecar and does not move**, so every artifact and kernel-program identity derived from it is byte-identical. Two requests built against different realization authorities are different requests, which is the pin doing its job rather than collateral damage.

### The principle, ruled on rather than the instance

`softmax-reduced-axis-rank` has no watched perturbation because it is **unreachable from a verified occurrence** — the family's own inferencer refuses an absent, duplicated, or second axis before a subject exists. Both nodes asked whether a reviewer would rather have no unreachable check than an untested one. **Tom ruled: unreachable-but-stated refusals stand, for realization laws.**

**The ground, and its fence.** A law is interpreted against a *subject*, not against the inferencer that produced it, so a hand-built or re-read subject can reach these rules even though no construction path does. That is a **reinterpretation boundary**, and it is what distinguishes these from a construction-path refusal — the mixed-width refusal proposed for the BF16 reference on the same day was **rejected** precisely because no constructible program could ever reach it, making it a maturity claim the evidence could not support. The two are not in tension: state a refusal a re-read subject can reach; do not state one nothing can.

Carried to where law authors will read it by [`state-the-unreachable-refusal-convention-where-law-authors-read-it`](state-the-unreachable-refusal-convention-where-law-authors-read-it.md), because a convention recorded only in two closed acceptance nodes is one the next law will not find.

## Current-state correction — 2026-08-09

That carrier is complete. `IndexRealizationLaw` now states the reinterpretation
boundary, its reachability fence, and the non-relaxation of watched reachable
refusals; the four named unreachable rules carry their local reasons. The
accepted softmax law and its identity consequence are unchanged.
