---
id: index-access-model
title: Validate the symbolic index and access-map model
status: todo
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract]
related: []
scopes: [research/indexing]
shared_scopes: []
paths: []
tags: [tiler-research, spike, indexing]
---
Represent iteration domains and affine or guarded tensor accesses for the first semantic slice. Test broadcast, permutation, reshape composition, non-contiguous affine layouts, tails, overflow, and guarded u32 fast paths with a wider correctness path.

Deliver canonicalization and verification rules, counterexamples that require non-affine or data-dependent access, and a decision boundary between semantic access maps and scheduled address arithmetic.
