---
id: admit-a-fusion-role-for-the-sub-tensor-selection-slice
title: Admit a fusion role for the sub-tensor selection slice
status: in-progress
priority: p2
dependencies: []
related: [scope-the-sub-tensor-selection-fusion-role, admit-a-fusion-role-for-the-sequence-extension-concatenate, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families, admit-the-sub-tensor-selection-family, lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, slice]
claimed_from: todo
assignee: w-admit-a-
lease_expires_at: 1786140923
---
## User-visible outcome

A cover region holding `tiler::slice-f32@1` beside another operation derives its fusion legality from a declared role instead of failing closed to `Unknown`, so the support matrix's R5 criterion is met for the sub-tensor selection family rather than skipped.

## Why this exists

**Fact — the family resolves to no legality at all today.** `FusionNumericalCapabilities::governed` (`crates/tiler-compiler/src/fusion_legality.rs:268-335`) registers nine keys and the slice is not among them; `derive_member` returns `Ok(None)` for an unregistered family (`:1037-1039`) and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:944-953`). Region formation holds no operation allowlist (`crates/tiler-compiler/src/region.rs:652-692`), so a program containing a slice does form candidates that reach that state.

**Fact — the elimination is done and one candidate survived.** [Sub-tensor selection fusion role](../docs/research/indexing/sub-tensor-selection-fusion-role.md) tests four candidates — no role, `ValueSource`, a new seventh role, and `CoordinateRelation` — against what `derive_obligations` decides at `3cca2a3f`, and only `CoordinateRelation` survives. `ValueSource` fails on the role doc's own distinction at `fusion_legality.rs:206-211`; a seventh role fails because non-surjectivity derives no obligation differently and a fifth `FusionRegionStructure` count would move the content identity of every region the vocabulary can already encode (`:511-538`).

**Fact — M4 does not wait on M5.** Neither `derive_fusion_legality` (`:922-967`) nor `derive_obligations` (`:1063-1163`) resolves an index-access capability, consults a realization law, or reaches the request boundary. This ticket is independent of [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md).

**Fact — this is p2 rather than p1, and the reason is the board.** The concatenate's role ticket is p1 because two live p1 decode tickets sit above it. The slice's two consumers — [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md) and [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) — are both p2 and both work at the IR and reference layers, so neither depends on a fusion role.

## What the work is

Register `slice_f32_op()` under `FusionOperationRole::CoordinateRelation` in `FusionNumericalCapabilities::governed`, with a comment stating the derivation rather than citing the record.

Extend the `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` (`fusion_legality.rs:1187-1189`) to the slice key. This arm is deliberately closed over exact keys so that each addition is decided rather than inherited, and the decision here is that the arm's own soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — transfers to a selection that introduces no multiply, no add, and no adjacency between them. Not extending it is not free: under a contraction-permitting contract a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1113-1116`) and `first_unknown` makes the whole candidate unknown.

Repair the `UNPLANNED_OPERATIONS` doc comment (`crates/tiler-compiler/src/policy.rs:789-810`). It explains the BF16 entries and the concatenate entry and says nothing about `tiler::slice-f32@1`, which was added to the list without its reason. The reason is the concatenate's: the family performs no arithmetic, so there is no dimension a capability row could list. This is folded here rather than filed separately because it is one comment in a file this ticket already edits.

Prove each new path can fail. A deliberate perturbation must show a slice-bearing region reaching `Unknown` when the role is removed — `governed_without` (`:357-362`) exists for exactly this — and a second showing the contraction obligation's outcome under a contraction-permitting contract with and without the arm extension.

Confirm on the merged tree whether the pinned explain digest at `crates/tiler-compiler/src/explain.rs:4054` moves. The record's reading is that it does not — `ExplainWriter::new` folds only `FusionNumericalCapabilities::governed().provider()` (`explain.rs:1219-1235`), not the role table, and the digest is a request-subject value whose concatenate-era movements were caused by the *semantic registry snapshot* that a `tiler-compiler` role addition does not touch — but that is an inference and must be observed rather than inherited.

## Explicit non-goals

- Any index-access lowering. That is [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md), and this ticket must not wait on it.
- Lifting the request boundary. A slice program is refused under `operation-set` because the region vocabulary's `LogicalAccess` cannot spell the family's access relation (`crates/tiler-compiler/src/request.rs:4898-4922`), which is the same state the two existing coordinate relations are in and is [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md)'s subject. Registering the role neither lifts it nor depends on it, and the matrix row must not be written as though it did.
- An `OperationNumericalCapability` row. The family performs no arithmetic, so there is no dimension a row could list; the entry in `UNPLANNED_OPERATIONS` stays until a physical realization exists.
- A seventh `FusionOperationRole` variant or a fifth `FusionRegionStructure` count.
- Anything about the strided or symbolic relations, which the key does not admit.

## Closes when

The role is registered, the contraction arm is decided explicitly, a slice-bearing region derives `Legal` with the nine obligations discharged, both deliberate failure perturbations are shown to fail, the `UNPLANNED_OPERATIONS` comment names its fifth entry, and the matrix's `Sub-tensor selection` row records R5 with its evidence and without claiming request-boundary reachability.

## Graph maintenance

- `contracts/navigation` is declared because delivering R5 moves the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Sub-tensor selection` rung and its next-column text, exactly as [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) declares it for the same reason.
- The scoping record owns the derivation and this ticket owns the rung. Do not restate the elimination here.
