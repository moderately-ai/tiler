---
id: prototype-metal-aot-slice
title: Emit and offline-compile the serial Sum Metal kernel
status: todo
priority: p0
dependencies: [prototype-compiler-serial-sum, prototype-artifact-slice]
related: []
scopes: [implementation/metal]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [implementation, prototype, metal]
---
Implement pure verified-KIR-to-MSL emission, explicit exact math/deployment flags, xcrun metal/metallib invocation, compiler provenance, and the publish=false prototype-compile producer. Produce a validated bounded artifact for the fixed workload; no generalized cache or proc macro.
