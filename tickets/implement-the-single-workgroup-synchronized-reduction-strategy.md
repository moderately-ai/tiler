---
id: implement-the-single-workgroup-synchronized-reduction-strategy
title: Implement the single-workgroup synchronized reduction strategy
status: todo
priority: p1
dependencies: [admit-the-first-typed-synchronization-point-and-atomic-target-authority]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A bounded reduction can select one verified single-workgroup tree schedule whose staged dataflow, synchronization point, accumulation dtype, order, and numerical permissions agree.

## Implementation keys

Build only on the accepted cooperative dataflow and synchronization receipt. Define the tree topology, active lanes at every phase, tail handling, workgroup storage, accumulation dtype, and deterministic contributor order. Tree reassociation requires reassociation permission; a nondeterministic or atomic arrival order additionally requires contributor-permutation permission.

Keep serial and single-workgroup alternatives together. Missing synchronization authority, insufficient workgroup resources, divergent convergence, or withheld numerical permission rejects before executable-frontier admission rather than receiving an arbitrary cost.

## Required evidence

Power-of-two, uneven-tail, one-element, and empty extents agree with the reference. Independent reassociation/permutation mutations reject the exact affected strategy. Every phase reaches the exact synchronization point uniformly, and every workgroup read is visibility-covered. Identity changes with topology, accumulation, point, and resource realization.

## Closes when

The target-neutral synchronized alternative is verified beside serial, exact public drafts are reviewed by Tom, every check is mutation-proved, and targeted tests/Clippy plus the batch gate pass. Metal support remains downstream.
