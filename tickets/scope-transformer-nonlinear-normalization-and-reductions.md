---
id: scope-transformer-nonlinear-normalization-and-reductions
title: Scope the workload's transformer nonlinear, normalization, and reduction families
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface]
related: [implement-parallel-reduction-strategies, research-region-accuracy-contracts-and-analyzable-error-budgets, own-operation-family-support-matrix]
scopes: [research/numerics, contracts/numerics, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, transformer, normalization, softmax, language-model]
---
Define the exact activation, normalization, softmax, masking, and reduction
families required by the selected workload. Similar names are not sufficient:
for example, exact and approximate GELU are different semantic operations, as
are LayerNorm and RMSNorm.

## Required analysis

- Give each required family an exact formula, dtype signature, conversion
  behavior, exceptional-value behavior, and accuracy or order contract.
- Derive softmax and normalization requirements from small tensor examples,
  including extrema reduction, exponentiation, accumulation, division or
  reciprocal, empty domains, masks, and materialization boundaries.
- Evaluate the Metal feasibility of required transcendental realizations using
  bounded source inspection or measurement.
- Separate a composite graph spelling from a justified atomic semantic
  operation and from a fused physical implementation.
- Identify which requirements are already covered by generic reduction,
  numerical-policy, and accuracy-contract work.

## Ticket-producing outcome

File coherent operation-family verticals—such as activation, normalization, and
softmax—rather than tickets organized around private modules. Each vertical
must include reference behavior, compiler legality, Metal realization,
explainable refusal, and bounded conformance evidence.

## Closes when

Every nonlinear, normalization, mask, and reduction requirement of the selected
workload has a precise contract or a named unresolved decision; Metal
feasibility boundaries are recorded; and all justified delivery work has
dependency-ordered tickets.
