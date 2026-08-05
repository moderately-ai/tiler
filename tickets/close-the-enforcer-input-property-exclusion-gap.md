---
id: close-the-enforcer-input-property-exclusion-gap
title: Close the enforcer input-property exclusion gap
status: todo
priority: p3
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, optimizer, enforcers, boundary-properties]
---
## User-visible outcome

A derivation of whether Tiler's enforcer insertion needs Volcano's *excluding physical property vector*, and if so where it belongs — recorded so the answer is not re-derived the first time an enforcer's input search re-derives the property the enforcer was about to supply.

## Why this exists

**Fact.** Volcano (`volcano-icde-1993`, preserved under [the formalism record's sources](../docs/research/region-search/sources/README.md)) carries a parameter the optimizer contract does not: when optimizing an enforcer's input, the required-property vector is relaxed *and* an excluding vector forbids algorithms that already deliver the property being enforced. Its worked case is that under a sort, hybrid hash join must apply and merge-join must not.

**Fact.** [The optimizer contract](../docs/compiler/optimizer.md#boundary-requirements-and-guarantees) says only that "enforcer insertion is cycle-checked". A cycle check catches an enforcer feeding itself; it does not stop the input search from choosing a producer that already guarantees the property, which is a redundant plan rather than a cyclic one.

**Inference.** Tiler has one enforcer family in flight (materialization, layout conversion, encoding repacking) and no evidence this has bitten. The gap is real and its cost today is unmeasured — which is why this is a derivation ticket rather than an implementation one.

## What the record owes

- Whether the redundancy is reachable in the current planner at all, checked by reading the frontier and selection paths rather than assumed.
- If reachable: whether exclusion belongs in the boundary-property system (as a third vector beside requirement and guarantee) or in dominance, and what its identity encoding is — the contract's own admission rule for a new property dimension applies.
- If unreachable today: the exact condition that would make it reachable, so this becomes a deferral with a trigger rather than an open note.

## Non-goals

Implementing exclusion; adding a property dimension without meeting the contract's admission rule.
