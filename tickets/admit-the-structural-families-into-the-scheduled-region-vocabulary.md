---
id: admit-the-structural-families-into-the-scheduled-region-vocabulary
title: Admit the structural families into the scheduled-region vocabulary
status: todo
priority: p1
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: [reach-a-verified-kernel-through-the-structural-families, admit-the-reindex-and-broadcast-operation-families]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, compiler, structural]
---
## User-visible outcome

A program stating `tiler::reindex-f32@1` or `tiler::broadcast-f32@1` reaches the optimizer instead of refusing at the request boundary under `operation-set`, so the two families the pinned workload cannot be written without stop being statable-but-unrecognizable.

## Why the elementary admission did not carry these two with it

**Fact — the activation was admissible and these are not, and the difference is which vocabulary is missing.** [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) admitted `tiler::silu-f32@1` by *projecting* its per-point body into `PointwiseF32Node`, from one shared statement the governed index-access lowering also drives. No `PointwiseF32Node` spells a sigmoid-weighted linear unit either — but the body is expressible in the nodes that exist, so a projection is available.

**Fact — what these two lack is the access relation, and `LogicalAccess` has no spelling for it.** `crates/tiler-ir/src/schedule/model.rs`'s `LogicalAccess` carries `LinearIdentity`, `ScalarBroadcast`, `PackedU4LsbZeroTail`, `ReductionContributor`, and `ContractionOperand`. There is no reindex map at all, and the only broadcast is `ScalarBroadcast` — "every invocation reads the single scalar parameter element", a rank-zero operand read once — which does not express the workload's `[1024]` to `[T, 1024]` widening. A projection into the per-point vocabulary cannot substitute for a missing coordinate map, because the two families compute nothing: each result element is an operand element with the same bits, and what varies is *which* element.

Reproduce the absence in one line: `rg -n 'enum LogicalAccess' -A 60 crates/tiler-ir/src/schedule/model.rs`.

**Inference — the shape of the widening, not yet the design.** `LogicalAccess` is `#[non_exhaustive]` under ADR 0074 convention 5a precisely so a new coordinate map lands additively, and no out-of-crate consumer classifies it by exhaustive match. So the additive seam exists; what does not exist is the map itself, its bounds-proof obligation, its write-ownership consequence, and its identity encoding. `crates/tiler-compiler/src/governed.rs`'s `GovernedReindexF32` already emits the *index-region* half for every admitted `ReindexForm`, and `GovernedBroadcastF32` for both many-to-one relations, so the derivation of the coordinate maps exists and is tested — it is the schedule-level vocabulary that is absent.

## Boundaries

- **A copy variant is not the answer, and the admission ticket's non-goals already ruled it out.** Adding a `ScalarProgram` copy would realize a standalone reindex as a materializing copy kernel. What should reach a kernel is a *fused* region where the structural occurrence contributes an access map and an arithmetic neighbour contributes the scalar program.
- Every new `LogicalAccess` variant owes what the existing ones owe: a bounds-proof obligation the region verifier can discharge, write-ownership consequences, an identity encoding, and a total map at every site inside `tiler-ir` that matches it exhaustively.
- The request boundary's refusal stays until the vocabulary lands. `a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason` in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` and `perturbing_one_occurrence_out_of_the_vocabulary_refuses_by_name` in `crates/tiler-compiler/tests/composed_family_recognition.rs` are what keep it observed, and both now assert it *beside* an admitted elementary neighbour so the rule is attributable.

## Closes when

A program containing a `Reindex` or a non-scalar `Broadcast` is recognized at the request boundary and reaches a verified scheduled region, its result is bit-compared against the reference evaluator, and the two boundary tests above are updated to assert the new admission beside whatever still refuses.
