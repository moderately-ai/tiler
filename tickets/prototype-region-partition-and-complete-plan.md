---
id: prototype-region-partition-and-complete-plan
title: Implement complete region partition planning
status: todo
priority: p0
dependencies: [prototype-fusion-legality-and-numerical-proof]
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, optimizer, partitioning]
---
Build complete program covers from legal regions. Cover every operation and named output, require boundary agreement, conservatively materialize fan-out unless duplication is explicitly legal and costed, retain fused and materialized alternatives, and reject partial programs.

Complete-plan identity must combine semantic graph meaning with exact bound
region occurrences and implementations, coverage, dependencies, deliberate
duplication, and materializations; a set of reusable region-content digests is
not sufficient proof of graph coverage.
