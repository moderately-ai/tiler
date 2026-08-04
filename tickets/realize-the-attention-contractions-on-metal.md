---
id: realize-the-attention-contractions-on-metal
title: Realize the attention score and value contractions on Metal
status: todo
priority: p1
dependencies: [admit-the-attention-contraction-structures, realize-the-tiled-contraction-schedule-and-its-metal-emission, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, plan-the-materialized-attention-decomposition, admit-reassociated-contraction-schedule-alternatives, scope-causal-structure-aware-attention-schedules, implement-parallel-reduction-strategies]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, attention, language-model]
---
## User-visible outcome

The two attention contractions become scheduled Metal kernels whose results are bit-identical to the reference evaluator — and the conformance row's value contraction is realized by the schedule that is *correct there*, not by the one that is fastest elsewhere.

## Why the conformance row selects a different realization

**Measurement — from [the L3 elimination](../docs/research/scheduling/first-metal-contraction-realizations.md).** The surviving strict realization, `tiled`, refuses a contracted extent that is not a multiple of 16 rather than padding it. Structure 2's contracted extent is the static 128 and always passes. **Structure 3's contracted extent is `S`**:

| Row | `S` | `tiled` admissible for structure 3? |
| --- | --- | --- |
| C1 prefill | 10 | **no** |
| C1 decode, steps 1–8 | 11 … 18 | only at `S = 16` |
| B1-a prefill | 128 | yes |
| B1-a decode | 129 … 256 | at 8 of the 128 steps |
| B1-d prefill | 8,192 | yes |

**Inference — so `direct` is the only strict realization covering the conformance row's value contraction**, and this refusal fires on the workload's most-run shape rather than on a hypothetical one. A plan that selected `tiled` for structure 3 would need a per-step routing decision over `S mod 16`, which is the first place in this workload where a validity guard and a profitability route are both genuinely required.

## Evidence prerequisite

**Measurement — the realization elimination, restated for these structures.** `direct` and `tiled` are attributed uniquely to the strict fold and consume no permission; `ksplit_contiguous` needs reassociation and `ksplit_strided` needs reassociation *and* permutation; `simdgroup` delivers a fused multiply-add where ADR 0015's contraction dimension is Forbidden **and** seeds its accumulator at `+0.0` where the profile declares no seed; `opaque_mps` is refuted against all twenty-two named topologies with a shape-dependent evaluation on one device.

**Measurement — no cell of either structure has been timed at any shape.** L3 deliberately left the batched forms unmeasured. This ticket's numbers are the first, and nothing in the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md) extrapolates structure 1's table onto them: multiply-accumulate counts are arithmetic, and two schedules with the same count differ by an order of magnitude in L3's own measured table.

**Inference — the arithmetic weight inverts between the two bounded rows.** The block's four projections perform `T · 6,291,456` multiply-accumulates and its two attention contractions perform `4,096 · T²` at `S = T`; they are equal at `T = 1,536`. At C1 the projections dominate 154×, at B1-a 12×, at B1-b 3×; at B1-c the attention contractions dominate 1.3× and at B1-d 5.3×. So a measurement taken only at C1 would rank these kernels on the row where they barely matter.

## Required delivery

- **`direct` for both structures, unconditionally**, bit-identical to the reference evaluator at the C1 prefill extents and at least one B1 extent. It has no structural precondition beyond a positive contracted extent, and it is what makes the conformance row realizable at all.
- **`tiled` for both structures, gated on its own precondition as a typed refusal.** The refusal must name the realization's precondition and the observed extent, and it must be demonstrated firing at `S = 10` before it is trusted — a precondition that has never rejected anything is not a precondition.
- **No K-padding, and the reason recorded.** Padding structure 3's contracted extent to a multiple of 16 would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires, and here the padding is measurably wrong in the same way the masked contributors are: the padded contributors are `+0.0 × v`, whose sign follows `v`, and the fold's seed is the first product rather than `+0.0`.
- **A schedule set bounded by the absence of barriers.** [IR](../docs/ir.md)'s kernel verifier admits no barrier under the implemented zero-synchronization schedule profile, so a reduction spanning more than one SIMD group has no synchronization construct to be built from. State which of the five reduction implementations each realization uses and why the others are unavailable, rather than leaving the absence to be discovered.
- **Timings for both structures at the C1 prefill row and at least two B1 rows**, under L3's own procedure — settled minimum over interleaved A/B rounds, round 0 reported separately, spread stated — so the D-A-versus-D-B comparison in [`plan-the-recomputing-attention-decomposition`](plan-the-recomputing-attention-decomposition.md) has a baseline that exists.
- **A refusal for every realization whose reduction topology is unstated or uncovered**, naming reassociation, permutation, or the absent distributivity separately, because those are three different explanations.

## Non-goals

The reassociated split alternatives, which are [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md)'s; the simdgroup route, which [`qualify-the-simdgroup-matrix-contraction-realization`](qualify-the-simdgroup-matrix-contraction-realization.md) owns and which does not survive the governed contract; any opaque provider; any schedule that skips masked contributors, which is [`scope-causal-structure-aware-attention-schedules`](scope-causal-structure-aware-attention-schedules.md)'s and is forbidden until it lands; and the cover and cost decisions, which are [`plan-the-materialized-attention-decomposition`](plan-the-materialized-attention-decomposition.md)'s.

## Closes when

Both structures have a `direct` realization bit-identical to the reference at the C1 prefill extents, `tiled` is available where its precondition holds and demonstrated refusing where it does not, and both are timed at the C1 row and at least two B1 rows with the measurement boundary stated.
