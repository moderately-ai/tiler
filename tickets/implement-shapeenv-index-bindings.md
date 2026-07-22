---
id: implement-shapeenv-index-bindings
title: Bind ShapeEnv sources into canonical index regions
status: todo
priority: p1
dependencies: [implement-shapeenv-core]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing]
---
Extend canonical index domains and expressions with sourceable ShapeEnv
InputDimension, InterfaceParameter, and phased TargetProperty bindings. Preserve
mathematical-integer identity, phase ordering, guards/proofs, and explicit
rejection of free, ambiguous, tensor-data-derived, or too-late sources. Do not
create an index-local duplicate symbol authority.
