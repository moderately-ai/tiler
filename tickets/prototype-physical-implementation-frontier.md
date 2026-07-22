---
id: prototype-physical-implementation-frontier
title: Implement the physical implementation frontier
status: todo
priority: p0
dependencies: [prototype-scheduled-region-ir, prototype-target-feasibility-authority, prototype-fusion-legality-and-numerical-proof]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
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

Frontiers are checked local authorities for individual legal regions. Their
enumeration does not depend on a complete cover and does not prove global
coverage; complete physical-plan selection joins the independent authorities.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
