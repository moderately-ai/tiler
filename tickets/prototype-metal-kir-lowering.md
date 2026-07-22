---
id: prototype-metal-kir-lowering
title: Lower verified kernel IR to deterministic MSL
status: todo
priority: p0
dependencies: [prototype-structured-kir-slice, prototype-target-feasibility-authority]
related: []
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, codegen]
---
Lower verified structured KIR plus explicit target facts to deterministic MSL for the bounded proof profile. Emit every required entry point and deterministic helpers, with typed diagnostics, golden/negative tests, and no graph-pattern reconstruction or hidden semantic special cases.
