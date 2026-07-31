---
id: retain-contraction-conformance-evidence
title: Retain contraction conformance evidence for the profile's cells and corpus
status: todo
priority: p2
dependencies: [integrate-the-contraction-vertical-into-the-runtime]
related: [design-model-level-qualification-and-optimization, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [implementation/reference, implementation/compiler, contracts/numerics, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, conformance, contraction, numerics]
---
## User-visible outcome

A later change to the contraction's schedule, emitter, or toolchain is a *failure* rather than a drift, because the exact bits the profile produces today are retained and compared.

## What to retain, and why each part

**Fact — the evidence already exists in a spike and is not a gate.** [The realization probe](../spikes/scheduling/metal_contraction_vertical/README.md) retains eight adversarial cases with every named topology's exact bits, and six workload cells with per-cell `result_sha256`. No `make` target reaches `spikes/`, so only re-running that spike by hand detects drift from it. This ticket moves the part that is a *guarantee about Tiler* into the repository's own conformance surface, and leaves the part that is a measurement about Metal where it is.

**Proposal — the two halves, kept apart.**

- **Reference conformance**, target-independent: the eight adversarial cases against the reference evaluator. The execution witness, order absorption, the fused-against-separately-rounded discriminator, the signed-zero accumulator seed, a non-canonical NaN payload, `inf * 0` formed inside the reduction, a subnormal product, and the vector separating the contiguous from the strided split. These are exact-bit assertions with no tolerance.
- **Realization conformance**, bounded to a host row: the six workload cells' `result_sha256` against the executed result, valid only where the environment row matches, announcing the difference and declining to compare where it does not — the discipline the Apple numerical harness already uses.

**Fact — the reduction contract names the coverage this owes.** [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md)'s adversarial list includes signed zeros in both orders, subnormals, infinities, qNaN and sNaN in every contributor position, three-element reassociation and permutation witnesses, contiguous multi-pass and noncontiguous lane trees, and verifier rejections naming the missing permission. The spike's corpus covers some of these and not all; state which, rather than implying the list is discharged.

## Non-goals

A model-level tolerance, which [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) owns and which L1 already fixes cannot be composed from per-operation bounds. Conformance for structures 2 and 3, which are not in the profile.

## Closes when

Both halves exist in the ordinary test surface, each was watched failing under a deliberate perturbation, the realization half declines rather than passes on a non-matching environment row, and the coverage statement says exactly which of the reduction contract's adversarial cells are covered and which are not.
