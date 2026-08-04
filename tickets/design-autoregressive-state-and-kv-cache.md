---
id: design-autoregressive-state-and-kv-cache
title: Design autoregressive state and KV-cache ownership
status: done
priority: p1
dependencies: [design-attention-program-vertical]
related: [device-placement-and-memory-domain-contract, transfer-synchronization-and-resource-lifetime-contract, prototype-candle-metal-adapter, admit-the-sequence-extension-concatenate-family, admit-an-additive-extent-relation, define-the-runtime-kv-state-boundary, bind-the-kv-cache-through-the-artifact-and-runtime-interface, execute-the-stateful-prefill-path, execute-the-decode-step-path, integrate-the-autoregressive-decode-loop, test-the-autoregressive-state-failure-cases, prove-the-c1-stateful-attention-vertical, scope-a-windowed-kv-append-into-retained-capacity, admit-a-position-selecting-slice-for-the-rotary-table, scope-the-sequence-extending-tensor-family]
scopes: [research/runtime, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, runtime, kv-cache, prefill, decode, language-model]
---
## User-visible outcome

Prefill-then-decode has a designed execution contract: KV state with stated identity, layout, growth, aliasing, and lifetime — kept strictly apart from the immutable artifact and compilation caches, so mutable inference state never contaminates cache identity.

Design the state and execution contract required to run prefill followed by
repeated token decoding. Do not conflate mutable model execution state with the
immutable artifact and compilation caches already owned elsewhere.

## Required design

- Specify the semantic inputs and outputs of prefill and one decode step.
- Define KV-state identity, layout, capacity, valid range, growth, update,
  placement, aliasing, retention, and final-use lifetime.
- State which facts belong to the semantic program, physical plan, artifact,
  runtime instance, and consumer.
- Bound sequence length, batch behavior, masking, and any shape specialization.
- Derive preflight, routing-commit, allocation, dispatch, synchronization, and
  failure behavior across repeated executions.
- Test the design with a small attention example that exposes incorrect
  position, stale-state, partial-update, and cross-device reuse cases.

## Ticket-producing outcome

File vertical tickets for the state representation, artifact/runtime bindings,
prefill path, decode-step path, consumer integration, negative tests, and
end-to-end stateful-attention proof. Public boundaries remain drafts until Tom
reviews their exact implementation.

## Closes when

Ownership and correctness invariants are explicit at every layer; the design
can reject invalid state before program work; prefill and decode have bounded
user-visible outcomes; and the necessary delivery tickets are linked and
dependency ordered.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L5** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L4 delivers a complete transformer block.

**Rests on:** L4.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Its first state contract is an ordinary dense-decoder KV cache; recurrent and convolution state remain owned by the later Qwen3.5 hybrid ticket. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

## Outcome (2026-07-31)

The durable record is [Autoregressive state and KV-cache ownership](../docs/research/runtime/autoregressive-state-and-kv-cache.md), filed under `docs/research/runtime/` and indexed in the research catalog. **The home was chosen by comparison:** the analysis is about ownership across executions, routing commits, retention, and cache identity — the subjects [the runtime execution contract](../docs/research/runtime/runtime-execution-contract.md) and [semantic validation enforcement](../docs/research/runtime/semantic-validation-enforcement.md) already own in that directory — rather than about shapes or program planning, where the two records it consumes live. It takes no measurement and says so.

**Trigger note, recorded because nothing else records it.** This ticket's activation trigger reads "L4 delivers a complete transformer block", which is capability wording. Tom fired it on 2026-07-31 under the **design-rung** reading, and the ladder row now carries that interpretation with its date. The ground: every delivered rung so far (L1–L4, L7) fired on record delivery rather than on capability delivery, and L4's own record states that the block itself is its delivery ticket 7 rather than part of its outcome — so holding a *design* behind the attention implementation chain would buy no evidence the state model needs. `contracts/navigation` was added as a shared scope for the same reason `scope-first-quantized-lm-profile` needed it: the ladder-row edit is required by this ticket's own graph-maintenance section and `docs/roadmap.md` maps to that scope. No open ticket declares it exclusively.

### The state contract, one line each

Identity is `(program interface key, layer ordinal, the live device and context the adapter bound, generation)` and is deliberately not an artifact subject. The logical input/output shapes are `[8, C, 128]` and `[8, S, 128]` F32. Capacity is a fixed logical bound selected at creation from the row's declared maximum context — 18 at C1, 8,320 at B1-d — under `max_position_embeddings = 32768`; the valid range is `[0, C)` and the cursor `C` is its single authority. Growth advances `C` by exactly `T` on observed terminal success and never otherwise; logical `capacity` does not grow and `C + T > capacity` refuses before program work. The update and publication are logically out of place: read the published logical value, produce a distinct replacement value, and replace its governed storage population and cursor together. Placement is the one symbolic affinity's memory domain under ADR 0047 — a state is not a new memory domain and needs no transfer. The later layout authority selects two alternating capacity-sized buffers per member with exact-live head-major payload packing, one active bank, and no physical stride fact; old and replacement banks remain disjoint and retained through exact final use. Lifetime is runtime-instance ownership, consumer destruction, and a terminal poisoned status after any post-commit failure.

### Layer ownership

Eleven facts placed, each with why moving it one layer breaks something nameable. Semantic program: the concatenation's meaning, and the extent relation `S = C + T`. Physical plan: governed layout/addressing, and which contraction realization this `S` admits. Artifact: the accessible-range and launch formulas over the bound extents, and the canonical identity — **`encode_identity` takes `&ArtifactEnvelope` and nothing else**, so no state value can reach it. Runtime instance: logical capacity, the governed storage population, generation, retention, device scoping, and cursor. Consumer: the absolute position of the new tokens, sampling, and termination. **One combination is silently wrong and is named rather than left to taste:** independently observable per-layer cursor advances can expose a partially advanced model; the later model boundary preserves one model-level cursor with atomic publication across every logical member.

### Cache-identity contamination, as three checked invariants

The expansion cache is unreachable by construction — compilation is at expansion time, before any state exists. Artifact identity is already enforced by a type, so the way to break it is not to smuggle a value in but to compile per decode step; the negative test is that C1's nine executions produce one identity. **The reachable one is the third:** the runtime pipeline cache is keyed on its specialization values, so `S`, `C`, and the cursor must be ABI-bound extents and never specialization values — specializing on `S` mints eight cold pipelines at C1 and 128 at B1-d, and makes a mutable inference quantity a cache key. That invariant is currently unbreakable for a reason that will not last: `grep -rniE 'pipelinecache|librarycache|pipeline_cache|library_cache' crates/` returns nothing, positive control `grep -rn 'PipelineCacheKey' docs/` returns the contract that specifies one.

### Four planning decisions — three taken here, one handed to L6

**Prefill is the same program with `C = 0`** (P2), not a second program (P1). `Extent` wraps a plain `u64` and its constructor documents that zero represents an empty axis, so the shape layer does not refuse it; and at `C = 0` the concatenation degenerates to exactly the materialization L4's boundary table already lists for o1 and o2, so P1's saving exists only in a fused plan that no registered fusion role makes reachable. **Publication is logically out of place**, while an in-place/windowed representation still owes a recovery mechanism after partial device work. The earlier capacity-strided realization and singular-allocation residency alternatives are retained only as historical candidate evidence. The later layout authority instead selects exact-live dense packing in two capacity-sized pool banks and measures the allocator/address consequences.

### The worked example and its four cases

Nine C1 executions at the real logical extents: prefill `T = S = 10`, decode steps at `S = 11 … 18`, with the mask and rotary shapes and tiled admissibility. The recorded cache byte columns and `4,128,768`-byte final figure are exact logical live payload and bytes touched for the selected dense packing, but not physical pool reservation or process residency. **The tiled realization is admissible at one of nine executions**, `S = 16`, so eight route direct through the same artifact and the same guard.

**Incorrect position — nothing in Tiler refuses it, and the record says so rather than inventing a check.** A wrong `cos`/`sin` row is a `[1, 128]` F32 tensor with the same shape, dtype, accessible range, and launch geometry as the right one; the envelope decodes, the guard holds, `plan_dispatch`'s byte comparison passes, and the result is a plausible logit vector with a wrong argmax. The structural half of the answer — one cursor authority, and a slice that makes the *inconsistency* mode unrepresentable — is filed with its limit stated: it does not remove the wrong-cursor mode. **Stale state** is refused by the additive extent relation and by nothing else in the stack. **Partial update** leaves the state bit-identical under U-A, so the refusal is the poisoned status, not corruption. **Cross-device reuse** is undetectable by the loader — `ExecutionEnvironment` has three fields and two devices of one family classify identically — and `LiveExecutionContext` deliberately carries no device handle, so the check is necessarily the adapter's.

### One prior judgement superseded, deliberately

`scope-the-sequence-extending-tensor-family` recorded "no capability ticket — filing the additive-extent-relation gap as its own ticket would duplicate a constraint the record hands to the contract work that will need it." That was correct while nothing needed it. This rung is that consumer and it makes the gap load-bearing rather than latent, so [`admit-an-additive-extent-relation`](admit-an-additive-extent-relation.md) is filed with the stale-state case as its motivating evidence. The supersession is stated rather than silent.

### Tickets filed

Eleven, dependency-ordered, none for the flash shape, batching, speculative decoding, or recurrent state:

1. `admit-the-sequence-extension-concatenate-family` — deps `scope-the-sequence-extending-tensor-family`; scopes `implementation/ir`, `implementation/reference`, `contracts/foundation`.
2. `admit-an-additive-extent-relation` — deps 1; scopes `implementation/ir`, `contracts/foundation`.
3. `define-the-runtime-kv-state-boundary` — deps 1 and `establish-a-dynamic-kv-physical-layout-authority`; scopes `contracts/integrations`, `contracts/foundation`, `research/runtime`, `research/program-planning`, `research/numerics`. D-15's semantics now compose with the selected exact-live/capacity-pool descriptor; the complete public boundary remains a tested draft for Tom.
4. `bind-the-kv-cache-through-the-artifact-and-runtime-interface` — deps 1, 3; scopes `implementation/artifact`, `implementation/runtime`, `implementation/build`.
5. `execute-the-stateful-prefill-path` — deps 4, `integrate-the-attention-block-into-the-runtime`.
6. `execute-the-decode-step-path` — deps 5.
7. `integrate-the-autoregressive-decode-loop` — deps 6.
8. `test-the-autoregressive-state-failure-cases` — deps 7.
9. `prove-the-c1-stateful-attention-vertical` — deps 8. **This is the rung's user-visible outcome.**
10. `scope-a-windowed-kv-append-into-retained-capacity` — `deferred`, deps 9 and `establish-a-dynamic-kv-physical-layout-authority`, with survivor-specific activation requiring the selected layout, a binding B1 measurement, a complete recovery contract, and its five obligations.
11. `admit-a-position-selecting-slice-for-the-rotary-table` — deps 7, carrying the incorrect-position case's structural half.

Every public boundary among them is a draft: the concatenation key's spelling, the state object's surface, and the device-scoping question are Tom's.

### Roadmap edits

The L5 ladder row moved from "none" to its delivered outcome and its trigger cell now carries the dated design-rung interpretation. Two support-matrix trigger cells gained dated notes: sequence extension names the two tickets L5 filed and states that the rung does not move because nothing is registered; sub-tensor selection gains a **third** trigger, the position-identity one, which is about correctness rather than bytes. The two prose paragraphs under the ladder that enumerate which rungs carry a record were corrected to include L5.

### Verification

`tkt lint`; `git diff --check`; `tkt guard --base 03a10ae` (verdict `ok`, with the expected non-failing shared-scope overlap on `project/tickets`); `make full`. Every local Markdown link in all fifteen changed files was resolved by a checked script that names its population and counts it — 459 links across the files `git diff --name-only 03a10ae HEAD` lists, 0 missing — and the check was proved able to fail: appending one Markdown link to a nonexistent sibling path made the script name that exact target and exit 1, and the file was restored.

### Deliberately not done

**Dated scope statement for the 2026-07-31 outcome.** No contract edit, ADR, measurement, or implementation was made by this ticket. The later physical-layout ticket supplied the D-14 measurement on 2026-08-04; that correction does not retroactively turn this ticket's original research into measured evidence.
