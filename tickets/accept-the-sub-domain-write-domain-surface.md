---
id: accept-the-sub-domain-write-domain-surface
title: Accept the sub-domain write-domain surface
status: done
priority: p1
dependencies: [admit-sub-range-write-domains-for-unequal-partitions]
related: [lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir]
---
## What is being accepted

[`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) landed a public surface as a **draft**. It is tested and its evidence is in that ticket's Outcome; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it.

Two of the three items are behaviour changes to an existing accepted surface rather than new names, which is the reason this node exists at all: nothing about a relaxed refusal announces itself in a signature.

## The exact surface

In `tiler_ir::index`:

- **`IndexRegionBuilder::write`'s admitted domain widens from equality to subset.** `domain` may be any subset of the region's parallel dimensions, where it previously had to be all of them. No signature moves. Every existing caller is unaffected because the full set is a subset of itself — verified by the workspace suite, in which every `context.write(...)` site in `tiler-compiler` passes the complete parallel vector and none changed.
- **`IndexBuildError::InvalidWriteDomain` keeps its name and loses half its meaning.** It now refuses only a write domain naming a non-parallel dimension. A caller matching on it will still compile and will now see it for strictly fewer inputs.
- **`IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain { access: TensorAccessId, value: ScalarValueId, dimension: DimensionId }`** — additive under that enum's existing `#[non_exhaustive]`. Raised when an output root's stored value varies along a parallel dimension the root's write does not iterate.

## The choices worth objecting to

- **Subset of the parallel dimensions, rather than a sub-*range* annotation on an access.** The deriving ticket named both as candidates. The subset construct reuses the dimension vocabulary that already exists and adds no field to `AccessData`, so no canonical identity moves; the sub-range construct adds a per-access range annotation, which enters `encode_region` and forces an identity-domain step, and it duplicates what a fresh dimension plus an offset coordinate already spells. The full elimination is in the deriving ticket's Outcome.
- **A new diagnostic rather than reusing `FreeReductionDimension`.** The two are the same defect seen from two roles, and folding them would report every unreduced value under a name about write domains. They are kept apart deliberately, and the arm order that keeps them exclusive is load-bearing — reversing it was watched failing.
- **The obligation living at verification rather than at `output`.** It is decidable at `output`, where `OutputTypeMismatch` is. It sits at verification instead so it joins the diagnostic set a caller gets in one build, beside `FreeReductionDimension`, which is its sibling and is also decidable early.
- **`UnusedDomainDimension` now also fires for a parallel dimension nothing iterates.** This refuses a region that was previously buildable in one narrow case — a parallel dimension declared after the write that would have had to name it. Compaction retains every declared dimension, so leaving it would put a dimension in the canonical identity of a region whose meaning does not include it.

## Evidence

The deriving ticket's Outcome carries: the construct elimination; the re-derivation of both partition obligations (per-root injectivity and the rectangle-volume identity) showing each quantifies over the root's own domain and never over the parallel set; the pin survey (no identity moved, gate green on the whole workspace); and six watched-failing perturbations covering every new admitting and refusing path.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the surface is in use and labelled a draft at its declaration site.

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.
