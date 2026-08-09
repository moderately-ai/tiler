---
id: scope-the-monoid-reducers-beyond-the-strict-sum
title: Scope the monoid reducers beyond the strict sum
status: deferred
priority: p2
dependencies: []
related: [implement-parallel-reduction-strategies, scope-the-standalone-extrema-and-clamp-families, scope-the-predicate-tensor-vertical, scope-the-index-producing-reduction-family, admit-a-parallel-topology-for-the-identity-less-extrema-fold, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, reductions, deferred]
---
## User-visible outcome

`Product`, a standalone extrema reduction, a non-identity seed, and the variadic multi-input form each have a scoped answer, so that "reductions beyond strict sum" stops being one matrix row listing four semantic families and two physical topologies together.

## Why this is deferred rather than open, and what this track is *not*

**Fact — the reduction schema is already fixed and this track instantiates it rather than designing it.** [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md) fixes `Reduction { input, axes, reducer, input_conversion, accumulator_dtype, initial, empty_without_initial, result_conversion, result_dtype, result_value_policy, order, determinism }`, the canonical axis set as unique, sorted, in range, and nonempty, `keepdims` as frontend sugar, and the reassociation-by-permutation legality table. It is `adopted_by` ADRs 0012, 0013, 0014, 0022, and 0025.

**Fact — exactly one reducer is registered standalone.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s reductions row records `tiler::strict-serial-sum-f32@1` as "the only *standalone* registered reduction", and records twice that its own scope narrowed without its rung moving — the normalization's prologue-carrying sum and the softmax's embedded extrema fold are both *embedded* forms. "Every entry in this row's own list — product, logical `any` and `all`, extrema reductions, a non-identity seed, and tree topologies — is still unregistered."

**Fact — two of that row's five members do not belong to this track, and separating them is half of why this ticket exists.** Logical `any` and `all` are gated on the predicate decision: [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-03` blocks "the logical-reduction case of F-28" along with four whole families, and [`scope-the-predicate-tensor-vertical`](scope-the-predicate-tensor-vertical.md) owns it. Tree and multi-pass *topologies* are physical, not semantic: cross-family invariant 3 states that "reduction topology is physical, never semantic" under [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md), and [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) owns them.

**Fact — one question in this track is stated and unowned.** `RQ-OP-06` asks whether the reduction family admits the variadic multi-input, multi-result form, blocks F-28, and notes that F-29 depends on the answer. Its closure test: "a workload needing two reductions over one traversal where fusing them post hoc is provably harder than expressing them together. Absent that, the single-input form stands, because the variadic form's cost is that every reducer becomes a tuple contract." It also warns that adopting StableHLO's *shape* does not adopt its implementation-defined ordering, which the corpus rejects on independent grounds.

## Activation trigger

A named workload requires a product reduction, a standalone extrema reduction, a seeded reduction whose initial value is not the reducer's identity, or two reductions over one traversal.

## What the work would be, when it starts

Per reducer: instantiate the fixed schema, which means stating the identity and empty-domain behaviour, the contributor order, the NaN and signed-zero policy, and the reassociation and permutation permissions independently — and for a non-identity seed, the rule that makes `empty_without_initial` meaningful, since the delivered extrema scalar program carries no `empty_identity_bits` field at all because no binary32 value is an identity for `Maximum`. Then answer `RQ-OP-06` against its own test, recording that the variadic *shape* may be adopted without the ordering model that comes with it in the source.

## Explicit non-goals

- Logical `any` and `all`, which are the predicate track's.
- Tree, split, and multi-pass topologies, which are physical and owned by [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md).
- The standalone elementwise extrema operations, which are [`scope-the-standalone-extrema-and-clamp-families`](scope-the-standalone-extrema-and-clamp-families.md)'s; this track carries only the reduction form, and admitting one does not admit the other.
- An arbitrary binary region as the reducer, which the taxonomy states is not the extension mechanism.

## Closes when

Each undelivered reducer has an instantiated schema with its identity, order, and permissions stated, and `RQ-OP-06` is answered against a workload or explicitly left standing with the single-input form as the recorded default.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-39** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers the undelivered reducers of F-28 — product, standalone extrema, non-identity seed, and the variadic form and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires a product, a standalone extrema reduction, a non-identity seed, or two reductions over one traversal; the pinned workload's reductions are the contraction fold, the normalization's squared sum, and the softmax's two embedded folds, each inside a registered composite key. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** The registered composite/staged families have widened, but no named workload asks for a standalone product, extrema reducer, non-identity seed, or two reducers over one traversal. Embedded folds and additional physical tree topologies do not constitute a standalone semantic reducer admission.
