---
id: prototype-artifact-slice
title: Implement the bounded artifact and routing slice
status: closed
priority: p0
dependencies: [prototype-semantic-reference-slice]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [implementation, prototype, artifact]
closed_reason: superseded
closed_note: Folded into prototype-target-neutral-baseline-slice so artifact contracts are proven through a complete target-neutral plan.
---
Implement the minimum target-neutral envelope, serial-Sum ABI roles, checked expressions, digest validation, preflight states, and one-way RoutingCommit types needed by the proof. Keep serialization private and lockstep; test corruption and forbidden post-commit fallback without importing compiler passes.
