---
id: implement-the-target-neutral-multi-pass-reduction-strategy
title: Implement the target-neutral multi-pass reduction strategy
status: done
priority: p1
dependencies: []
related: [implement-parallel-reduction-strategies]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A reduction can produce and consume explicit partial tensors across multiple kernel-program stages, retaining a legal serial alternative and without requiring an intra-workgroup barrier.

## Implementation keys

Define each pass's reduction order, accumulation dtype, partial shape/storage, materialization, dispatch dependency, visibility transition, and empty-domain identity. Preserve reassociation and contributor permutation as independent permissions. The program verifier must prove each partial is initialized before use and that the final pass covers every contributor exactly under the selected order.

Retain serial and multi-pass alternatives together. Hard feasibility rejects unsupported storage, dispatch, arithmetic, or numerical permissions; cost remains separate and this ticket does not make the multi-pass plan win by preference.

## Required evidence

One program retains serial and multi-pass alternatives with distinct identities and explain records. Empty, one-element, uneven-tail, and multi-pass extents match the reference under each admitted numerical contract. Missing partial initialization, wrong dependency order, narrowed accumulation, and independently denied reassociation/permutation each reject. If the boundary-enforcer test reaches a real mismatch, activate its owner rather than widening constants.

## Closes when

The target-neutral multi-pass alternative is verified and carries stable verified kernel-program identity, every new check is mutation-proved, public schedule/program boundary changes are reviewed by Tom, and targeted tests/Clippy plus the batch gate pass. Artifact packaging/replay, Metal realization, and calibrated selection remain downstream.

## Review packet (2026-07-31)

**Public boundary changes for Tom, none self-accepted.** `tiler_ir::schedule` gains `ReductionPass`, `ContributorPartition`, `ReductionTopology::MultiPass { pass, partition, axes, order, accumulation, permits_reassociation, permits_permutation }`, `partial_reduction_shape`, and `partial_reduction_axis`. `tiler_ir::program` gains `PartialReduction`, `PartialReductionRef`, `KernelProgramBuilder::push_partial_reduction`, `VerifiedKernelProgram::partial_reductions`, and `MAX_PROGRAM_PARTIAL_REDUCTIONS`. `tiler_reference` gains `strict_partial_sums` and `strict_partitioned_sum`. `KernelProgramBuildError::EmptyCoverage` is replaced by the whole-program `KernelProgramDiagnostic::UncoveringStage`, because a split's final pass legitimately computes no operation and the question is only answerable once the split is declared.

**Identity domains.** `PROGRAM_DOMAIN` steps `tiler.kernel-program.v5` → `v6` to fold the split contracts; `contributors_per_partition` is the one field program scope cannot derive, which is why folding it is required rather than optional. No goldens moved — every program-identity assertion in the corpus is relational. The schedule domain deliberately does **not** step: `MultiPass` takes an appended tag `0x33`, so no previously encodable region's bytes move and `STRICT_F32_REGION_IDENTITY_HEX` is unchanged. Prose naming `tiler.kernel-program.v5` in `crates/tiler-artifact/src/program/mod.rs:64` and `docs/artifact-abi.md` is now stale and out of this ticket's declared scopes.

**Numerical independence.** A split consumes reassociation and nothing else: `verify_multi_pass_semantics` admits the topology only when `permits_reassociation` is true, and reads `permits_permutation` only to carry the declared realization forward. Both directions are driven — permitted permutation with forbidden reassociation still rejects, forbidden permutation with permitted reassociation still admits.

**Remainder, not absorbed.** Frontier enumeration of the split as a retained alternative beside the serial one, with its explain records, is blocked on two facts found here and needs its own ticket: (1) a split realizes one semantic occurrence with two dispatches, which the bounded profile expresses only through the reserved `ProposalBody::KernelSubprogram`, and `selection::reconcile_boundaries` admits at most one intermediate per region; (2) `DeterministicBudgets::governed` fixes `regions: 2` and `buffers: 3`, and `verify_request` requires both — a three-stage split program needs three regions and four buffers, so the governed budgets must be widened deliberately rather than to make a test pass. The compiler side therefore lands as the region constructors, the split-choosing authority, and the request-subject binding, without a program assembler. A ragged final partition is likewise unimplemented and rejects with a typed reason: `ContributorPartition::covers` requires the product to be exact, because a ragged tail needs a second constant trip count the structured-kernel loop vocabulary does not carry.

**Boundary-enforcer trigger.** `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` still passes. It compares two compile-time constants that this change does not touch, so `implement-boundary-property-enforcers` does not become startable from here.

## Accepted (2026-07-31)

Tom accepted the reviewed boundary as merged at `a1859d3`: the `tiler_ir::schedule` split vocabulary (`ReductionTopology::MultiPass`, `ReductionPass`, `ContributorPartition`, `partial_reduction_shape`/`partial_reduction_axis`), the `tiler_ir::program` `PartialReduction` contract with `push_partial_reduction`/`partial_reductions`/`MAX_PROGRAM_PARTIAL_REDUCTIONS`, the `tiler_reference` `strict_partial_sums`/`strict_partitioned_sum` oracles, the `EmptyCoverage` → whole-program `UncoveringStage` move, and the `tiler.kernel-program.v6` domain step. Frontier enumeration is `enumerate-the-split-reduction-on-the-planning-frontier`.

## Graph maintenance

- Keep artifact encoding and replay in `realize-parallel-reduction-strategies-on-metal`; this ticket closes on target-neutral verified program identity within its declared scopes.
- Activate `implement-boundary-property-enforcers` only if its named mismatch test actually fails on the new cross-stage boundary.
- Preserve both serial and multi-pass alternatives for downstream realization and measured calibration rather than selecting one here.
