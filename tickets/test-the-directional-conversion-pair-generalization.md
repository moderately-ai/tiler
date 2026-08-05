---
id: test-the-directional-conversion-pair-generalization
title: Test the directional conversion-pair generalization on a second pair
status: in-progress
priority: p2
dependencies: []
related: [scope-the-in-type-precision-reduction-family, conform-the-bf16-vertical-end-to-end, carry-bf16-through-the-artifact-encoding-and-identity, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, conversion, dtype]
claimed_from: todo
assignee: agent-conversion-pair
lease_expires_at: 1785963860
---
## User-visible outcome

`RQ-OP-04` is answered against evidence rather than by analogy: either every conversion pair decomposes into two directional families with disjoint field sets, or one keyed family parameterized by source, destination, and mode is correct at scale — and the corpus stops carrying an `n²`-growth question it has never tested.

## Why this is dispatchable now rather than deferred

**Fact — the question, and its falsifiable test.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-04` asks whether [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)'s directional-pair decision "generalize[s] to every conversion pair, or is one keyed family parameterized by source, destination, and mode correct at scale?", blocks F-18 and F-19, and fixes the test: "Closes when a second pair is examined field by field. The test is falsifiable: if a second directional pair's field set is *not* disjoint in the way BF16/binary32's is, the generalization is refuted and the parameterized form wins. The `n²` growth is a cost to be stated, not the deciding argument."

**Fact — nothing about the test needs a workload, a target, or a measurement.** ADR 0091 is accepted and states the derivation to be generalized: narrowing owes a rounding rule, an overflow rule, and a subnormal rule that widening does not, and widening owes an exactness claim that narrowing cannot make. [ADR 0041](../docs/decisions/0041-separate-float-to-integer-conversion-families.md) independently accepts four float-to-integer families differing in exactly three fields. A second pair — the float-to-integer directions, or an IEEE binary pair — can be laid out field by field against those two accepted decisions today.

**Fact — the answer is close to load-bearing rather than academic.** BF16 reached R4 on 2026-08-01 with three registered keys and an exact-rational oracle, and ADR 0091's two conversion families are "registered in neither direction". [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s cast-and-convert row states the forcing condition in its own words: "Admitting any second dtype into a profile forces this row, because a mixed-dtype program cannot be expressed without an explicit conversion operation and no implicit promotion exists after semantic admission." A second dtype is admitted at the semantic and reference layers now.

**Inference — deciding the decomposition *after* the first conversion key is registered would be deciding it by accident.** Whichever shape the first registered conversion takes becomes the precedent every later pair is read against, and migrating it later migrates every identity that named it — the same cost ADR 0087 recorded when it chose one keyed family for the contraction.

## What the work is

Pick a second pair and walk its fields against ADR 0091's: what narrowing owes, what widening owes, and whether the two field sets are disjoint in the same way. Record the `n²` growth as a stated cost rather than as the argument. Then answer, and state the consequence for the *four* accepted float-to-integer families, which are already a directional decomposition of one logical conversion and therefore evidence on one side of the question rather than a neutral case.

## Explicit non-goals

- Registering any conversion key or choosing a Rust spelling. Both are Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md); this ticket produces the decomposition answer and the derivation behind it.
- Reopening ADR 0091 or ADR 0041. Both are accepted; this tests whether the first *generalizes*, and a refutation narrows its scope rather than superseding it.
- In-type precision reduction, which is not a conversion at all — [`scope-the-in-type-precision-reduction-family`](scope-the-in-type-precision-reduction-family.md) owns it, and the taxonomy is explicit that its result type never changes.
- Bit reinterpretation, which changes no numeric value and is a different question entirely.

## Closes when

`RQ-OP-04` is answered with the second pair's field-by-field derivation recorded, the `n²` cost is stated rather than assumed decisive, and the taxonomy's `RQ-OP-04` row names the answer — or the examination shows the two candidates are indistinguishable on this pair and names the third pair that would separate them.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-22** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), covering F-18 and F-19 together because a float-to-integer conversion is a directional pair under ADR 0041 exactly as a float-to-float conversion is under ADR 0091, and the question is whether that shape is the general one.
- Filed at `todo` rather than `deferred` deliberately: unlike every other track this record filed, its closure test names no workload, no target, and no measurement, so its trigger is already satisfied by the two accepted decisions it compares.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s cast-and-convert row is the delivery ledger and this ticket moves no rung.
