---
id: admit-the-rms-normalization-family
title: Admit the RMS normalization family
status: todo
priority: p1
dependencies: [scope-transformer-nonlinear-normalization-and-reductions, admit-the-silu-activation-family, admit-the-reindex-and-broadcast-operation-families, implement-the-typed-accuracy-contract-vocabulary, record-the-metal-elementary-function-accuracy-guarantee]
related: [admit-the-softmax-family, implement-parallel-reduction-strategies, own-operation-family-support-matrix, design-attention-program-vertical]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, normalization, reduction, transcendental, language-model, breadth]
---
## User-visible outcome

A program can state `rms_norm(x, weight, eps)` over a named axis and have it execute — the operation the selected workload performs 113 times per forward pass, and the second-largest requirement in it after the contraction.

## Evidence prerequisite

**Fact — the exact formula, from `Qwen3RMSNorm.forward` at lines 71–76 of the pinned `modeling_qwen3.py` (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`).** `variance = hidden_states.pow(2).mean(-1, keepdim=True)` then `hidden_states * torch.rsqrt(variance + self.variance_epsilon)` then `self.weight * hidden_states.to(input_dtype)`. Three decisions the usual spelling hides: nothing is subtracted, so this is **not** layer normalization; `eps` is inside the `rsqrt` argument, not outside the root; and the operation uses `rsqrt`, not `1 / sqrt`. The [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) records all three with the F32 consequences.

**Measurement — `eps` is a semantic term and not a guard.** From the [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md): with `eps`, a zero row normalizes to zeros and a subnormal row to a normal `0x02081cb9`; without it, the same rows give NaN and `+inf`. `eps` also changes the result at an ordinary input, so it perturbs every output rather than activating near zero.

**Measurement — the silent-wrongness case.** Squaring overflows at `0x5f7fffff` (≈ 1.845 × 10¹⁹). A row of `1e20` gives a mean of squares of `+inf`, an `rsqrt` of zero, and a result of **all positive zeros** — finite, plausible, and wrong, with no NaN or infinity to reveal it. Whether the operation refuses is decision **D-3** of the derivation and this ticket settles it.

**Fact — volume and extents.** 113 occurrences: 57 over a static extent of 1024 (`input_layernorm` and `post_attention_layernorm` per layer, plus `model.norm`) and 56 over a static extent of 128 (`q_norm` and `k_norm` per layer). One operation, two extent classes: `Qwen3Attention.__init__` at line 195 constructs the per-head norms from the same class. Per forward pass that is 144,384·`T` squared contributors and 729·`T` reciprocal square roots.

**Fact — the broadcast operand.** The weight is `[1024]` against `[T, 1024]`, or `[128]` against `[T, 16, 128]`. `docs/ir.md` admits no implicit broadcasting and the rank-zero scalar admission does not cover it, which is why [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) is a dependency rather than a note.

**Measurement — Metal supplies the intrinsic.** The [emission probe](../spikes/numerics/metal_transcendental_emission/README.md) records `air.rsqrt.f32` under the governed flag set and `air.fast_rsqrt.f32` under the compiler default. What is absent is on Tiler's side: no reciprocal square root exists in the structured-kernel vocabulary.

## Required delivery

One vertical. It must carry:

- **Reference behaviour.** A governed `OpKey` carrying a reduced-axis attribute and the exact `eps` bits — `rms_norm_eps` is `1e-06`, not exactly representable in F32, and two normalizations differing only in that constant are different operations that must not share an identity, a cache subject, or a golden. The normative reference pins the mean-of-squares, the `eps` position inside the `rsqrt` argument, the choice of `rsqrt`, and the fact that the weight multiply follows the (F32-identity) conversion rather than preceding it. A `tiler-reference` evaluator implementing exactly that.
- **Compiler legality.** A fusion role for a sum reduction carrying an elementwise squaring prologue. `OrderedReduction` is the shape this was defined for, but it is held by the single registered strict-serial-sum key, so a role for this family is required or it yields no fusion legality at all. The division by the static extent is exact here because 1024 and 128 are powers of two; do not encode that exactness into the formula, because a non-power-of-two extent would then acquire a silent rounding.
- **Order and accumulation.** A strict ordered fold over the canonical contributor sequence unless a registered permission authorizes otherwise, with reassociation and permutation checked independently. The accumulator dtype is explicit and is decision **D-5**, owned by [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md); consume that authority rather than defaulting from the element dtype.
- **Metal realization.** Structured-kernel constructs for the reciprocal square root and for a prologue-carrying sum, and an emission that selects the intrinsic the resolved accuracy contract admits rather than the one the compiler default would pick.
- **Explainable refusal.** Separate typed refusals for: a non-positive, non-finite, or NaN `eps` (rejected at construction — a zero `eps` is a different operation with a different domain, not a degenerate parameter); an absent, duplicated, or out-of-range reduced axis, naming the violated rule; a reduction topology the order permission does not cover, naming the missing dimension; and an accumulator narrower than the contract allows. Settle **D-3** and, if the answer is a refusal, note that it is a semantic precondition requiring a proof or a costed runtime scan rather than a free guard.
- **Bounded conformance evidence.** The zero row, a subnormal row (which diverges between the CPU reference and the subnormal-flushing qualified Metal row — record the divergence, do not tune it away), a row above the squaring-overflow threshold, both extent classes, and the exact worked example the derivation retains (`x = [3.0, 4.0]`, `w = [1.0, 2.0]`, whose F32 bits are recorded there). State exactly which extents and rows the evidence covers.
- **The matrix row.** Update the reductions row and the transcendentals row of the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) in the same change.

## Non-goals

Layer normalization, a general `Rsqrt` key, a general mean reduction, and any normalization with a bias or a mean subtraction. The derivation establishes that this workload needs none of them, and widening the family to absorb layer normalization would silently change what an existing occurrence means.

## Reconsideration trigger

Active now: 113 occurrences per forward pass with no alternative spelling. If the workload is superseded, re-derive the formula and the `eps` value from the replacement's own reference rather than carrying these forward — the `eps` position in particular is not shared across architectures.
