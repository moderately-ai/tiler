---
id: carry-the-parametric-broadcast-relation-through-index-and-schedule-ir
title: Carry the parametric broadcast relation through index and schedule IR
status: in-progress
priority: p1
dependencies: [replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics]
related: []
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, scheduling, broadcast, identity]
claimed_from: todo
assignee: worker-parametric-broadcast
lease_expires_at: 1786640549
---
# Carry the parametric broadcast relation through index and schedule IR

## User-visible outcome

Index realization and schedule verification carry one broadcast relation over its whole symbolic domain, including its bijective binding at one, without lying that it is always replication or always reindexing.

## Work

- Add one explicitly tagged parametric broadcast access relation carrying the sourced operand/result relation and exact environment identity needed to interpret it.
- Keep `BroadcastReplication` and `ReindexBijection` unchanged. The new carrier is neither concrete variant; consumers must match it explicitly.
- Extend the governed index law/lowering, canonical schedule encoder, builders, verifier, realization witnesses, request-subject projection, fusion classification, costing, and exhaustive tag/population tests.
- Prove bounds and coordinate equality for every admitted binding. Permit replication-only transformations only when the environment proves actual widening; otherwise conservatively decline them.
- Preserve all existing access/schedule bytes by adding fresh discriminants. Step a domain only if an old payload must be reinterpreted.
- Keep the relation symbolic. Do not bind an extent, select a concrete access variant, or introduce a runtime fallback in this layer.

## Acceptance

- The same carrier verifies at bindings one, two, ten, and the admitted upper bound.
- Forged zero-capable, foreign-environment, wrong-equality, and concrete-variant substitutions fail under distinct typed rules.
- A replication-only fusion/cost path declines when actual widening is unproved and admits the proved-widening neighbour.
- Existing concrete reindex and broadcast canonical bytes remain unchanged; new tag injectivity is perturbed and observed failing.

## Stop conditions

Stop if the carrier would require runtime-bound values in semantic identity, if any consumer treats it as concrete replication through a wildcard, or if one-artifact lowering needs a different coordinate language than the accepted sourced relation.
