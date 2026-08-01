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
last_verified: "2026-08-01"
ticket: "scope-transformer-nonlinear-normalization-and-reductions"
---

# Transformer reference-semantics probe

## The named question

**What does the pinned reference actually compute** for the workload's softmax, causal mask, RMS normalization, and SiLU — at the boundary inputs where two plausible formulas stop agreeing?

Each family in this workload has at least two spellings that a reader would call the same operation, and the spellings differ observably in F32. Whether the softmax subtracts its row maximum, whether it then divides by the denominator or multiplies by the denominator's reciprocal, whether the causal mask fills with negative infinity or with the most negative finite value, where `eps` sits relative to the reciprocal square root, and whether SiLU is `x / (1 + exp(-x))` or `x * sigmoid(x)` are five such choices. A Tiler contract must pin one of each, and pinning it from memory is how a contract acquires a defect that no test written from the same memory will find.

The `softmax_form_width_*` rows deserve their own note, because a naive count would answer them wrongly. Divide and reciprocal-multiply agree on most elements, so an unrestricted agreement rate mostly measures how often the question was not asked; the rows therefore count only elements where the two forms produce different bits. They also stratify by row width, because at four contributors and above something starts contributing disagreements that belong to neither form — visible as the `matches_neither` column, which is exactly zero at widths two and three and nonzero above them. The narrow widths are what isolate the normalization form; the wide ones are what show that a wide-row count could not have.

**Added 2026-08-01 — what `matches_neither` is, measured rather than attributed.** This section previously named the cause outright: "the denominator's own accumulation order disagreeing with the naive sum". That reading survives the measurement, but it was an attribution when it was written, and at that resolution a second hypothesis fitted the same counts equally — a normalization constant that is not the correctly rounded reciprocal of *any* denominator, which is what an approximate reciprocal would produce. The `softmax_constant_width_*` rows separate them, and the lever is the maximum subtraction: it makes the largest score's exponential exactly `1.0`, so the reference's output at that position **is** the constant it multiplied the row by, read off exactly rather than solved for.

Two results follow, and they point in different directions. First, at every width, all 20,000 rows are *exactly* one scalar multiple of these exponentials — so the reference's exponentials agree bit for bit and the whole divergence is that one scalar. That is also the strongest available evidence for the **reciprocal-multiply form**, and it holds at every width rather than only at the narrow ones: a division by a denominator is not a single-constant multiply, and the perturbation below shows it failing this check on most rows. Second, the constant is not always the correctly rounded reciprocal of the naive sum — 14,680 of 20,000 at width four, within three ULP — and at width four, where the summation orders are enumerable, **19,895 of 20,000 constants are the correctly rounded reciprocal of a denominator these same exponentials reach under some strict left fold or the balanced tree**. So the original attribution is confirmed and no approximate reciprocal is needed to explain the data. The enumeration is not every legal grouping, so the count is a lower bound on reachability: it eliminates the second hypothesis where it is high and does not establish it where it falls short. Widths eight and eighteen are not enumerable and the question is left open there rather than generalized.

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

[`results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`](results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv) is the current retained observation. Each directory name carries the whole of its boundary — one host class, CPU, F32, and the two pinned package versions — because every row is a fact about that combination and about nothing wider.

[`results/2026-07-31-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`](results/2026-07-31-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv) is retained beside it rather than replaced, because the L3′ derivation and the SiLU and RMS-normalization landings were all written against those exact bytes and a citation should reach what its author read. The environments are identical; only the corpus grew. **Measurement — the earlier record is reproduced exactly, not merely believed.** On 2026-08-01 the probe as it stood reproduced the 2026-07-31 record byte for byte, and the corpus additions are a pure insertion: `diff results/2026-07-31-*/record.tsv results/2026-08-01-*/record.tsv` reports added lines only and no changed or deleted line, so every earlier claim is still readable at its original value.

**What 2026-08-01 added, and why each row had to exist.** The record previously carried none of the softmax evidence that the family's admission actually turned on, so that evidence had to be re-measured from a document rather than read from the record — the failure this directory exists to prevent. The additions close three such gaps. The `softmax_worked_example_*` rows carry the L3′ derivation's own four-position row, both spellings, the reference's output, and the reference's implied normalization constant with the reordered denominator that produces it, so the divergence between the reference and the pinned formula is checkable here instead of quoted from there. The `softmax_row_with_a_nan_score` and `torch_max_*` rows close a gap the softmax admission had to measure outside this record: `grep -i nan results/2026-07-31-*/record.tsv` returned only `silu_inputs` and the SiLU result rows. And the `softmax_constant_*` rows are the `matches_neither` attribution above.

**One of those rows contradicts a claim made about it.** The signed-zero rows are recorded in *both* operand orders and in two spellings because one order cannot tell an ordering rule apart from an order dependence. `torch.max` returns `-0.0` from `[+0.0, -0.0]` and `+0.0` from `[-0.0, +0.0]`; `torch.amax` does the opposite. Neither implements the `-0.0 < +0.0` total ordering that ADR 0023's Tiler extrema families share — each returns a fixed *position* rather than a fixed value, and the two spellings do not even agree on which position. A single-order measurement would have reported whichever answer its order produced as if it were the rule.

## The checks can say no

Two deliberate perturbations were run on 2026-07-31, and a third on 2026-08-01.

1. **The `eps` placement is load-bearing and the probe detects it.** Removing `eps` from the reciprocal square root's argument changed `rms_zero_vector` from four `0x00000000` to four `0x7fc00000` (NaN) and `rms_subnormal_vector` from `0x02081cb9` to `0x7f800000` (positive infinity). Restoring it reproduced the retained record byte for byte. So the record's normalization rows report the formula under test rather than a constant.
2. **The corpus discriminates between SiLU spellings.** `silu_x_times_sigmoid_x_inputs_differing_from_reference` names `0xc2b00000` (`-88.0`), where `x * sigmoid(x)` and the reference differ by one ULP while `x / (1 + exp(-x))` agrees exactly. An earlier corpus without an input near the exponential's overflow threshold reported all three spellings identical — a uniform pass over inputs none of which discriminated, which is the failure signature this repository distrusts by default.
3. **`explained_by_one_constant` reports a property of the reference, not of the arithmetic.** It reads `20000` at every one of the five widths, and a number that uniform over a heterogeneous population is exactly the signature to distrust — so it was run against a case that must fail. Substituting the divide form `numer / denom` for the reference and leaving the check otherwise untouched drops it to 16,963, 13,875, 11,091, 5,505, and 2,356 of 20,000 at widths two, three, four, eight, and eighteen, falling as the row widens because a longer row gives the single constant more elements to fail at. The check can say no, it says no to the spelling the record's own counts rule out, and the uniform pass is therefore a result rather than a check that did not run.

## Traceability

- **Supported claim:** [Transformer non-linear, normalization, and reduction contracts](../../../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md).
- **Workload the inputs come from:** [First Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`scope-transformer-nonlinear-normalization-and-reductions`](../../../tickets/scope-transformer-nonlinear-normalization-and-reductions.md).
