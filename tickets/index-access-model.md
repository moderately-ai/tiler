---
id: index-access-model
title: Validate the symbolic index and access-map model
status: done
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract]
related: []
scopes: [research/indexing, contracts/core, contracts/compiler, contracts/artifacts, contracts/integrations]
shared_scopes: []
paths: []
tags: [tiler-research, spike, indexing]
---
Represent iteration domains and affine or guarded tensor accesses for the first semantic slice. Test broadcast, permutation, reshape composition, non-contiguous affine layouts, tails, overflow, and guarded u32 fast paths with a wider correctness path.

Deliver canonicalization and verification rules, counterexamples that require non-affine or data-dependent access, and a decision boundary between semantic access maps and scheduled address arithmetic.

## Outcome

- Research: [symbolic index and access model](../docs/research/indexing/index-access-model.md)
- Experiment: [index and access-model experiment](../spikes/indexing/README.md)
- Adopted decision: [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md)
- Result: separated width-independent logical tensor coordinates from physical buffer views, address widths, masks, and storage encoding.
