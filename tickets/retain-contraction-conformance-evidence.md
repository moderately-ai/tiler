---
id: retain-contraction-conformance-evidence
title: Retain contraction conformance evidence for the profile's cells and corpus
status: done
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

**Historical Fact — the evidence began in a spike rather than a gate.** [The realization probe](../spikes/scheduling/metal_contraction_vertical/README.md) retains eight adversarial cases with every named topology's exact bits, and six workload cells with per-cell `result_sha256`. The ordinary test surface described below now retains the parts that are guarantees about Tiler while the Metal measurement remains bounded to its environment row.

**Proposal — the two halves, kept apart.**

- **Reference conformance**, target-independent: the eight adversarial cases against the reference evaluator. The execution witness, order absorption, the fused-against-separately-rounded discriminator, the signed-zero accumulator seed, a non-canonical NaN payload, `inf * 0` formed inside the reduction, a subnormal product, and the vector separating the contiguous from the strided split. These are exact-bit assertions with no tolerance.
- **Realization conformance**, bounded to a host row: the six workload cells' `result_sha256` against the executed result, valid only where the environment row matches, announcing the difference and declining to compare where it does not — the discipline the Apple numerical harness already uses.

**Fact — the reduction contract names the coverage this owes.** [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md)'s adversarial list includes signed zeros in both orders, subnormals, infinities, qNaN and sNaN in every contributor position, three-element reassociation and permutation witnesses, contiguous multi-pass and noncontiguous lane trees, and verifier rejections naming the missing permission. The spike's corpus covers some of these and not all; state which, rather than implying the list is discharged.

## Non-goals

A model-level tolerance, which [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) owns and which L1 already fixes cannot be composed from per-operation bounds. Conformance for structures 2 and 3, which are not in the profile.

## Closes when

Both halves exist in the ordinary test surface, each was watched failing under a deliberate perturbation, and the realization half declines rather than passes on a non-matching environment row.

**Correction — 2026-08-10.** An earlier close condition also required that "the coverage statement says exactly which of the reduction contract's adversarial cells are covered and which are not." That clause is not a residual of this ticket: on 2026-08-09 the coverage ledger was rehomed to [`state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract`](state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract.md).

## Outcome audit — 2026-08-09

**Fact — the two implementation halves landed.** `crates/tiler-reference/tests/contraction_conformance.rs` now retains the eight exact-bit cases through the public semantic and reference boundary. Its source states the independent spike provenance, the strict numerical contract, the target-independent boundary, and the named-case rather than whole-domain extent. The ordinary conformance surface pins the six retained `direct` workload digests, checks that the executed and embedded bytes agree, compares against the retained digest only on the matching environment row, and exercises the mismatch and decline paths. The source anchors `the_pinned_cells_are_the_retained_records_own_direct_rows`, `a_retained_comparison_separates_the_executed_bytes_from_the_published_record`, and `the_contraction_members_route_and_the_gates_cells_carry_their_retained_digests` are the current evidence.

**Fact — one documentation remainder stays open.** Neither half is a complete ledger against the governing record's `Required adversarial tests` population. That bounded remainder is now [`state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract`](state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract.md). It must not duplicate the delivered corpus or widen the evidence claim while stating coverage.

The implementation outcome this ticket requested is complete. The coverage ledger is split so dependents no longer treat already-retained tests as missing work.
