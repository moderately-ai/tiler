---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.first-metal-lm-workload"
kind: "research"
title: "First Metal language-model workload profile"
topics: ["program-planning", "language-model", "workload", "metal", "inference", "qwen"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.correctness-and-testing"]
depends_on: ["tiler.research.apple-targets.numerical-behaviour"]
ticket: "define-first-metal-lm-workload"
---

# First Metal language-model workload profile

**Status:** durable workload profile for rung L1 of the language-model inference ladder. The model selection is an accepted ticket outcome; the profile below is the bounded record that L2 through L8 derive from. Nothing here authorizes implementation, and no rung of the ladder is built.

## Traceability

- **Work record:** [`define-first-metal-lm-workload`](../../../tickets/define-first-metal-lm-workload.md), which holds the elimination run, the rejected candidates, and the 2026-07-30 selection.
- **Ladder position:** rung L1 of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). L2 ([`derive-transformer-operation-and-shape-surface`](../../../tickets/derive-transformer-operation-and-shape-surface.md)) consumes the operation and shape surface below; L5 ([`design-autoregressive-state-and-kv-cache`](../../../tickets/design-autoregressive-state-and-kv-cache.md)) consumes the state arithmetic; L8 ([`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md)) consumes the two bounded rows and owns the measurement harness and the comparison budget.
- **Target authority:** [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md), specifically its unified MSL 4 Apple9/F32 replay section.
- **Contract destinations:** the model-level oracle form below is written against [Correctness and testing](../../correctness-and-testing.md) and [Numerical semantics](../../numerical-semantics.md); neither has consumed it yet, which is why `disposition` reads `pending` rather than `adopted`.

Claims are labelled **Fact** when traced to inspected primary source at a recorded immutable revision, **Inference** when derived from stated facts, **Measurement** when tied to an exact environment and procedure, and **Proposal** when not yet accepted or tested.

## What is fixed, and what this document adds

**Fact — the selection.** The first complete language-model workload is `Qwen/Qwen3-0.6B-Base` at immutable revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, widened to F32, batch 1, one Apple GPU, with bounded prompt, context, and decode lengths. Tom delegated the choice and it was recorded on 2026-07-30 in the work record above; the elimination that discarded encoder-only, single-block, 7B-class, mixture-of-experts, state-space, encoder-decoder, and hybrid-recurrent candidates lives there and is not restated.

This document adds what the selection did not by itself supply: the pinned identity manifest and its acquisition policy, the configuration and tensor facts read from the pinned revision rather than from remembered architecture defaults, the F32 memory arithmetic, two bounded workload rows, the correctness oracle's exact observables, the qualified target row, and the exclusions.

**Fact — GPT-2 stays a diagnostic fixture family, not a workload.** The work record keeps GPT-2-shaped blocks useful because their smaller operation surface localizes failures, and explicitly denies them authority to weaken the Qwen-derived delivery graph. **Fact — Qwen3.5 hybrid recurrence stays downstream.** [`exercise-qwen35-hybrid-text-tower-after-the-dense-vertical`](../../../tickets/exercise-qwen35-hybrid-text-tower-after-the-dense-vertical.md) owns the architecture stress that follows dense model qualification; nothing in this profile admits Gated DeltaNet, recurrent or convolution state, partial RoPE, multi-token prediction, or vision conditioning.

## Pinned identity manifest

**Fact — the manifest, and exactly how each digest was obtained.** Every row names the file at repository `Qwen/Qwen3-0.6B-Base`, revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`. The `SHA-256` column is the digest of the file's content bytes. Rows marked *local* were fetched from the revision-pinned `resolve` endpoint on 2026-07-31 and hashed with `shasum -a 256`, and each fetched size was cross-checked against the size the repository API reported for the same blob; the two agreed for every row.

**Fact — the checkpoint row is no longer an unverified identity.** It was originally recorded as the Git-LFS object id the repository API returns: an LFS object id is by construction the SHA-256 of the object's content, but it had not been computed from the bytes, because the ticket that wrote this profile deliberately did not acquire the checkpoint. [`retain-the-qwen-conformance-reference-logit-fixture`](../../../tickets/retain-the-qwen-conformance-reference-logit-fixture.md) acquired it on 2026-07-31 and hashed all nine files locally with `shasum -a 256`; every digest and every byte size matched the table below, so the checkpoint row is now a locally reproduced digest on the same footing as the others. The fixture's producer re-runs that verification on every invocation and stops rather than warns on a mismatch.

| File | Bytes | SHA-256 | Digest source |
| --- | --- | --- | --- |
| `config.json` | 727 | `504a6b58c4271583724e66584b6b7698aea18450209df6b2f7582df0e89cee59` | local |
| `generation_config.json` | 138 | `8c970692323e3ea0e9b8b0a4dca79388d31226e41f83c9fd6014804280ebf6e8` | local |
| `model.safetensors` | 1,192,135,096 | `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba` | local |
| `tokenizer.json` | 7,031,645 | `c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539` | local |
| `tokenizer_config.json` | 9,678 | `3c04ed3ca964ea2f6b2b5faf0dc4d31aec1cb1e8b4bcf63f402d295046b422b5` | local |
| `vocab.json` | 2,776,833 | `ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910` | local |
| `merges.txt` | 1,671,853 | `8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5` | local |
| `LICENSE` | 11,343 | `832dd9e00a68dd83b3c3fb9f5588dad7dcf337a0db50f7d9483f310cd292e92e` | local |
| `README.md` | 2,973 | `910d9be25c648ab1cb5a7b1d20d67ca6d43d43559a705010198886f9af68e8f1` | local |

**Fact — there is one weight shard and no index file.** The revision's complete file list contains `model.safetensors` and no `model.safetensors.index.json`, so no shard index participates in identity. The exact check, reproducible in one line, is that `curl -sL "https://huggingface.co/api/models/Qwen/Qwen3-0.6B-Base/revision/da87bfb608c14b7cf20ba1ce41287e8de496c0cd?blobs=true"` lists ten files — the nine above plus `.gitattributes` — and no file whose name ends in `index.json`.

**Fact — the license.** The checkpoint is Apache-2.0. Two independent statements agree: the repository's card metadata reports `license: apache-2.0`, and the `LICENSE` blob digested above is the 202-line Apache License 2.0 text, opening `Apache License / Version 2.0, January 2004`. Apache-2.0 permits the redistribution this profile does not perform and the derivative use a conformance fixture would perform, and nothing in this profile depends on a narrower grant.

**Fact — the reference implementations, pinned.** The correctness oracle needs an implementation that defines what the weights mean; the checkpoint's own `config.json` declares `"transformers_version": "4.51.0"` and `"architectures": ["Qwen3ForCausalLM"]`, which makes that implementation the definitional reference rather than one candidate among several. The rows below were fetched on 2026-07-31 from the pinned commit and hashed locally, with each size cross-checked against the API-reported blob size.

| Reference | Revision | Path | Bytes | SHA-256 |
| --- | --- | --- | --- | --- |
| `huggingface/transformers` v4.51.0 (commit `0720e206c6ba28887e4d60ef60a6a089f6c1cc76`) | annotated tag object `8910bd3a3880e48a5c1ba4e5d0c742e67dbcbe76` | `src/transformers/models/qwen3/modeling_qwen3.py` | 51,968 | `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2` |
| same | same | `src/transformers/models/qwen3/configuration_qwen3.py` | 11,202 | `87f0d17326c44f2dfe1bfc329faf9201ab4b19a89ad555da085b4cc81461b201` |
| same | same | `src/transformers/modeling_rope_utils.py` | 32,952 | `c28b3e88edca8fdb5497e5c36091bf753db49bd94ace33a84e9f9c61cbf66032` |
| `huggingface/candle` 0.11.0 (commit `31f35b147389700ed2a178ee66a91c3cc25cc80d`) | — | `candle-transformers/src/models/qwen3.rs` | 16,556 | `5970178aae1b8b44d3a73b176d23a620f46331c18ac80dce6306e647c9f11671` |

**Fact — the published `transformers` 4.51.0 distribution carries those exact bytes.** Installing `transformers==4.51.0` from PyPI and hashing the three installed files reproduces all three digests above, and `modeling_rope_utils.py` measures 32,952 bytes — the size this table could not state while the file had only been fetched by digest. So the pinned git revision and the installed package are the same source for these files, and a run under the pinned environment can *check* that it evaluated the definitional reference instead of inferring it from a version string. The conformance fixture below performs that check on every invocation.

**Inference — the two references play different roles and must not be merged.** The `transformers` revision is the semantic definition: it is the implementation whose serialization format the checkpoint carries and whose module structure names every tensor in the inventory below. The Candle revision is the *intended consumer* — [Candle integration](../../integration/candle.md) records that no manifest in this workspace declares Candle and that `31f35b14` is the upstream revision the corpus cites rather than one Cargo resolves — so it is a useful independent cross-check that the traced surface is not an artefact of one framework, and it is not an authority over meaning. Neither is a Tiler normative contract; [Correctness and testing](../../correctness-and-testing.md) already fixes that a consumer runtime is an oracle only where its documented behaviour matches the selected contract.

**Proposal — repository storage policy for the checkpoint.** No checkpoint, tokenizer, or reference-source bytes are stored in this repository at any path. The manifest above *is* the retained identity; bytes are reconstructed on demand into a cache outside the repository, and any spike that needs them declares its own local directory in its own README and covers exactly that directory with a narrow gitignore entry. Nothing enforces this — the repository has no gate, and a diff adding a 1.19 GB blob is caught by review or not at all — which is the ordinary condition of every rule in this corpus and is stated rather than assumed. This profile adds no gitignore entry because it creates no spike directory; the fixture ticket named below owns adding one when it creates the directory it applies to. **Fact — that entry now exists and is one line.** `spikes/program-planning/qwen3-conformance-fixture/.gitignore` ignores `/local-work/` and nothing else, covering the regenerable F32 logit bytes and the spike's uv-managed environment; the checkpoint itself is never under this repository, so no rule mentions it.

**Fact — the reproducible acquisition route.** Either form below fetches the exact revision, and the third command is the verification that makes the acquired bytes usable as evidence rather than merely present. It was executed on 2026-07-31 by the `hf download` form, and all nine manifest files matched.

```sh
# Either: the Hugging Face client, which places files in the local HF cache outside this
# repository. `hf` is the current spelling (huggingface_hub 0.36.2 on the host that wrote
# this); an older hub installs the same command as `huggingface-cli`.
hf download Qwen/Qwen3-0.6B-Base --revision da87bfb608c14b7cf20ba1ce41287e8de496c0cd

# Or: direct, per file, with no client and no repository-local state.
curl -L -o model.safetensors \
  https://huggingface.co/Qwen/Qwen3-0.6B-Base/resolve/da87bfb608c14b7cf20ba1ce41287e8de496c0cd/model.safetensors

# Then, mandatory before the bytes carry any claim:
shasum -a 256 model.safetensors
# must print cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba
```

## Configuration facts read from the pinned revision

**Fact — the complete `config.json` at the pinned revision**, whose digest is in the manifest above. These are read values, not recalled defaults.

| Key | Value | Key | Value |
| --- | --- | --- | --- |
| `architectures` | `["Qwen3ForCausalLM"]` | `model_type` | `qwen3` |
| `num_hidden_layers` | 28 | `hidden_size` | 1024 |
| `intermediate_size` | 3072 | `hidden_act` | `silu` |
| `num_attention_heads` | 16 | `num_key_value_heads` | 8 |
| `head_dim` | 128 | `attention_bias` | `false` |
| `rms_norm_eps` | 1e-06 | `attention_dropout` | 0.0 |
| `rope_theta` | 1000000 | `rope_scaling` | `null` |
| `max_position_embeddings` | 32768 | `max_window_layers` | 28 |
| `sliding_window` | `null` | `use_sliding_window` | `false` |
| `vocab_size` | 151936 | `tie_word_embeddings` | `true` |
| `bos_token_id` | 151643 | `eos_token_id` | 151643 |
| `torch_dtype` | `bfloat16` | `use_cache` | `true` |

**Fact — `generation_config.json` declares greedy decoding.** It sets `do_sample: false`, `max_new_tokens: 2048`, and repeats `bos_token_id` and `eos_token_id` as 151643. **Inference.** Greedy is the checkpoint's own declared generation mode, so this profile's greedy-only scope imports no sampling policy from outside the checkpoint and needs no temperature, top-k, or top-p semantics.

**Fact — head dimension is independent of `hidden_size / num_attention_heads`.** 1024 / 16 = 64, and `head_dim` is 128. A planner that derives head width by division produces a silently wrong shape on this checkpoint, and the reference implementation reads `getattr(config, "head_dim", config.hidden_size // config.num_attention_heads)` precisely so that the declared value wins. This is one of the two reasons the work record preferred this checkpoint over a generic small Llama.

**Fact — two declared length limits disagree and only one is the position budget.** `config.json` declares `max_position_embeddings: 32768` while `tokenizer_config.json` declares `model_max_length: 131072`. The tokenizer's field bounds the tokenizer's own input handling and is not a claim about the model's rotary position range; every bound in this profile is taken from 32768. **Inference.** A workload row derived from the tokenizer's number would exceed the checkpoint's declared position range with no error from either file, so the discrepancy is recorded rather than silently resolved.

**Fact — the RoPE frequency construction, from the pinned reference.** `rope_scaling` is `null`, so the reference selects `_compute_default_rope_parameters`, which returns `attention_factor = 1.0` and `inv_freq = 1 / theta ** (arange(0, 128, 2) / 128)` with `theta = 1000000`: 64 frequencies over the full 128-wide head, computed in float32 and, in `Qwen3RotaryEmbedding.forward`, evaluated inside an explicitly disabled autocast so `cos` and `sin` are float32 regardless of model dtype. There is no partial rotary factor and no scaling multiplier on the table.

## Tensor inventory

**Fact — read from the checkpoint's own safetensors header without acquiring the checkpoint.** A safetensors file begins with an 8-byte little-endian header length followed by that many bytes of JSON describing every tensor's dtype, shape, and byte range. Two HTTP range requests against the pinned `resolve` URL — `Range: bytes=0-7` then `Range: bytes=8-35255` — returned a header length of 35,248 and the complete header, at a cost of about 34 KiB rather than 1.19 GB. The header declares 310 tensors, every one `BF16`, `__metadata__` of `{"format": "pt"}`, and a maximum data-offset end of 1,192,099,840. **Inference — the header describes the whole file.** 8 + 35,248 + 1,192,099,840 = 1,192,135,096, which equals the manifest's file size exactly, so no bytes are unaccounted for and the inventory is complete rather than a prefix of one.

| Tensor role | Count | Shape each | Elements each |
| --- | --- | --- | --- |
| `model.embed_tokens.weight` | 1 | `[151936, 1024]` | 155,582,464 |
| `model.layers.{L}.input_layernorm.weight` | 28 | `[1024]` | 1,024 |
| `model.layers.{L}.self_attn.q_proj.weight` | 28 | `[2048, 1024]` | 2,097,152 |
| `model.layers.{L}.self_attn.k_proj.weight` | 28 | `[1024, 1024]` | 1,048,576 |
| `model.layers.{L}.self_attn.v_proj.weight` | 28 | `[1024, 1024]` | 1,048,576 |
| `model.layers.{L}.self_attn.q_norm.weight` | 28 | `[128]` | 128 |
| `model.layers.{L}.self_attn.k_norm.weight` | 28 | `[128]` | 128 |
| `model.layers.{L}.self_attn.o_proj.weight` | 28 | `[1024, 2048]` | 2,097,152 |
| `model.layers.{L}.post_attention_layernorm.weight` | 28 | `[1024]` | 1,024 |
| `model.layers.{L}.mlp.gate_proj.weight` | 28 | `[3072, 1024]` | 3,145,728 |
| `model.layers.{L}.mlp.up_proj.weight` | 28 | `[3072, 1024]` | 3,145,728 |
| `model.layers.{L}.mlp.down_proj.weight` | 28 | `[1024, 3072]` | 3,145,728 |
| `model.norm.weight` | 1 | `[1024]` | 1,024 |

**Fact — the inventory contains no `lm_head.weight`**, which is the structural form of `tie_word_embeddings: true`: the output projection reads the embedding matrix, and the reference declares `_tied_weights_keys = ["lm_head.weight"]` to say so. **Inference.** One 151,936 × 1024 matrix serves both the input lookup and the output projection, so a plan that allocates two copies doubles the largest single allocation in the model for no semantic reason, and a plan that shares one must respect that the two uses have different access relations — a gather on one side, a contraction on the other.

**Fact — the GQA projection asymmetry is visible in the shapes.** `q_proj` is `[2048, 1024]` because 16 query heads × 128 = 2048, while `k_proj` and `v_proj` are `[1024, 1024]` because 8 key/value heads × 128 = 1024, and `o_proj` is `[1024, 2048]`. The query and key/value projections are therefore different shape classes within one attention block, and `num_key_value_groups` is 16 / 8 = 2.

**Fact — the parameter total and its arithmetic.** Summing the inventory gives 155,582,464 embedding elements, 15,730,944 per layer × 28 layers, and 1,024 final-norm elements, for 596,049,920 parameters — the count the work record states, here recomputed from the checkpoint's own header rather than carried forward.

## F32 memory arithmetic

The three quantities below are separate and are stated separately, because collapsing them is how a workload that fits becomes a workload that does not.

**Fact — weight budget.** 596,049,920 parameters at 4 bytes is **2,384,199,680 bytes** (2.2205 GiB) of F32 weights. The same parameters at the checkpoint's own BF16 width occupy 1,192,099,840 bytes, so widening to F32 exactly doubles the weight residency; the 1,192,135,096-byte file size is that BF16 payload plus the 35,256-byte header.

**Fact — KV-cache budget, per token and per context.** One cached token holds a key and a value for every layer and every key/value head: 2 × 28 layers × 8 heads × 128 = 57,344 F32 elements, or **229,376 bytes (224 KiB) per token**. This is a function of `num_key_value_heads`, not `num_attention_heads`; using 16 heads would overstate it by exactly 2×.

| Context (tokens) | KV-cache F32 | Weights + KV |
| --- | --- | --- |
| 18 | 4,128,768 B (0.0038 GiB) | 2.2243 GiB |
| 256 | 58,720,256 B (0.0547 GiB) | 2.2751 GiB |
| 640 | 146,800,640 B (0.1367 GiB) | 2.3572 GiB |
| 2,176 | 499,122,176 B (0.4648 GiB) | 2.6853 GiB |
| 8,320 | 1,908,408,320 B (1.7773 GiB) | 3.9978 GiB |
| 32,768 | 7,516,192,768 B (7.0000 GiB) | 9.2205 GiB |

**Inference — the declared maximum context is not a workload row.** At the checkpoint's declared 32,768 positions the F32 KV cache alone is 7.00 GiB and total resident state is 9.22 GiB before any activation or workspace. That is not infeasible on a large unified-memory Apple part, but it is a residency claim no measurement in this repository supports, and it would make every early performance number a statement about memory pressure rather than about the compiler. The benchmark row below therefore stops well inside it and says so, rather than quoting 32,768 as a supported length.

**Unknown — peak workspace.** Peak transient memory is a property of the selected physical plan, not of the model, so this profile cannot state it and does not. One bound is worth recording because it constrains planning rather than merely costing it: a prefill that materializes the full attention score matrix needs 16 heads × P × P × 4 bytes, which is 1,048,576 B at P = 128, 268,435,456 B at P = 2,048, and **4,294,967,296 bytes (4.00 GiB) at P = 8,192**. **Inference.** At the long end of the benchmark row a materialized-score plan costs more transient memory than the entire model, so the choice between materializing scores and streaming them is a hard-feasibility question at some prompt length rather than a cost comparison at every prompt length — exactly the separation the architectural contract requires. Which lengths those are, and what the real peak is for a chosen plan, belongs to L5 and L6.

**Fact — logits are small and this matters for the oracle.** One position's logits are 151,936 F32 values, or 607,744 bytes. Retaining every logit of the conformance row below is therefore about 10.4 MiB of regenerable data, which is why that row can be fully retained and the benchmark row cannot.

## The two bounded rows

One universal sequence cap would either be too small to be representative or too large to retain, so the profile closes on two rows with different jobs. Both are batch 1, F32 throughout, one Apple GPU, greedy, and inside `max_position_embeddings = 32768`.

### C1 — conformance row

**Proposal — the row.** A fixed 10-token prompt and a fixed 8-step decode budget, reaching 18 positions.

| Property | Value |
| --- | --- |
| Prompt text | `The quick brown fox jumps over the lazy dog.` |
| Prompt token IDs | `[785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]` |
| Prompt tokens | `The`, `Ġquick`, `Ġbrown`, `Ġfox`, `Ġjumps`, `Ġover`, `Ġthe`, `Ġlazy`, `Ġdog`, `.` |
| Prompt length | 10 |
| Decode budget | 8 steps |
| Maximum context reached | 18 |
| Termination | EOS token 151643, or the fixed 8-step budget, whichever comes first |
| Retained observables | every prefill and every decode-step logit vector — 18 × 151,936 F32 values |
| Retention size | 10,939,392 bytes (10.43 MiB) raw F32 |

**Fact — the token IDs were produced, not recalled.** They come from encoding the prompt text with the pinned `tokenizer.json` whose digest is in the manifest, via `tokenizers` 0.22.1 on 2026-07-31; decoding the IDs returns the prompt text unchanged. The prompt is pure ASCII, contains no special or added token, and needs no chat template — which is deliberate, because a base checkpoint has no chat semantics and the work record excludes them.

**Inference — why the row is fixed as token IDs rather than as text.** The Tiler workload boundary begins at token IDs. Tokenization is string processing, not a tensor program, and nothing in Tiler's semantic graph can express it; recording the IDs makes the conformance row reproducible even for a consumer holding a different tokenizer implementation, and makes any future tokenizer drift a visible mismatch against the recorded IDs rather than a silent change of input.

**Inference — why 10 and 8.** The row exists so that *every* logit is retainable and reproducible, which caps it: at 18 positions the complete logit set is about 10.4 MiB, whereas a 512-token prompt would be 296.8 MiB and would force the row to retain a summary instead. Ten prompt tokens still exercise multi-position prefill, causal masking over a non-trivial window, and the prefill-to-decode transition, and eight decode steps exercise repeated single-token decode against a growing cache. A one-token prompt would exercise none of the first group; a one-step decode would not distinguish a broken cache update from a correct one.

**Proposal — what is retained where.** The complete F32 logit bytes are regenerable local data and are not checked in. What is checked in with the fixture is small and sufficient to detect drift: a SHA-256 over each position's exact F32 logit bit pattern, the full-precision top-32 entries per position with their indices, the greedy token and its runner-up gap per position, and the 18 emitted token IDs. **Inference — the digest is not the tolerance evidence.** A digest proves the reference regenerates exactly; it cannot support a bounded-error comparison, which needs values. Both are retained for that reason.

**Measurement — the fixture exists and is retained.** [Qwen3-0.6B-Base C1 conformance and attribution reference fixture](../../../spikes/program-planning/qwen3-conformance-fixture/README.md) produced it from the locally verified checkpoint and the locally verified `transformers` v4.51.0 sources, in F32 on CPU with `attn_implementation="eager"` and `logits_to_keep=0`, and the retained record is [`results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0/`](../../../spikes/program-planning/qwen3-conformance-fixture/results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0). It was first produced on 2026-07-31 carrying the logits alone; the 2026-08-01 record extends it with [L6](complete-model-ingestion-and-execution.md)'s attribution surface and supersedes it, and `sequence.tsv`, `positions.tsv`, `top32.tsv`, and `envelope.tsv` regenerated byte-identically across the extension, so every measurement below is the same measurement. Three of its outcomes belong in this row rather than only in the spike:

- **The 18-token sequence.** The 10 prompt tokens followed by `576, 3974, 13876, 38835, 34208, 916, 279, 15678` — the base model restarts the pangram. Termination was the fixed 8-step budget; EOS 151643 never appeared, so the row exercises the budget arm of the termination rule and not the EOS arm.
- **The tie branch is unexercised.** At all 18 positions exactly one index attains the maximum and the top two logits are never bit-identical, so the tie policy is declared and implemented but not demonstrated by this row. That is a fact about this prompt, not evidence that the branch is unreachable.
- **`logits_to_keep=0` means every position.** The pinned reference turns it into `slice(0, None)`, so the value that reads like "keep none" is the special case that keeps all — the reason this row can retain prefill logits at all.

**Measurement — the 18 positions and the 8 decode steps are consistent, and here is the arithmetic.** Prefill covers positions 0–9 in one pass; eight decode passes cover positions 10–17. The eighth pass consumes the eighth generated token, which is what makes the retained set 18 vectors and the maximum context 18 while the budget stays at 8 steps. The argmax at position 17 is retained per position but is not appended, because appending it would spend a ninth step.

### B1 — representative benchmark row

**Proposal — the matrix.** Four prompt lengths against one fixed decode budget, so prompt length is the only varied axis and per-token decode latency keeps a constant denominator across rows.

| Row | Prompt tokens | Decode steps | Context reached | KV-cache F32 at end | Weights + KV |
| --- | --- | --- | --- | --- | --- |
| B1-a | 128 | 128 | 256 | 58,720,256 B | 2.2751 GiB |
| B1-b | 512 | 128 | 640 | 146,800,640 B | 2.3572 GiB |
| B1-c | 2,048 | 128 | 2,176 | 499,122,176 B | 2.6853 GiB |
| B1-d | 8,192 | 128 | 8,320 | 1,908,408,320 B | 3.9978 GiB |

**Inference — why the matrix stops at 8,320 of 32,768 declared positions.** Three separate quantities cross a threshold before the declared maximum: total resident state reaches 9.22 GiB at 32,768, a materialized-score prefill reaches 4.00 GiB at P = 8,192, and neither figure is bounded by any measurement this repository holds. B1-d already sits at the point where a plan choice becomes a feasibility question, which makes it the most informative long row available without turning the first benchmark into a memory-pressure experiment. Extending the matrix upward is legitimate work; it needs a residency measurement on a named host first, and it belongs to L8.

**Proposal — the observables, and what they are not.** Prefill latency and per-token decode latency, each as min-of-N on a quiet host; peak resident bytes; dispatch count and materialization count. Not tokens per second aggregated across a batch, because batch is 1 and a throughput figure would need batching that this profile deliberately excludes. Every one of these is a `Measurement` bound to an exact host, toolchain, and procedure when it is taken, and none of them is a number this document supplies — the harness, the host discipline, and the regression policy are L8's, and this row exists so that L8 measures a workload someone already bounded.

## Effective numerical policy and the correctness oracle

**Fact — the effective F32 policy is derived from the qualified target, not chosen for this workload.** The qualified row's governed baseline is `-fmetal-math-mode=safe`, `-fmetal-math-fp32-functions=precise`, and `-ffp-contract=off`, fixed at expansion time and carried in artifact identity, as [Candle integration](../../integration/candle.md) records while contrasting it with Candle's own fast-math default. **Measurement — subnormals flush.** On the qualified Apple9/macOS/F32 row, both compilation paths flush F32 input and result subnormals to sign-preserving zero, which is why the only realization the retained runtime proof executed under is `FlushSubnormalsToZeroF32` and why the governed Metal profile rejects strict subnormal-preserving F32 rather than delivering it. The effective policy for this workload is therefore subnormal-flushing, contraction-free, safe-math F32 — a derived consequence of the target, and one that [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) forbids any authority from silently narrowing or substituting to make a plan feasible.

**Inference — widening to F32 moves the executed program toward the reference, not away from it.** The pinned reference already computes several stages in float32 regardless of model dtype: `Qwen3RMSNorm.forward` upcasts to float32 for the mean-of-squares and `rsqrt`, `eager_attention_forward` computes its softmax with `dtype=torch.float32`, and `Qwen3RotaryEmbedding.forward` builds `cos` and `sin` under a disabled autocast. In an all-F32 realization those upcasts become identities, so the F32 workload is closer to the reference's own internal precision than a BF16 one would be, and the surviving divergence sources are reduction order, subnormal handling, and elementary-function results rather than storage width.

**Inference — the reference does not flush subnormals and the target does.** A CPU float32 reference preserves subnormal intermediates that the qualified Metal row flushes to zero. That is a real, named divergence source rather than a nuisance, and it is the reason the oracle's comparison must be stated against the *effective* realization instead of against an idealized IEEE float32 — a bound derived from the wrong realization would be either unachievable or vacuous.

**Proposal — the oracle, as five observables rather than one.** The work record's correctness consequence is that token-sequence equality alone is insufficient because materially wrong logits can retain the same argmax. The oracle therefore compares, after prefill and after every decode step:

1. **Logit agreement** against the pinned reference evaluated in F32 for the same token IDs, under a bounded-error conformance level whose bound is derived rather than assumed — see the next paragraph.
2. **Greedy-token equality** against the reference's greedy token at every position.
3. **Tie handling**, declared in advance: the greedy token is the lowest vocabulary index among all indices attaining the maximum logit, and any position where the top-two logits are bit-identical is recorded as a tie rather than resolved silently, because at such a position token equality carries no information about logit agreement.
4. **Termination**, which must be EOS token 151643 or the row's fixed budget, and never an implicit stop; a run that terminates for any other reason is a failure regardless of its logits.
5. **Plan determinism** on the Tiler side alone: N repeated executions of the same artifact with the same inputs on the same device produce bit-identical logits. This needs no reference, it is the `plan deterministic` conformance level [Numerical semantics](../../numerical-semantics.md#conformance-levels) already defines, and it separates "disagrees with the reference" from "disagrees with itself" — two failures with entirely different causes.

**Unknown — the numeric bound, and why stating one here would be the defect.** [Correctness and testing](../../correctness-and-testing.md) requires every oracle to name a conformance level explicitly, and the level here is `bounded error`, whose bound must come from the effective realization and the reference comparison contract. It cannot be composed from per-operation tolerances: [Region accuracy contracts and analyzable error budgets](../numerics/region-accuracy-contract.md) establishes that an error bound is a relation between two complete computations and is not generally the sum of per-operation tolerances, because cancellation, correlated reuse, deleted materialization rounding points, and exceptional-value discontinuities all break the sum. A model-level constant written down now would be exactly the ad hoc threshold the L8 outcome forbids — a number chosen before any evidence, which a later measurement would either quietly relax or quietly fail against.

**Proposal — how the bound gets derived, so the gap is a procedure and not a hole.** Before comparing any Tiler result, measure the reference's own sensitivity on this exact checkpoint and this exact prompt: evaluate the reference twice under two independently legal F32 orderings — the ordinary float32 path and a float64 path rounded to F32 at the observable — and record the resulting per-position logit deviation. That envelope is a property of the computation rather than of Tiler, it is measurable today without any Tiler execution, and it is the smallest deviation any correct F32 realization could be required to fall inside. The admissible bound is then that measured envelope combined with the declared realization's own subnormal and elementary-function behaviour. This is a bounded experiment with stated inputs, outputs, and a stop condition, and its owner is named below.

**Fact — a float64 pass through the pinned reference is not uniformly float64, and the procedure above has to say which one it means.** The three float32 sites this profile records above — `Qwen3RMSNorm.forward` at `modeling_qwen3.py:73`, the softmax in `eager_attention_forward` at line 162, and the `.float()` calls building the RoPE table at lines 336–344 — are unconditional. They are upcasts for a BF16 or F32 model, which is how this profile described them; at model dtype float64 the same lines are **downcasts**, and they sit at the mean-of-squares normalization, the attention softmax, and the rotary table. In an unmodified float64 pass those three stages therefore round identically to the F32 pass and contribute exactly zero to the measured deviation, at three of the most cancellation-prone points in the model.

**Measurement — the envelope, measured both ways, on the host row the fixture records.** The fixture evaluates both readings rather than choosing one: `f64_unmodified` is the pinned reference verbatim at dtype float64, and `f64_promoted` promotes those three sites to float64 with line-for-line copies that change nothing else. Across all 18 positions the largest whole-vocabulary deviation from the F32 pass is **2.048e-4** unmodified and **2.007e-4** promoted; restricted to the top-32 entries it is 7.82e-5 and 7.44e-5, at most 78 ULP in both. The greedy token agrees at every position under both. Between 483 and 3,863 of the 151,936 logits per position are bit-identical between the two orderings — under 3% — so argmax agreement coexists with almost every individual logit differing.

**Inference — the dominant divergence source on this row is contraction reduction order.** Promoting the three sensitive stages moved the envelope by about 2%, so the normalization, softmax, and rotary-table rounding are not what the deviation is made of here; the contractions are. The unmodified pass was therefore not materially understating the envelope on this row — but that is a measured outcome rather than something the unmodified pass could have established about itself, which is why both variants are retained. **Inference — the margin is wide and is a property of this row.** The worst deviation is about 1,300× smaller than the smallest runner-up gap of 0.266, which is why greedy agreement survives reordering everywhere. None of this is a bound, and none of it generalizes to a row with a narrower margin; the numbers qualify one prompt and one checkpoint on one host.

## Target qualification

**Fact — the current authority row, verified against the retained records.** The qualified profile is `apple9-f32-unified-msl4-macos26`: the macOS family, F32, offline `-target air64-apple-macos26.0`, offline `-std=metal4.0`, runtime `MTLLanguageVersion4_0`, and an Apple9 device, selected indivisibly. The offline compiler and linker are Metal/AIR-LLD 32023.883 (`metalfe-32023.883`); the runtime path in the replay loaded `GPUCompiler.framework` build `metalfe-32023.921`. The exact retained records are [`results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`](../../../spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv) and [`results/2026-07-31-numerics-exhaustive-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`](../../../spikes/apple-targets/results/2026-07-31-numerics-exhaustive-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv), both schema `tiler.apple-numerical-behaviour/v7`, described in the [unified MSL 4 replay section](../apple-targets/numerical-behaviour.md) of the numerical-behaviour record; the 2026-07-30 pair they replace is retained beside them as the previous row, with every value this row depends on reproduced byte for byte. The earlier MSL 3.1 records remain evidence for their own exact inputs and are not this workload's target row.

**Fact — which claims need a live device.** Every execution and delivered-numerics claim. Compile-side claims — emitted operations, module options, artifact identity — do not.

**Fact — the runtime compiler identity is a qualifier for one comparison, not an AOT input.** The replay's runtime half deliberately used `newLibraryWithSource` to compare two compilers, and the separate native-AOT observer established that pipeline preparation from an offline metallib cannot attribute its private translator identity. `metalfe-32023.921` therefore qualifies the source-JIT comparison rows and is not evidence about the AOT route; exact native translation identity remains `Unknown`. A workload measurement taken through the AOT route inherits that unknown and must not substitute the source-JIT build for it.

## Operation and shape surface handed to L2

L2 owns the derivation into Tiler operation families, extent classes, and capability tickets. What this profile hands it is the traced surface at the pinned revision, so that derivation starts from inspected source rather than from a generic transformer.

**Fact — the per-layer computation, from `Qwen3DecoderLayer`, `Qwen3Attention`, and `Qwen3MLP` at the pinned reference.** Each of the 28 layers is a pre-norm residual block: `h = h + Attn(RMSNorm(h))` then `h = h + MLP(RMSNorm(h))`, with `model.norm` applied once after the last layer and the tied embedding matrix projecting to 151,936 logits. Inside attention: project to Q `[T, 2048]`, K `[T, 1024]`, V `[T, 1024]` with no bias; view Q and K as `[T, heads, 128]` and apply an RMSNorm over the 128-wide head dimension only — `q_norm` and `k_norm`, which is the per-head normalization a generic Llama does not have; apply rotary embedding to the normalized Q and K over the full 128 dimensions using the half-split `rotate_half` form; append K and V to the cache; expand K and V from 8 to 16 heads by group repetition; contract Q against Kᵀ scaled by `128 ** -0.5`; add the additive causal mask; softmax over the key axis; contract against V; and project `[T, 2048]` back to `[T, 1024]`. Inside the MLP: `down_proj(silu(gate_proj(x)) * up_proj(x))`, a SwiGLU with SiLU as `hidden_act` declares.

**Inference — the families this requires, and where each stands on the support matrix.** Tensor contraction is required nine times per layer — the Q, K, and V projections, the Q·Kᵀ score contraction, the scores·V contraction, the output projection, and the gate, up, and down projections — plus once per step for the vocabulary projection. Seven of the nine are weighted and two are weight-free, and across the whole model the weighted ones use six distinct weight shapes: `[2048, 1024]`, `[1024, 1024]`, `[1024, 2048]`, `[3072, 1024]`, `[1024, 3072]`, and the `[151936, 1024]` embedding. It sits at R1 with no registered key and an unsettled keyed-family question that [Milestone 6](../../roadmap.md#framing-what-a-tensor-contraction-family-would-impose) owns, and the split between weighted and weight-free contractions matters because only the second kind has an extent that grows during decode. Softmax, SiLU, `rsqrt`, and the RoPE table's `cos` and `sin` are transcendental or elementary families at R2 with no operation, evaluator, or structured-kernel construct — the roadmap's own absence check 1 returns no output at all, and this trace is what prompted correcting that check, which had silently started matching ordinary `log` identifiers while its comment still claimed silence. Reindex, broadcast, transpose, slice, concatenate, and the group repetition are structural families at R2 with no registered key. A general mean-reduction for RMSNorm and a max-and-sum reduction for softmax are reductions other than the single registered strict serial sum, so they resolve to no fusion legality at all. **Inference.** Every family this workload needs is at R1 or R2 today; nothing in the ladder is partially built, and this trace is the evidence for that rather than a contradiction of it.

**Fact — the extent classes.** Batch is the constant 1. Hidden 1024, intermediate 3072, head dimension 128, head counts 16 and 8, layer count 28, and vocabulary 151,936 are static. Exactly two extents vary: the prefill token count `T` and the cached context length `S`, both bounded per row by the tables above and both well inside 32,768. **Inference.** The workload needs bounded symbolic extents, which the sourced-extent profile already provides, and needs no unconstrained dynamic shape — which is what keeps the roadmap's deferral of unconstrained dynamic shapes intact rather than triggered.

**Fact — one dtype boundary the workload cannot avoid.** The checkpoint's stored weights are BF16 and the workload executes in F32, so a BF16-to-F32 conversion exists at ingestion. It is a widening conversion, exact for every finite BF16 value — BF16 is a truncated F32, so even a BF16 subnormal widens to an F32 normal and the target's subnormal flush cannot touch it — and it happens once at load rather than inside the executed program; whether it is a Tiler semantic operation or a host-side ingestion step is L6's question, and the answer changes whether the cast-and-convert row of the support matrix is triggered. Recorded so the question is visible rather than assumed away.

## Exclusions

Each of these is a tracked position with a trigger, not an omission.

| Excluded | Why | Reconsideration trigger |
| --- | --- | --- |
| Batching beyond 1 | Adds a batched-contraction shape class before the unbatched one is proven, and converts every latency observable into a throughput one | The unbatched contraction path is realized and a throughput claim is actually wanted |
| Sampling (temperature, top-k, top-p) | The checkpoint's own `generation_config.json` declares `do_sample: false`; sampling adds a randomness source no `OperationEffect` variant admits | A product goal requires non-greedy generation; `OperationEffect` is `Pure`-only and would have to widen first |
| Chat templates and thinking-mode semantics | This is a base checkpoint with no chat semantics; the template in `tokenizer_config.json` belongs to the instruct sibling | An instruct or thinking checkpoint becomes a named workload |
| Tokenization | String processing, not a tensor program; nothing in the semantic graph can express it | Never for the compiler; a consumer-side concern only |
| Quantization | L7's decision and dependent on milestone 2Q, and forbidden here as a cost-motivated preselection | [`scope-first-quantized-lm-profile`](../../../tickets/scope-first-quantized-lm-profile.md) activates |
| Sliding-window attention | `sliding_window` is `null` and `use_sliding_window` is `false` at the pinned revision | A workload whose config enables it |
| Training, distributed execution, speculative decoding, unconstrained dynamic shapes | Carried unchanged from the roadmap's deferral table; none has a reserved seam | The triggers in that table |
| Contexts beyond 8,320 tokens | 32,768 positions cost 7.00 GiB of F32 KV cache alone, unsupported by any measurement here | A residency measurement on a named host, under L8 |
| MoE, vision conditioning, hybrid recurrence, multi-token prediction | Each needs a capability family the ladder does not name | The named downstream hybrid ticket, after dense qualification |

## What remains open, and who owns it

- **The conformance fixture exists, and now carries an attribution surface beside it.** [`retain-the-qwen-conformance-reference-logit-fixture`](../../../tickets/retain-the-qwen-conformance-reference-logit-fixture.md) acquired the checkpoint under the policy above, verified every manifest digest locally, and retained C1's per-position logit digests, top-32 slices, greedy tokens, runner-up gaps, tie states, and the 18-token sequence in [the fixture spike](../../../spikes/program-planning/qwen3-conformance-fixture/README.md); [`retain-the-c1-model-attribution-fixture`](../../../tickets/retain-the-c1-model-attribution-fixture.md) then extended the same producer and the same evaluated pass with the reference's per-layer hidden states, its per-layer post-RoPE `K` and `V`, and the four host computations, under this row's retention policy — full bytes regenerable, digests and bounded comparison values checked in. What remains open is portability rather than existence: the digests are bound to one host, thread count, and BLAS, so a mismatch elsewhere is expected and is not by itself a defect. A second host would turn that expectation into evidence; no ticket owns doing so, because nothing yet needs it. The one column that *is* portable is the exactly-rounded `l2_norm` the attribution records carry, which depends on neither summation order nor host.
- **The comparison bound is no longer `Unknown`, and it was derived rather than chosen.** [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) still owns the reading; [`measure-the-model-level-comparison-envelope-under-the-target-realization`](../../../tickets/measure-the-model-level-comparison-envelope-under-the-target-realization.md) measured the band that rung's record specifies, before any Tiler result exists. The retained quantity is in [the fixture spike](../../../spikes/program-planning/qwen3-conformance-fixture/README.md)'s `joint.tsv` and `perturbation.tsv`: **2.2101e-4** over the whole vocabulary and **1.0872e-4**, at most **87 ULP**, restricted to the reference's own top-32 order, under P-reorder, P-flush and P-elem applied *together* rather than summed. The greedy token agrees at all 18 positions under all four joint variants, and the band is about **1,204×** below this row's smallest runner-up gap of 0.266, so the exact-greedy gate holds on C1. Both halves this entry previously called missing are now measured: P-elem is sized from the registered `Ulp(tiler::ulp-reference-gap@1, 12)` and `Faithful` contracts and widens the reordering envelope by about 8%, and the CPU reference's subnormal preservation against the Metal row's flush-to-zero is established by two positive controls and then measured to be the identity on this row, because no arithmetic site of this prompt reaches the F32 subnormal range. What remains open is reach rather than existence: the band qualifies one prompt, one checkpoint, and 18 positions, it samples the admitted per-element sign assignments at full magnitude rather than searching them, and it is not a threshold — this profile still files no competing ticket for one.
- **Peak workspace is plan-dependent.** [`design-autoregressive-state-and-kv-cache`](../../../tickets/design-autoregressive-state-and-kv-cache.md) and [`design-model-ingestion-and-complete-execution`](../../../tickets/design-model-ingestion-and-complete-execution.md) own the real figure; only the materialized-score bound above is stated here.
- **Every operation family this workload needs is at R1 or R2.** [`derive-transformer-operation-and-shape-surface`](../../../tickets/derive-transformer-operation-and-shape-surface.md) turned the traced surface into capability requirements; the delivered derivation is [Transformer operation and shape surface derivation](../shapes/transformer-operation-and-shape-surface.md), and this profile files none of them.
