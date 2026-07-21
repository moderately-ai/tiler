---
id: prototype-physical-implementation-frontier
title: Implement the physical implementation frontier
status: todo
priority: p0
dependencies: [prototype-region-partition-and-complete-plan]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, physical-planning, scheduling]
---
Add the typed provider surface for structured physical implementations and
opaque calls, then enumerate their proposals with typed boundary
requirements/guarantees, target/applicability predicates, exact feasibility
resources, estimated costs, provider provenance, and a minimal serial schedule.
Multiple physical providers contribute additive alternatives rather than a
singular-capability ambiguity. Every proposal must re-enter ordinary checked IR
verification. Keep infeasibility distinct from cost and malformed compiler
output distinct from a valid no-plan result.
