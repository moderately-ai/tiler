---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.qwen3-conformance-fixture"
kind: "experiment"
title: "Qwen3-0.6B-Base C1 conformance and attribution reference fixture"
topics: ["program-planning", "language-model", "conformance", "attribution", "numerics", "qwen"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.first-metal-lm-workload", "tiler.research.program-planning.complete-model-ingestion-and-execution", "tiler.research.program-planning.model-level-qualification"]
entrypoints: ["spikes/program-planning/qwen3-conformance-fixture/produce_fixture.py", "spikes/program-planning/qwen3-conformance-fixture/verify_fixture.py"]
last_verified: "2026-08-01"
ticket: "measure-the-model-level-comparison-envelope-under-the-target-realization"
---

# Qwen3-0.6B-Base C1 conformance and attribution reference fixture

Reference evidence for the C1 conformance row of [the first Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md): what the pinned `transformers` v4.51.0 `Qwen3ForCausalLM` implementation computes, in F32 on CPU, at every prefill position and every decode step of the profile's fixed 10-token prompt and 8-step decode budget.

The profile is the authority for every constant here — the pinned revision, the per-file manifest, the prompt token IDs, the decode budget, the termination rule, the tie policy. This spike transcribes them and checks against them; it re-derives none of them. The attribution surface's content is fixed the same way by [the L6 ingestion and execution record](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md).

The record answers two questions that are not the same question, and it keeps them apart:

- **Conformance** — does a candidate agree with the reference at the model boundary? That is the five observables over the retained logits, and it is the pass or fail.
- **Attribution** — *where* does it disagree? A forward pass is thirty executions over ten operation families and four host computations, and a logit vector carries no execution ordinal. The reference's own intermediate values are retained beside its logits so that a model-level disagreement lands on one of them.

Two different things are retained for each, and for one reason: a SHA-256 proves a reference regenerates *exactly* but cannot support a bounded-error comparison, which needs values.

**The record now also carries the joint comparison band, and still sets no threshold.** The band is the deviation of a pass carrying all three of the qualification record's named perturbations — P-reorder, P-flush, P-elem — against the plain F32 pass, measured before any Tiler result exists. What that band *gates* is not decided here: [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns the reading, and the corpus and regression tickets own any threshold over it.

## Prerequisites

- macOS on Apple silicon, and `uv`. The pinned Python environment is this directory's `pyproject.toml` and `uv.lock`; it is the one Python harness in this repository that pins, because here the dependency *is* the evidence — the retained digests are a fingerprint of one exact implementation's reduction order, so a floating resolution would silently re-baseline the fixture. `pyproject.toml` records that reasoning inline.
- About 1.2 GB of free disk for the checkpoint, which lands in the Hugging Face cache **outside this repository**, roughly 16 MB under `local-work/` for the regenerable F32 bytes, and roughly 5 GB of RAM for the float64 passes. The passes are sequential and each model is released before the next is loaded, so the peak is one model rather than the ten the run loads.
- Network access on the first run only. The producer calls `snapshot_download` at the pinned revision, so a populated cache makes later runs offline.

## Run it

No `make` target reaches a spike. From this directory:

```sh
# Produce (first run downloads the checkpoint; subsequent runs are about a quarter
# of an hour, most of it the eight float64-carrier passes and the 2.22 GiB weight
# digest). The invocation is unchanged by the joint band: one producer, one
# record, one evaluated F32 pass that everything else is compared against.
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python produce_fixture.py \
  --out results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0

# Verify a retained record without owning a model:
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python verify_fixture.py \
  results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0 \
  --logit-dir local-work/logits --attribution-dir local-work/attribution

# Reproducibility: regenerate everything and demand byte equality:
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python produce_fixture.py \
  --compare results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0
```

`UV_PROJECT_ENVIRONMENT` keeps the virtual environment inside `local-work/`, which the one gitignore entry beside this file already covers. Omitting it puts a `.venv/` in this directory that nothing ignores.

## What the producer checks before it computes anything

Each of these is fail-closed — a mismatch exits non-zero and writes nothing, because a run against different bytes is not a weaker fixture, it is a fixture for a different question.

1. **The checkpoint.** All nine manifest files are hashed locally after acquisition. This is the step that converts the profile's `model.safetensors` row from an API-reported Git-LFS object id into a digest computed from bytes; the profile's manifest table now records it as locally verified for that reason.
2. **The reference implementation.** The *installed* `modeling_qwen3.py`, `configuration_qwen3.py`, and `modeling_rope_utils.py` are hashed against the profile's pinned-commit digests. They match byte for byte, so "the pinned reference was evaluated" is a checked claim rather than an inference from a version string.
3. **The prompt.** The profile's recorded token IDs are re-encoded from the pinned `tokenizer.json` and round-tripped back to the prompt text.

Four more stop during the run, on the attribution surface rather than on identity: a hidden state, cache tensor, or rotary table whose shape is not the profile's; a mask entry that is neither of [L4](../../../docs/research/program-planning/first-attention-program-vertical.md)'s two admitted values; a retained byte total that is not L6's arithmetic; and a widened weight set that is not 310 tensors at 2,384,199,680 bytes.

Six more stop on the joint band: a run whose own smallest runner-up gap is no longer the transcribed 0.2660789489746094 the exact-greedy gate is derived against; a joint population that is not exactly the four declared variants; an unknown P-elem sign policy; an elementary result at the F32 range limit, where the perturbation's step is not finite and `tiler::ulp-reference-gap@1` is undefined anyway; an MLP activation that is not a SiLU, which `tiler::silu-f32@1` is the authority for and nothing else; and a joint variant that produced a different number of positions than the baseline.

## What is retained, and where the rest goes

Under `results/<slug>/`, all small enough to read:

| File | Contents |
| --- | --- |
| `environment.tsv` | The workload constants, the run configuration, the attribution layout rules, Python/torch/transformers/numpy versions, the host row, and every verified checkpoint and reference-source digest |
| `sequence.tsv` | The 18-token sequence: 10 prompt tokens and 8 generated tokens, each labelled |
| `positions.tsv` | Per position: SHA-256 over the exact F32 logit bit pattern, greedy token, runner-up, the gap, how many indices attained the maximum, and whether the top two are bit-identical |
| `top32.tsv` | The top-32 logits and indices per position, each as both an exact bit pattern and a round-trip decimal |
| `envelope.tsv` | Per position and variant: the deviation between the F32 pass and a float64 pass rounded to F32 |
| `joint.tsv` | Per position and joint variant: the deviation against the plain F32 pass under P-reorder, P-flush and P-elem applied together, whole-vocabulary and top-32, absolute and ULP, with the joint greedy token, its agreement, the position's runner-up gap and the ratio against it |
| `perturbation.tsv` | How each perturbation is defined and which authority sizes it, the P-flush controls in both arms, the two joint-pass controls, and the band summary with its ratio to the smallest runner-up gap |
| `hidden.tsv` | Per layer and position: SHA-256 over the 1,024-wide `h_out` slice, its exactly-rounded norm, and its largest-magnitude lane |
| `hidden_top.tsv` | The four largest-magnitude lanes of each `h_out` slice, as bit pattern and round-trip decimal |
| `cache.tsv` | Per layer, tensor, and position: SHA-256 over the `[8, 128]` post-RoPE `k_rope` or `v_heads` slice, its norm, and its largest-magnitude (head, lane) |
| `cache_top.tsv` | The four largest-magnitude entries of each cache slice |
| `rotary.tsv` | The rotary `cos` and `sin` rows in full — every one of the 18 × 128 entries, as bit pattern and round-trip decimal |
| `mask.tsv` | The additive causal mask in full — every entry of all nine passes, as a bit pattern |
| `host.tsv` | The four host computations' digests and populations, including the one digest over the widened F32 weights |
| `manifest.tsv` | SHA-256 over each retained file and over the producer, the validator, `pyproject.toml`, and `uv.lock` |

Three sets of complete F32 bytes are regenerable local data and are not version controlled:

| Under `local-work/` | Layout | Bytes |
| --- | --- | --- |
| `logits/position-NN.f32le.bin` | `[151936]` per position, 18 files | 10,939,392 |
| `attribution/hidden/layer-LL.f32le.bin` | `[18, 1024]` per layer, position-major, 28 files | 2,064,384 |
| `attribution/cache/layer-LL-{k_rope,v_heads}.f32le.bin` | `[8, 18, 128]` per layer and tensor, head-major, 56 files | 4,128,768 |

Every one is little-endian IEEE-754 binary32 and C-contiguous, and every retained digest is taken over exactly those bytes, so `verify_fixture.py --logit-dir --attribution-dir` re-hashes them whenever they are present.

**The digest unit is one position of one tensor, and that is what makes it attribution rather than a checksum.** A (layer, position) pair names one of the twenty-eight layer executions *and* the pass that produced the position — prefill for 0–9, decode *n* for 9 + *n* — which is exactly the resolution the model boundary lacks. Both units are 4,096 bytes, because a hidden slice is 1,024 F32 values and a cache slice is 8 × 128. They are reached differently and the record says so: a hidden position is the byte range `[p·4096, (p+1)·4096)` of its layer file, while a cache position is a *strided gather*, `tensor[:, p, :]` re-serialized contiguously, because the file keeps L4's and L5's declared `[8, S, 128]` head-major shape rather than being transposed to make the digest a byte range.

**The retained values are the reference's, at the reference's coordinates.** `hidden_top.tsv` and `cache_top.tsv` rank by descending |value| with ties toward the lower flat index. Magnitude rather than the signed order `top32.tsv` uses, because a hidden state has no ranking semantics to preserve — what a bounded comparison wants is the coordinates where an absolute deviation would be largest. A candidate is *indexed at* those coordinates rather than re-ranked, which is the same discipline `envelope.tsv` already uses when it restricts a deviation to the reference's own top-32 order.

**The norms are portable and nothing else here is.** `l2_norm` is computed by exactly-rounded summation over exact float64 squares — an F32 value's square needs at most 48 significand bits and stays inside float64's exponent range, and `math.fsum` is exactly rounded — so it depends on neither summation order nor host. It is the one figure a reader on another machine can compare directly, and it is what remains usable when the per-lane ranking has permuted.

## The four host computations

L6 fixes that removing a computation from the executed program moves it into the oracle's comparison surface rather than out of the system, so each is retained bit-exactly rather than assumed to agree. All four are here, and the first two are retained *in full* because they are small enough that "checkable in full" costs nothing.

- **The rotary `cos` and `sin` rows.** `[18, 128]` each, 9,216 bytes each, every entry in `rotary.tsv`. L2 moved their construction out of the executed program; this is where it reappears.
- **The additive causal mask.** All nine passes in `mask.tsv`, 216 entries, and the producer admits exactly two bit patterns: `0xff7fffff` masked and `0x80000000` attended — negative zero, because the reference multiplies the fill by a boolean rather than writing a zero.
- **The token IDs.** Already in `sequence.tsv`; `host.tsv` adds a digest over one declared serialization of them, little-endian int32 in sequence order, so the same evidence can be compared as bytes.
- **The BF16-to-F32 widening.** One digest over the widened bytes, taken from the *loaded parameters the F32 pass actually used* rather than from a re-derivation of them, absorbing for each of the checkpoint's 310 tensors in lexicographic name order the name's UTF-8 bytes, one NUL, and the tensor's little-endian C-contiguous F32 bytes.

Per-tensor weight digests are deliberately **not** retained. A total, injective map from checkpoint tensor name to interface key is [`define-the-model-weight-binding-manifest`](../../../tickets/define-the-model-weight-binding-manifest.md)'s subject, and a second naming authority for one subject is what that ticket exists to avoid.

## The joint comparison band

The band is one deviation between two complete computations, and that is the whole method. [Region accuracy contracts](../../../docs/research/numerics/region-accuracy-contract.md) establishes that an error bound is a relation between two complete computations and is not generally the sum of per-operation tolerances, because cancellation, correlated reuse, deleted materialization rounding points, and exceptional-value discontinuities all break the sum. Three separately measured maxima added together would be that same forbidden composition at a coarser granularity, so this record does not produce them: the four `joint_*` variants each apply all three perturbations at once, and the retained band is the maximum over positions and variants of that single jointly perturbed quantity.

The protocol is the L8 qualification record's, drafted at [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) in its *The bound: three named perturbations* section and pending its carrier.

| Perturbation | How this producer realizes it | Where its size comes from |
| --- | --- | --- |
| **P-reorder** | the two float64 orderings this record already carried, `f64_unmodified` and `f64_promoted`, reused rather than a third being invented | the retained `envelope.tsv` |
| **P-flush** | `torch.set_flush_denormal`, which sets ARM `FPCR.FZ` on this host, in force across the whole pass | the qualified `apple9-f32-unified-msl4-macos26` row's own measured sign-preserving flush of F32 input *and* result subnormals |
| **P-elem** | the exponential subordinate to the attention softmax and to the SiLU gate, and the reciprocal square root subordinate to RMS normalization, each moved to the edge of its admitted band | the **registered** contracts: `Ulp(tiler::ulp-reference-gap@1, 12)` and `Faithful` |

**The authority for P-elem is the registered contract and not Table 8.1, and the difference is a factor of three.** The Metal Shading Language Specification bounds `exp` at 4 ULP *under Apple's own ULP definition*; [the elementary-function accuracy record](../../../docs/research/numerics/metal-elementary-function-accuracy.md) derives that crossing to Tiler's metric as a factor of three under the conservative reading, so what the compiler promises is 12 ULP under `tiler::ulp-reference-gap@1`. A perturbation sized at 4 would measure a bound Tiler does not claim and would under-state the admissible band threefold. `verify_fixture.py` refuses a record whose retained size is not 12.

The perturbation is applied in the pass's own dtype but stepped by ULPs of the **F32** result the target would produce, because the reordering perturbation already stands in for the F32 rounding of the pass as a whole and re-rounding here would count it twice. `Faithful` admits one of the two F32 values bracketing the exact result, so its deviation is strictly below one ULP; the perturbation uses one ULP, the supremum of that band.

**Two sign policies, because neither is worst for both structures, and neither is a search.** `outward` moves every perturbed result away from zero: maximally correlated, and therefore worst where a perturbation propagates through a sum — the SiLU gate into `down_proj`, the RMS scale across the whole hidden vector — while cancelling exactly inside a softmax normalization, which divides by its own sum. `alternating` takes the sign from the result's own low mantissa bit, which is decorrelated from magnitude and so does not cancel there. Both are full-magnitude samples of the admitted band rather than a search over the 2^N per-element sign assignments, which is combinatorial and per output. **The consequence is stated rather than hidden: the true worst case within these contracts is at least the measured band and is not bounded above by it.**

**The softmax and the SiLU are expanded, and what that costs is measured rather than assumed.** The pinned spellings are fused kernels with no reachable exponential, so the joint passes evaluate `exp(x − max) / Σ exp(x − max)` and `x · σ(x)` through their two stable branches. That expansion is itself another legal ordering, which is admissible only because these passes already carry P-reorder — and the `elem_zero` control, the same pass with the perturbation at zero ULPs, is what shows the expansion is not the thing being measured.

### The P-flush mechanism is proved, not assumed

`torch.set_flush_denormal` returns `True` for **both** directions on this host, so a run that trusted its return value would report a flush it never performed. Two positive controls decide instead, in the same process, each watched in both arms:

- **elementwise** — `float32 (-1e-38) * 0.01`, whose exact result `-1e-40` is subnormal. Mode off returns `0x800116c2`, the exact subnormal; mode on returns `0x80000000`, a sign-preserving zero.
- **BLAS** — a `[64, 2] @ [2, 64]` `torch.matmul` whose exact sum is `-2^-133`. Mode off returns `0x80010000`; mode on returns `0x80000000`. Checked separately from the elementwise path because the two need not share the mode.

**The BLAS control is built so the sign is decided by the flush and not by the accumulator, and that construction is load-bearing.** A negative subnormal *product* summed against a `+0` accumulator yields `+0` by IEEE addition whether or not the flush preserved the sign — `(−0) + (+0) = +0` — so the obvious control cannot distinguish a sign-preserving flush from a sign-losing one. The control therefore contracts two normal terms, `2^-110` and `−(2^-110 + 2^-133)`, whose exact sum is the negative subnormal and in which no input or intermediate is subnormal, leaving the flush of the *result* as the only step that can set the sign. A `K = 64` variant of the same construction padded with zeros returns `+0` under the mode, for exactly the accumulator reason and not a mechanism defect.

## The 18 positions, and why the count is what it is

Prefill covers positions 0–9 in one pass with `logits_to_keep=0`, which the pinned source turns into `slice(0, None)` and therefore into every position rather than only the last. Eight decode passes then cover positions 10–17. The eighth pass consumes the eighth generated token, so the retained set is 18 logit vectors and the maximum context reached is 18 — both figures the profile's C1 table states. The argmax at position 17 is retained per position but is not appended, because appending it would spend a ninth decode step the budget does not have.

Greedy selection applies the profile's declared tie policy explicitly — the lowest vocabulary index among all indices attaining the maximum — rather than inheriting whatever `torch.argmax` returns for a tie.

## Findings

**Measurement — the generated continuation, on the host row in `environment.tsv`.** The 18-token sequence is the 10 prompt tokens followed by `576, 3974, 13876, 38835, 34208, 916, 279, 15678`: the base model restarts the pangram. Termination was the 8-step budget; EOS 151643 never appeared.

**Measurement — there are no bit-identical ties.** At all 18 positions exactly one index attains the maximum, and the top two logits are never bit-identical. The tie branch of the oracle is therefore unexercised by this row, which is a fact about this row and not evidence that the branch is unreachable. The smallest runner-up gap is 0.2660789489746094 at position 10; the largest is 7.850418 at position 6.

**Measurement — the F32 sensitivity envelope.** Across all 18 positions, the largest whole-vocabulary deviation between the F32 pass and a float64 pass rounded to F32 is 2.048e-4 (position 0); restricted to the top-32 entries it is 7.82e-5, or at most 78 ULP. The greedy token agrees at every position under both float64 variants. Between 483 and 3,863 of the 151,936 logits per position are bit-identical between the two orderings — that is, under 3% — so agreement of the argmax coexists with almost every individual logit differing.

**Inference — the decision margin is wide here, and that is a property of this row.** The worst deviation, 2.05e-4, is about 1,300× smaller than the smallest runner-up gap, 0.266, which is why greedy agreement survives reordering at every position. This says nothing about a row with a narrower margin, and it is not a bound: an envelope measured on one prompt and one checkpoint qualifies that prompt and that checkpoint.

**Fact — the pinned reference's float32 spellings are unconditional, so a float64 run needs a decision.** Three sites in `modeling_qwen3.py` cast to float32 regardless of model dtype: `Qwen3RMSNorm.forward` at line 73, the softmax in `eager_attention_forward` at line 162, and the `.float()` calls building the RoPE table in `Qwen3RotaryEmbedding.forward` at lines 336–344. The profile already records these as upcasts, which is what they are for a BF16 or F32 model. At model dtype float64 they are **downcasts**, and they sit at the mean-of-squares normalization, the attention softmax, and the rotary table — three of the most cancellation-prone stages in the model. In an unmodified float64 pass they round identically to the F32 pass and contribute exactly zero to the measured deviation.

That is why `envelope.tsv` carries two variants rather than one:

- `f64_unmodified` — the pinned reference verbatim at dtype float64. No patching, reproduces from the checked-in environment alone, and is the conservative floor.
- `f64_promoted` — the same three sites promoted to float64, each a line-for-line copy of the pinned source with only the float32 spelling changed.

**Measurement — promoting the three sites barely moves the envelope.** `f64_promoted` reaches 2.007e-4 against `f64_unmodified`'s 2.048e-4, and the top-32 ULP maximum is 78 for both. **Inference.** On this row the dominant divergence source is the reduction order of the contractions, not the rounding of the normalization, softmax, or rotary table. The unmodified pass was therefore not materially understating the envelope — but that is a measured outcome, not something the unmodified pass could have told you, which is why both are retained.

**Measurement — the joint band.** Across all 18 positions and all four joint variants, the largest whole-vocabulary deviation from the plain F32 pass is **2.2101e-4**; restricted to the reference's own top-32 entries it is **1.0872e-4**, or at most **87 ULP**. The band is set at position 0 by `joint_unmodified_alternating`; the top-32 ULP maximum of 87 is set at position 6 by `joint_promoted_outward`. Between 397 and 3,636 of each position's 151,936 logits are bit-identical to the baseline under a joint variant — under 2.4%.

**Measurement — the greedy token agrees at every position under every joint variant.** All 72 rows of `joint.tsv` agree with the baseline's greedy token. Whether a joint pass produced a bit-identical top-two pair is *not* measured: `joint.tsv` records the joint greedy token and its agreement, and the tie state stays a property of the baseline in `positions.tsv`.

**Measurement — the band against the smallest runner-up gap, and the gate that rests on it.** The smallest runner-up gap across the 18 positions is **0.2660789489746094**, at position 10. The band's ratio against it is **8.3063e-4**, so the band is about **1,204×** smaller than the gap and the exact-greedy gate the qualification record derives **holds** on this row. It is not close: the loosest position-by-position ratio is 1.52e-4 at position 1, and no position exceeds 1.6e-4.

**Measurement — P-elem widened the envelope by about eight per cent, and the expansion it needed cost nothing.** The joint band, 2.2101e-4, sits 7.9% above the reordering-only envelope's 2.048e-4. The `elem_zero` control — the same pass as `joint_unmodified_outward` with the perturbation at zero ULPs — reaches exactly `0.0002048015594482422`, the `f64_unmodified` figure to the last bit, so the expanded softmax and SiLU spellings contribute nothing of their own and everything above the envelope is the contract-sized perturbation. **Inference.** On this row reduction order remains the dominant term: 12 ULP of admitted exponential error and one ULP of admitted reciprocal-square-root error move the model boundary by under a tenth of what reordering does.

**Measurement — and the two sign policies differ by almost an order of magnitude, which is why both are kept.** Against the same `elem_zero` control, `joint_unmodified_outward` reaches `0.0002067089080810547` — a **0.93%** move — while `joint_unmodified_alternating` reaches 2.2101e-4, a **7.9%** move. **Inference — that gap is the softmax normalization behaving exactly as the policy design predicted.** A uniform relative perturbation of every exponential very nearly cancels in a quotient by their own sum, so `outward` reaches the model boundary mostly through the SiLU gate and the RMS scale; a sign taken from the result's own low mantissa bit does not cancel there. A record that had run only the correlated policy would have measured P-elem's contribution at roughly an eighth of its size, reported a band of 2.067e-4 rather than 2.2101e-4, and looked like a clean measurement while doing it.

**Measurement — P-flush is established, and it is the identity on this row.** Both controls passed in both arms with the sign preserved on the elementwise *and* the BLAS path, so the mechanism is proved rather than assumed. Its effect was then measured rather than presumed: the plain F32 pass re-evaluated with the mode in force is **bit-identical to the baseline at all 18 positions**, so no arithmetic site of this row produced or consumed an F32 subnormal, and the flush changes nothing to change. The joint band therefore carries all three terms. The carrier control agrees from the other side — the float64 joint pass is byte-identical with the mode off, which is what a pass with no subnormals in range must be. **Inference — and this is a fact about this row's dynamic range, not about the mechanism.** The weights supply no subnormal; the masked softmax entries underflow to exactly zero rather than passing through the subnormal range; and no attended score, gate activation, or normalized state on this prompt reaches the roughly `2^-126` floor. A prompt or a checkpoint that did would make P-flush a live term and would need this measurement re-derived rather than reused.

**Correction — 2026-08-02, on why the weights supply none.** This paragraph previously gave the reason as "L1 records BF16 as a truncated F32 so even a BF16 subnormal widens to an F32 *normal*", and that reason is false: BF16 shares binary32's exponent width, so widening preserves the subnormal class, measured exhaustively at 254 of 254 in [the BF16 conversion record](../../../docs/research/numerics/bf16-computation-accumulator-and-conversion.md) and stated from the target side in [the Apple numerical-behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md). The measured reason is a counted property of the pinned revision rather than of the format: **0 subnormal, 0 infinite, and 0 NaN stored values over all 596,049,920 elements of all 310 tensors**, from [the corpus reachability probe](../qwen3-corpus-reachability/README.md). Nothing measured in this record changes — the bit-identity above was measured, not inferred from the removed reason — and the sentence that follows it, that another checkpoint would make P-flush live, is now the operative one rather than a remote hypothetical.

**Fact — `output_hidden_states=True` does not return the twenty-eight `h_out` tensors, and using it would have retained a different surface.** `Qwen3Model.forward` appends `hidden_states` at the *top* of each layer iteration and appends `self.norm(hidden_states)` after the loop, so the returned tuple is the embedding output, twenty-seven layer outputs, and the normed final state. Layer 27's own `h_out` — one of the twenty-eight tensors this surface exists for — never appears in it. The producer therefore hooks each `Qwen3DecoderLayer`, which returns all twenty-eight and changes no value. The mask and the rotary rows are read from the same call for the same reason: recomputing them beside the run would retain a lookalike of the host computation rather than the one the retained logits were produced under.

**Measurement — the widening is bit-exact at every tensor.** All 310 checkpoint tensors are BF16, all 310 widen to F32 with identical bit patterns to `stored.to(torch.float32)`, and the widened total is 2,384,199,680 bytes — L1's F32 weight budget, recomputed here from the loaded parameters rather than carried forward.

**Measurement — the decode masks are fully attended, and the prefill mask is the exact lower triangle.** The prefill instance is `[10, 10]` at 400 bytes with 55 attended entries — the row counts `1 2 3 4 5 6 7 8 9 10` L4 recorded. Each decode pass presents a `[1, S]` mask with all `S` entries attended, because a single new position at cache position `S − 1` is causally allowed every key. No entry in any of the nine passes carried a third bit pattern.

**Measurement — extending the producer did not perturb the pass it measures.** `sequence.tsv`, `positions.tsv`, `top32.tsv`, and `envelope.tsv` are byte-identical to the superseded 2026-07-31 conformance-only record, and `environment.tsv` differs only by the added `attribution.*` keys. The hooks observe; they do not participate. Adding the joint band held the same line a second time: every previously retained file except `environment.tsv` and `manifest.tsv` regenerated byte-identically, `environment.tsv` gained five `joint.*` keys and removed none, and the F32 pass the whole record is built on is the one it always was. `produce_fixture.py --compare` then regenerated the complete production a second time and all **15** retained files matched byte for byte, so the joint passes — the flush mode among them — are as deterministic on this host as the passes they were added beside.

## How drift is actually caught

Three layers, all demonstrated against deliberate perturbations rather than assumed:

- The **identity manifests** stop the producer before it computes anything: a checkpoint file, a reference source, or a prompt tokenization that is not the pinned one exits 4 and writes nothing.
- `manifest.tsv` catches any byte that changed in a retained file, and also catches a record whose producer, validator, or pinned environment is no longer the one that made it.
- `verify_fixture.py`'s cross-file checks catch an edit made consistently enough to survive a re-hashed manifest: a greedy token that no longer agrees with the top-32 head, a gap that is no longer the difference of the two logits it derives from, a generated token that is no longer the argmax of the position that produced it, a top-32 ordering that violates descending logit with ties broken toward the lower index, a top-32 deviation exceeding the whole-vocabulary deviation it is a subset of, an attribution head whose rank-0 entry disagrees with the extremum it summarizes, a recorded norm below the largest component magnitude it must dominate, a rotary table whose two 64-wide halves are no longer the duplication `cat((freqs, freqs))` builds, a position-0 rotary row that is not exactly `cos 1.0` and `sin 0.0`, and a mask entry that is either not two-valued or admitted where causality forbids it.

Every one of those attribution checks is exact — a structural identity, a decode round trip, or an inequality that holds by definition — so none of them smuggles in a threshold. Every check names its population and counts it, so a clean run prints how many files were re-hashed, how many slices were examined, and how many half-duplication pairs were compared. A validator that silently examined nothing would otherwise be indistinguishable from a validator that found nothing wrong.

The joint band adds a fourth layer, and it is the same discipline applied to a summary rather than to a value: a band that is not the maximum over the rows it summarizes, a ratio that is not the deviation divided by the gap the same row records, a gap that is not the one `positions.tsv` records for that position, an agreement flag that disagrees with the greedy token it is derived from, a gate verdict that disagrees with the band it rests on, an exponential resized away from `Ulp(tiler::ulp-reference-gap@1, 12)` toward Table 8.1's 4, a P-flush control whose two arms returned the same bit pattern and therefore cannot say no, a term state that does not follow from its own controls and reachability count, and a zero-magnitude P-elem control that is not below the variant it controls, does not name a variant `joint.tsv` carries, or summarizes that variant with something other than its own maximum. **Twelve deliberate one-value perturbations were applied to a scratch copy of the record with `manifest.tsv` consistently re-hashed each time, and all twelve were refused at exit 5, with the unperturbed copy passing in the same matrix.** On the producer side, six guards were driven to their stop with the input each exists to refuse — a moved runner-up-gap minimum, a joint population one variant short, an unknown sign policy, an elementary result at the F32 range limit, an activation that is not a SiLU, and a joint variant that produced 17 positions — each exiting 4, and each with its positive arm watched passing.

## Measurement boundary

- Every number here is bound to the exact host, toolchain, and thread count in `environment.tsv`: an Apple M4 Max on macOS 27.0 build 26A5388g, `torch` 2.6.0 with `torch.set_num_threads(1)`. Single-threaded evaluation removes intra-op reduction-order variation as a source of digest drift between runs on this host; it does **not** make the digests portable to a different CPU, a different BLAS, or a different thread count. A digest mismatch on another host is expected and is not by itself evidence of a defect. The `l2_norm` columns are the exception and say why above.
- The envelope is a bounded measurement over one prompt, one checkpoint, and 18 positions. It is the smallest deviation any correct F32 realization could be required to fall inside for *this* row. It is not a universal claim, not a tolerance, and not a budget. **The attribution surface carries no envelope at all** — the float64 passes are not hooked, so nothing here says how far an intermediate may deviate, only what it was.
- The attribution surface is the reference's intermediates, not Tiler's. No Tiler execution, no Metal work, and no comparison has been run against it; what exists is the surface a comparison would run over. Whether a per-execution disagreement *implies* a model-level one, and by how much, is a question about error propagation that no measurement here touches.
- The float64 passes are teacher-forced on the F32 pass's token sequence so that every position compares the same inputs. Without forcing, a single argmax flip would make every later position a comparison of two different computations. Each pass still records its own argmax per position, so a flip would remain visible; none occurred.
- The reference is a CPU float32 implementation that preserves subnormal intermediates, and the qualified Apple9/F32 Metal row flushes F32 subnormals to sign-preserving zero. That divergence source is now measured rather than only named — the mechanism is proved by two positive controls and the flush is bit-identical to the baseline on this row — but what is measured is that *this* prompt and *this* checkpoint never reach the F32 subnormal range. It is not a claim that the reference and the target agree on subnormals, and a row that reached them would need the measurement re-derived.
- **The joint band is admissibility, not proof, and the asymmetry is what makes it usable.** A Tiler result *outside* the band is a defect: no legal realization of this program under these contracts on this target could produce it. A Tiler result *inside* the band is not thereby proven correct — it is only indistinguishable, at the model boundary, from a legal realization. That is why the bound is one of five observables rather than the whole oracle.
- **The band samples the admitted band at full magnitude; it does not search it.** Two sign policies are retained and neither is a search over the 2^N per-element sign assignments, so the true worst case within these registered contracts is at least the measured band and is not bounded above by it. A tighter statement would need a per-output adversarial search this measurement does not perform.
- The joint band qualifies one prompt, one checkpoint revision, one reference revision, 18 positions, batch 1, greedy, F32, on the host row in `environment.tsv`. It says nothing about a B1-length row, another prompt, another checkpoint, or the quantized path, and it is not a threshold: what it gates belongs to the corpus and regression tickets.
- No B1 benchmark row. It is excluded by the ticket, and its logits alone would be 296.8 MiB at one prompt length.
