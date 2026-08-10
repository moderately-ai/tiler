---
id: retain-the-c1-attention-block-conformance-evidence
title: Retain the C1 attention-block conformance evidence
status: todo
priority: p2
dependencies: [integrate-the-attention-block-into-the-runtime, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, retain-contraction-conformance-evidence, retain-the-qwen-conformance-reference-logit-fixture, admit-the-softmax-family, scope-causal-structure-aware-attention-schedules]
scopes: [implementation/reference, implementation/compiler, contracts/numerics, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, conformance, attention, numerics, language-model, class-conformance-fixture]
---
## User-visible outcome

A later change to the block's schedule, emitter, or toolchain is a *failure* rather than a drift, because the exact bits the C1 attention block produces today are retained and compared — including the three cases where a plausible reimplementation would silently differ.

## Why three cases matter more than the corpus size

**Inference — from the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md).** A broad corpus of ordinary attention rows would pass under several materially different implementations, because the differences this program admits are concentrated at boundaries an ordinary row never reaches. Three cases carry almost all of the discriminating power, and one of them is not even reachable from the workload:

1. **The masked-position signed zero.** At query position 0 the probability row is `0x3f800000` followed by nine exact `0x00000000`. With a negative `v` at the attended key, the value contraction's seed — the first product, since the profile declares no `initial` — is `0x80000000`, and the completed strict ascending fold is `0x00000000`. **Measurement**, retained by the [attention-block probe](../spikes/program-planning/attention-block-reference/README.md). A schedule that skipped masked contributors as a causal-structure optimization returns the other sign, and no ordinary row would notice.
2. **The fully masked row, which C1 cannot reach.** **Measurement — the finite mask fill and a `-inf` fill produce bit-identical results at all 1,600 C1 score elements**, because every masked argument drives `Exp` to exactly zero under both. The same comparison at a fully masked width-10 row returns uniform `0x3dcccccd` under the finite fill and ten `0x7fc00000` NaNs under `-inf`. **Inference — so a corpus that only ran C1 would pass with the wrong mask convention installed.** The synthetic row is the only case that tests decision D-1's answer at all.
3. **The row sum.** **Measurement — 111 of the C1 score tensor's 160 rows sum to exactly `0x3f800000` and 49 do not.** Whether a softmax row sums to exactly one in F32 is a per-row accident, so a check may assert neither unit nor non-unit as a universal, and one written from a single example alone is wrong on the others. **Correction — 2026-08-10.** The bare claim that [the L3′ record](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md)'s four-wide worked example sums to `0x3f7ffffe` is the pre-2026-08-01 attribution. After that correction the four-wide row splits: the **reference model** outputs sum to `0x3f7ffffe`; the **pinned formula** sums to exactly `0x3f800000` on that same row. The non-unit discipline under the pinned formula is carried by other L3′ widths (`[0.0, 2.0]` → `0x3f7fffff`, `[0.0, 1.0, 0.0]` → `0x3f800001`) and by the C1 111/49 census above — not by the four-wide pinned-formula sum.

## Required delivery

- **The retained record**: exact F32 bit patterns for the block's three outputs at the C1 prefill shape over a recorded operand seed, plus a SHA-256 over each, plus the per-stage intermediates the design's worked example names — the raw score row, the scaled row, the masked row, and the probability row at query head 0, position 2.
- **The three discriminating cases above**, each as a test that fails before the behaviour it protects exists. The mask-convention case is a synthetic fully masked row and is not part of the workload; state that it is deliberate coverage of an unreached case rather than a workload row.
- **The structural equivalences and their perturbations**, so that a later refactor of the structural families is caught: the rotary composition at 0 of 20,480 with its swap-removed and sign-reversed perturbations at 20,480; the grouped-query mapping at 0 differing with the `h mod 8` reading at 17,920.
- **The realization boundary, stated per row.** Every retained bit pattern names the exact host, offline toolchain, numerical realization, and selected schedule. A record whose boundary is implicit generalizes itself the first time someone reads it.
- **A `direct`-versus-`tiled` cross-check where both are admissible**, because two realizations attributed to the same topology must return identical bits and a divergence is a defect rather than a tolerance. At `S = 10` only `direct` is admissible, so the cross-check runs at a B1 extent and says so.
- **The link from the design record to the retained evidence**, so a reader can reach the bits from the claim.

**Correction — 2026-08-10 (partial delivery census).** The three discriminating cases and the structural rotary/GQA perturbations already land as ordinary host tests under other tickets; they are not open first-creation work on this ticket:
- masked-position signed zero — `a_masked_position_contributes_a_signed_zero_to_the_value_contraction` in `crates/tiler-reference/tests/causal_self_attention_block.rs` (landed under [`assemble-the-causal-self-attention-block-program`](assemble-the-causal-self-attention-block-program.md));
- fully-masked synthetic row (D-1) — `a_fully_masked_row_follows_the_pinned_formula_under_either_mask_convention` in `crates/tiler-reference/src/softmax/tests.rs` (landed under [`admit-the-softmax-family`](admit-the-softmax-family.md));
- row-sum non-assertion discipline — `a_rows_outputs_do_not_sum_to_exactly_one` (and related corpus comments) under the same softmax tests, with the C1 111/49 census restated on the pinned score-row test;
- rotary composition 0 of 20,480 with swap-removed and sign-reversed perturbations at 20,480 — `the_query_operand_matches_the_rotary_formula_at_sixteen_heads` in `crates/tiler-reference/tests/rotary_position_embedding.rs` (compose-rotary landing);
- GQA mapping 0 vs `h mod 8` at 17,920 — `crates/tiler-reference/tests/grouped_query_head_layout.rs` (admit-grouped-query-head-layout landing).

This ticket's residual is therefore: (a) SHA-256 digests of the three C1 block outputs with a complete host / offline-toolchain / numerical-realization / selected-schedule boundary after [`integrate-the-attention-block-into-the-runtime`](integrate-the-attention-block-into-the-runtime.md); (b) the B1 `direct`↔`tiled` cross-check where both are admissible; (c) a durable design→retained-evidence link to that surface. Do not re-create a second copy of the already-green host cases unless consolidating them into one named suite is deliberate and stated. Per-stage intermediate bits and whole-block recompute already exist on the ordinary test surface; digests with the complete Tiler realization boundary do not.

## Non-goals

A model-level tolerance or a whole-model comparison, which are [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md)'s and [`retain-the-qwen-conformance-reference-logit-fixture`](retain-the-qwen-conformance-reference-logit-fixture.md)'s. Any B1-row conformance retention — those rows exist to be measured, not retained. Portability: the digests are bound to one host and a mismatch elsewhere is expected and is not by itself a defect.

## Closes when

The C1 block's exact bits are retained with their complete boundary, the three discriminating cases each fail before their fix, and the structural perturbations are demonstrated differing.

**Correction — 2026-08-10.** The discriminating-case and structural-perturbation conjuncts are already satisfied on the ordinary host test surface (see the Required delivery census above). Status remains `todo` because the residual conjunction — complete-boundary digests of the three C1 outputs, the B1 `direct`↔`tiled` cross-check, and the durable design→retained-evidence link — is unmet, and the hard dependency [`integrate-the-attention-block-into-the-runtime`](integrate-the-attention-block-into-the-runtime.md) is still open.
