---
id: implement-shapeenv-index-bindings
title: Implement ShapeEnv-backed symbolic index bindings
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, mature-product]
---
Implement the accepted ShapeEnv root-symbol and binding model, then extend canonical index domains and expressions with sourceable InputDimension, InterfaceParameter, and phased TargetProperty references. Preserve exact mathematical-integer identity and reject free, ambiguous, tensor-data-derived, or too-late sources. Do not create a competing index-local binding vocabulary.
