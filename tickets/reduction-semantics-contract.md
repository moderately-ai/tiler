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

## Outcome

[Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md)
defines ordered contributors, conversions, seeds, empty behavior, order
permissions, and the semantic/physical boundary. The retained
[reduction contract probe](../spikes/numerics/reduction_contract/README.md)
checks the host-side model. ADRs
[0012](../docs/decisions/0012-physical-reduction-topology.md),
[0014](../docs/decisions/0014-reassociation-vs-permutation.md),
[0022](../docs/decisions/0022-reduction-identities-and-initial-values.md), and
[0025](../docs/decisions/0025-reduction-empty-results-and-padding.md) record the
accepted invariants. Parallel topology verification and backend execution are
still unimplemented.
