---
id: prototype-inline-proc-macro-frontend
title: Implement the inline proc-macro frontend proof
status: todo
priority: p1
dependencies: [prototype-public-compiler-api, prototype-neutral-artifact-codec]
related: []
scopes: [implementation/frontend, implementation/compiler, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, frontend, proc-macro, inline-dx]
---
Implement a bounded inline Rust proc-macro frontend that parses one visible tensor region, constructs the public logical program, invokes the ordinary compiler boundary, reports span-aware typed errors, and emits generated Rust. Preserve no consumer build.rs, registry, source scan, prepare step, or runtime JIT. Tom reviews public syntax and ergonomics.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.
