---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.qwen3-weight-quantization-profiles"
kind: "experiment"
title: "Qwen3-0.6B-Base candidate quantization profile probe"
topics: ["numerics", "quantization", "dtypes", "language-model", "qwen", "memory"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.first-quantized-lm-profile"]
entrypoints: ["spikes/numerics/qwen3-weight-quantization-profiles/weight_error.py", "spikes/numerics/qwen3-weight-quantization-profiles/model_error.py", "spikes/numerics/qwen3-weight-quantization-profiles/calibration_sensitivity.py"]
last_verified: "2026-07-31"
ticket: "scope-first-quantized-lm-profile"
---

# Qwen3-0.6B-Base candidate quantization profile probe

Evidence for [the first quantized language-model profile](../../../docs/research/numerics/first-quantized-lm-profile.md): what each candidate weight representation costs in bytes and what it does to the workload's own conformance row, measured on the pinned checkpoint rather than argued from the reputation of a format.

[The workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md) is the authority for every constant here — the pinned revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, the `model.safetensors` digest, the C1 prompt token IDs, the 8-step decode budget, the tie policy, the F32 weight budget. This spike transcribes them and checks against them; it re-derives none of them.

Nothing here is a Tiler execution, a Metal measurement, or an accuracy budget. It answers two bounded questions — how much smaller, and how much different — and stops.

## Prerequisites

- The pinned checkpoint in the Hugging Face cache **outside this repository**, about 1.2 GB. Acquire it exactly as the workload profile records: `hf download Qwen/Qwen3-0.6B-Base --revision da87bfb608c14b7cf20ba1ce41287e8de496c0cd`. `TILER_QWEN3_SAFETENSORS` overrides the resolved path.
- An interpreter that already provides `numpy` for Stage A, plus `torch` and `transformers` for Stage B. **This spike deliberately pins nothing**, which is the opposite of the choice [the C1 conformance fixture](../../program-planning/qwen3-conformance-fixture/README.md) made, and the reason is that the two are answering different questions. There the retained digests *are* a fingerprint of one implementation's reduction order, so a floating resolution would silently re-baseline the evidence. Here every reading is a *difference* between two runs in one process, and the F32 baseline of that difference is recomputed on whatever host runs it. `environment.tsv` records the exact versions that produced the retained record, and Stage B re-derives its own baseline rather than comparing to a stored one.

## Run it

No `make` target reaches a spike. From this directory:

```sh
# Stage A -- weight-space error and exact byte cost over all 197 candidate tensors (~20 s):
python3 weight_error.py --out results/2026-07-31-weight-error-qwen3-0.6b-base-da87bfb6

# Stage A' -- how much of Stage A's ordering is the calibration's doing (~40 s):
python3 calibration_sensitivity.py --out results/2026-07-31-weight-error-qwen3-0.6b-base-da87bfb6

# Stage B -- the model-visible C1 observable, 16 runs plus a baseline (~25 min on the host row below):
HF_HUB_OFFLINE=1 python3 model_error.py \
  --out results/2026-07-31-model-observable-qwen3-0.6b-base-da87bfb6
```

## What runs before anything is computed

Each is fail-closed — a mismatch exits non-zero and writes nothing.

1. **The checkpoint.** `model.safetensors` is size-checked and SHA-256'd against the workload profile's manifest. **Fault-proved:** pointing `TILER_QWEN3_SAFETENSORS` at `/etc/hosts` exits with `checkpoint size 253 != pinned 1192135096`.
2. **The baseline anchor** (Stage B). The F32 baseline's own 18-token C1 sequence is compared against the retained C1 fixture's `sequence.tsv`. The retained run reports `baseline_anchored_to_retained_fixture=True`: this host, at `transformers` 4.57.6 rather than the fixture's pinned 4.51.0, emits the identical 18 tokens. Absolute logit bits are **not** claimed to reproduce the fixture, and the harness warns rather than stopping when the anchor is unavailable, because a differential reading survives an unanchored baseline while an *absolute* one would not.
3. **Restore exactness** (Stage B). After all sixteen profile runs, the F32 baseline is recomputed and required to be bit-identical to the first one. Without this, a leaking restore would silently make each later profile a comparison against a different model. **Fault-proved:** dropping one tensor from every restore exits with `restore drifted: the post-run F32 baseline is not bit-identical`.

The conversion itself has a positive control too: for a linear ramp, `roundtrip`'s maximum absolute error is `0.06666672` against a half-scale of `0.06666667` at U4 and `0.00392163` against `0.00392157` at U8 — an affine round trip's error is bounded by half a step and reaches it, so a round trip that had quietly done nothing, or had wrapped instead of clamping, would not produce that number.

## What each stage computes

**Conversion.** Exactly what `tiler::strict-affine@1` registers in [`crates/tiler-ir/src/semantic/quantization.rs`](../../../crates/tiler-ir/src/semantic/quantization.rs): encode is f32 divide, add zero point, clamp to the inclusive code domain, round to nearest ties-to-even; decode is widen code and zero point to i32, subtract, convert to f32, multiply by the scale. The code domain is `[0, 15]` at U4 and `[0, 255]` at U8, and the scale is stored and applied as F32.

**Calibration is not that contract.** Choosing *which* scale and zero point a weight gets is an ingestion-side decision that Tiler does not define. The default here is asymmetric min-max with `+0.0` forced representable — `span = max(hi, 0) - min(lo, 0)`, `scale = span / code_max`, `zero_point = clamp(round_ties_even(-min(lo, 0) / scale))`. `calibration_sensitivity.py` exists because that choice is load-bearing enough to deserve a measurement rather than a caveat.

**Granularity.** `per-tensor` is one scale and zero point for the whole weight, the only parameter map Tiler implements. `per-channel` is one per output row — one per `o` in the workload's `td,od->to` structure, so the parameter is constant along the contracted axis. `per-groupN` is one per contiguous run of `N` elements *along the contracted axis*, so the parameter varies inside the reduction. That difference is what the research record turns into a legality result rather than an accuracy one.

**Byte cost.** Packed codes at the declared width, plus one F32 scale and one code-width zero point per group. The zero point costs the code width because its declared component type *is* the code type; charging it a byte would overstate every fine-grained profile.

**The BF16 control** (Stage B). Not a quantization profile: one storage width, no scheme, no scale, no zero point. It exists because the checkpoint's own weights are BF16 and the workload widens them to F32, so narrowing back is exact by construction, and any quantized candidate has to be judged against it rather than against F32.

## Retained records

| File | Contents |
| --- | --- |
| `results/2026-07-31-weight-error-…/per-tensor-error.tsv` | 1,576 rows: every (tensor, profile) pair's relative Frobenius error, maximum absolute error, scale range, and stored bytes |
| `…/profile-summary.tsv` | The eight profiles aggregated over all 595,984,384 elements |
| `…/calibration-sensitivity.tsv` | 192 rows: eight granularity/width pairs against three calibrations over layer 0 plus the embedding |
| `…/environment.tsv` | Pinned identity, tensor count, interpreter and numpy versions, host row |
| `results/2026-07-31-model-observable-…/model-observable.tsv` | 16 rows: model weight bytes, C1 sequence equality, per-position greedy agreement, whole-vocabulary and top-32 logit deviation from the F32 baseline, minimum runner-up gap |
| `…/environment.tsv` | The C1 constants, both sequences, the anchor and restore-exactness results, the installed reference source and its digest, and every version |

## Findings

**Measurement — per-tensor U4, the one profile Tiler has an executable vertical for, destroys the model.** Relative Frobenius error 0.4968 over the whole weight set; at the model level it agrees with the F32 baseline's greedy token at **0 of 18** C1 positions and emits a different sequence. Per-tensor U8 reaches 0.0315 and 8–9 of 18. Neither reproduces the C1 sequence.

**Measurement — finer parameter maps buy accuracy at almost no byte cost, and the ordering is calibration-robust.** At U8, per-channel is 0.00823 and per-group128 is 0.00605, against per-tensor's 0.0315 — a 4–5× error reduction for 0.3% and 3.9% more bytes respectively. Across all three calibrations and both widths, error decreases monotonically from per-tensor to per-channel to per-group128 to per-group32; no calibration reorders them.

**Measurement — calibration rescues coarse U4 partially and hurts U8.** Two-sided 99% clipping moves per-tensor U4 from 0.4736 to 0.1423 on the sampled set, and moves per-tensor U8 the wrong way, from 0.0287 to 0.0891: at eight bits the tail is worth resolving and at four bits it is not. Per-tensor U4's *best* calibration is still worse than per-group32 U4's worst.

**Measurement — every measured scale is a normal F32 by more than thirty orders of magnitude.** The smallest scale over all eight profiles and all 197 tensors is `1.358331e-05` (per-group128 U8); the largest is `1.536458e-01`. The F32 minimum normal is `2^-126 ≈ 1.1755e-38`, so the smallest measured scale exceeds it by a factor of about `1.2e33`.

**Measurement — the BF16 control is exact and halves the weight budget.** Replacing all 197 tensors with their BF16 round trip produces **bit-identical logits at every one of the 18 C1 positions** — maximum deviation `0.000000e+00` — at 0.500 of the F32 weight bytes. That is not a surprise, it is the definition: the checkpoint is BF16 and the workload widened it.

**Measurement — sequence equality is weaker than greedy agreement, demonstrated.** Per-channel U4 over the projections reproduces the C1 18-token sequence exactly while disagreeing with the baseline's argmax at 3 of 18 positions. A harness that checked only the emitted tokens would have reported that profile as matching.

**Measurement — one candidate agrees at every position.** Per-group128 U8 reaches 18 of 18 greedy agreement and the exact C1 sequence in both the projections-only and the projections-plus-embedding variant. Per-channel U8 reaches 17 of 18 and the exact sequence in both. Every U4 candidate fails at least one of the two in at least one variant.

**Measurement — quantization moves the logits far outside the F32 realization envelope.** L1 measured the pinned reference's own F32 reordering envelope at a whole-vocabulary maximum of `2.048e-4` and a top-32 maximum of `7.82e-5`. The gentlest candidate here, per-group128 U8, has a *median* whole-vocabulary deviation of `5.6e-2` and a top-32 maximum of `3.93`. **Inference.** A quantized program is a different computation, not a different realization of the same one, so the F32 comparison bound cannot be reused for it and the normative reference has to be the quantized program's own.

## Measurement boundary

- Every number is bound to the host and versions in each `environment.tsv`: an Apple M4 Max on macOS 27.0 arm64, CPython 3.11.13, numpy 2.4.6, and for Stage B `torch` 2.10.0 with `torch.set_num_threads(1)` and `transformers` 4.57.6.
- **Stage B does not run the pinned definitional reference.** The workload profile pins `transformers` 4.51.0 and digests three of its source files; the interpreter here provides 4.57.6, whose `modeling_qwen3.py` digest is recorded and is not one of the pinned three. Every reading is therefore a difference against a baseline computed by the *same* implementation in the *same* process, and the anchor check establishes only that this implementation emits the same 18 C1 tokens. A bounded-error claim against the retained fixture's values would exceed this record.
- One prompt, one checkpoint, 18 positions, batch 1, greedy. The C1 row was chosen because it is retainable and reproducible, not because it is representative of accuracy; it exercises no long context, no B1 row, and no second prompt. A profile that agrees at all 18 C1 positions has not been shown to agree anywhere else.
- Min-max and two quantile calibrations only. No MSE-optimal, no GPTQ/AWQ-style error-compensating calibration, no mixed-precision assignment, and no per-tensor search. A better calibration would improve the absolute numbers; the sweep bounds how much it can reorder the profiles, and does not bound how much it can improve them.
- **No Metal, no GPU, no Tiler execution.** Nothing here measures packed extraction, integer subtraction, integer-to-float conversion, or a quantized contraction on any device. Those are named as unmeasured in the research record and are not inferred from anything here.
- The activation path is not measured at all: only weights are replaced. An activation-quantized profile has different error behaviour and is out of scope.
