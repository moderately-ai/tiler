---
id: prototype-generic-region-formation
title: Implement generic fusion-region formation
status: in-progress
priority: p0
dependencies: [prototype-semantic-normalization]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, milestone-0b]
claimed_from: todo
assignee: agent-prototype-generic-region-formation
lease_expires_at: 1784828687
---
Replace proof-graph recognition and hard-coded occurrences with deterministic
bounded enumeration of connected convex regions from arbitrary supported DAGs.
Include singleton coverage, boundaries, retained named/multi-result outputs,
fan-out handling, stable identity and budgets; compare small graphs with an
exhaustive oracle. Define separate canonical region-content and graph-
occurrence identities so identical content at distinct occurrences remains
shareable without losing exact coverage or boundary bindings.
