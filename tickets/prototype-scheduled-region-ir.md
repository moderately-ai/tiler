---
id: prototype-scheduled-region-ir
title: Implement checked scheduled-region IR
status: todo
priority: p0
dependencies: [prototype-semantic-index-refinement, prototype-target-feasibility-authority]
related: [scheduled-region-model]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, verifier]
---
Implement reviewed target-neutral ScheduledRegion and KernelSchedule builders,
canonical identities, and intrinsic verifier. Validate axes, work ownership,
loops, vector/tail organization, staging, reduction topology, synchronization,
launch expressions, and specialization before target feasibility is queried.
No cost or provider callback can repair malformed schedule intent.
