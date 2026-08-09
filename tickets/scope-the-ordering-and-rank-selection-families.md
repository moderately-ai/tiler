---
id: scope-the-ordering-and-rank-selection-families
title: Scope the ordering and rank-selection families
status: deferred
priority: p3
dependencies: []
related: [scope-the-data-dependent-extent-representation, scope-the-index-producing-reduction-family, scope-the-ordered-search-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, sorting, deferred]
---
## User-visible outcome

Sort, argsort, and top-k have a stated float total order and a stated tie-break, so that two implementations of one program cannot disagree about where the NaNs and the signed zeros went.

## Why this is deferred rather than open, and why these two are one track

**Fact — the sources agree that they do not define the order.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) records that Array API 2025.12 states of `sort` and `argsort` that "For floating-point input arrays, the sort order of NaNs and signed zeros is unspecified and thus implementation-dependent", and that StableHLO's `sort` takes a caller-supplied `comparator`, pushing the whole question to the caller. "Neither source supplies an order Tiler could adopt; one declines to specify and the other delegates."

**Fact — the proposed resolution is a reuse, and it is a proposal.** The taxonomy suggests reusing [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md)'s deterministic `-0.0 < +0.0` ordering "so that two families do not carry two float orders", and marks it explicitly: "That reuse is a proposal, and `RQ-OP-11` is where it would be tested rather than assumed."

**Fact — `RQ-OP-11` is a structural question, not a preference.** A caller-supplied comparator "would be the first region-bearing operation in the graph". [IR](../docs/ir.md) states the initial compilation unit "has no semantic functions/calls, recursion, region-bearing control flow, data-dependent branches, or semantic loops". The taxonomy's own reading is that "the fixed-order candidate should be assumed to win unless a workload refutes it".

**Inference — F-37 and F-38 are one track because one implementation covers them.** F-38's own D7 says its physical fallback is "a full sort followed by a slice", so the sort's realization contains the selection's; both owe the same float total order, both owe a tie-break, and both produce an index result whose type is independent of the value's. What F-38 adds is a `k` — an attribute when static and an operand when symbolic — and the symbolic case belongs to [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md) rather than here.

## Activation trigger

A named workload requires an in-graph sort, argsort, or top-k. Consumer-side top-k sampling does not fire it, for the same reason consumer-side argmax does not: the L6 record fixes logits as a consumer-sampled output.

## What the work would be, when it starts

Answer `RQ-OP-11` first, because it decides the shape of everything else: fixed stated order, or a caller-supplied comparator region, priced against a workload needing a composite key and against what a nested region costs canonical identity, verification, and the extension seam. Then state the float total order — reusing ADR 0023's rather than minting a second, if the test supports it — the stability flag, the tie-break among equal values, the `k`-exceeds-extent rule as a refusal or a defined shorter result rather than a silent choice, and the sort-then-slice baseline realization.

## Explicit non-goals

- A comparator region admitted without pricing it. That is the whole of `RQ-OP-11`.
- A second float order. If ADR 0023's does not fit, say why rather than adding one.
- A symbolic `k`, which needs the data-dependent extent representation.
- Consumer-side selection, which needs no family at all.

## Closes when

`RQ-OP-11` is answered, one float total order covers both families, the tie-break and the `k`-overflow rule are stated, and a sort-then-slice baseline exists — or the group is recorded as consumer-owned with the derivation.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-31** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-37 and F-38 and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires an in-graph sort, argsort, or top-k; the pinned workload's sampling happens outside the program by the L6 record's own boundary. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** Model-level greedy and top-k inspection remain consumer-side observables, not graph operations. No in-graph sort, argsort, or top-k workload has appeared, so neither the fixed-order nor comparator-region candidate has a named consumer.
