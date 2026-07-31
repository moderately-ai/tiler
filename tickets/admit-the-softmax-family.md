---
id: admit-the-softmax-family
title: Admit the softmax family
status: todo
priority: p1
dependencies: [scope-transformer-nonlinear-normalization-and-reductions, admit-the-rms-normalization-family]
related: [admit-the-silu-activation-family, implement-parallel-reduction-strategies, own-operation-family-support-matrix, design-attention-program-vertical, promote-the-symbolic-index-profile-to-a-public-boundary, assemble-the-causal-self-attention-block-program, retain-the-c1-attention-block-conformance-evidence]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, softmax, attention, reduction, transcendental, language-model, breadth]
---
## User-visible outcome

A program can state `softmax(scores, axis)` and have it execute — the operation that turns attention scores into attention weights, 28 times per forward pass, over the one extent in this workload that grows during decode.

## Why this one last of the three

**Inference — from the [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md).** SiLU has one transcendental and no reduction; RMS normalization adds one order-sensitive reduction over a static extent; softmax adds a second reduction, a second combiner family with an unsettled NaN contract, and the workload's only bounded-symbolic growing reduction extent. Each rung adds one obligation to the one below it.

## Evidence prerequisite

**Fact — the exact expression, from `eager_attention_forward` at lines 157–162 of the pinned `modeling_qwen3.py` (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`).** The scale multiplies the contraction *result* (line 157), the mask is *added* (line 160), and the softmax is over the last axis with `dtype=torch.float32` (line 162). Pre-scaling the query instead is a different F32 computation.

**Measurement — the reference subtracts the row maximum.** On `[1000.0, 1000.0]` it returns a half each while the naive quotient-of-exponentials returns NaN, because `exp(1000)` overflows F32. Not a tolerance: finite against NaN.

**Measurement — the reference multiplies by the denominator's reciprocal; it does not divide.** Counting only elements where the two forms produce different bits, at row width 2 all 3,037 discriminating elements match the reciprocal form and none matches division; at width 3, all 7,224. Above width 4 a third bucket appears that matches neither, which is the denominator's own accumulation order and is why the narrow widths are the ones that isolate the question. The [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) retains the counts. **Inference.** A key that pinned "divide" would diverge from the reference on ordinary inputs while looking correct; the choice belongs in the pinned formula, not in a `reciprocal_math` permission.

**Measurement — the causal mask's fill value is finite, not `-inf`.** It is `torch.finfo(f32).min` (`0xff7fffff`), and an attended entry is that value times a boolean false, which is **negative zero** (`0x80000000`). This decides the fully-masked-row behaviour: uniform under the finite fill, NaN under `-inf`.

**Fact — volume and extent.** 28 occurrences covering 448·`T` rows in total, each row reducing `S` contributors, where `S` is bounded symbolic and grows during decode. At the B1-d benchmark row's prefill that is 3.0 × 10¹⁰ exponentials in one pass.

**Measurement — Metal supplies the primitives.** `air.exp.f32`, `air.fmax.f32`, `air.simd_sum.f32`, and `air.simd_max.f32` under the governed flag set, with `air.fast_*` variants selected by the compiler default; the [emission probe](../spikes/numerics/metal_transcendental_emission/README.md) retains the table. `air.fmax.f32` is number-preferring with an order-dependent signed-zero result, so it does not implement both extrema families.

## Required delivery

One vertical. It must carry:

- **Reference behaviour.** A governed `OpKey` carrying a reduced-axis attribute, whose normative reference pins: the row-maximum subtraction, the exponential, the sum, and the **reciprocal multiplication** rather than a division. A `tiler-reference` evaluator implementing exactly that. The result does not sum to exactly one in F32 — the derivation's worked example sums to `0x3f7ffffe` — so no check may assert that it does.
- **Settle decision D-2, the extrema family.** `Maximum` propagates NaN; `MaximumNumber` prefers the numeric operand. ADR 0023 makes them separate operations, and the choice determines whether one NaN score poisons its whole row. The reduction form is a separate obligation from the elementwise form; admitting one does not admit the other.
- **Settle decision D-1, the fully masked row.** Uniform, NaN, or a typed refusal. The workload does not reach the case — causal masking at batch 1 without padding leaves every row at least one attended position, and the reference's own repair path is guarded to `sdpa` on `cuda`/`xpu` and does not run here — so this is a decision to make deliberately rather than an answer to inherit from whichever mask convention got written first. **Measurement added 2026-07-31 by [the L4 attention design](../docs/research/program-planning/first-attention-program-vertical.md): the conformance row cannot falsify either answer.** Replacing the finite fill with `-inf` over the whole `[8, 2, 10, 10]` C1 score tensor changes **0 of 1,600** softmax outputs, because every masked argument drives `Exp` to exactly `+0.0` under both conventions; a fully masked width-10 row returns uniform `0x3dcccccd` under the finite fill and ten `0x7fc00000` NaNs under `-inf`. **Inference — so a corpus that only ran C1 would pass with the wrong mask convention installed**, and the synthetic fully-masked row is the only case that tests this decision at all rather than optional extra coverage. [The attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains both comparisons.
- **Compiler legality, with the two reductions treated separately.** The maximum is associative and commutative wherever its family is total, so any tree over the same contributors gives the same bits; the sum is neither. A schedule may therefore parallelize the first pass freely and the second only under a permission, and a legality check that assumed one permission covered both passes would be wrong in exactly one direction. Reassociation and permutation stay independent. An online single-pass form that rescales a running sum when the maximum changes regroups the contributor sequence and requires reassociation — a legality question, not a cost one.
- **Shape semantics.** Softmax is shape-preserving. A zero-length reduced axis yields a zero-length output and evaluates no scalar softmax, so the reduction contract's empty-domain rules do not apply and the family must say so rather than appearing to inherit them.
- **Symbolic extent.** The reduced extent is `S`, which needs [`promote-the-symbolic-index-profile-to-a-public-boundary`](promote-the-symbolic-index-profile-to-a-public-boundary.md) to reach a public boundary; without it every distinct `S` is a separate compiled artifact. An extent symbol with no proved upper bound refuses rather than compiling a generic program.
- **Metal realization.** Structured-kernel constructs for the exponential and for a maximum reduction, and an emission that selects the intrinsic the resolved accuracy contract admits. `air.fmax.f32` may be selected only if the settled extrema family's full behaviour agrees, including its signed-zero order dependence, or with a fixup.
- **Explainable refusal.** A missing order permission names which of the two passes and which dimension; an unbounded reduced extent names the missing constraint; an unhonourable accuracy contract carries the declaring profile's identity and the refusing fact's measurement boundary. Perturb each so it actually fires before trusting it.
- **Bounded conformance evidence.** The derivation's worked example (`[1.0, 2.0, 3.0, mask]`, whose F32 bits are retained there, including the masked position contributing and receiving exactly `+0.0`), a row of equal large scores that would overflow without the maximum subtraction, the underflow band where a contributor more than about 104 below the maximum contributes exactly zero, and whichever fully-masked-row behaviour D-1 settles. State exactly which reduced extents the evidence covers; do not generalize from a static one to `S`.
- **The matrix row.** Update the reductions row and the transcendentals row of the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) in the same change, and the `Select` row's trigger note if D-1 lands as a refusal.

## Non-goals

A general `Exp` key, a standalone maximum-reduction key, log-softmax, a derived index-domain causal mask, and a fused flash-attention form. The mask stays an F32 program input at every row this workload declares; the derived predicate route needs a boolean dtype the registry does not admit and an index-domain comparison ADR 0084's vocabulary excludes by construction, and its activation trigger is a row where the `T × S` mask outgrows the program rather than this ticket.

## Reconsideration trigger

Active now: 28 occurrences per forward pass and no alternative spelling. If the workload is superseded, re-derive the mask convention and the normalization form from the replacement's own reference — both were established here by measurement precisely because they are not what the conventional spelling suggests.
