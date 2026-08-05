---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.qwen3-corpus-reachability"
kind: "experiment"
title: "Qwen3-0.6B-Base conformance-corpus reachability probe"
topics: ["program-planning", "language-model", "conformance", "qwen", "numerics", "subnormals"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["exhaustive-finite", "bounded-measurement"]
supports: ["tiler.research.program-planning.model-level-qualification", "tiler.research.program-planning.first-metal-lm-workload"]
entrypoints: ["spikes/program-planning/qwen3-corpus-reachability/probe_corpus.py"]
last_verified: "2026-08-02"
ticket: "define-the-model-level-conformance-corpus"
---

# Qwen3-0.6B-Base conformance-corpus reachability probe

**What it answers.** Three rows of [the model-level adversarial corpus](../../../docs/research/program-planning/model-level-qualification.md#the-adversarial-corpus-derived-from-refusals-that-already-exist) turn on whether the pinned checkpoint can reach a condition at all, and each would otherwise be settled by an assumption. This probe answers all three from the same verified bytes in one run, so a reader gets one environment row and one manifest instead of three probes that could have run against different files.

| Question | Corpus consequence | Answer this probe measured |
| --- | --- | --- |
| Can a position of any prompt reach a bit-identical top-two logit pair? | whether `A-tie` is a demonstrating row or a recorded negative | **Not found.** 330 evaluated positions over 19 prompts, 0 with a bit-identical top-two pair. The structural route exists — 2,226 of 151,936 vocabulary entries share an embedding row with another — and did not fire: the best-placed duplicate-group member ranked 86,718th at its best and sat 17.45 logits below the maximum. |
| Does a BF16 subnormal widen to an F32 normal? | the stated ground for the corpus's deliberately absent subnormal-weight row | **No, and the population is exhaustive.** All 65,536 BF16 bit patterns were widened and classified: a stored subnormal widens to an F32 **subnormal** in all 254 cases and to a normal in none. Every class is preserved. |
| Does the checkpoint hold a subnormal, NaN, or infinite stored value? | whether the absent subnormal-weight row and the ingestion ticket's non-finite check have anything to catch here | **No.** Over 596,049,920 stored elements in all 310 tensors: 596,049,699 normal, 221 zero, and 0 subnormal, 0 infinite, 0 NaN. |

**What it does not answer.** It runs no Tiler execution, compiles no Metal, and touches no device. It sets no threshold and qualifies no other checkpoint, prompt, host, or reference revision.

## Running it

No `make` target reaches a spike. From this directory:

```sh
UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python probe_corpus.py \
  --out results/<slug>
```

`--structural-only` runs stages 0 and 1 alone; they need neither `torch` nor the memory an F32 forward pass wants, and they answer both exceptional-value questions on their own. The checkpoint is fetched into the Hugging Face cache outside this repository under [the workload profile's storage policy](../../../docs/research/program-planning/first-metal-lm-workload.md#pinned-identity-manifest); add `HF_HUB_OFFLINE=1` to require an already-cached copy rather than a network fetch. This directory carries no `rust-toolchain.toml`, deliberately: a directory-local file would silently select another compiler for a nested workspace, and this spike has no Rust in it at all.

The probe **stops** rather than warning. Every manifest digest — four checkpoint files and three pinned `transformers` sources — is recomputed on every invocation, and a mismatch exits 4 before any measurement, because a run against other bytes is not a weaker probe, it is a probe of a different checkpoint.

## The three stages

**Stage 0 — exceptional stored values.** Two independent passes, kept apart because they are different evidence classes. The first widens all 65,536 BF16 bit patterns and classifies each result by its exponent and significand fields; the population is finite and completely covered, so it is *exhaustive finite evidence* about the conversion rather than a sample of it. The second classifies every stored element of every tensor the safetensors header declares, which is a *measurement* about one checkpoint revision. Neither uses a float comparison: a comparison against `float('inf')` cannot distinguish a subnormal from a small normal, and that distinction is the whole question.

**Stage 1 — the structural tie route.** The checkpoint declares `tie_word_embeddings: true` and carries no `lm_head.weight`, so one `[151936, 1024]` matrix is both the gather source and the vocabulary projection's weight. Two bit-identical rows of it are two bit-identical *columns* of that projection, so their logits are the same contraction over the same operand sequence at every position of every prompt. If the maximum is ever attained by a member of such a group, every other member attains it too, and the tie is a property of the checkpoint rather than of the prompt. The stage groups all 151,936 rows by exact stored bit pattern, re-checking byte equality inside every group so a digest collision cannot manufacture one.

**Stage 2 — prompts through the pinned reference.** One prefill pass per prompt in F32 on CPU with `attn_implementation="eager"` and `logits_to_keep=0`, which the pinned reference turns into `slice(0, None)` and is what makes a prefill pass offer more than one candidate position. Each position records the tie observable the oracle would report *and* how far the best-placed duplicate-group member is from the maximum — its identity, its logit, its rank, and its gap. A negative that reported only "no tie found" would say nothing about how close the search came; the rank and the gap are what let a later reader decide whether a larger search is worth running.

## The search, stated so it can be reproduced or refuted

Nineteen prompts, 330 positions. One is the control below. The other eighteen are repetitions of a duplicate-group member at lengths 8, 16, and 32, spanning the measured group-size range from the 505-member group down to five of the 2-member groups. The construction is derived rather than guessed: the model cannot distinguish two tokens whose embedding rows are bit-identical, in either direction, so a prompt that drives the model to predict the token it has just been shown lands the maximum inside a duplicate group whenever it lands on that token at all, and repetition is the cheapest such driver on a base checkpoint. Every candidate names the partner the structural argument says must tie with it, and the probe **stops** if stage 1 does not group the two together — so the candidate list cannot drift away from the measurement it rests on.

What the search does not cover, stated because it is what a later reader would extend: it varies neither prompt content beyond repetition nor decode-step positions, it evaluates no gradient-guided or beam-style search over prompts, and it does not enumerate the coincidence route, which would need a top-two gap of exactly zero in F32 between two entries whose embedding rows differ.

## Controls, and every one watched failing

**The C1 positive control.** The first prompt is the conformance row's own ten-token prefill, and ten transcribed rows from [the retained C1 conformance fixture](../qwen3-conformance-fixture/README.md) — greedy token, top-one and top-two logit bit patterns, and runner-up gap per position — must be reproduced exactly. That is a check that this probe evaluated the same reference the fixture did, and it can fail, which a probe that only re-ran a prompt and compared against itself cannot. It doubles as the negative control for the tie observable: the fixture measures no bit-identical top-two pair at any C1 position, so a probe reporting one there is reporting a defect in itself.

**The tie-detector positive control.** Every reported position says "no bit-identical top-two pair", and a detector that could only ever say that would be indistinguishable from one that works. So a synthetic logit row whose maximum is attained by exactly two indices is evaluated through the same `analyse_position` the measured rows go through, and the run stops unless it reports the tie, counts exactly two attaining indices, chooses the lowest of them as the greedy token — the oracle's declared policy — and reports a runner-up gap of zero.

**Three deliberate perturbations, each watched stopping the run**, on a scratch copy of the probe outside this directory:

| Perturbation | Result |
| --- | --- |
| one hex digit of `model.safetensors`'s manifest digest changed | `PROBE STOP: model.safetensors hashed to cd2a5120…eba, the manifest says cd2a5120…ebb`, exit 4 |
| the control's position-0 greedy token moved from 2701 to 2702 | `PROBE STOP: control position 0: greedy token 2701, the retained conformance fixture records 2702`, exit 4 |
| the greedy policy flipped from the lowest attaining index to the highest | `PROBE STOP: the tie detector chose 9 as the greedy token; the declared policy is the lowest attaining index, which is 7`, exit 4 |

The unperturbed probe exits 0 in the same matrix.

**Reproducibility.** The tie-detector control was added after a first complete run and the record regenerated with `duplicate_groups.tsv`, `exceptional.tsv`, `positions.tsv`, and `widening.tsv` byte-identical; `environment.tsv` gained exactly one key, `probe.tie_detector_positive_control`, and removed none.

## The retained record

[`results/2026-08-02-corpus-reachability-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0/`](results/2026-08-02-corpus-reachability-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0)

| File | What it holds |
| --- | --- |
| `environment.tsv` | the host row, the workload identity, every verified checkpoint and reference-source digest, and every counted population this README quotes |
| `widening.tsv` | the exhaustive BF16→F32 class map: five (stored class, widened class) pairs over 65,536 patterns |
| `exceptional.tsv` | one row per tensor holding a subnormal, infinite, or NaN stored value. **It is a header and no rows, and that is the measurement** — a counted zero over 310 tensors rather than a file nobody wrote |
| `duplicate_groups.tsv` | the 28 duplicate embedding-row groups, with size, lowest member, the group's exactly-rounded row `l2_norm`, and up to sixteen member IDs |
| `positions.tsv` | 330 rows, one per evaluated position, carrying the tie observable and the best-placed duplicate-group member's identity, logit, rank, and gap |
| `manifest.tsv` | a SHA-256 over each retained file and over `probe_corpus.py`, `pyproject.toml`, and `uv.lock`, so the record names the exact producer that wrote it |

**Measurement boundary.** Apple M4 Max, macOS 27.0 build 26A5388g, arm64, 36 GiB; Python 3.11.12, `torch` 2.6.0, `transformers` 4.51.0, `numpy` 2.2.5, `torch.set_num_threads(1)`. One checkpoint revision, one reference revision, 19 prompts, 330 positions, batch 1, F32, CPU. The tie result is a negative over exactly that search and is not a proof that no prompt reaches a tie; the widening result is exhaustive over BF16 and says nothing about any other stored dtype; the stored-value counts qualify this checkpoint revision alone.

## Provenance

Stages 1 and 2 are rebuilt from work abandoned mid-ticket under `define-the-model-level-conformance-corpus`, preserved at tag `abandoned/define-the-model-level-conformance-corpus` (commit `c0260fd`), which had produced the structural argument and the candidate construction and had recorded no result. Nothing from it was carried as a conclusion: stage 1 was re-run here, and its own candidate list is now checked against that run rather than trusted.
