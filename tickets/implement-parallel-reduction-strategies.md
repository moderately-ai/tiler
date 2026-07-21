---
id: implement-parallel-reduction-strategies
title: Implement parallel reduction strategies
status: todo
priority: p1
dependencies: [implement-first-profile-numerical-policies, implement-boundary-properties-and-enforcers]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, reduction, scheduling, numerics]
---
Add single-workgroup and multi-pass reductions beyond the serial schedule. Define empty identities, accumulation dtype, deterministic/relaxed orders, synchronization, partial storage, feasibility and numerical evidence; selection may deliberately choose multiple kernels.
