---
id: shape-environment-contract
title: Define symbolic shape and extent sourceability
status: done
priority: p0
dependencies: []
related: []
scopes: [research/shapes, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research]
---
Determine the scope and invariants of ShapeEnv: static rank, symbolic extents, equality and divisibility constraints, where runtime extent values originate, and which host-side shape and launch expressions are derivable. Bound or defer data-dependent shapes explicitly.

Deliver a decision memo with positive and negative examples, validation rules, and implications for ABI guards, specialization, and index width. Do not assume a consumer framework.

## Outcome

- Research: [shape environment](../docs/research/shapes/shape-environment-contract.md) and [constraint prover boundary](../docs/research/shapes/constraint-prover-boundary.md)
- Adopted decision: [ADR 0008](../docs/decisions/0008-typed-root-bindings.md)
- Result: separated exact shape algebra from typed root-binding provenance and bounded the initial incomplete prover without weakening validation.

## Evidence correction (2026-07-21)

The [shape-environment report](../docs/research/shapes/shape-environment-contract.md)
and [constraint-prover report](../docs/research/shapes/constraint-prover-boundary.md)
now distinguish the partial private static slice from the proposed full
environment and prover. The architectural contract remains accepted; no
tracked executable currently proves the complete model.
