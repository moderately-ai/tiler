---
id: prototype-candle-metal-adapter
title: Prototype the Candle Metal adapter
status: todo
priority: p1
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/candle, implementation/runtime, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, integration, candle]
---
Implement the first consumer adapter without contaminating compiler semantics: storage/layout validation, output allocation, device-scoped runtime cache identity, ABI binding, asynchronous lifetimes, preflight before custom-op application, and wrapper-level fallback. Start with the explicit contiguous/no-autograd subset and reject unsupported cases.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.
