---
id: implement-shapeenv-index-bindings
title: Bind ShapeEnv sources into canonical index regions
status: in-progress
priority: p1
dependencies: [implement-shapeenv-core, implement-shapeenv-constraints]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing]
claimed_from: todo
assignee: agent-shapes2
lease_expires_at: 1785002816
---
Extend canonical index domains and expressions with sourceable ShapeEnv
InputDimension, InterfaceParameter, and phased TargetProperty bindings. Preserve
mathematical-integer identity, phase ordering, guards/proofs, and explicit
rejection of free, ambiguous, tensor-data-derived, or too-late sources. Do not
create an index-local duplicate symbol authority.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
