---
id: admit-a-multi-region-index-realization-law
title: Admit a multi-region index realization law
status: todo
priority: p1
dependencies: []
related: [lower-a-two-region-occurrence-through-one-index-access-capability, admit-the-rms-normalization-family, admit-the-softmax-family, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, lowering, normalization]
---
## User-visible outcome

An operation whose canonical realization needs more than one index region — a reduction producing a shared intermediate, then an elementwise pass consuming it — can carry an `IndexRealizationLaw`, be verified against an ordered region sequence, and mint a refinement receipt that binds every region. This is the authority a normalization or a softmax needs before any *capability* vocabulary for region sequences can mean anything.

## Why this is filed

Filed from the discovery stop on [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md), whose premise was that widening the compiler-side `IndexAccessLoweringProvider` would release `tiler::rms-norm-f32@1`. Measurement falsified that: the refusal arrives before the provider is driven, from `tiler-ir`. The evidence is `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs`, whose `refining_the_normalization_refuses_before_the_provider_is_driven` observes a driven-provider count of exactly zero.

**Fact.** `crates/tiler-ir/src/semantic/registry.rs` registers an `IndexRealizationLaw` for exactly nine operations, and the normalization and the softmax are deliberately absent. The comment above that list states the intent: absence "fails closed later".

**Fact.** `FrozenIndexRealizationLawRegistry::resolve` (`crates/tiler-ir/src/index/refinement.rs`) returns `IndexRefinementVerificationError::MissingRealizationLaw` for an operation with no law row, and `refine_index_region` (`crates/tiler-compiler/src/legality.rs`) calls `resolve` *before* `emit_region`. So no provider runs for a lawless family.

**Fact.** `IndexRealizationLaw::realize` (`crates/tiler-ir/src/index/law.rs`) builds one `IndexRegionBuilder` and returns one `VerifiedIndexRegion`. `ResolvedIndexRealization::verify` (`crates/tiler-ir/src/index/refinement.rs`) takes one `&VerifiedIndexRegion` and requires `expected.canonical_identity() == region.canonical_identity()`.

**Inference — the sequence vocabulary must exist here first.** Verification is an identity comparison against a law-reconstructed region. An ordered region sequence has no canonical identity for that comparison to consume, and `IndexRefinementReceipt` binds one region's operands and results. A capability that declared a region sequence today would therefore have nothing able to certify it: the declaration would be a type-system reservation wearing the shape of implemented support, which is the conflation the architectural contract forbids.

**Fact — the two-region shape exists at a different layer.** `KernelSubprogram` / `SubprogramStage` (`crates/tiler-compiler/src/frontier.rs`) is an ordered chain with an internal intermediate, and `derive_subprogram_boundary_contract` proves the chain well formed. That operates on `VerifiedScheduledRegion` (the physical/schedule IR), not on `tiler_ir::index::VerifiedIndexRegion` (the index-refinement IR). It is a model to mirror, not a mechanism to reuse — and conflating the two IRs is what made the original ticket look reachable from `implementation/compiler`.

## Closes when

1. `IndexRealizationLaw` can express an ordered sequence of canonical regions with a named intermediate between them, and the intermediate's shape, ownership, and lifetime are explicit contracts rather than implied by stage order.
2. `realize` and `verify` agree on a canonical identity for the *sequence*, so a candidate sequence is compared as a whole and a truncated or reordered one is refused with a typed reason.
3. `IndexRefinementReceipt` binds every region in the sequence, and the reached-scalar containment check covers the union of the sequence's regions rather than one region's.
4. The law encoding stays append-only per tag, with per-tag injectivity reasoning recorded at the encoding site — `IndexRealizationLaw::encode` already carries tags 1..=8 and the tag-8 comment is the precedent to follow.
5. A deliberate perturbation — a law declaring one region for a two-region occurrence, and a sequence whose intermediate is never read — each refuses with a typed reason rather than minting a receipt.

## Non-goals

Registering the normalization's or the softmax's own law, which belongs to the family tickets once this vocabulary exists; widening `select_supported_strategy`, owned by [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md); and any compiler-side capability surface, which is [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md)'s once unblocked.

## Decision boundary

The law enum, the sequence identity encoding, and the receipt's public shape are all `tiler-ir` public surface. A tested implementation is a draft; acceptance of the exact interface is Tom's, and reaches him as an `awaiting-decision` acceptance node carrying the surface and its evidence.
