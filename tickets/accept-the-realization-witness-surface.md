---
id: accept-the-realization-witness-surface
title: Accept the realization witness surface
status: done
priority: p2
dependencies: []
related: [enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, derive-the-oracle-for-a-permitted-divergence-candidate, accept-the-reference-conformance-threading-surface]
scopes: []
shared_scopes: []
paths: []
tags: [tiler-research, numerics, reference, conformance]
---
## User-visible outcome

Tom decides, or declines, the public surface a permissive conformance oracle needs: one witness type, one refusal enum, and one additional constructor on `ReferenceNumericalConformance`. Nothing is released on it until he does.

## Why this exists

**Fact.** [The freedom-sites enumeration](../docs/research/reference/plan-freedom-sites.md) Part 7 drafts the exact surface with its evidence. Under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) a witness type, any change to `ReferenceNumericalConformance`'s construction, and any new plan-side field are each a public boundary, so the record drafts and parks rather than adopting.

**Fact — the acceptance is not one question but three, and they are separable.** Item A (`RealizationWitness` in `tiler_ir::schedule`, aggregated by `RealizationWitness::of(&VerifiedScheduledRegion)`) and Item B (`UnpinnedFreedomSite`, a refusal enum with no `Conforms`-shaped arm) stand alone. Item C (`ReferenceNumericalConformance::from_witness`) carries a consequence the record names explicitly: it would make `tiler-reference` name a plan structure for the first time. Today `grep -rn "tiler_ir::schedule" crates/tiler-reference/src --include='*.rs'` returns three lines, all behaviour vocabularies and no plan structure, and `tiler-compiler` depends on `tiler-reference` only as a dev-dependency — so the two sides can only meet in `tiler-ir`.

**Fact — an alternative that avoids the dependency is drafted beside it.** Keep the reference's arguments plain scalars, as `strict_partitioned_sum_under` already does, and site the aggregation alone in `tiler_ir::schedule`. Cheaper on dependency direction, more verbose at every call site, and it cannot carry the pointwise expression at all.

## What this ticket must produce

Tom's decision, recorded with who accepted, the date, and the venue. The shapes discipline admits: acceptance of A and B with C excluded behind its own ticket; acceptance of all three; or a redirection to the plain-scalar alternative.

## Explicit non-goals

Implementing any of it; deciding the `PointwiseF32Expression` evaluability fork, which is its own ticket.

## Closes when

Tom has accepted, excluded, or redirected each of the three items by name, and the record's Part 7 states which.

## Graph maintenance

Filed by the freedom-sites enumeration as the public boundary it was forbidden to self-accept.

## Decided 2026-08-06 — A and B accepted; C redirected to the plain-scalar form

**Tom decided at the live session's decision round, relayed and executed by the coordinator:** Item A (`RealizationWitness` in `tiler_ir::schedule`) and Item B (`UnpinnedFreedomSite`, no `Conforms`-shaped arm) are accepted as drafted. Item C is **redirected to the plain-scalar alternative**: the reference keeps taking plain scalars (as `strict_partitioned_sum_under` does) and the aggregation sites in `tiler_ir` alone — `tiler-reference` never names a plan structure. The ground: the evaluability fork's resolution (the reference evaluates the retained semantic program) removed the plain-scalar form's only stated disqualifier, so the `from_witness` dependency-direction commitment would be taken for a convenience the resolved fork no longer requires. Implementation is [`implement-the-realization-witness-vocabulary`](implement-the-realization-witness-vocabulary.md).
