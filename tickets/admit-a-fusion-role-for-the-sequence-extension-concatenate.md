---
id: admit-a-fusion-role-for-the-sequence-extension-concatenate
title: Admit a fusion role for the sequence-extension concatenate
status: todo
priority: p1
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, concatenate]
---
## User-visible outcome

A cover region holding `tiler::concatenate-f32@1` beside another operation derives its fusion legality from a declared role instead of failing closed to `Unknown`, so the support matrix's R5 criterion is met for the sequence-extension family rather than skipped.

## Why this exists

**Fact — the family resolves to no legality at all today.** `FusionNumericalCapabilities::governed` (`crates/tiler-compiler/src/fusion_legality.rs:268-335`) registers nine keys and the concatenate is not among them; `derive_member` returns `Ok(None)` for an unregistered family (`:1037-1039`) and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:940-953`).

**Fact — the elimination is done and one candidate survived.** [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) tests four candidates — no role, `ValueSource`, a new seventh role, and `CoordinateRelation` — against what `derive_obligations` actually decides, and only `CoordinateRelation` survives. `ValueSource` fails on the role doc's own distinction at `fusion_legality.rs:205-212`; a seventh role fails because it derives no obligation differently and a fifth `FusionRegionStructure` count would move the content identity of every region the vocabulary can already encode (`:511-538`).

**Fact — M4 does not wait on M5.** Neither `derive_fusion_legality` (`:922-967`) nor `derive_obligations` (`:1063-1163`) resolves an index-access capability, consults a realization law, or reaches the request boundary. This ticket is independent of the concatenate lowering chain and of Q-SHAPE-006.

## What the work is

Register `concatenate_f32_op()` under `FusionOperationRole::CoordinateRelation` in `FusionNumericalCapabilities::governed`, with a comment stating the derivation rather than citing the record.

Extend the `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` (`fusion_legality.rs:1187-1189`) to the concatenate key. This arm is deliberately closed over exact keys so that each addition is decided rather than inherited, and the decision here is that the arm's own soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — transfers verbatim to a join that introduces no multiply, no add, and no adjacency between them. Not extending it is not free: under a contraction-permitting contract a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1113-1116`) and `first_unknown` makes the whole candidate unknown.

Prove each new path can fail. A deliberate perturbation must show a concatenate-bearing region reaching `Unknown` when the role is removed — `governed_without` (`:357-362`) exists for exactly this — and a second showing the contraction obligation's outcome under a contraction-permitting contract with and without the arm extension.

Confirm on the merged tree whether the pinned explain digest in `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` (`crates/tiler-compiler/src/explain.rs` — cited by test name because its line number has already drifted once, when the softmax fact correction rebaselined it to `a95ad77532352d7f`) moves. The record's reading is that it does not — `ExplainWriter::new` folds only `FusionNumericalCapabilities::governed().provider()` (`explain.rs:1219-1235`), not the role table, and `GOVERNED_PROVIDER_REVISION` did not move when the reindex and broadcast roles were added — but that is an inference from a precedent and must be observed rather than inherited, because the ledger comments at `explain.rs:4008-4021` record two occasions on which a concatenate-related change moved it for a different reason.

## Explicit non-goals

- Any index-access lowering. That is [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) and [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md), and this ticket must not wait on either.
- An `OperationNumericalCapability` row. The family performs no arithmetic, so there is no dimension a row could list; `UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs:788-817`) records that reasoning and the entry stays until a physical realization exists.
- A seventh `FusionOperationRole` variant or a fifth `FusionRegionStructure` count.

## Closes when

The role is registered, the contraction arm is decided explicitly, a concatenate-bearing region derives `Legal` with the nine obligations discharged, both deliberate failure perturbations are shown to fail, and the matrix's `Sequence extension` row records R5 with its evidence.

## Graph maintenance

- `contracts/navigation` is declared because delivering R5 moves the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Sequence extension` rung and its next-column text, exactly as [`admit-a-fusion-role-for-the-tensor-contraction`](admit-a-fusion-role-for-the-tensor-contraction.md) declares it for the same reason.
- The scoping record owns the derivation and this ticket owns the rung. Do not restate the elimination here.
