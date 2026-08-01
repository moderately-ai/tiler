---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.qwen3-conformance-fixture"
kind: "experiment"
title: "Qwen3-0.6B-Base C1 conformance and attribution reference fixture"
topics: ["program-planning", "language-model", "conformance", "attribution", "numerics", "qwen"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.first-metal-lm-workload", "tiler.research.program-planning.complete-model-ingestion-and-execution"]
entrypoints: ["spikes/program-planning/qwen3-conformance-fixture/produce_fixture.py", "spikes/program-planning/qwen3-conformance-fixture/verify_fixture.py"]
last_verified: "2026-08-01"
ticket: "retain-the-c1-model-attribution-fixture"
---

# Qwen3-0.6B-Base C1 conformance and attribution reference fixture

Reference evidence for the C1 conformance row of [the first Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md): what the pinned `transformers` v4.51.0 `Qwen3ForCausalLM` implementation computes, in F32 on CPU, at every prefill position and every decode step of the profile's fixed 10-token prompt and 8-step decode budget.

The profile is the authority for every constant here — the pinned revision, the per-file manifest, the prompt token IDs, the decode budget, the termination rule, the tie policy. This spike transcribes them and checks against them; it re-derives none of them. The attribution surface's content is fixed the same way by [the L6 ingestion and execution record](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md).

The record answers two questions that are not the same question, and it keeps them apart:

- **Conformance** — does a candidate agree with the reference at the model boundary? That is the five observables over the retained logits, and it is the pass or fail.
- **Attribution** — *where* does it disagree? A forward pass is thirty executions over ten operation families and four host computations, and a logit vector carries no execution ordinal. The reference's own intermediate values are retained beside its logits so that a model-level disagreement lands on one of them.

Two different things are retained for each, and for one reason: a SHA-256 proves a reference regenerates *exactly* but cannot support a bounded-error comparison, which needs values.

**Nothing here sets a comparison budget.** [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns that. This spike supplies the measured half it would otherwise have to produce from scratch, and the surface a diagnosis would run over.

## Prerequisites

- macOS on Apple silicon, and `uv`. The pinned Python environment is this directory's `pyproject.toml` and `uv.lock`; it is the one Python harness in this repository that pins, because here the dependency *is* the evidence — the retained digests are a fingerprint of one exact implementation's reduction order, so a floating resolution would silently re-baseline the fixture. `pyproject.toml` records that reasoning inline.
- About 1.2 GB of free disk for the checkpoint, which lands in the Hugging Face cache **outside this repository**, roughly 16 MB under `local-work/` for the regenerable F32 bytes, and roughly 5 GB of RAM for the float64 passes.
- Network access on the first run only. The producer calls `snapshot_download` at the pinned revision, so a populated cache makes later runs offline.

## Run it

No `make` target reaches a spike. From this directory:

```sh
# Produce (first run downloads the checkpoint; subsequent runs are a few minutes,
# most of it the two float64 passes and the 2.22 GiB weight digest):
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

## What is retained, and where the rest goes

Under `results/<slug>/`, all small enough to read:

| File | Contents |
| --- | --- |
| `environment.tsv` | The workload constants, the run configuration, the attribution layout rules, Python/torch/transformers/numpy versions, the host row, and every verified checkpoint and reference-source digest |
| `sequence.tsv` | The 18-token sequence: 10 prompt tokens and 8 generated tokens, each labelled |
| `positions.tsv` | Per position: SHA-256 over the exact F32 logit bit pattern, greedy token, runner-up, the gap, how many indices attained the maximum, and whether the top two are bit-identical |
| `top32.tsv` | The top-32 logits and indices per position, each as both an exact bit pattern and a round-trip decimal |
| `envelope.tsv` | Per position and variant: the deviation between the F32 pass and a float64 pass rounded to F32 |
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

**Fact — `output_hidden_states=True` does not return the twenty-eight `h_out` tensors, and using it would have retained a different surface.** `Qwen3Model.forward` appends `hidden_states` at the *top* of each layer iteration and appends `self.norm(hidden_states)` after the loop, so the returned tuple is the embedding output, twenty-seven layer outputs, and the normed final state. Layer 27's own `h_out` — one of the twenty-eight tensors this surface exists for — never appears in it. The producer therefore hooks each `Qwen3DecoderLayer`, which returns all twenty-eight and changes no value. The mask and the rotary rows are read from the same call for the same reason: recomputing them beside the run would retain a lookalike of the host computation rather than the one the retained logits were produced under.

**Measurement — the widening is bit-exact at every tensor.** All 310 checkpoint tensors are BF16, all 310 widen to F32 with identical bit patterns to `stored.to(torch.float32)`, and the widened total is 2,384,199,680 bytes — L1's F32 weight budget, recomputed here from the loaded parameters rather than carried forward.

**Measurement — the decode masks are fully attended, and the prefill mask is the exact lower triangle.** The prefill instance is `[10, 10]` at 400 bytes with 55 attended entries — the row counts `1 2 3 4 5 6 7 8 9 10` L4 recorded. Each decode pass presents a `[1, S]` mask with all `S` entries attended, because a single new position at cache position `S − 1` is causally allowed every key. No entry in any of the nine passes carried a third bit pattern.

**Measurement — extending the producer did not perturb the pass it measures.** `sequence.tsv`, `positions.tsv`, `top32.tsv`, and `envelope.tsv` are byte-identical to the superseded 2026-07-31 conformance-only record, and `environment.tsv` differs only by the added `attribution.*` keys. The hooks observe; they do not participate.

## How drift is actually caught

Three layers, all demonstrated against deliberate perturbations rather than assumed:

- The **identity manifests** stop the producer before it computes anything: a checkpoint file, a reference source, or a prompt tokenization that is not the pinned one exits 4 and writes nothing.
- `manifest.tsv` catches any byte that changed in a retained file, and also catches a record whose producer, validator, or pinned environment is no longer the one that made it.
- `verify_fixture.py`'s cross-file checks catch an edit made consistently enough to survive a re-hashed manifest: a greedy token that no longer agrees with the top-32 head, a gap that is no longer the difference of the two logits it derives from, a generated token that is no longer the argmax of the position that produced it, a top-32 ordering that violates descending logit with ties broken toward the lower index, a top-32 deviation exceeding the whole-vocabulary deviation it is a subset of, an attribution head whose rank-0 entry disagrees with the extremum it summarizes, a recorded norm below the largest component magnitude it must dominate, a rotary table whose two 64-wide halves are no longer the duplication `cat((freqs, freqs))` builds, a position-0 rotary row that is not exactly `cos 1.0` and `sin 0.0`, and a mask entry that is either not two-valued or admitted where causality forbids it.

Every one of those attribution checks is exact — a structural identity, a decode round trip, or an inequality that holds by definition — so none of them smuggles in a threshold. Every check names its population and counts it, so a clean run prints how many files were re-hashed, how many slices were examined, and how many half-duplication pairs were compared. A validator that silently examined nothing would otherwise be indistinguishable from a validator that found nothing wrong.

## Measurement boundary

- Every number here is bound to the exact host, toolchain, and thread count in `environment.tsv`: an Apple M4 Max on macOS 27.0 build 26A5388g, `torch` 2.6.0 with `torch.set_num_threads(1)`. Single-threaded evaluation removes intra-op reduction-order variation as a source of digest drift between runs on this host; it does **not** make the digests portable to a different CPU, a different BLAS, or a different thread count. A digest mismatch on another host is expected and is not by itself evidence of a defect. The `l2_norm` columns are the exception and say why above.
- The envelope is a bounded measurement over one prompt, one checkpoint, and 18 positions. It is the smallest deviation any correct F32 realization could be required to fall inside for *this* row. It is not a universal claim, not a tolerance, and not a budget. **The attribution surface carries no envelope at all** — the float64 passes are not hooked, so nothing here says how far an intermediate may deviate, only what it was.
- The attribution surface is the reference's intermediates, not Tiler's. No Tiler execution, no Metal work, and no comparison has been run against it; what exists is the surface a comparison would run over. Whether a per-execution disagreement *implies* a model-level one, and by how much, is a question about error propagation that no measurement here touches.
- The float64 passes are teacher-forced on the F32 pass's token sequence so that every position compares the same inputs. Without forcing, a single argmax flip would make every later position a comparison of two different computations. Each pass still records its own argmax per position, so a flip would remain visible; none occurred.
- The reference is a CPU float32 implementation that preserves subnormal intermediates. The qualified Apple9/F32 Metal row flushes F32 subnormals to sign-preserving zero. That divergence source is real, is named in the workload profile, and is **not** measured here.
- No B1 benchmark row. It is excluded by the ticket, and its logits alone would be 296.8 MiB at one prompt length.
