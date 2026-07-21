---
id: prototype-generic-region-formation
title: Implement generic fusion-region formation
status: todo
priority: p0
dependencies: [prototype-canonical-index-region-slice]
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, optimizer, fusion, milestone-0b]
---
Replace proof-graph recognition and hard-coded occurrences with deterministic bounded enumeration of connected convex regions from arbitrary supported DAGs. Include singleton coverage, boundaries, retained named/multi-result outputs, fan-out handling, stable identity and budgets; compare small graphs with an exhaustive oracle.
