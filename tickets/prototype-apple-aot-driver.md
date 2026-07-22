---
id: prototype-apple-aot-driver
title: Implement the Apple offline compiler driver
status: todo
priority: p0
dependencies: [repair-apple-target-experiment-integrity, prototype-target-feasibility-authority, enforce-repository-validation-gate-integrity]
related: []
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, aot, toolchain]
---
Implement a bounded driver with explicit SDK, platform family, deployment minimum, MSL version, output-affecting flags, metal/metallib invocation, diagnostics, fingerprint and provenance. Use one selected SDK and never inherit output-affecting defaults silently; exclude cache and proc-macro concerns.
