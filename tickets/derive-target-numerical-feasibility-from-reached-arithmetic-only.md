---
id: derive-target-numerical-feasibility-from-reached-arithmetic-only
title: Derive target numerical feasibility from reached arithmetic only
status: todo
priority: p1
dependencies: [admit-the-partitioned-copy-scheduled-region]
related: [admit-an-explicit-non-arithmetic-region-and-delivery-state, plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/compiler, implementation/ir, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility, correctness, strict]
---
## Outcome

Target numerical feasibility asks only about value-changing floating-point computation actually reached by the selected program. A copy-only program carries the caller's stated contract as program intent but produces the verifier-derived `BitPreservingCopy` classification and cannot be rejected for a target honourability row it never consumes.

## Boundary

This is not a default-to-feasible rule. The owning region must be a verified `PartitionedCopy`; an unclassified or missing state refuses. Mixed programs continue to require every floating-point entry's complete numerical realization, and copy entries may neither introduce nor discharge a floating-point obligation.

## Closes when

Copy-only, arithmetic-only, and mixed programs exercise distinct paths; deleting the explicit copy classification fails closed; moving one arithmetic operation into or out of a stage changes the requested feasibility population; and no compile-host, backend name, or nearby target row is used as an inference.
