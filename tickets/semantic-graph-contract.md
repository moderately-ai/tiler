---
id: semantic-graph-contract
title: Define the semantic tensor graph contract
status: done
priority: p0
dependencies: []
related: []
scopes: [research/semantic-graph]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research]
---
Specify what operations, values, result ports, constants, parameters, and named graph outputs mean in Tiler's target-independent logical representation. Resolve multi-result operations, multiple externally visible outputs, purity, acyclicity, ownership, and validation boundaries without introducing schedule choices.

Deliver an evidence-backed decision memo with candidate invariants, counterexamples, a minimal end-to-end tensor pipeline, and explicit deferred cases. Distinguish facts, inferences, proposals, and measurements. Do not rewrite the contract docs in this ticket.

## Outcome

- Research: [semantic graph contract memo](../docs/research/semantic-graph/contract-memo.md)
- Adopted decisions: [ADR 0005](../docs/decisions/0005-public-semantic-tensor-graph.md) and [ADR 0006](../docs/decisions/0006-operation-value-graph.md)
- Result: established the pure acyclic operation/value graph, ordered named outputs, multi-result operations, canonical identity boundary, and explicit effect-model deferral.
