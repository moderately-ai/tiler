---
id: prototype-apple-aot-driver
title: Implement the Apple offline compiler driver
status: todo
priority: p0
dependencies: [repair-apple-target-experiment-integrity, prototype-target-feasibility-authority, enforce-repository-validation-gate-integrity]
related: []
scopes: [implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, metal, aot, toolchain]
---
Implement a bounded driver with explicit SDK, platform family, deployment minimum, MSL version, output-affecting flags, metal/metallib invocation, diagnostics, fingerprint and provenance. Use one selected SDK and never inherit output-affecting defaults silently; exclude cache and proc-macro concerns.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.
