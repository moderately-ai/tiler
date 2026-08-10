---
id: realign-the-l1-and-l2-operation-family-standing-after-post-close-matrix-moves
title: Realign L1 and L2 operation-family standing after post-close matrix moves
status: done
priority: p2
dependencies: []
related: [refresh-the-l1-operation-family-standing, refresh-the-l2-derivation-operation-family-standing, admit-a-fusion-role-for-the-sub-tensor-selection-slice, register-the-softmax-realization-law, refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause]
scopes: [research/program-planning, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

[`first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md) (L1) and [`transformer-operation-and-shape-surface.md`](../docs/research/shapes/transformer-operation-and-shape-surface.md) (L2) again state operation-family rungs **and bounds** that agree with the roadmap family-state table, so nothing downstream derives capability from post-close stale cells.

## The finding, from the 2026-08-10 ticket audit of `refresh-the-l1-operation-family-standing`

**Fact.** [`refresh-the-l1-operation-family-standing`](refresh-the-l1-operation-family-standing.md) closed on 2026-08-06 with L1 standing aligned to the matrix at delivery base `428d201d` / correction commit `67156c5e`. That close is historically warranted for the three-site R1/R2-floor repair. It is **not** a true statement that the close condition still holds against the matrix after later landings.

**Fact — two movers landed after that close.**

1. **Slice R5.** [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md) (`done`, 2026-08-07) admits `FusionOperationRole::CoordinateRelation` for `tiler::slice-f32@1`. The roadmap cell is **R5** for the F32 literal-offset family (strided and symbolic stay **R1**). Live L1 still says **R4** and `R5 awaits a fusion role`, and still bounds "nothing … fuses". Live L2 still restates Slice **R4** / `R5 awaits a fusion role` from the 2026-08-06 L2 refresh (`80a48705`), which copied L1 before the fusion-role landing.

2. **Softmax bound prose.** [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md) registered `IndexRealizationLaw::StagedSoftmaxF32` the same evening as the L1 close; [`refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause`](refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause.md) recorded that the governed maximum key and multi-reader sequence rule landed, and that refusal moved from `operation-set` to `UnsupportedCapability { rule: "missing-capability" }`. Live L1 Softmax bound still says no `IndexRealizationLaw` is registered, refuses under `operation-set`, and that two named prerequisites remain. Rung **R5** itself still matches.

**Fact — L1 handoff of 2026-08-09 did not re-read bounds.** The `Current handoff correction — 2026-08-09` only discharged the L2 owner pointer; it did not re-verify L1 or L2 family bounds against the matrix after the Slice and Softmax matrix moves.

**Fact — what must survive.** Other family rungs and bounds from the 2026-08-06 L1 table that still match the matrix; reductions-beyond-strict-sum **R2**; cast-and-convert **R2**; the BF16→F32 ingestion paragraph and its 2026-08-02 Correction; L8's non-restatement claim; the composition claim that no composition of these families executes on the Tiler side. Rung numbers alone are not the claim — each moved row's **bound** must be stated.

## Source-first Fact audit — 2026-08-10

Performed at exact base `86db3609463b7b4d52a4532d01a970b649eebcdb`, before either research record was edited. The complete owning ticket, both complete target records, the complete L8 record, the governing roadmap sections and complete Slice/Softmax cells, the five related completed tickets, and the relevant semantic construction, law/refinement, fusion-legality, request-recognition, lowering, physical-plan, policy, and boundary-test sites were read rather than inferred from searches.

1. **Verified — historical close and later drift.** [`refresh-the-l1-operation-family-standing`](refresh-the-l1-operation-family-standing.md), under the source anchors `Outcome — 2026-08-06` and `Fact audit — 2026-08-10 (post-close matrix drift)`, records both the warranted original delivery and the later Slice/Softmax drift. L1's `Operation and shape surface handed to L2` table and L2's `Family-by-family disposition` table still carried those two stale bounds at this base.
2. **Verified — both movers, with their distinct live refusal layers made explicit.** `FusionNumericalCapabilities::governed` registers `slice_f32_op()` as `FusionOperationRole::CoordinateRelation`, and `slice_role_tests::a_region_holding_a_selection_derives_legality_instead_of_failing_closed` proves the formed candidate legal while role withdrawal returns `unsupported-operation-capability`. `SliceSelectionError::diagnostic_code` pins `slice.selection.strided-window-unsupported` and `slice.selection.symbolic-offset-unsupported`. Separately, `recognize_structural_read` admits only Reindex and Broadcast, while `the_family_region_sequence_query_agrees_with_the_resolved_law` proves Slice carries no sequence law, so the Slice request still refuses under `operation-set`. For Softmax, `IndexRealizationLaw::staged_softmax_f32()` is registered and the same query proves sequence recognition; `a_softmax_program_is_refused_for_want_of_an_installed_lowering` asserts all five statable contracts return `missing-capability`; and `staged_plan` has only a `StagedRootMeanSquareScaleF32` arm, with every other law failing closed to `None`.
3. **Verified — the 2026-08-09 handoff was pointer-only.** L1's source anchor `Current handoff correction — 2026-08-09` changes the L2 ownership statement and says the remaining capability limits are unchanged; it does not re-derive either moved bound.
4. **Verified — preservation set.** L1's reduction and cast rows and its source anchor `Correction — 2026-08-02` remain independent of these moves. L8's anchors `It moves no row` and `no operation family moved a rung` describe what L8 itself delivered, not a restatement of the current matrix. L1's `No composition of these families executes` and L2's closing corrected-standing inference remain live. No Fact is false and the ticket purpose is unchanged.

## The work

1. Re-read in full the roadmap family-state cells for Slice and Softmax (and any cell a live L1/L2 bound still names) at the repair base — bounds, refusal vocabulary, remaining walls, not rungs alone.
2. In L1 standing: Slice row to **R5** F32 literal-offset (strided/symbolic **R1**); drop `R5 awaits a fusion role`; bound must not claim fusion is impossible — state the admitted `CoordinateRelation` role and what R5 still does not deliver (lowering / emission / current request-boundary refusal). Softmax bound: replace lawless / `operation-set` / two-open-prerequisites prose with the current roadmap ceiling (`StagedSoftmaxF32` registered as the roadmap states, occurrence recognized, refusal `missing-capability` / no installed staged lowering provider, plus whatever walls the full Softmax cell still holds open). Update any *What remains open* bullet that still restates Slice at R4. Follow L1's dated-**Correction** convention quoting the false clauses.
3. In L2: the same Slice R4 / `R5 awaits a fusion role` restatements, and any Softmax bound language still copied from the pre-law picture. Same dated-Correction discipline; keep disposition, BF16, gather, and other must-survive content.
4. Optional consistency only: re-read L1's "five family groups" / status correction text after the Slice rung change; do not invent a new floor claim.

No crate, key, identity, or public-boundary change. No matrix rung movement in this ticket — only research-document standing prose.

## Closes when

L1's and L2's operation-family standing agree with the roadmap's family-state table, verified by a full read of both documents and every named cell, with each moved row's **bound** stated rather than only its rung.

## Outcome — 2026-08-10

The two live standing records now agree on the moved cells and preserve the unrelated workload, operation, dtype, and composition conclusions.

- **L1 Softmax:** the live R5 row substitutes registered `StagedSoftmaxF32`, successful occurrence recognition, the five-contract `missing-capability` refusal, no installed lowering capability, and no physical Softmax `staged_plan` arm for the retired no-law / `operation-set` / two-prerequisite bound. A dated correction quotes and classifies the retired clauses.
- **L1 Slice:** the live row substitutes R5 and `CoordinateRelation` for R4 / `R5 awaits a fusion role`, names the two R1 diagnostic codes, and keeps the narrower live wall: no Slice sequence law, no request recognition, `operation-set`, no lowering or emission, and no `VerifiedKernel`. The *What remains open* restatement now says literal-offset Slice R5 without changing its no-composition conclusion.
- **L2 Softmax:** the live family row and the later obligations correction carry the same law/recognition/lowering/physical-plan boundary. The original 2026-08-06 table-cell quotations remain historical evidence, and a 2026-08-10 correction classifies every retired lawless phrase.
- **L2 Slice:** the live no-slice paragraph now distinguishes R5 fusion legality from the absent sequence law and request/physical walls, while preserving the strided/symbolic R1 refusals and the derivation that this workload's layer interior asks for no Slice.

**Residual census.** Across the two records, searches for `No/no IndexRealizationLaw`, `MissingRealizationLaw`, `R5 awaits a fusion role`, `Nothing/nothing lowers, fuses, or emits`, and `Slice at R4` return only five dated correction sites: L1's table correction and current-standing correction, plus L2's Softmax table correction, Slice correction, and Softmax obligations correction. Each hit quotes or names retired text and immediately states its replacement. The live positive census finds `StagedSoftmaxF32` / `missing-capability` in both live Softmax cells and their corrections, and `CoordinateRelation` plus both Slice diagnostic codes in both live Slice statements.

**Scope and gate.** The delta is confined to `research/program-planning`, `research/shapes`, and shared `project/tickets`; no crate, API, key, identity, matrix-policy, or status path changes. The fresh green `make full` on parent `a1b373f2a2da8664b60fa6620acd19ff22326041` carries because this delta touches only the two research Markdown records and this ticket, none of the repository's gate-invalidating paths. `make citations`, `tkt lint --format json`, `git diff --check`, and exact-base `tkt guard` are rerun on this delta.
