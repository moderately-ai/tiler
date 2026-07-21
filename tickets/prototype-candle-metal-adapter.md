---
id: prototype-candle-metal-adapter
title: Prototype the Candle Metal adapter
status: todo
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/candle, implementation/runtime]
shared_scopes: []
paths: []
tags: [implementation, integration, candle]
---
Implement the first consumer adapter without contaminating compiler semantics: storage/layout validation, output allocation, device-scoped runtime cache identity, ABI binding, asynchronous lifetimes, preflight before custom-op application, and wrapper-level fallback. Start with the explicit contiguous/no-autograd subset and reject unsupported cases.
