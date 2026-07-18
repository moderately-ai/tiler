---
id: kernel-program-buffer-plan
title: Define whole-program planning and conservative buffer reuse
status: todo
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract, scheduled-region-model]
related: []
scopes: [research/program-planning]
shared_scopes: []
paths: []
tags: [tiler-research, spike, program-plan]
---
Specify how multiple scheduled regions become a KernelProgram with dependencies, materialized values, scratch allocations, dispatch order, host expressions, and named outputs. Develop a conservative liveness and buffer-reuse model under an explicit single-device execution contract.

Deliver a whole-program verifier proposal and examples covering fan-out, multiple outputs, multi-dispatch reductions, scratch lifetime, aliasing rejection, and fallback-before-partial-work requirements.
