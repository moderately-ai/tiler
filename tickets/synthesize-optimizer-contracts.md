---
id: synthesize-optimizer-contracts
title: Synthesize optimizer, schedule, program, and numerical contracts
status: todo
priority: p1
dependencies: [reduction-semantics-contract, region-search-oracle, scheduled-region-model, kernel-program-buffer-plan, cost-model-bootstrap, reference-evaluator-slice, index-access-model, structured-kernel-ir-verifier]
related: [synthesize-core-contracts]
scopes: [contracts/compiler]
shared_scopes: []
paths: []
tags: [tiler-research, synthesis, decision]
---
Update the optimizer, fusion and scheduling, cost-model, numerical-semantics, and correctness contracts from the completed evidence. Keep legality, target feasibility, estimated cost, and measured cost visibly distinct.

Acceptance requires named optimizer stages, verifier boundaries, bounded-search policy, schedule normalization, multi-output and multi-kernel examples, reduction rules, numerical conformance obligations, and rejection explanations.
