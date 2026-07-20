---
id: reduction-semantics-contract
title: Define reduction semantics and legality
status: done
priority: p0
dependencies: [semantic-graph-contract, numerical-policy-contract]
related: []
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research, numerics, reductions]
---
Specify reduction axes, accumulation dtype, empty identities, ordering guarantees, determinism, NaN behavior, materialization boundaries, and the numerical freedoms required by single-pass and multi-pass implementations.

Deliver normative examples and adversarial tests, plus a legality table separating semantic feasibility from physical cost. Call out which reduction forms are excluded from the first vertical slice.
