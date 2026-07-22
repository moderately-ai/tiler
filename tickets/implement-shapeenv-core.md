---
id: implement-shapeenv-core
title: Implement the core ShapeEnv authority
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [shape-environment-contract]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, mature-product]
---
Implement the accepted graph/session-owned ShapeEnv: typed root symbols,
constraints, exact mathematical integers, binding/source phases, canonical
identity, validation, and explicit unresolved/ambiguous errors. It must not
depend on index IR. Index bindings and predicates consume this authority in
separate downstream tickets.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
