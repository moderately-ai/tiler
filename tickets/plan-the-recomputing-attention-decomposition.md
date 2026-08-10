---
id: plan-the-recomputing-attention-decomposition
title: Plan the recomputing attention decomposition that never materializes the scores
status: todo
priority: p2
dependencies: [integrate-the-attention-block-into-the-runtime, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, plan-the-materialized-attention-decomposition, enumerate-the-split-reduction-on-the-planning-frontier, implement-analytical-component-cost-model, decide-whether-to-admit-a-distributivity-permission, scope-causal-structure-aware-attention-schedules, reconcile-the-first-attention-planning-record-with-landed-fusion-roles-and-budgets]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, fusion, feasibility, attention, language-model, class-generic-capability]
---
## User-visible outcome

A second complete decomposition of the attention chain that materializes **no** `[8, 2, T, S]` tensor — only two `[8, 2, T]` statistics — so the transient requirement is low enough that a target profile can accept it where the materialized plan's residency predicate rejects or stays `Unknown` under D-11, without consuming any numerical permission. At the B1-d prefill row its design arithmetic is 1,150,287,880 bytes, against 18,329,108,488 for the historical unfused materialized design and 5,444,206,600 for that design's one-tensor handoff case. The materialized implementation ticket must supply the actual current comparison; this ticket does not keep calling the proposal-era unfused form the reachable plan after its fusion roles landed.

## What it is, precisely

**Proposal — from the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md).** A two-stage `KernelSubprogram`. Stage 1 computes the scores, applies the scale and the mask, and materializes only the per-row maximum `m` and denominator `d`, each `[8, 2, T]`. Stage 2 **recomputes** the score for each `(g, r, t, s)`, forms `p = Exp(s − m) · (1/d)` exactly as the pinned softmax formula does, and accumulates `p · v` in a strict ascending fold over `s`.

**Inference — it consumes no numerical permission, and that is the whole reason it is written this way rather than as a flash-attention kernel.** Every probability is formed by the same three roundings the reference performs, in the same order; both reductions run over the same contributor sequences the materialized plan uses. A recomputation is a physical implementation of one logical DAG, not a new logical equivalence group, so nothing about it is a relaxation.

**Inference — the online single-pass form is a different arithmetic and is rejected, not deferred.** It computes `O = (Σ_s e_s · v_s) · r` where the reference computes `O = Σ_s (e_s · r) · v_s`. Factoring the common multiplier out of the sum consumes **distributivity**, the third numerical dimension, for which [Numerical semantics](../docs/numerical-semantics.md#distributivity-is-outside-the-order-contract) admits no permission, [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) is the accepted classification, and [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) is the product decline. The online-softmax rescaling fold additionally consumes **elementary-function identity**, the fourth numerical dimension ([ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md)): the exp telescoping through a running maximum is a rewrite through a registered elementary function, not only a ring regroup of `+` and `*`. A multi-dimension refusal must name every missing dimension (ADR 0080 item 5; ADR 0101 decision 6); naming only distributivity, or stating reassociation as the sole second ground, is incomplete. This ticket is what survives in its place.

**Correction — 2026-08-10.** The L4 proposal-era account of the online form named distributivity plus reassociation. ADR 0101 (2026-08-06) requires elementary-function identity to be named alongside distributivity for flash/online refusal, so this ticket is not the sole authority for a reassociation-only second ground.

## Evidence prerequisite

**Fact — the transient requirement, from the design's arithmetic.** The statistics are `2 · 16 · T · 4` bytes: 1,280 at C1 prefill and 1,048,576 at B1-d, against the 4,294,967,296-byte tensor they replace.

| Row | `T = S` | Materialized, historical unfused | Materialized, design best case | This plan | Ratio against historical unfused |
| --- | --- | --- | --- | --- | --- |
| C1 prefill | 10 | 1,101,208 | 1,082,008 | 1,076,888 | 1.02× |
| B1-b prefill | 512 | 123,207,688 | 72,876,040 | 56,164,360 | 2.2× |
| B1-c prefill | 2,048 | 1,310,720,008 | 505,413,640 | 237,240,328 | 5.5× |
| B1-d prefill | 8,192 | 18,329,108,488 | 5,444,206,600 | 1,150,287,880 | **15.9×** |

**Fact — the cost is one extra evaluation of the score contraction.** Structure 2 performs `16 · T · S · 128` multiply-accumulates and this plan performs it twice, which is an additional 1.374 × 10¹¹ per layer at B1-d.

**Inference — which plan wins is unmeasured and this ticket is where it stops being unmeasured.** The design deliberately makes no claim: it holds no timing of either attention contraction at any shape, and multiply-accumulate counts are arithmetic rather than measurements. That is why this ticket depends on the materialized plan running first.

**Fact — the stage handoff needs no *program-level kernel barrier* because the pass boundary is the dispatch boundary; the precedent is in the tree.** `crates/tiler-ir/src/program/model.rs` records for the split reduction that "a split reduction needs no barrier because the pass boundary *is* the dispatch boundary", and `crates/tiler-compiler/src/program.rs` builds that ordinary `Data` edge at the site. That claim is program-stage ordering only: it does not decide kernel-internal synchronization, and the schedule vocabulary admits `ReductionTopology::CooperativeWorkgroup` / `SynchronizationPoint` (same separation already corrected on [`plan-the-materialized-attention-decomposition`](plan-the-materialized-attention-decomposition.md)). **Inference —** this plan's statistics handoff is the same program-stage mechanism at a different subject, which is also why [`enumerate-the-split-reduction-on-the-planning-frontier`](enumerate-the-split-reduction-on-the-planning-frontier.md) is the shape to follow rather than to reinvent.

## Required delivery

- **The two-stage subprogram as a frontier candidate**, with a typed partials declaration for the statistics, a `Data` dependency making them visible, and the reduction occurrence claimed exactly once so the graph is not double-covered.
- **Proof that stage 2's probabilities are bit-identical to the materialized plan's**, at the C1 prefill shape and at least one B1 shape. This is the correctness claim the whole plan rests on: if recomputing the score does not reproduce the score, the plan is a different computation and the permission argument collapses.
- **The same masked-contributor discipline.** Recomputation does not license skipping masked positions; that is [`scope-causal-structure-aware-attention-schedules`](scope-causal-structure-aware-attention-schedules.md)'s question and it is forbidden until that lands.
- **A measured comparison against the materialized plan** at the C1 prefill row and at least two B1 rows, under the same procedure, with the correctness oracle compared before the speed. Report regressions, variance, and environment; a row where this plan is slower is a result, not a failure.
- **The feasibility crossover stated as a predicate, not a preference.** Where the materialized plan's residency predicate rejects and this one's does not, the choice is feasibility; where both are feasible, it is cost. Those are different findings and the explain output must not merge them.

## Non-goals

The online single-pass form, on the multi-dimension grounds above (distributivity and elementary-function identity; ADR 0080, ADR 0095, ADR 0101). Any distributivity permission — [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) owns that closed choice and this ticket takes no position. A three-pass variant that also recomputes for the maximum. The decode step. Making this the default: it is a second candidate on the frontier, and preference belongs to measured calibration.

## Closes when

The subprogram is a checked frontier candidate whose stage-2 probabilities are bit-identical to the materialized plan's, it is measured against that plan at three rows, and the feasibility-versus-cost boundary between them is explicit in explain output.
