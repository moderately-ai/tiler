---
id: prototype-structured-kir-slice
title: Implement the structured kernel IR slice
status: todo
priority: p0
dependencies: [prototype-complete-program-selection]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, kernel-ir, compiler-foundation]
---
Implement backend-consumable structured KIR with typed values, address spaces, explicit indexing, loads/stores, conversions, loops, predicates, reductions, and effects/barriers where applicable. Verify scope, type, ownership, bounds, effect ordering, and output coverage; backends must not reconstruct graph-specific semantics.
