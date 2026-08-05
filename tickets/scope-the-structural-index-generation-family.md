---
id: scope-the-structural-index-generation-family
title: Scope the structural index-generation family
status: deferred
priority: p3
dependencies: []
related: [admit-a-position-selecting-slice-for-the-rotary-table, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, structural, indexing, deferred]
---
## User-visible outcome

A program that needs a materialized coordinate — a position index, a range, an identity matrix — has one governed way to say so, instead of a frontend baking the coordinates into a constant whose bytes are a shape in disguise.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-02 classifies structural index generation as atomic, zero operands and one result, with a static rank, one axis attribute selecting which coordinate is materialized, and a numerical rule that is a refusal rather than a rounding: exact for every coordinate representable in the result type, and "a coordinate that is not exactly representable is a refusal". Its D7 records that the physical route is an index expression, "which the [index model](../docs/research/indexing/index-access-model.md) already admits as affine", and that materializing an index a fused consumer would have computed "is a cost question, never a legality one".

**Fact — no key exists and no matrix row tracks it.** F-02 is one of the twenty-three families the taxonomy's join table lists under *(no matrix row today)*. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.

**Fact — the workload reaches its coordinates without one.** The pinned workload's rotary tables are bound as `cos`/`sin` program inputs and its causal mask as an additive `f32` input; neither is generated in the graph. The one place a generated coordinate would help is the decode step's rotary row selection, and that is owned as a *slice* at a bound cursor by [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md), which needs an `IndexNode` carrier rather than a generator.

**Inference.** The family is cheap and its physical route already exists, which makes it tempting to admit early — and admitting it early would fix an axis attribute, a result-type admissible set, and a representability refusal for a producer nobody has named. The cost of waiting is nil, because nothing composes around its absence.

## What the work would be, when it starts

State the exact result-type admissible set (the taxonomy narrows it below the catalog because the value is a coordinate), the axis attribute's canonical form, the representability refusal and its diagnostic, the host oracle, and the index expression the lowering emits — then check the claim that materialization is never a legality question by exhibiting one occurrence where a fused consumer computes the coordinate and one where the value is a program output.

## Explicit non-goals

- Any data-dependent range. A generated extent is F-36's problem and [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md)'s.
- The `IndexNode` carrier for a symbolic offset, which the slice family's symbolic half owns.
- An `eye`/`meshgrid` convenience surface: the taxonomy classifies those as frontend sugar and admitting one would put frontend convenience into durable graph identity.

## Closes when

A named producer requires a materialized coordinate, the five obligations above are stated together, and the representability refusal is watched firing — **or** the family is recorded as permanently frontend-resolved with the reason.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-14** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-02 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named producer requires a materialized coordinate: the workload binds its rotary tables and its mask as program inputs, and the one candidate occurrence is owned as a slice rather than a generator. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
