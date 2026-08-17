---
id: define-the-model-weight-binding-manifest
title: Define the model weight binding manifest and its digest gate
status: in-progress
priority: p1
dependencies: [assemble-the-decoder-layer-program, assemble-the-embedding-and-vocabulary-projection-programs, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, define-first-metal-lm-workload, retain-the-qwen-conformance-reference-logit-fixture, ingest-the-checkpoint-as-f32-program-inputs]
scopes: [contracts/integrations, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [design, ingestion, weights, identity, validation, language-model, class-conformance-fixture]
claimed_from: todo
assignee: worker-weight-manifest
lease_expires_at: 1786999769
---
## User-visible outcome

The map from a checkpoint's 310 tensors to a program's interface keys is a checked record rather than a convention, so binding the wrong weight to the right-shaped operand is a stopping failure at load instead of a plausible logit vector with a wrong argmax.

## Why this exists

**Inference, from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).** Every check Tiler performs on a bound weight is shape, dtype, rank, symbol, operand-count, or adapter-capability (plus dense storage-length on the dispatch path); none of those checks content identity. The pinned checkpoint's 310 tensors fall into eight shape classes whose members are mutually interchangeable under all of them:

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

So `57! · 56!³ · 28!³` bindings pass every check in the stack and one is right (one size-1 class contributes the trivial factor `1!`). This is not "unchecked": the checks that exist all pass.

## Why it belongs to the consumer, stated so it can be refuted

Folding a per-tensor content digest into artifact identity would compile one artifact per checkpoint, which is worse than the per-decode-step compilation [rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) already refuses. Carrying it as a runtime-validated fact under ADR 0021 means hashing 2,384,199,680 bytes either per execution — absurd — or once when the consumer builds its bound weight set, and hashing once at that load *is* this manifest, reached by a longer route. (ADR 0021's decision scope is value-domain facts that enable correctness-sensitive rewrites; the analogy is load-once validation, not that the ADR defines weight-binding identity.)

## Required content

- One record that is a **total, injective map from each of the checkpoint's 310 tensor names** (or equivalently from fully qualified layer+role slots) to the program interface key that receives that tensor, together with the expected shape and expected stored scalar for that binding. Layer programs reuse the same interface keys across all 28 layers, so bare interface keys alone are not an injective domain of size 310; totality and injectivity are over the checkpoint inventory (or the qualified slots), each entry carrying the interface key. Aligns with the fixture README's phrasing: "a total, injective map from checkpoint tensor name to interface key".
- **Total and injective** over that checkpoint inventory, checked against the safetensors header rather than against a remembered list. [L1](../docs/research/program-planning/first-metal-lm-workload.md) records that two HTTP range requests read the complete 35,248-byte header without acquiring the 1.19 GB payload, so the check costs about 34 KiB.
- Bound to the pinned file digest `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba` at revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, verified locally with `shasum -a 256`.
- **It stops rather than warns** on any mismatch, exactly as L1's conformance fixture producer already does.
- The record states in its own words which mistakes Tiler catches — rank, stored scalar, literal extent, symbol consistency, operand count, each by its `BindError` variant — and which permutation it cannot, so a reader is not left inferring the boundary.

## Closes when

The manifest exists for the pinned checkpoint, its totality and injectivity are checked against the header under the pinned digest, a deliberately permuted map is watched failing, and a deliberately altered digest is watched stopping the run.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Three live-prose fixes from the ticket audit; status and graph edges unchanged (outcome still undelivered).

1. **False factorial product.** The shape-class table above has three size-56 classes (`[1024,1024]`, `[128]`, `[3072,1024]`) and three size-28 classes (`[2048,1024]`, `[1024,2048]`, `[1024,3072]`), so the shape-preserving permutation count is `57! · 56!³ · 28!³`, not the inherited `57! · 56!² · 28!⁴` (wrong by a factor of `28!/56!`). The qualitative claim — many wrong bindings pass every Tiler check — stands. The same false product still appears in L6 (`docs/research/program-planning/complete-model-ingestion-and-execution.md`), the L6 work-record ticket, L8 (`docs/research/program-planning/model-level-qualification.md`), and the L8 work-record ticket; those sites are residual product debt outside this ticket-only repair.

2. **"model-state creation".** After the 2026-08-04 supersession under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md), L6 states that Tiler retains nothing between invocations and holds no model state. The hash-once timing is consumer load (when the bound weight set is built), not model-state creation.

3. **Map domain.** Required content had "pairing each program interface key with a checkpoint tensor name" while demanding totality and injectivity over the 310-tensor inventory. Interface keys are reused per layer invocation; the injective domain is checkpoint tensor names (or fully qualified layer+role slots), each carrying the interface key — matching the fixture README anchor `total, injective map from checkpoint tensor name to interface key`.

4. **Bind-check summary (non-blocking precision).** The opening sentence had been limited to "shape, dtype, rank, or symbol"; `bind_region` also raises `OperandCountMismatch` and `UnsupportedCapability`, and the dispatch path can raise `StorageLengthMismatch`. Content/identity of weight bytes is still never checked — that remains the load-bearing gap this ticket owns.
