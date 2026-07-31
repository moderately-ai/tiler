---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.qwen3-conformance-fixture"
kind: "experiment"
title: "Qwen3-0.6B-Base C1 conformance reference logit fixture"
topics: ["program-planning", "language-model", "conformance", "numerics", "qwen"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.first-metal-lm-workload"]
entrypoints: ["spikes/program-planning/qwen3-conformance-fixture/produce_fixture.py", "spikes/program-planning/qwen3-conformance-fixture/verify_fixture.py"]
last_verified: "2026-07-31"
ticket: "retain-the-qwen-conformance-reference-logit-fixture"
---

# Qwen3-0.6B-Base C1 conformance reference logit fixture

Reference evidence for the C1 conformance row of [the first Metal language-model workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md): what the pinned `transformers` v4.51.0 `Qwen3ForCausalLM` implementation computes, in F32 on CPU, at every prefill position and every decode step of the profile's fixed 10-token prompt and 8-step decode budget.

The profile is the authority for every constant here — the pinned revision, the per-file manifest, the prompt token IDs, the decode budget, the termination rule, the tie policy. This spike transcribes them and checks against them; it re-derives none of them.

It exists so that a later rung can detect drift without re-deriving what the reference should have said. That is why two different things are retained rather than one: a SHA-256 per position proves a reference regenerates *exactly*, but it cannot support a bounded-error comparison, which needs values; the top-32 slices are the values.

**Nothing here sets a comparison budget.** [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns that. This spike supplies the measured half it would otherwise have to produce from scratch.

## Prerequisites

- macOS on Apple silicon, and `uv`. The pinned Python environment is this directory's `pyproject.toml` and `uv.lock`; it is the one Python harness in this repository that pins, because here the dependency *is* the evidence — the retained digests are a fingerprint of one exact implementation's reduction order, so a floating resolution would silently re-baseline the fixture. `pyproject.toml` records that reasoning inline.
- About 1.2 GB of free disk for the checkpoint, which lands in the Hugging Face cache **outside this repository**, and roughly 5 GB of RAM for the float64 passes.
- Network access on the first run only. The producer calls `snapshot_download` at the pinned revision, so a populated cache makes later runs offline.

## Run it

No `make` target reaches a spike. From this directory:

```sh
# Produce (first run downloads the checkpoint; subsequent runs are ~6 s):
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python produce_fixture.py \
  --out results/2026-07-31-c1-conformance-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0

# Verify a retained record without owning a model:
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python verify_fixture.py \
  results/2026-07-31-c1-conformance-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0 \
  --logit-dir local-work/logits

# Reproducibility: regenerate everything and demand byte equality:
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python produce_fixture.py \
  --compare results/2026-07-31-c1-conformance-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0
```

`UV_PROJECT_ENVIRONMENT` keeps the virtual environment inside `local-work/`, which the one gitignore entry beside this file already covers. Omitting it puts a `.venv/` in this directory that nothing ignores.

## What the producer checks before it computes anything

Each of these is fail-closed — a mismatch exits non-zero and writes nothing, because a run against different bytes is not a weaker fixture, it is a fixture for a different question.

1. **The checkpoint.** All nine manifest files are hashed locally after acquisition. This is the step that converts the profile's `model.safetensors` row from an API-reported Git-LFS object id into a digest computed from bytes; the profile's manifest table now records it as locally verified for that reason.
2. **The reference implementation.** The *installed* `modeling_qwen3.py`, `configuration_qwen3.py`, and `modeling_rope_utils.py` are hashed against the profile's pinned-commit digests. They match byte for byte, so "the pinned reference was evaluated" is a checked claim rather than an inference from a version string.
3. **The prompt.** The profile's recorded token IDs are re-encoded from the pinned `tokenizer.json` and round-tripped back to the prompt text.

## What is retained, and where the rest goes

Under `results/<slug>/`, all small enough to read:

| File | Contents |
| --- | --- |
| `environment.tsv` | The workload constants, the run configuration, Python/torch/transformers/numpy versions, the host row, and every verified checkpoint and reference-source digest |
| `sequence.tsv` | The 18-token sequence: 10 prompt tokens and 8 generated tokens, each labelled |
| `positions.tsv` | Per position: SHA-256 over the exact F32 logit bit pattern, greedy token, runner-up, the gap, how many indices attained the maximum, and whether the top two are bit-identical |
| `top32.tsv` | The top-32 logits and indices per position, each as both an exact bit pattern and a round-trip decimal |
| `envelope.tsv` | Per position and variant: the deviation between the F32 pass and a float64 pass rounded to F32 |
| `manifest.tsv` | SHA-256 over each retained file and over the producer, the validator, `pyproject.toml`, and `uv.lock` |

The complete F32 logit bytes — 18 × 607,744 = 10,939,392 bytes — are regenerable local data under `local-work/logits/`, one little-endian IEEE-754 binary32 C-contiguous file per position, and are not version controlled. The per-position digests in `positions.tsv` are taken over exactly those bytes, so `verify_fixture.py --logit-dir` re-hashes them whenever they are present.

## The 18 positions, and why the count is what it is

Prefill covers positions 0–9 in one pass with `logits_to_keep=0`, which the pinned source turns into `slice(0, None)` and therefore into every position rather than only the last. Eight decode passes then cover positions 10–17. The eighth pass consumes the eighth generated token, so the retained set is 18 logit vectors and the maximum context reached is 18 — both figures the profile's C1 table states. The argmax at position 17 is recorded per position but is not appended to the sequence: appending it would spend a ninth decode step the budget does not have.

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

## How drift is actually caught

Two layers, both demonstrated against deliberate perturbations rather than assumed:

- `manifest.tsv` catches any byte that changed in a retained file, and also catches a record whose producer, validator, or pinned environment is no longer the one that made it.
- `verify_fixture.py`'s cross-file checks catch an edit made consistently enough to survive a re-hashed manifest: a greedy token that no longer agrees with the top-32 head, a gap that is no longer the difference of the two logits it derives from, a generated token that is no longer the argmax of the position that produced it, a top-32 ordering that violates descending logit with ties broken toward the lower index, a top-32 deviation exceeding the whole-vocabulary deviation it is a subset of.

Every check names its population and counts it, so a clean run prints how many files were re-hashed and how many positions were examined. A validator that silently examined nothing would otherwise be indistinguishable from a validator that found nothing wrong.

## Measurement boundary

- Every number here is bound to the exact host, toolchain, and thread count in `environment.tsv`: an Apple M4 Max on macOS 27.0 build 26A5388g, `torch` 2.6.0 with `torch.set_num_threads(1)`. Single-threaded evaluation removes intra-op reduction-order variation as a source of digest drift between runs on this host; it does **not** make the digests portable to a different CPU, a different BLAS, or a different thread count. A digest mismatch on another host is expected and is not by itself evidence of a defect.
- The envelope is a bounded measurement over one prompt, one checkpoint, and 18 positions. It is the smallest deviation any correct F32 realization could be required to fall inside for *this* row. It is not a universal claim, not a tolerance, and not a budget.
- The float64 passes are teacher-forced on the F32 pass's token sequence so that every position compares the same inputs. Without forcing, a single argmax flip would make every later position a comparison of two different computations. Each pass still records its own argmax per position, so a flip would remain visible; none occurred.
- The reference is a CPU float32 implementation that preserves subnormal intermediates. The qualified Apple9/F32 Metal row flushes F32 subnormals to sign-preserving zero. That divergence source is real, is named in the workload profile, and is **not** measured here.
- No Tiler execution, no Metal work, and no B1 benchmark row. Those are excluded by the ticket.
