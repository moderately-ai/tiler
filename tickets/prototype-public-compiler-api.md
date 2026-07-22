---
id: prototype-public-compiler-api
title: Implement the reviewed public compiler boundary
status: todo
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, dx]
---
Implement ADR 0069's consumer-agnostic CompilationRequest, session/provider
inputs, checked compilation result, stable diagnostics/explain, and ordinary
call-site ergonomics over the verified pipeline. Tom reviews consequential
public crate, trait, type, and call-site boundaries before acceptance. Frontends
consume this API; backend feasibility components need not depend on it.
