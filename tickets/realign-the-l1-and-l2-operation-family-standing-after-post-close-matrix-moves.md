---
id: realign-the-l1-and-l2-operation-family-standing-after-post-close-matrix-moves
title: Realign L1 and L2 operation-family standing after post-close matrix moves
status: in-progress
priority: p2
dependencies: []
related: [refresh-the-l1-operation-family-standing, refresh-the-l2-derivation-operation-family-standing, admit-a-fusion-role-for-the-sub-tensor-selection-slice, register-the-softmax-realization-law, refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause]
scopes: [research/program-planning, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: terra-l1-l2-standing
lease_expires_at: 1786408310
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

## The work

1. Re-read in full the roadmap family-state cells for Slice and Softmax (and any cell a live L1/L2 bound still names) at the repair base — bounds, refusal vocabulary, remaining walls, not rungs alone.
2. In L1 standing: Slice row to **R5** F32 literal-offset (strided/symbolic **R1**); drop `R5 awaits a fusion role`; bound must not claim fusion is impossible — state the admitted `CoordinateRelation` role and what R5 still does not deliver (lowering / emission / current request-boundary refusal). Softmax bound: replace lawless / `operation-set` / two-open-prerequisites prose with the current roadmap ceiling (`StagedSoftmaxF32` registered as the roadmap states, occurrence recognized, refusal `missing-capability` / no installed staged lowering provider, plus whatever walls the full Softmax cell still holds open). Update any *What remains open* bullet that still restates Slice at R4. Follow L1's dated-**Correction** convention quoting the false clauses.
3. In L2: the same Slice R4 / `R5 awaits a fusion role` restatements, and any Softmax bound language still copied from the pre-law picture. Same dated-Correction discipline; keep disposition, BF16, gather, and other must-survive content.
4. Optional consistency only: re-read L1's "five family groups" / status correction text after the Slice rung change; do not invent a new floor claim.

No crate, key, identity, or public-boundary change. No matrix rung movement in this ticket — only research-document standing prose.

## Closes when

L1's and L2's operation-family standing agree with the roadmap's family-state table, verified by a full read of both documents and every named cell, with each moved row's **bound** stated rather than only its rung.
