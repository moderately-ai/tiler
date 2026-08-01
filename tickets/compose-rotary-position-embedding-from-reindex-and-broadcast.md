---
id: compose-rotary-position-embedding-from-reindex-and-broadcast
title: Compose rotary position embedding from Reindex and Broadcast
status: todo
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
