---
id: compose-rotary-position-embedding-from-reindex-and-broadcast
title: Compose rotary position embedding from Reindex and Broadcast
status: done
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families]
related: [design-attention-program-vertical, admit-the-grouped-query-head-layout-reindex-profile, assemble-the-causal-self-attention-block-program, derive-transformer-operation-and-shape-surface]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, structural, rope, attention, language-model]
---
## User-visible outcome

A program can apply rotary position embedding to a `[T, heads, 128]` operand using only families this workload already requires — and the derivation that removed a slice family and a concatenate family from the workload's requirements becomes a checked composition rather than a claim.

## Why this is not absorbed into the Reindex admission

**Inference.** [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) admits the two families and their normative semantics. What it does not do is answer whether the *particular* coordinate map rotary embedding needs is one of the admitted initial forms, and that answer decides whether this workload needs a structural family the corpus does not have. Filing the question against the composition that raises it keeps the admission ticket from having to anticipate it and keeps the answer attached to the operation that fails closed without it.

## Evidence prerequisite

**Fact — the reference's form, from `modeling_qwen3.rotate_half` and `apply_rotary_pos_emb` at the pinned digest.** `rotate_half(x) = cat(-x₂, x₁)` over the half-split of the 128-wide head axis, and `y = x · cos + rotate_half(x) · sin` with `cos` and `sin` unsqueezed at the head axis. There is no partial rotary factor and no scaling multiplier: `rope_scaling` is `null` and `attention_factor` is 1.0 at the pinned revision.

**Proposal — the composition, from the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md).** A `Reindex` bijective split of the 128 axis into `(2, 64)`; a `Reindex` coordinate swap `i -> 1 − i` on the resulting size-2 axis; a `Multiply` by a `[2, 1]` sign operand broadcast over the 64 axis; a `Reindex` merge back to 128. Then two broadcast multiplies and one add.

**Measurement — the composition reproduces the reference bit-for-bit, and both of its parts are load-bearing.** On a `[1, 16, 10, 128]` operand it differs from `modeling_qwen3.rotate_half` at **0 of 20,480** elements; dropping the coordinate swap differs at all 20,480 and reversing the sign operand differs at all 20,480. The complete rotary application differs from `apply_rotary_pos_emb` at 0 of 20,480 for the query and 0 of 10,240 for the key. The [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains the counts and the perturbations.

**Fact — the sign operand cannot be a constant.** `ConstantF32::infer` pushes `Shape::new([])`, so `tiler::constant-f32@1` produces rank zero only and a two-element dense constant is not expressible. The `[2, 1]` sign tensor enters as a program input; it is eight bytes and it is the workload's only new input.

**Fact — the tables are program inputs, not in-program transcendentals.** `cos` and `sin` depend on position and `rope_theta = 1000000` alone and are host-precomputable, which removes `Sin` and `Cos` from the executed program entirely. The trade is explicit: the host construction joins the conformance oracle's comparison surface and must agree with the reference's own table construction rather than being assumed to.

## Required delivery

- **D-10 is settled — corrected 2026-08-01; consume the admitted form, do not reopen the question.** `admit-the-reindex-and-broadcast-operation-families` answered it in `tiler::reindex-f32@1`'s registered normative reference on 2026-07-31: the within-axis coordinate permutation is admitted as exactly the `reverse-axis` form `i -> extent − 1 − i` (at extent 2, this composition's swap), any other within-axis map refusing as `reindex.form.unadmitted-kind`; `propagate-the-d10-resolution-into-the-contract-corpus` carried the answer into `docs/ir.md` and the research records. This bullet previously asked the question this ticket must now merely consume.
- **A checked composition, not a new key.** Rotary embedding stays a graph shape over admitted families; nothing here admits a `Rope` operation. The composition's normative reference is the composition.
- **Reference equivalence at the workload's own extents.** Bit-compare against the pinned reference at head dimension 128 for both the 16-head query and the 8-head key operand, and retain the two perturbations, because a comparison whose failure mode was never demonstrated is not evidence.
- **The broadcast axis mappings, explicitly.** `cos` and `sin` are `[T, 128]` against `[T, heads, 128]` and the sign is `[2, 1]` against `[…, 2, 64]`. [IR](../docs/ir.md) admits no implicit broadcasting and the rank-zero scalar admission covers a scalar operand only, so every one of these is an explicit `Broadcast` with a stated axis mapping.
- **Explainable refusal.** A `Reindex` whose coordinate function is not among the admitted forms refuses at construction naming the form, not naming totality. Perturb it so it fires before trusting it.

## Non-goals

A `Rope` operation family, the rotary table's construction inside the program, partial rotary factors, rope scaling of any kind, and the interleaved (GPT-J) pairing convention. The pinned checkpoint uses the half-split form with `rope_scaling: null`, and a workload that used another would re-derive this from its own reference.

## Closes when

The composition verifies over admitted families and reference-evaluates bit-identically to the pinned reference at both operand shapes with its perturbations retained. (D-10 was answered in the registered normative reference on 2026-07-31; this ticket consumes the `reverse-axis` form rather than settling anything.)

## Outcome

**Fact — the composition is a checked program and nothing was registered.** `crates/tiler-reference/tests/rotary_position_embedding.rs` builds rotary position embedding as ten occurrences over the four already-registered families, in this order and with these axis mappings:

| # | key | form or mapping | shape |
| --- | --- | --- | --- |
| 1 | `tiler::reindex-f32@1` | `split-axis` on axis 2 into `(2, 64)` | `[T, h, 128] -> [T, h, 2, 64]` |
| 2 | `tiler::reindex-f32@1` | `reverse-axis` on axis 2, the admitted D-10 form | `[T, h, 2, 64]` |
| 3 | `tiler::broadcast-f32@1` | `replicate, replicate, from-operand 0, stretch-unit 1` | `[2, 1] -> [T, h, 2, 64]` |
| 4 | `tiler::multiply-f32@1` | — | `[T, h, 2, 64]` |
| 5 | `tiler::reindex-f32@1` | `merge-axes` over axes 2 and 3 | `[T, h, 2, 64] -> [T, h, 128]` |
| 6 | `tiler::broadcast-f32@1` | `from-operand 0, replicate, from-operand 1` | `[T, 128] -> [T, h, 128]` |
| 7 | `tiler::multiply-f32@1` | — | `x · cos` |
| 8 | `tiler::broadcast-f32@1` | `from-operand 0, replicate, from-operand 1` | `[T, 128] -> [T, h, 128]` |
| 9 | `tiler::multiply-f32@1` | — | `rotate_half(x) · sin` |
| 10 | `tiler::add-f32@1` | — | `[T, h, 128]` |

Occurrences 1–5 are `rotate_half`, and the program names both it and the rotary result as ordered outputs so every perturbation is shared between the two comparisons. No `Rope` key exists, no form or relation was added, and the public surface of `tiler-ir` and `tiler-reference` is unchanged.

**Measurement — the composition at the workload's own extents, against an independently recomputed expectation.** `rotate_half` is compared at the bit level (negating a normal binary32 flips one bit, so the expectation needs no floating-point arithmetic); the whole formula is recomputed as two separate multiplies and an add.

| comparison | 16-head query | 8-head key |
| --- | --- | --- |
| elements | 20,480 | 10,240 |
| `rotate_half` differing | 0 | 0 |
| `x · cos + rotate_half(x) · sin` differing | 0 | 0 |
| swap dropped, `rotate_half` differing | 20,480 | 10,240 |
| signs reversed, `rotate_half` differing | 20,480 | 10,240 |
| swap dropped, whole formula differing | 20,480 | 10,240 |
| signs reversed, whole formula differing | 20,480 | 10,240 |

Both perturbation counts are exact rather than probable: the fixture's payloads are pairwise distinct by construction, so the two halves differ at every lane, and reversing the sign negates a tensor with no zero in it. The dropped-swap perturbation is checked structurally too — that program carries nine occurrences and two reindexes, so its counts measure the reversal alone.

**Fact — the pinned lanes are reproduced.** `the_pinned_rotate_half_lanes_are_reproduced` drives the attention-block probe's four `rotate_half_input_lanes_0_3` and four `rotate_half_input_lanes_64_67` payloads through the composition and requires `rotate_half_output_lanes_0_3` and `rotate_half_output_lanes_64_67` back, bit for bit, with both perturbations shown to move exactly those lanes.

**Measurement boundary, stated rather than hidden.** The full-shape comparison against the pinned reference's own numbers stays out of tree. [The attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains counts and eight lane payloads, not operand tensors, and its operands come from a seeded `torch` generator this workspace cannot reproduce; it is what measured this composition against `modeling_qwen3.rotate_half` and `apply_rotary_pos_emb` at 0 of 20,480, 0 of 20,480, and 0 of 10,240. In tree the evidence is that the composition denotes `cat(−x₂, x₁)` and the rotary formula at the workload's extents, plus the eight-lane tie to the probe's retained bits. Neither substitutes for the other, and a re-pin of the reference would need the probe re-run.

**Fact — the explainable refusal fires.** A hand-assembled mapping record naming `rotate-axis` — the within-axis rotation the index vocabulary can express and D-10 deliberately leaves unadmitted — is refused under `reindex.form.unadmitted-kind` by `ReindexForm::from_canonical_value` and again through `SemanticProgramBuilder::apply` at the exact point of the composition where the swap belongs, with the message naming the rejected form. The admitted neighbour, `reverse-axis` at the same axis, is applied immediately afterwards and succeeds, so the refusal discriminates the form rather than the occurrence.

**Fact — the sign operand cannot be a constant.** A one-output constant program evaluates to a rank-zero tensor carrying `0xbf800000`, and the `[2, 1]` sign mapping refuses that operand under `broadcast.mapping.operand-axes-unconsumed`; a broadcast of one constant could not have supplied two different signs regardless. The `[2, 1]` input is admitted by the same mapping.

**Every check was watched failing.** Dropping the negation from the bit-level `rotate_half` expectation moved the query comparison to 10,240 of 20,480 and the key to 5,120 of 10,240 — the first half, where the negation lives. Corrupting one pinned output lane failed `the_pinned_rotate_half_lanes_are_reproduced` on that lane alone. Renaming the unadmitted form to `reverse-axis` made the record decode and admitted the occurrence, failing the refusal test.

Roadmap: the structural row and the slice row each gained a dated fact, because both previously rested on a derivation this landing turns into a program.
