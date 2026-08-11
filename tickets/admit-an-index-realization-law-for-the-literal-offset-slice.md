---
id: admit-an-index-realization-law-for-the-literal-offset-slice
title: Admit an index realization law for the literal-offset slice
status: todo
priority: p2
dependencies: [lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability]
related: [lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, slice, identity]
---
## User-visible outcome

The literal-offset `tiler::slice-f32@1` occurrence has a registered `IndexRealizationLaw` that independently reconstructs its exact one-region access relation, so refinement can compare a provider's emitted region with semantic authority rather than refusing `MissingRealizationLaw`.

## Why this exists

**Fact — exposed 2026-08-11 at `099c6e2d`.** [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) registered the exact unary-F32 capability and drove its real provider to a structurally verified region, but `refine_index_region` refused that same occurrence as `IrVerifier(MissingRealizationLaw)` before comparing the provider output with an expected realization. The source-safe query anchor `None for an operation the registry carries no law for` documents that fail-closed behavior, and `family_realization_law(&slice_f32_op())` returns `None` at this base.

**Fact — this remainder is identity-bearing, but the exact version consequence is not permission to move every domain.** `FrozenIndexRealizationLawRegistry::from_semantic` builds its canonical identity from the semantic and scalar snapshots plus the count-prefixed `encode_index_realization_law_sidecar`. `IndexRealizationLaw` is a public, `#[non_exhaustive]` typed vocabulary whose `realize`, numerical-contract, sequence-shape, and `encode` matches enumerate every current variant. The standard registry registers every first law for an operation at law-row revision `1`, and the current law encoder uses append-only tags through `12` under `tiler.ir.index-realization-law-registry.v1`. Adding a slice law therefore adds one standard sidecar row and changes the complete frozen law-registry identity (and every derived pin that retains it), while existing law-row bytes, the semantic-registry snapshot, and the `v1` domain can remain unchanged only if the new spelling is proved append-only. A new law variant is also additive growth of an existing public `#[non_exhaustive]` vocabulary; this ticket records that exact public delta for the coordinator to classify under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), but does not accept its included/excluded surface or authorize a merge.

## What the work is

- Re-derive the exact literal slice law from the complete semantic selection grammar and the independent compiler provider; decide the smallest law variant and constructor that state `WholeAxis -> d` and `Window { offset, .. } -> d + offset` with a total relation match.
- Register that law for `slice_f32_op()` in the semantic sidecar and update every exhaustive interpretation, encoding, identity census, and public-boundary label the new variant reaches. Derive whether the first slice row is revision `1` and an append-only next tag under the existing `v1` domain; preserve every old row byte if so, and otherwise stop for an explicit domain/version decision rather than silently stepping it. Recompute every standard-law-registry consumer and pin rather than assuming the blast radius.
- Perturb the law and provider independently by dropping a nonzero offset, with the other side unchanged, and quote the refinement mismatch in both directions. Prove an exact match refines.
- Record the exact ADR 0075/current-working-contract classification and any required public acceptance carrier instead of treating the ticket, a tested draft, or a `#[non_exhaustive]` marker as acceptance authority.

## Explicit non-goals

- The compiler-local capability and provider, owned by the dependency.
- Strided or source-bearing offsets, scheduled-region vocabulary, `VerifiedKernel`, view-versus-copy planning, or backend work.
- Reusing `Reindex` by erasing the selection attribute's distinct identity or admission rules.

## Stop conditions

- Stop for Tom if the law variant's public included/excluded surface has more than one defensible shape after source derivation.
- Stop and split if the exact law needs any `IndexNode`, access, ownership, schema, or semantic slice widening beyond the admitted literal grammar.

## Closes when

The exact literal-offset slice law is registered and identity-coherent; matching provider output refines; independent dropped-offset perturbations fail with quoted mismatches; the public draft/acceptance status is stated accurately; and the operation-family delivery graph can mark O-06 M5 fully delivered without implying scheduled-region or physical feasibility.
