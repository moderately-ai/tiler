---
id: define-the-model-weight-binding-manifest
title: Define the model weight binding manifest and its digest gate
status: in-progress
priority: p1
dependencies: [assemble-the-decoder-layer-program, assemble-the-embedding-and-vocabulary-projection-programs, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, define-first-metal-lm-workload, retain-the-qwen-conformance-reference-logit-fixture, ingest-the-checkpoint-as-f32-program-inputs, correct-residual-qwen-weight-permutation-products]
scopes: [contracts/integrations, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [design, ingestion, weights, identity, validation, language-model, class-conformance-fixture]
claimed_from: todo
assignee: worker-weight-manifest
lease_expires_at: 1786999769
---
## User-visible outcome

The map from the pinned checkpoint's 310 tensors to globally qualified program interface slots is a checked record rather than a convention. A consumer that applies this record while constructing its bound weight set can stop a wrong name-to-slot binding before execution instead of producing a plausible logit vector with a wrong argmax. This ticket retains and verifies that record; it does not add the production consumer load path.

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

Folding a per-tensor content digest into artifact identity would compile one artifact per checkpoint, which is worse than the per-decode-step compilation [rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) already refuses. Carrying the checkpoint digest as a runtime-validated fact under ADR 0021 means hashing 2,384,199,680 bytes either per execution — absurd — or once when the consumer builds its bound weight set. That digest check and this manifest's name-to-qualified-slot check are distinct, complementary load-once facts: the digest binds the complete checkpoint bytes, while the manifest binds each named tensor to its intended program input. Neither substitutes for the other. (ADR 0021's decision scope is value-domain facts that enable correctness-sensitive rewrites; the analogy is load-once validation, not that the ADR defines weight-binding identity.)

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

## Fact audit — 2026-08-17

Read at exact base `07aca5cd8f67824019d8c183fd3a9584ce84b670` before any implementation edit.

1. **Verified — current Tiler binding checks.** Complete reads of `crates/tiler/src/expansion.rs`, `crates/tiler/src/value.rs`, and `crates/tiler/src/route.rs` verify the opening anchor `Every check Tiler performs on a bound weight`: `bind_region` checks capability, operand count, rank, stored scalar, literal extents, and symbol consistency, while the delivering route additionally checks dense storage length. None reads a checkpoint tensor name or establishes payload identity.
2. **Verified — exact checkpoint population.** The pinned file is locally present under the exact `da87bfb608c14b7cf20ba1ce41287e8de496c0cd` snapshot. Its 35,248-byte safetensors header names 310 tensors, all `BF16`, with the eight table counts exactly as stated; `57! · 56!³ · 28!³` follows from that census.
3. **Imprecise, repaired above — `hashing once at that load *is* this manifest`.** A complete-file digest binds checkpoint bytes, but it does not state or validate the name-to-slot map. The repaired paragraph now keeps checkpoint integrity and binding identity as separate load-once checks. This changes no outcome or graph edge.
4. **Verified with historical scope — `two HTTP range requests read the complete 35,248-byte header without acquiring`.** The complete L1 record and retained fixture show that was the original evidence method. This ticket can now revalidate the locally present exact bytes directly and needs no network access.
5. **Verified — pinned source anchors.** Local `shasum -a 256` over the 1,192,135,096-byte file returns `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`; its snapshot directory is the pinned revision. The local `config.json` independently reports 28 layers and the model dimensions used by the program fixtures.
6. **Verified — producer stops rather than warns.** The complete `produce_fixture.py` read verifies that `CHECKPOINT_MANIFEST` pins revision and digest and that `acquire_and_verify_checkpoint` fails before model load on a mismatch.
7. **Verified — qualified ownership is required.** The complete P1/P3 fixture and P2 decoder-layer fixture reuse bare weight keys. P2's eleven weight keys repeat across all 28 layer invocations; P1 and P3 both consume the one tied embedding tensor through graph-local `W_embed` keys. A total, injective global map therefore needs layer-qualified slots plus one shared embedding slot with two declared use sites.

## Outcome — 2026-08-17

The retained record is [`results/2026-08-17-qwen3-0.6b-base-da87bfb6-weight-bindings/manifest.json`](../spikes/program-planning/qwen3-conformance-fixture/results/2026-08-17-qwen3-0.6b-base-da87bfb6-weight-bindings/manifest.json), under the existing Qwen conformance-fixture evidence boundary. Its canonical byte digest is `7044ad5173ee123d8970f7a8f782fc24b607d19628a3af5b036995109de250ee`.

The standard-library-only producer authenticates the complete local checkpoint before emitting. The independently derived verifier proves 310 header tensors, 310 unique checkpoint names, and 310 unique qualified slots; checks every name, `Shape`, checkpoint `BF16`, expected program `StorageScalar::F32`, span, and payload framing; and binds the record to both the exact revision and complete-file SHA-256. Layer ownership is explicit as `P2.layer-NN.<interface_key>`. The tied embedding remains one `P1+P3.shared.W_embed` slot with `P1.W_embed` and `P3.W_embed` use sites.

The unchanged verifier was deliberately exercised against a same-shape K/V map permutation, independently altered checkpoint digest and revision fields, an omission, a duplicate checkpoint name, a duplicate qualified slot, a wrong shape, a wrong program stored scalar, and a foreign name. All nine subjects stopped with their named reason; the unperturbed record passed in the same run.

**Boundary.** The complete checkpoint digest catches any content change inside the source file, while the manifest catches a source-name-to-slot permutation. This retained research record does not implement loading, widening, binding, execution, artifact construction, compiler support, runtime support, or Metal support. It cannot observe a consumer that validates and then ignores the record, nor a same-shape F32 buffer swap after source authentication and named extraction. No L-rung or support-matrix row advances, so no navigation/support claim changes.

The false `57! · 56!² · 28!⁴` product still live at four L6/L8 document and work-record anchors is now owned by [`correct-residual-qwen-weight-permutation-products`](correct-residual-qwen-weight-permutation-products.md); it was not silently repeated or opportunistically repaired here.
