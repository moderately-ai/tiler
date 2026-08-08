---
id: admit-the-indirect-access-class-into-the-index-layer
title: Decide whether the index layer admits a data-dependent access class
status: done
priority: p2
dependencies: []
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, emit-the-indirect-gather-on-metal, implement-index-domain-predicates, revise-adr-0108-with-a-complete-data-dependent-index-vertical]
scopes: [implementation/ir, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, indexing, gather, class-generic-capability, needs-tom]
---
## User-visible outcome

The original research pass identified the explicit index-layer decision behind
indirect gather and drafted ADR 0108. The draft was returned for revision after an
independent source audit; the successor ticket now owns the corrected comparison.

## Corrected outcome — 2026-08-08

This ticket remains `done` because drafting a proposed decision was its completed
research outcome. It did **not** accept or admit an access class, and it no longer
satisfies backend emission. The live successor is
[`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](revise-adr-0108-with-a-complete-data-dependent-index-vertical.md).

The source audit preserved the useful diagnosis and rejected the proposed remedy:

- **Fact.** Gather is registered and reference-evaluated under
  [ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md),
  but no index form or scheduled access relation represents it. The current typed
  refusal remains correct.
- **Fact.** `IndexNode` has five forms, `IndexExprClass` has three members, and
  `IndexDomainUnknownReason` has three reasons. Those counts describe the current
  no-admission boundary and are pinned from their types.
- **False in the former outcome.** An access-level representation need not move
  old canonical bytes: `encode_region` already tags reads as `1` and writes as
  `2`, so a fresh tag `3` with a framed payload can be append-only.
- **Imprecise in the former outcome.** Seven proof functions zip coordinates and
  extents, but `IndexRegionBuilder::prepare_access` first enforces equal rank.
  Those zips do not silently choose an expression representation.
- **False in the former outcome.** The three unknown reasons do not promise
  closure, and a gather bound is not undecidable in principle. ADR 0107 permits
  static proof or named validation, and `decide_gather_index` is factored for a
  future host-side validator.
- **Incomplete in the former outcome.** A tensor-reading expression is a nested
  logical read. The proposal did not define its source bounds, reachability,
  `u32` semantics, proof subject, compaction, identity, authoring, reference, or
  compiler contract. `IndexNode` is private, so the public-boundary census also
  undercounted construction, view, error, and validation surfaces.
- **Circular in the former outcome.** Metal emission cannot trigger a decision
  and IR admission on which that emission already depends.

## Handoff

The successor compares a first-class verified nested read/value expression with
an append-only tagged access representation across the whole logical-to-physical
vertical. It may revise or defer ADR 0108, but it implements nothing. The current
5/3/3 boundary, ADR 0046's non-weakening requirement, and ADR 0107's semantic-only
admission remain authoritative meanwhile.
