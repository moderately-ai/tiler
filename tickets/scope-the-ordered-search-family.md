---
id: scope-the-ordered-search-family
title: Scope the ordered search family
status: deferred
priority: p3
dependencies: []
related: [scope-the-ordering-and-rank-selection-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, search, deferred]
---
## User-visible outcome

A binary search over sorted data is a family whose sortedness precondition is a *declared and validated* value assumption, not an undocumented expectation that turns a wrong answer into a plausible one.

## Why this is deferred rather than open, and why it is not grouped with sorting

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-39 is atomic, two operands and one result, data and needle of one identical type with an index result, one side attribute selecting left or right insertion, and one D5 sentence that separates it from everything else in its group: it "requires the data to be sorted, which is a **value assumption** rather than a shape constraint". Its D8 routes that assumption: "A sortedness assumption declared but unproved falls under [ADR 0021](../docs/decisions/0021-validated-value-assumptions.md) and needs proof or runtime validation."

**Inference — that is why it is not in the ordering track.** Sort and top-k *produce* an order and owe a total order and a tie-break; this family *consumes* an order it cannot see and owes a validated precondition. The implementations differ too — a sort, versus a binary search parallel over the needles — so grouping them would put one track's correctness argument on two families and one implementation on two physical shapes.

**Inference — the failure mode is the reason to state the assumption rather than assume it.** A binary search over unsorted data returns an index, in range, of the right type, for every input. Nothing downstream can tell. That is the same silent-wrongness shape the corpus concentrates scrutiny on, and it is why this family's deliverable is the precondition rather than the search.

## Activation trigger

A named workload requires an ordered search — a bucketing step, an interpolation table lookup, or a quantile evaluation.

## What the work would be, when it starts

The key, the identical-type data and needle admissible set, the index result type, the side attribute, the binary-search oracle, and the sortedness assumption expressed as an ADR 0021 value assumption with its two discharge routes stated — proved where the producer is a sort the compiler can see, and a typed host-side pre-dispatch validation where it is not. State the total order the assumption is *about*, which must be the one the ordering track selects, so the two families cannot disagree about what sorted means.

## Explicit non-goals

- Sorting itself, which is [`scope-the-ordering-and-rank-selection-families`](scope-the-ordering-and-rank-selection-families.md)'s.
- A search that silently defines a result for unsorted data. That is the failure this family exists to prevent.
- Interpolation on top of the search, which is a separate arithmetic family.

## Closes when

The family has a key, a binary-search oracle, and a sortedness precondition routed through ADR 0021 with both discharge routes stated and the refusal watched firing — or is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-32** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-39 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires an ordered search. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
