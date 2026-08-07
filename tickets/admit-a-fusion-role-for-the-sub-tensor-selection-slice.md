---
id: admit-a-fusion-role-for-the-sub-tensor-selection-slice
title: Admit a fusion role for the sub-tensor selection slice
status: done
priority: p2
dependencies: []
related: [scope-the-sub-tensor-selection-fusion-role, admit-a-fusion-role-for-the-sequence-extension-concatenate, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families, admit-the-sub-tensor-selection-family, lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, slice]
---
## User-visible outcome

A cover region holding `tiler::slice-f32@1` beside another operation derives its fusion legality from a declared role instead of failing closed to `Unknown`, so the support matrix's R5 criterion is met for the sub-tensor selection family rather than skipped.

## Why this exists

**Fact — the family resolves to no legality at all today.** `FusionNumericalCapabilities::governed` (`crates/tiler-compiler/src/fusion_legality.rs:339-606` at base `2180ece3`) registers **fourteen** keys and the slice is not among them; `derive_member` returns `Ok(None)` for an unregistered family (`:1338-1340`) and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:1238-1244`). Region formation holds no operation allowlist — `RegionGraph`'s construction reads only `definition.effect()` from a reached definition (`crates/tiler-compiler/src/region.rs:914-947`) — so a program containing a slice does form candidates that reach that state.

**Correction, 2026-08-07 — the key count above was false and every line citation in this ticket was stale.** This ticket was written against `3cca2a3f`, where the table held nine keys. The three BF16 families landed under `establish-bf16-optimizer-legality`, and the concatenate's and the contraction's roles landed under their own tickets, so the table holds fourteen at this ticket's base: `grep -c 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns `14`. The count was only ever used to say that the slice is absent, which is true at fourteen as it was at nine, so no conclusion below changes. Every line number in this ticket has been re-read against `2180ece3` and corrected in place; a reader following the originals would have landed in unrelated code.

**Fact — the elimination is done and one candidate survived.** [Sub-tensor selection fusion role](../docs/research/indexing/sub-tensor-selection-fusion-role.md) tests four candidates — no role, `ValueSource`, a new seventh role, and `CoordinateRelation` — against what `derive_obligations` decides at `3cca2a3f`, and only `CoordinateRelation` survives. `ValueSource` fails on the role doc's own distinction, at `fusion_legality.rs:262-269` on this base; a seventh role fails because non-surjectivity derives no obligation differently and a fifth `FusionRegionStructure` count would move the content identity of every region the vocabulary can already encode (`:784-809`, with the reason stated on the two reduction variants and the `coordinate_relations` field).

**Fact — M4 does not wait on M5.** Neither `derive_fusion_legality` (`:1193-1260`) nor `derive_obligations` (`:1364-1476`) resolves an index-access capability, consults a realization law, or reaches the request boundary. Re-read in full at `2180ece3` and confirmed. This ticket is independent of [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md).

**Fact — this is p2 rather than p1, and the reason is the board.** The concatenate's role ticket is p1 because two live p1 decode tickets sit above it. The slice's two consumers — [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md) and [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) — are both p2 and both work at the IR and reference layers, so neither depends on a fusion role.

## What the work is

Register `slice_f32_op()` under `FusionOperationRole::CoordinateRelation` in `FusionNumericalCapabilities::governed`, with a comment stating the derivation rather than citing the record.

Extend the `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` (`fusion_legality.rs:1558-1561`) to the slice key. This arm is deliberately closed over exact keys so that each addition is decided rather than inherited, and the decision here is that the arm's own soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — transfers to a selection that introduces no multiply, no add, and no adjacency between them. Not extending it is not free: under a contraction-permitting contract a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1417-1422`) and `first_unknown` makes the whole candidate unknown.

Repair the `UNPLANNED_OPERATIONS` doc comment (`crates/tiler-compiler/src/policy.rs:1092-1133` at this base, not `:789-810`). It explains the BF16 entries and the concatenate entry and says nothing about `tiler::slice-f32@1`, which was added to the list without its reason. The reason is the concatenate's: the family performs no arithmetic, so there is no dimension a capability row could list. This is folded here rather than filed separately because it is one comment in a file this ticket already edits.

Prove each new path can fail. A deliberate perturbation must show a slice-bearing region reaching `Unknown` when the role is removed — `governed_without` (`:628-633`) exists for exactly this — and a second showing the contraction obligation's outcome under a contraction-permitting contract with and without the arm extension.

Confirm on the merged tree whether the pinned explain digest moves. It sits at `crates/tiler-compiler/src/explain.rs:3883` and reads `de9ad4cc087697d8` at this base, not `:4054`. The record's reading is that it does not move — `ExplainWriter::new` folds only `FusionNumericalCapabilities::governed().provider()` (`explain.rs:1295-1306`), not the role table, and the digest is a request-subject value whose concatenate-era movements were caused by the *semantic registry snapshot* that a `tiler-compiler` role addition does not touch — but that is an inference and must be observed rather than inherited.

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

## Outcome

**Fact — the family no longer fails closed.** `FusionNumericalCapabilities::governed` registers `tiler::slice-f32@1` under the existing `FusionOperationRole::CoordinateRelation`, so a formed region holding a selection beside a multiply derives `FusionLegality::Legal` with all nine obligations discharged where it previously returned `Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"`. The registration comment states the derivation in place — why a role at all, why not `ValueSource`, and why the family's own non-surjectivity does not earn a seventh role — rather than citing the scoping record.

**Fact — the contraction arm was decided, not inherited.** `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` arm now names the slice key beside `reindex`, `broadcast`, and `concatenate`, on the transfer this ticket states. The one way the transfer could have failed is answered in the arm's own comment: the argument is that inserting a pure data movement between two adds introduces no product to fuse, and it turns on the movement introducing no *operation* rather than on its being total, so a map that reads strictly less of its source than a reindex does not weaken it. No seventh role, no fifth `FusionRegionStructure` count, no `OperationNumericalCapability` row, and no index-access lowering were added.

**Measurement — three deliberate perturbations, each of the thing guarded rather than of an assertion, run on this branch's tree one at a time.** (A) Registering `FusionOperationRole::ValueSource` instead: all three of `fusion_legality::slice_role_tests` fail — the role assertion reports `left: Some(ValueSource)`, the structure assertion reports `coordinate_relations` `left: 0` against `right: 1`, and the contraction test reports `Unknown(FusionUnknown { obligation: ArithmeticContraction, reason: "unrealized-contraction", ... })`. (B) Removing the `roles.insert` line entirely: all three fail, the region test reporting `Unknown(FusionUnknown { obligation: OperationCapabilitiesResolved, reason: "unsupported-operation-capability", ... })` where `Legal` is asserted. (C) Dropping the slice from the contraction arm's match guard and nothing else: exactly one test fails, at the contraction assertion — `left: Unknown { reason: "unrealized-contraction" }`, `right: Discharged` — and the other two stay green, which is what makes the arm extension separately load-bearing. Under the strict governed contract (C) is invisible, contraction being `Forbidden` so the obligation discharges by normative guarantee regardless, which is why the test states it against `StrictF32NumericalContract::governed_relaxed()` and asserts that contract does not forbid contraction before relying on it.

**Measurement — the pinned explain digest did not move, observed rather than inherited.** `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` passes unchanged; `explain.rs` still pins `"tiler-explain-v7 request=de9ad4cc087697d8"` at `:3883`, and `git diff` reports the file untouched by this branch. No identity-domain step was taken and none was owed. **Inference:** this matches the record's reading — `ExplainWriter::new` folds the fusion provider identity, not the role table, and `GOVERNED_PROVIDER_REVISION` is unmoved.

**Fact — the `UNPLANNED_OPERATIONS` comment now names its fifth entry.** It gave a reason for the three BF16 rows and for the concatenate and was silent about `tiler::slice-f32@1`. The reason is the concatenate's — the family performs no arithmetic, so there is no dimension a capability row could list — and the new paragraph also records that holding a `CoordinateRelation` fusion role is not in tension with carrying no capability row, for the identical reason the concatenate's paragraph gives.

**Fact — the matrix rung moved.** `Sub-tensor selection` reads `R5 for the F32 literal-offset family; the strided and symbolic forms stay R1`, with the R5 evidence Fact, the vacuous-versus-substantive discharge stated explicitly, the digest observation dated, and a closing Fact naming what R5 does *not* deliver: no lowering, no emission, no `VerifiedKernel`, and an unchanged `operation-set` refusal at the request boundary.

**The maturity claim, stated exactly.** A role is admitted into the capability table, and a *formed candidate* containing a selection derives `Legal`. That is two of the four distinct claims the brief separates. No region containing a selection reaches a `VerifiedKernel`, and nothing here is device-verified. The evidence tier is a tested guarantee over the `fusion_legality` authority driven directly, not through a compile — the request boundary still refuses the family under `operation-set` because the region vocabulary's `LogicalAccess` cannot spell a selection's access relation, which is [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md)'s subject and was never this ticket's.

**Public boundary.** None. `FusionOperationRole` is a private enum in a private module, `FusionNumericalCapabilities` is `pub(crate)`, and `fusion_legality` carries no `pub` item and no re-export, so nothing publicly reachable was added or altered and no ADR 0075 draft label is owed.

**Out of scope, flagged rather than absorbed.** [Sub-tensor selection fusion role](../docs/research/indexing/sub-tensor-selection-fusion-role.md) carries a `2026-08-06` restatement saying the role table holds **eleven** keys; it holds fourteen, the three BF16 rows having landed since. That file maps to `research/indexing`, which this ticket does not declare, so the stale count is reported rather than repaired. It changes no conclusion of that record — the count is used only to say the slice is absent — and the record's own `disposition: pending` and `implementation_status: not-started` frontmatter is now behind this landing.
