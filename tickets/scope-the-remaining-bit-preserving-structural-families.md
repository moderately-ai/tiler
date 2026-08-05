---
id: scope-the-remaining-bit-preserving-structural-families
title: Scope the remaining bit-preserving structural families
status: deferred
priority: p3
dependencies: []
related: [reach-a-verified-kernel-through-the-structural-families, admit-the-structural-families-into-the-scheduled-region-vocabulary, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, structural, deferred]
---
## User-visible outcome

The identity copy and the axis-uniform repetition reach the same governed treatment the reindex and broadcast families already have, so that "structural and data-movement families" stops naming two delivered families and two that exist only as words in a matrix row.

## Why this is deferred rather than open, and why these two are one track

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) classifies F-03 identity and bit-preserving copy as "atomic with a declared decomposition capability" — result type and shape equal to the operand exactly, "bit-preserving by definition, including NaN payloads and signed zero", reference is the identity, "physical fallback is a copy, and eliding it is an optimization with a written-ownership obligation" — and F-27 repetition and tiling as atomic, per-axis repeat counts, no numerical content, "reference reads a modular map; physical fallback is a read map, so like F-23 it need not materialize".

**Inference — they are one track under this ticket's own splitting rule.** The rule splits on numerical contract, compound storage, effects, or backend feasibility. These two share every one: no numerical content at all, no storage question, `Pure`, and a realization that is a read map or a copy. They share an implementation with the delivered families too — an identity is a `Reindex` whose mapping form is the empty permutation, and a repetition is a many-to-one read map of exactly the shape `tiler::broadcast-f32@1` already emits. Splitting them would answer bit preservation, write ownership, and read-map lowering twice.

**Fact — the matrix row already names them and holds them at R2.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Structural and data-movement families` row reads "R5 for the two admitted families; views and bit-preserving copies stay R2", and closes "Views and bit-preserving copies have no key and no contract and stay at R2". Repetition appears in no row at all, so the row's title is narrower than its name suggests. The record this ticket was filed by corrects that reading rather than the rung.

**Fact — one member of that row is not a semantic family at all.** A *view* is a physical realization of a selection or a copy, not an operation; the delivery graph records it as a physical-candidate choice, and this ticket does not carry it.

## Activation trigger

A named producer emits an explicit identity — a materialization barrier, a written-ownership boundary, or an extension-typed copy that must not canonicalize its payload — **or** a workload needs axis-uniform repetition that broadcasting cannot express because the repeated axis is not extent-one.

## What the work would be, when it starts

For each family: the canonical attribute set (none for the identity, per-axis repeat counts for the repetition), the derivation of the result shape from the operand rather than a declared one, a bit-preserving evaluator that deliberately does not apply the arithmetic NaN canonicalization — the rule `StandardReferenceProvider` already follows for the structural families — the read-map or copy lowering, and the elision rule with its written-ownership obligation. Then state the boundary the taxonomy makes load-bearing: a per-element `repeat` driven by a tensor of counts is a data-dependent extent and belongs to F-36, not here.

## Explicit non-goals

- Views, which are physical and have no semantic identity to admit.
- Per-element repetition, which is [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md)'s.
- Any relaxation of the reindex family's refusals; a repetition is a new relation, never a seventh mapping form of a bijection.

## Closes when

Both families have a stated attribute schema, a bit-preserving oracle, a read-map or copy lowering with its ownership obligation, and a conformance corpus over exceptional payloads — or one of them is recorded as permanently subsumed by an admitted family, with the subsumption proved rather than asserted.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-15** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-03 and F-27 and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named producer needs an explicit identity or an axis-uniform repetition: the workload's structural occurrences are the rotary split, reverse, merge, and the two table broadcasts, all of which the two admitted families already express, and `crates/tiler-reference/tests/rotary_position_embedding.rs` builds that composition over them. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
