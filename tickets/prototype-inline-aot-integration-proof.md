---
id: prototype-inline-aot-integration-proof
title: Prove the complete inline AOT workflow
status: todo
priority: p1
dependencies: [prototype-macro-embedding-and-cargo-behavior, prototype-metal-runtime-proof]
related: []
scopes: [implementation/frontend, implementation/cache, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/runtime]
shared_scopes: []
paths: []
tags: [implementation, integration, inline-dx, milestone-0b]
---
Demonstrate one ordinary inline Rust invocation constructing and optimizing a program, sharing external compilation through the validated cache, embedding manifest/metallib bytes directly, and emitting guarded runtime selection with fallback authority before commit. Require no build script, registry, scan, prepare command, or runtime source compilation.
