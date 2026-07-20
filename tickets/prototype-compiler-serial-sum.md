---
id: prototype-compiler-serial-sum
title: Implement fixed fusion scheduling and structured lowering
status: todo
priority: p0
dependencies: [prototype-semantic-reference-slice]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets, contracts/optimizer]
paths: []
tags: [implementation, prototype, compiler]
---
Implement deterministic validation/canonicalization, the single pointwise-into-serial-Sum fusion choice, canonical index lowering, one-thread-per-output schedule, structured-kernel refinement, rejection explanations, and artifact-plan construction. Retain an explicit split reference plan and do not implement a general memo or cost model.
