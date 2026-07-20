---
id: synthesize-optimizer-contracts
title: Synthesize optimizer, schedule, program, and numerical contracts
status: done
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

## Outcome

- Contracts: [optimizer](../docs/compiler/optimizer.md), [fusion and scheduling](../docs/compiler/fusion-and-scheduling.md), and [cost model](../docs/compiler/cost-model.md)
- Evidence: [region oracle](../docs/research/region-search/exhaustive-region-oracle.md), [schedule model](../docs/research/scheduling/scheduled-region-model.md), and [program planning](../docs/research/program-planning/kernel-program-buffer-plan.md)
- Result: separated legality, intrinsic schedule verification, target feasibility, cost estimation, program selection, and structured-kernel refinement across named stages.
