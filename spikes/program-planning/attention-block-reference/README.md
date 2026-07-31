---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.attention-block-reference"
kind: "experiment"
title: "C1 attention-block reference probe"
topics: ["program-planning", "attention", "rope", "masking", "softmax", "transformer", "language-model"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.program-planning.first-attention-program-vertical"]
entrypoints: ["spikes/program-planning/attention-block-reference/probe.py"]
last_verified: "2026-07-31"
ticket: "design-attention-program-vertical"
---

# C1 attention-block reference probe

## The named question

**Does the attention program the L4 design writes down denote what the pinned reference computes**, at the C1 conformance row's own shapes — and where it does not, exactly which elements disagree and why?

The design states a graph: a `Reindex` spelling for `rotate_half`, a `(g, r)` head split for grouped-query attention, an index structure for the score contraction, a scale on the score rather than on an operand, a finite mask fill, and a value contraction whose contributors at masked positions are exact zeros. Each of those is a claim with a plausible neighbour that produces a correctly shaped tensor and different numbers, and the neighbours are the spellings a competent implementer reaches for first. This probe replaces the design's word with bit counts.

Five questions, each chosen because a wrong answer is invisible in a decimal rendering or in a shape check:

1. **Is `rotate_half` a slice-and-concatenate, or is it a `Reindex` split, a coordinate swap on the resulting size-2 axis, a broadcast sign multiply, and a `Reindex` merge?** The second reading is what removes a slice family and a concatenate family from the workload's requirements, so the derivation that saved two capability tickets rests on it.
2. **Which key head does query head `h` read?** `repeat_kv` is repeat-interleave (`h // n_rep`); repeat-tile (`h % num_kv_heads`) is the other reading, produces an identically shaped tensor, and is wrong for fourteen of the sixteen heads.
3. **Does the index structure `grtd,gsd->grts` denote the reference's repeat-then-matmul?** Structural agreement and bitwise agreement are different questions, and the probe reports them separately so that a reduction-order artefact is not read as a structural defect.
4. **What does a masked position contribute to the value contraction?** It contributes an exact zero whose *sign* follows the value operand, and a signed zero added to a signed-zero accumulator is not the identity.
5. **Can the C1 row discriminate the mask's fill convention at all?** Decision D-1 turns on it, and an unreachability claim that nothing measured is a claim about the author's reading of the mask builder.

## What it does and does not establish

**It establishes** what `transformers` 4.51.0 on `torch` 2.6.0 computes, in F32, on CPU, for the C1 prefill shape — ten new positions against ten context positions, sixteen query heads over eight key/value heads, head dimension 128 — on synthetic operands from a fixed seed. Agreement is reported as a count of elements whose F32 bits differ, because that is the only equality that distinguishes a spelling from its neighbour.

**It establishes no Tiler contract, no accuracy bound, and nothing about any GPU.** The divergence sources the workload profile names — reduction order, subnormal flushing, elementary-function results — are target properties this probe cannot see. Under ADR 0042 an observation is not a normative guarantee.

**It uses synthetic operands, not the checkpoint.** Every question here is about the reference's *composition* rather than about the weights, so no checkpoint is loaded and no network is touched. The consequence is stated rather than hidden: the retained bit patterns are properties of this seed, and only the zero-and-nonzero difference counts, the mapping rows, and the structural facts generalize past it. The [C1 conformance fixture](../qwen3-conformance-fixture/README.md) is what carries the real weights.

## Reproduce

From **this directory** (no `make` target reaches `spikes/`):

```sh
uv run --offline python probe.py                     # print the record
uv run --offline python probe.py > record.tsv        # capture it for comparison
```

Drop `--offline` on a host whose uv cache does not already hold the pinned wheels. The output is ordered and deterministic; two consecutive runs were byte-identical on 2026-07-31.

## Retained record

[`results/2026-07-31-c1-attention-block-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`](results/2026-07-31-c1-attention-block-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv) is the retained observation. Its name carries the whole of its boundary — the C1 prefill shape, one host class, CPU, F32, and the two pinned package versions — because every row is a fact about that combination and about nothing wider.

## The checks can say no

Every equivalence claim below is paired with a perturbation in the same invocation, so a zero is a property of the composition rather than of a comparison that cannot fail. All counts are from the retained record.

1. **The `rotate_half` composition is exact, and both of its parts are load-bearing.** `rotate_half_composition_differing_elements` is `0` of `20480`. Removing the coordinate swap differs at all `20480`; reversing the sign operand differs at all `20480`. So the swap and the sign are each necessary, and neither is a coincidence of the corpus.
2. **The grouped-query mapping discriminates.** `h // n_rep` matches `repeat_kv` at every element; `h % num_kv_heads` differs at `17920` elements over fourteen of the sixteen heads. A probe that compared only shapes would have passed both.
3. **The recomputation is the reference's own composition.** `eager_attention_weights_differing_from_recomputation` and `eager_attention_output_differing_from_recomputation` are both `0` against `modeling_qwen3.eager_attention_forward`, so the intermediates the record exposes describe the reference rather than a lookalike assembled beside it.
4. **The C1 row cannot discriminate the mask fill, and that is the measured result rather than the harness failing.** `softmax_finite_fill_vs_neginf_fill_differing_elements` is `0` over all 1,600 score elements — but the same comparison on a *fully masked* width-10 row returns uniform `0x3dcccccd` under the finite fill and NaN under `-inf`. The comparison discriminates; the C1 row is what does not reach the case.

## Traceability

- **Supported claim:** [First attention program vertical](../../../docs/research/program-planning/first-attention-program-vertical.md).
- **Workload the shapes come from:** [First Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md).
- **The families whose formulas it assumes:** [Transformer non-linear, normalization, and reduction contracts](../../../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md), whose own probe is [the reference-semantics probe](../../numerics/transformer_reference_semantics/README.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
