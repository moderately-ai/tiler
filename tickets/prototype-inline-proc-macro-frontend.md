---
id: prototype-inline-proc-macro-frontend
title: Implement the inline proc-macro frontend proof
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate, enforce-repository-validation-gate-integrity]
related: []
scopes: [implementation/frontend, implementation/compiler, implementation/workspace]
shared_scopes: []
paths: []
tags: [implementation, frontend, proc-macro, inline-dx]
---
Implement a bounded inline Rust proc-macro frontend that parses one visible tensor region, constructs the public logical program, invokes the ordinary compiler boundary, reports span-aware typed errors, and emits generated Rust. Preserve no consumer build.rs, registry, source scan, prepare step, or runtime JIT. Tom reviews public syntax and ergonomics.
