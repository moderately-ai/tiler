---
id: prototype-metal-runtime-preflight
title: Implement Metal runtime preflight
status: todo
priority: p0
dependencies: [prototype-runtime-artifact-validation, prototype-metal-aot-slice]
related: []
scopes: [implementation/runtime]
shared_scopes: []
paths: []
tags: [implementation, runtime, metal, correctness]
---
Preflight device, family, library, every selected function/pipeline, resources, bindings, launch expressions, and scratch before routing commit or program work. Distinguish route misses from corrupt artifacts and systemic failures with typed phases and injected failures.
