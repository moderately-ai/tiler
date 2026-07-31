---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.transformer-reference-semantics"
kind: "experiment"
title: "Transformer reference-semantics probe"
topics: ["numerics", "softmax", "normalization", "masking", "transformer", "language-model"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
entrypoints: ["spikes/numerics/transformer_reference_semantics/probe.py"]
last_verified: "2026-07-31"
ticket: "scope-transformer-nonlinear-normalization-and-reductions"
---

# Transformer reference-semantics probe

## The named question

**What does the pinned reference actually compute** for the workload's softmax, causal mask, RMS normalization, and SiLU — at the boundary inputs where two plausible formulas stop agreeing?

Each family in this workload has at least two spellings that a reader would call the same operation, and the spellings differ observably in F32. Whether the softmax subtracts its row maximum, whether it then divides by the denominator or multiplies by the denominator's reciprocal, whether the causal mask fills with negative infinity or with the most negative finite value, where `eps` sits relative to the reciprocal square root, and whether SiLU is `x / (1 + exp(-x))` or `x * sigmoid(x)` are five such choices. A Tiler contract must pin one of each, and pinning it from memory is how a contract acquires a defect that no test written from the same memory will find.

The `softmax_form_width_*` rows deserve their own note, because a naive count would answer them wrongly. Divide and reciprocal-multiply agree on most elements, so an unrestricted agreement rate mostly measures how often the question was not asked; the rows therefore count only elements where the two forms produce different bits. They also stratify by row width, because at four contributors and above the denominator's own accumulation order starts contributing disagreements that belong to neither form — visible as the `matches_neither` column, which is exactly zero at widths two and three and nonzero above them. The narrow widths are what isolate the normalization form; the wide ones are what show that a wide-row count could not have.

## What it does and does not establish

**It establishes** what `transformers` 4.51.0 on `torch` 2.6.0 computes, in F32, on CPU, for hand-authored boundary inputs, reported as exact bit patterns because a decimal rendering hides signed zero and NaN payloads. Where two spellings disagree, it names the exact input at which they disagree rather than reporting a summary verdict.

**It establishes no Tiler contract and no bound.** An observation of one implementation is evidence for what the definitional reference means, not a proof that a Tiler operation must reproduce it bit-for-bit, and certainly not an accuracy bound — under ADR 0042 an empirical observation cannot establish an unmeasured worst case. It also says nothing about any GPU: the divergence sources the workload profile names (reduction order, subnormal flushing, elementary-function results) are all target properties this probe cannot see.

**It loads no checkpoint and touches no network.** Unlike the [C1 conformance fixture](../../program-planning/qwen3-conformance-fixture/README.md), every question here is about the reference's formulas and their behaviour on synthetic inputs, so no weights are required.

## Reproduce

From **this directory** (no `make` target reaches `spikes/`):

```sh
uv run --offline python probe.py                     # print the record
uv run --offline python probe.py > record.tsv        # capture it for comparison
```

Drop `--offline` on a host whose uv cache does not already hold the pinned wheels. The output is ordered and deterministic; two consecutive runs were byte-identical on 2026-07-31.

## Retained record

[`results/2026-07-31-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`](results/2026-07-31-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv) is the retained observation. Its name carries the whole of its boundary — one host class, CPU, F32, and the two pinned package versions — because every row is a fact about that combination and about nothing wider.

## The checks can say no

Two deliberate perturbations were run on 2026-07-31.

1. **The `eps` placement is load-bearing and the probe detects it.** Removing `eps` from the reciprocal square root's argument changed `rms_zero_vector` from four `0x00000000` to four `0x7fc00000` (NaN) and `rms_subnormal_vector` from `0x02081cb9` to `0x7f800000` (positive infinity). Restoring it reproduced the retained record byte for byte. So the record's normalization rows report the formula under test rather than a constant.
2. **The corpus discriminates between SiLU spellings.** `silu_x_times_sigmoid_x_inputs_differing_from_reference` names `0xc2b00000` (`-88.0`), where `x * sigmoid(x)` and the reference differ by one ULP while `x / (1 + exp(-x))` agrees exactly. An earlier corpus without an input near the exponential's overflow threshold reported all three spellings identical — a uniform pass over inputs none of which discriminated, which is the failure signature this repository distrusts by default.

## Traceability

- **Supported claim:** [Transformer non-linear, normalization, and reduction contracts](../../../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md).
- **Workload the inputs come from:** [First Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`scope-transformer-nonlinear-normalization-and-reductions`](../../../tickets/scope-transformer-nonlinear-normalization-and-reductions.md).
