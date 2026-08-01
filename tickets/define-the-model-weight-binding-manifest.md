---
id: define-the-model-weight-binding-manifest
title: Define the model weight binding manifest and its digest gate
status: todo
priority: p1
dependencies: [assemble-the-decoder-layer-program, assemble-the-embedding-and-vocabulary-projection-programs]
related: [design-model-ingestion-and-complete-execution, define-first-metal-lm-workload, retain-the-qwen-conformance-reference-logit-fixture, ingest-the-checkpoint-as-f32-program-inputs]
scopes: [contracts/integrations, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [design, ingestion, weights, identity, validation, language-model]
---
## User-visible outcome

The map from a checkpoint's 310 tensors to a program's interface keys is a checked record rather than a convention, so binding the wrong weight to the right-shaped operand is a stopping failure at load instead of a plausible logit vector with a wrong argmax.

## Why this exists

**Inference, from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).** Every check Tiler performs on a bound weight is a shape, dtype, rank, or symbol check, and the pinned checkpoint's 310 tensors fall into eight shape classes whose members are mutually interchangeable under all of them:

| Shape | Count | Roles |
| --- | --- | --- |
| `[151936, 1024]` | 1 | the tied embedding |
| `[1024]` | 57 | 28 `input_layernorm`, 28 `post_attention_layernorm`, `model.norm` |
| `[2048, 1024]` | 28 | `q_proj` |
| `[1024, 1024]` | 56 | 28 `k_proj`, 28 `v_proj` |
| `[128]` | 56 | 28 `q_norm`, 28 `k_norm` |
| `[1024, 2048]` | 28 | `o_proj` |
| `[3072, 1024]` | 56 | 28 `gate_proj`, 28 `up_proj` |
| `[1024, 3072]` | 28 | `down_proj` |

So `57! · 56!² · 28!⁴` bindings pass every check in the stack and one is right. This is not "unchecked": the checks that exist all pass.

## Why it belongs to the consumer, stated so it can be refuted

Folding a per-tensor content digest into artifact identity would compile one artifact per checkpoint, which is worse than the per-decode-step compilation [rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) already refuses. Carrying it as a runtime-validated fact under ADR 0021 means hashing 2,384,199,680 bytes either per execution — absurd — or once at model-state creation, and hashing once at creation *is* this manifest, reached by a longer route.

## Required content

- One record pairing each program interface key with a checkpoint tensor name, its expected shape, and its expected stored scalar.
- **Total and injective** over the checkpoint's own tensor inventory, checked against the safetensors header rather than against a remembered list. [L1](../docs/research/program-planning/first-metal-lm-workload.md) records that two HTTP range requests read the complete 35,248-byte header without acquiring the 1.19 GB payload, so the check costs about 34 KiB.
- Bound to the pinned file digest `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba` at revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, verified locally with `shasum -a 256`.
- **It stops rather than warns** on any mismatch, exactly as L1's conformance fixture producer already does.
- The record states in its own words which mistakes Tiler catches — rank, stored scalar, literal extent, symbol consistency, operand count, each by its `BindError` variant — and which permutation it cannot, so a reader is not left inferring the boundary.

## Closes when

The manifest exists for the pinned checkpoint, its totality and injectivity are checked against the header under the pinned digest, a deliberately permuted map is watched failing, and a deliberately altered digest is watched stopping the run.
