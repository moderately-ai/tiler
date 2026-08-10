---
id: scope-the-index-producing-reduction-family
title: Scope the index-producing reduction family
status: deferred
priority: p3
dependencies: []
related: [admit-a-parallel-topology-for-the-identity-less-extrema-fold, scope-the-ordering-and-rank-selection-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, reductions, deferred]
---
## User-visible outcome

`ArgMin` and `ArgMax` have a stated tie-break and NaN policy and a decided result count, instead of being a reduction whose answer depends on which contributor a schedule happened to visit first.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-29 is atomic with one operand and one or two results, and is "the first family whose results are heterogeneously typed". Its D5 is the load-bearing part: "The tie-break and the NaN policy are semantic, not implementation detail; 'first minimal index wins' must be stated or the result is nondeterministic." Its D8 records why it cannot reuse the monoid reduction's permissions: "An identity-less fold has no seed, so it is not a monoid and cannot reuse F-28's topology permissions unchanged."

**Fact — `RQ-OP-07` owns the arity and states its closure test.** "Does an index-producing reduction return the index alone or the value and the index? Closes when the tie-breaking rule and NaN policy are stated, because those determine whether the value result is recoverable by a second gather without ambiguity. If it is recoverable, the one-result form is sufficient and the two-result form is a fusion concern."

**Fact — the deferral rests on an elimination.** The pinned language-model workload's argmax is on the *consumer* side: L6's logits contract states that the head program's single output is `logits` with no softmax, no scale, no temperature, no top-k or top-p, no vocabulary mask, no argmax, and no token — sampling and greedy selection stay outside the graph (a device-side argmax would need a max-with-index family that does not exist). The nearest in-graph candidate, the softmax's row maximum, is an extrema *value* fold and produces no index.

**Fact — an adjacent identity-less parallel permission is already delivered and is not this.** [`admit-a-parallel-topology-for-the-identity-less-extrema-fold`](admit-a-parallel-topology-for-the-identity-less-extrema-fold.md) is **done**: it delivered `EmptyDomainContract::NoIdentity` (non-empty domain proof for identity-less combines; not a carried `has_value` staged-partial). An index-producing fold is identity-less in the same way and would consume that empty-domain / identity-less combine permission rather than F-28 monoid topology permissions, which is why the two are related and why this track must not re-derive the empty-domain contract.

## Activation trigger

A named workload requires an in-graph index-producing reduction — a sampling step inside the program, a routing decision, or a consumer that cannot recover the index itself.

## What the work would be, when it starts

State the tie-break rule and the NaN policy first, because `RQ-OP-07`'s arity answer falls out of them: if the value is recoverable by a second gather without ambiguity, the one-result form suffices. Then the key, the reduced-axis attribute, the index result type as a coordinate type independent of the value type, the `(value, index)` carrying fold as the oracle, and the combine a parallel topology would need — which has no identity and therefore consumes the delivered `EmptyDomainContract::NoIdentity` / identity-less combine permission rather than F-28's monoid topology permissions.

## Explicit non-goals

- F-28's topology permissions, which do not transfer to an identity-less fold.
- A consumer-side argmax. If the selection happens outside the program it needs no family.
- Top-k, which is [`scope-the-ordering-and-rank-selection-families`](scope-the-ordering-and-rank-selection-families.md)'s: a rank selection at `k > 1` is a different physical problem.

## Closes when

The tie-break and NaN policy are stated, `RQ-OP-07`'s arity follows from them rather than being chosen beside them, and the family has a key, a carrying-fold oracle, and a parallel-combine story that names the contract it consumes.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-25** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-29 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires an in-graph index-producing reduction; the pinned workload samples its logits on the consumer side by the L6 record's own boundary. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** The language-model conformance work still keeps greedy selection outside the program, and no routing or sampling workload requires in-graph ArgMin/ArgMax. The embedded softmax maximum remains a value fold without an index result.
- 2026-08-10 — **not fired.** No workload requires an in-graph index-producing reduction; L6 still ends at logits with no argmax/token, and softmax remains a value Maximum fold. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 47 unique governed keys today; `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` — 19 operation `*_op` functions (the 2026-08-05 "eighteen" / "46" census is historical); none is an argmin/argmax or other index-producing reduction key.
