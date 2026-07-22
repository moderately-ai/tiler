---
id: implement-general-dag-partitioning
title: Implement general DAG partition search
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate, implement-boundary-property-enforcers, implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning, mature-product]
---
Extend partition planning to realistic DAGs with fan-out, named/multi-result outputs, legal shared-work duplication, materialization choices, and budgeted memoized search. Verify complete coverage and boundaries against exhaustive small-graph oracles and explain pruning.
