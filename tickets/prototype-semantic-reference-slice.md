---
id: prototype-semantic-reference-slice
title: Implement the serial Sum semantic and reference slice
status: todo
priority: p0
dependencies: [prototype-workspace-scaffold]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics]
paths: []
tags: [implementation, prototype, semantics]
---
Implement only the typed f32 input/constant/multiply/add/strict-Sum/output graph, validation, canonical contributor order, deterministic identity, and host reference evaluator required by ADR 0055. Include adversarial numerical and invalid-graph tests; do not add Metal or optimizer dependencies.
