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
