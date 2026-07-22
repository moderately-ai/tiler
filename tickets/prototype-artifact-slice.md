---
id: prototype-artifact-slice
title: Implement the bounded artifact and routing slice
status: closed
priority: p0
dependencies: [prototype-semantic-reference-slice]
related: [prototype-neutral-artifact-codec, prototype-kernel-program-ir]
scopes: [implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [implementation, prototype, artifact]
closed_reason: superseded
closed_note: The baseline retained only construction-plan evidence; target-neutral kernel-program IR, the artifact-facing program model, and the bounded codec now have separate owners.
---
Implement the minimum target-neutral envelope, serial-Sum ABI roles, checked expressions, digest validation, preflight states, and one-way RoutingCommit types needed by the proof. Keep serialization private and lockstep; test corruption and forbidden post-commit fallback without importing compiler passes.
