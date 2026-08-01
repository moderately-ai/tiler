---
id: design-model-ingestion-and-complete-execution
title: Design model ingestion and complete supported-model execution
status: done
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, design-autoregressive-state-and-kv-cache]
related: [prototype-public-compiler-api, prototype-candle-metal-adapter, prototype-inline-proc-macro-frontend, route-an-embedded-artifact-through-a-consumer-storage-seam, admit-an-indirect-gather-family-for-tied-embedding-lookup, scope-first-quantized-lm-profile, admit-a-storage-carrier-for-integer-program-inputs, assemble-the-decoder-layer-program, assemble-the-embedding-and-vocabulary-projection-programs, widen-the-deterministic-budgets-to-the-decoder-layer-program, define-the-model-weight-binding-manifest, ingest-the-checkpoint-as-f32-program-inputs, define-the-model-execution-state-boundary, drive-the-complete-forward-pass-over-three-artifacts, retain-the-c1-model-attribution-fixture, name-the-execution-ordinal-in-model-level-failures, prove-the-c1-complete-model-execution, project-only-the-final-position-logits]
scopes: [contracts/integrations, contracts/navigation, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [design, frontend, model, weights, integration, language-model]
---
## User-visible outcome

A consumer can hand Tiler a supported model (architecture, config, weights) and inputs, and receive logits — through a typed boundary that keeps every consumer format and Candle type out of compiler semantics.

Define how a consumer supplies a supported model architecture, configuration,
weights, and inference inputs and receives logits without making a consumer
format or Candle type part of compiler semantics.

## Required design

- Select the bounded model-description and weight-container boundary required
  by the representative workload.
- Map configuration and weights into typed semantic program inputs with
  complete identity, shape, dtype, layout, and validation rules.
- Define whole-model composition across layers, entrypoints, artifacts,
  runtime instances, and persistent decode state.
- State unsupported-model, unsupported-operation, and fallback behavior before
  routing commit.
- Separate tokenizer and sampling concerns from compiler ownership while
  identifying the integration contract needed to produce and consume logits.
- Define complete-model reference comparison and failure reporting.

## Ticket-producing outcome

File delivery tickets for model description or adapter work, weight validation
and binding, whole-model graph construction, artifact/program orchestration,
consumer integration, and a complete supported-model execution proof. Reuse
the existing public compiler, macro, Candle, artifact, and runtime tickets where
they already own a prerequisite.

## Closes when

One supported model can be described end to end without an unowned boundary;
the frontend/runtime dependency direction remains consumer-independent; every
unsupported case has an explicit behavior; and the complete-model vertical is
represented by scoped, dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L6** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L2 and L5 both deliver.

**Rests on:** L2 and L5.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Derive the model boundary from that exact revision rather than from a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

## Outcome (2026-08-01)

The durable record is [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md), filed under `docs/research/program-planning/` and indexed in the research catalog. **The home was chosen by comparison:** the analysis is about the shape of a whole program, its partition into artifacts, and its residency — the subjects [the attention program vertical](../docs/research/program-planning/first-attention-program-vertical.md), [the workload profile](../docs/research/program-planning/first-metal-lm-workload.md), and [the general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md) already own in that directory — rather than about runtime ownership, where L5's record lives. It takes no measurement and says so.

**Trigger note, recorded because nothing else records it.** This ticket's activation trigger reads "L2 and L5 deliver". Both delivered as design records; L5's own trigger read "L4 delivers a complete transformer block", which is capability wording, and Tom fired it on 2026-07-31 under the **design-rung** reading. That established interpretation extends here — every delivered rung so far fired on record delivery rather than on capability delivery — and the ladder row now carries the dated note in the same form L5's worker used. `contracts/navigation` was already declared on this ticket, which is what the ladder-row and catalog edits need.

**Base correction, disclosed rather than absorbed.** The worktree was created at `a3d61bd`, one commit behind the named base `d862c2b` — which is `origin/main` and is the commit recording this claim. The tree was clean and untouched, so it was fast-forwarded to the named base before any edit; `tkt guard --base d862c2b` therefore measures the intended diff.

### The four boundaries, one line each

**Model description:** it never crosses — the architecture and every configuration value except `T` and `S` enter as literal extents and literal attributes in the region the consumer wrote, so Tiler holds no `ModelDescription`, no architecture key, and no configuration record, and a second architecture is a second region rather than a second value. **Weights:** ordinary program inputs, the consumer's own values wrapped by the consumer's own `TensorAdapter`, with no weight container, no checkpoint format, and no tensor-name vocabulary anywhere in Tiler. **Inputs:** `[T]` token IDs at an admitted integer identity, the host-precomputed `cos` and `sin`, the host-built additive mask, the two-element `rope_sign`, and the per-layer cached `[8, C, 128]` key and value tensors. **Logits:** one output, `[T, 151936]` F32 row-major dense, unnormalized, with no softmax, temperature, top-k, mask, argmax, or token.

### The eliminations

**The model description is the region (M-C), not a compiler type (M-A) or a consumer-implemented trait (M-B).** M-A makes an architecture part of compiler identity and contradicts ADR 0069 rather than merely costing something; M-B's method set *is* the architecture surface and inverts the dependency direction during compilation. The consequence worth stating is that the layer count is the one configuration value that must not enter any program — it is a repetition count over an identical subgraph, and putting it inside makes a 28-layer and a 32-layer model two identities for one computation. The checkpoint-versus-region trap L1 named is then a refusal that exists today: a region compiled at the derived `head_dim = 64` binds a `[128]` weight and refuses as `BindError::LiteralExtentMismatch` with both numbers.

**The weight container is the consumer's map (W-C).** W-A is refused by an implemented budget — `verify_program` checks `buffers` against `4.max(input_count + 1)` against a limit of 4 — and W-B is a consumer format in disguise. The partition closes against L1: eleven weights per layer is 62,923,776 B, ×28 is 1,761,865,728, plus 622,329,856 for the tied matrix and 4,096 for `model.norm`, is exactly L1's 2,384,199,680-byte F32 budget.

**BF16 ingestion is host-side (I-B), and L6 adds the ground L2 did not have.** An operation inside a program runs on every execution of that program, so a `Cast` would convert 1,761,865,728 bytes on every token — 270 times over the C1 row — against once at load, and no hoisting capability could lift it out because the boundary is the consumer's loop.

**The embedding gather stays inside (IN-A), which is the explicit decision the gather ticket demanded.** IN-B is the cheaper option and saves nothing: it does not remove the tied matrix from the boundary, it does not save time (one gather against 253 contractions), it moves the bounds obligation to a layer that cannot state one, and it makes the failure undetectable in exactly L5's shape. It would also make the rung's own outcome false — a consumer that performs the model's first operation hands over most of a model.

### D-13 closed: two axes, not one

L5 handed over a choice between 3,816,587,264 B of peak KV residency for one program per step and 1,976,557,568 B for one program per layer. **The arithmetic is right and the framing conflates two independent decisions.** The per-layer figure holds one old allocation live at a time, which requires releasing each before the next layer runs — and L5's own retention rule makes an old allocation releasable only after the *completion condition*, which is per submission. So the second row is reachable only with 28 submissions and 28 host completion observations per token, and releasing early is precisely giving up the restorability that makes U-A's failure non-destructive.

The design takes **the per-layer program with the token transaction**: 3.5544 GiB, one host round trip, a post-commit failure leaving the state bit-identical. It declines 1.714 GiB that is worth 3.8 MB at the conformance row. **L5's cursor-granularity rule is refined rather than contradicted:** what it protects is observability, so per-layer programs with one model cursor and a generation per layer state, published atomically after one observed terminal success, satisfy it; 28 independently advanced cursors stay forbidden.

### The layer-ownership additions

Ten facts beyond L5's table, each with why moving it one layer breaks something nameable. The new *layer* is the runtime-value boundary (`tiler::value`), which L5's table does not have and which owns rank, stored scalar, literal extent, and symbol validation before any route is selected. The others place the model's meaning at the region text, the checkpoint's identity and the BF16 widening at the consumer, the token bounds split between the program and the consumer, the layer count as a consumer loop bound, the model cursor over 28 generations at the runtime instance, the tied matrix's single allocation at the consumer, and the logits' covered positions at the semantic program.

### The permutation nothing refuses

The checkpoint's 310 tensors fall into eight shape classes whose members are mutually interchangeable under every check in the stack: 57 `[1024]`, 56 `[1024, 1024]`, 56 `[128]`, 56 `[3072, 1024]`, 28 each of `[2048, 1024]`, `[1024, 2048]`, `[1024, 3072]`, and one `[151936, 1024]`. So `57! · 56!² · 28!⁴` bindings pass and one is right. This is not "unchecked" — the checks that exist all pass — and the named enforcement boundary is a consumer-side weight binding manifest under L1's manifested-by-digest discipline, placed there because a per-tensor digest in artifact identity would compile one artifact per checkpoint and a runtime-validated one is the same check by a longer route.

### Peak residency, the figure L1 deferred

2.2299 GiB at C1 prefill and at most 2.2287 GiB at C1 decode 8 — the weight budget plus 0.4%, so the conformance row says nothing about residency. At most 5.7777 GiB at B1-d's final decode step, which is 1,911,183,360 B (1.7799 GiB) above L1's B1-d row because that row counts one copy of the cache and an extending execution holds two. And at B1-d prefill the binding terms are neither the weights nor the cache: 26.1462 GiB unfused, 10.1472 GiB under D-B, and 5.5111 GiB under D-B with final-position logits. Every peak is the exact sum of its row's stated terms rather than an estimate beside them.

### Five refusals stand between this design and a compiled model

The deterministic budgets (`semantic_values` 16, `semantic_operations` 8, `buffers` 4 against `max(4, inputs + 1)`) refuse the layer program on three counts and pass the other two programs; the recognizer refuses all three, and it is a different refusal with a different remedy; the inline route refuses a symbolic region at `AotRefusal::SymbolicExtent`; the facade cannot dispatch, terminating at `RouteOutcome::NoDeviceAuthority` until the storage seam lands; and **the landed Candle adapter implements `CustomOp1` — one tensor input, one output — against a decoder layer's eighteen and three**, so the first complete-model consumer is not the Candle custom-op wrapper and no partition fixes it. Every budget is inside `VerifiedRequestSubject::canonical_bytes` and therefore inside artifact identity, which is what makes a widening a decision rather than a knob.

### Tickets filed

Twelve, dependency-ordered, reusing the existing capability, state, symbolic-extent, storage-seam, and fixture tickets rather than duplicating them:

1. `admit-a-storage-carrier-for-integer-program-inputs` — deps `admit-an-indirect-gather-family-for-tied-embedding-lookup`. Carries D-17 to Tom.
2. `assemble-the-decoder-layer-program` — deps `assemble-the-causal-self-attention-block-program`, `admit-the-silu-activation-family`, `admit-the-sequence-extension-concatenate-family`.
3. `assemble-the-embedding-and-vocabulary-projection-programs` — deps 1, `admit-the-rms-normalization-family`.
4. `widen-the-deterministic-budgets-to-the-decoder-layer-program` — deps 2. Carries D-18 to Tom.
5. `define-the-model-weight-binding-manifest` — deps 2, 3.
6. `ingest-the-checkpoint-as-f32-program-inputs` — deps 5, `route-an-embedded-artifact-through-a-consumer-storage-seam`.
7. `define-the-model-execution-state-boundary` — deps 2, `define-the-runtime-kv-state-boundary`. Carries D-16 to Tom.
8. `drive-the-complete-forward-pass-over-three-artifacts` — deps 4, 6, 7, `deliver-an-artifact-family-from-a-symbolic-region`, `integrate-the-autoregressive-decode-loop`.
9. `retain-the-c1-model-attribution-fixture` — deps `retain-the-qwen-conformance-reference-logit-fixture`; runs in parallel.
10. `name-the-execution-ordinal-in-model-level-failures` — deps 8.
11. `prove-the-c1-complete-model-execution` — deps 8, 9, `prove-the-c1-stateful-attention-vertical`. **This is the rung's user-visible outcome.**
12. `project-only-the-final-position-logits` — deps 3, `admit-a-position-selecting-slice-for-the-rotary-table`.

Every public boundary among them is a draft: the integer storage carrier, the model execution state's surface, and any budget widening are Tom's.

### Roadmap edits

The L6 ladder row moved from "none" to its delivered outcome and its trigger cell carries the dated design-rung interpretation. The two prose paragraphs under the ladder that enumerate which rungs carry a record were corrected to include L6, with a sentence stating what it closed. The support matrix's sub-tensor-selection trigger cell gained a dated note: L6 fires that row's **first** trigger — the final-position logits one — with the 4,978,027,008-byte arithmetic and the ticket that owns it, and the rung does not move because nothing is registered.

### Deliberately not done

No contract sentence was edited. `docs/integration/candle.md` and `docs/integration/frontends.md` are named in `informs` and were read as evidence only; the record's `disposition` is `pending` for that reason, as L1's, L2's, L4's, and L5's are. No tolerance, no measurement, and no latency or throughput claim — every byte figure is arithmetic over quantities L1, L4, and L5 already state. No ticket for a model-description type or adapter, a compiler-side weight container, a `Cast` family, a device-side argmax or sampler, a tokenizer, a Candle multi-operand launch path, the quantized calibration question L7 owns, or any model-level accuracy budget, which L8 owns. No crate admission is requested for a model driver; the record proposes a prototype member and says crate admission is Tom's.

### Verification

`tkt lint`; `git diff --check`; `tkt guard --base d862c2b`; `make full`. Local Markdown links in every changed file were resolved by a checked script that names its population and counts it, and the check was proved able to fail before it was trusted.
