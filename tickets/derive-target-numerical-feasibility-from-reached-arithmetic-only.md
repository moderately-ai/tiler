---
id: derive-target-numerical-feasibility-from-reached-arithmetic-only
title: Derive target numerical feasibility from reached arithmetic only
status: todo
priority: p1
dependencies: [admit-an-explicit-non-arithmetic-region-and-delivery-state]
related: [plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/compiler, implementation/ir, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility, correctness, strict]
---
## Outcome

Target numerical feasibility asks only about arithmetic actually reached by the selected computation. A copy-only program carries the caller's stated contract as program intent but produces explicit `NotApplicable` numerical requirements and cannot be rejected for a target honourability row it never consumes.

## Boundary

This is not a default-to-feasible rule. The owning region/entry must explicitly classify itself as non-arithmetic; an unclassified or missing state refuses. Mixed programs continue to require every arithmetic entry's complete numerical realization, and copy entries may neither introduce nor discharge an arithmetic obligation.

## Closes when

Copy-only, arithmetic-only, and mixed programs exercise distinct paths; deleting the explicit copy classification fails closed; moving one arithmetic operation into or out of a stage changes the requested feasibility population; and no compile-host, backend name, or nearby target row is used as an inference.
