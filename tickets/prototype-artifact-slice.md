---
id: prototype-artifact-slice
title: Implement the bounded artifact and routing slice
status: closed
priority: p0
dependencies: [prototype-semantic-reference-slice]
related: [prototype-neutral-program-and-artifact-types, prototype-neutral-artifact-codec]
scopes: [implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [implementation, prototype, artifact]
closed_reason: superseded
closed_note: The baseline retained only construction-plan evidence; neutral artifact ownership and the bounded codec are now tracked by prototype-neutral-program-and-artifact-types and prototype-neutral-artifact-codec.
---
Implement the minimum target-neutral envelope, serial-Sum ABI roles, checked expressions, digest validation, preflight states, and one-way RoutingCommit types needed by the proof. Keep serialization private and lockstep; test corruption and forbidden post-commit fallback without importing compiler passes.
