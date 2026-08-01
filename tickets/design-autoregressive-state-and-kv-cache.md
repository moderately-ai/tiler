---
id: design-autoregressive-state-and-kv-cache
title: Design autoregressive state and KV-cache ownership
status: in-progress
priority: p1
dependencies: [design-attention-program-vertical]
related: [device-placement-and-memory-domain-contract, transfer-synchronization-and-resource-lifetime-contract, prototype-candle-metal-adapter, admit-the-sequence-extension-concatenate-family, admit-an-additive-extent-relation, define-the-runtime-kv-state-boundary, bind-the-kv-cache-through-the-artifact-and-runtime-interface, execute-the-stateful-prefill-path, execute-the-decode-step-path, integrate-the-autoregressive-decode-loop, test-the-autoregressive-state-failure-cases, prove-the-c1-stateful-attention-vertical, scope-a-windowed-kv-append-into-retained-capacity, admit-a-position-selecting-slice-for-the-rotary-table, scope-the-sequence-extending-tensor-family]
scopes: [research/runtime, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, runtime, kv-cache, prefill, decode, language-model]
claimed_from: todo
assignee: worker-kv-cache
lease_expires_at: 1785558508
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

Identity is `(program interface key, layer ordinal, the live device and context the adapter bound, generation)` and is deliberately not an artifact subject. Layout is `[8, S, 128]` F32 row-major, L4's o1 and o2 shape unchanged. Capacity is a fixed `[8, capacity, 128]` allocation set at creation from the row's declared maximum context — 18 at C1, 8,320 at B1-d — under `max_position_embeddings = 32768`. The valid range is `[0, C)` and the cursor `C` is the single authority for it. Growth advances `C` by exactly `T` on observed terminal success and never otherwise; `capacity` does not grow and `C + T > capacity` refuses before program work. The update is out of place: read `[8, C, 128]`, write a distinct `[8, S, 128]`, replace allocation and cursor together. Placement is the one symbolic affinity's memory domain under ADR 0047 — a state is not a new memory domain and needs no transfer. Aliasing is none, which `verify_storage`'s `ForbiddenAlias` already requires and which is what makes a failed step non-destructive. Retention holds both allocations through their exact final device use, the old one releasable only after the completion condition and never after its last encoder call. Lifetime is runtime-instance ownership, consumer destruction, and a terminal poisoned status after any post-commit failure.

### Layer ownership

Eleven facts placed, each with why moving it one layer breaks something nameable. Semantic program: the concatenation's meaning, and the extent relation `S = C + T`. Physical plan: layout, and which contraction realization this `S` admits. Artifact: the accessible-range and launch formulas over the bound extents, and the canonical identity — **`encode_identity` takes `&ArtifactEnvelope` and nothing else**, so no state value can reach it. Runtime instance: capacity, the allocation, the generation, the retention lease, the device scoping, and the cursor. Consumer: the absolute position of the new tokens, sampling, and termination. **One combination is silently wrong and is named rather than left to taste:** the cursor's granularity must equal the program-boundary granularity, because a single model-level cursor with per-layer programs advances on a per-layer completion while twenty-seven other layers have not.

### Cache-identity contamination, as three checked invariants

The expansion cache is unreachable by construction — compilation is at expansion time, before any state exists. Artifact identity is already enforced by a type, so the way to break it is not to smuggle a value in but to compile per decode step; the negative test is that C1's nine executions produce one identity. **The reachable one is the third:** the runtime pipeline cache is keyed on its specialization values, so `S`, `C`, and the cursor must be ABI-bound extents and never specialization values — specializing on `S` mints eight cold pipelines at C1 and 128 at B1-d, and makes a mutable inference quantity a cache key. That invariant is currently unbreakable for a reason that will not last: `grep -rniE 'pipelinecache|librarycache|pipeline_cache|library_cache' crates/` returns nothing, positive control `grep -rn 'PipelineCacheKey' docs/` returns the contract that specifies one.

### Four planning decisions — three taken here, one handed to L6

**Prefill is the same program with `C = 0`** (P2), not a second program (P1). `Extent` wraps a plain `u64` and its constructor documents that zero represents an empty axis, so the shape layer does not refuse it; and at `C = 0` the concatenation degenerates to exactly the materialization L4's boundary table already lists for o1 and o2, so P1's saving exists only in a fused plan that no registered fusion role makes reachable. **The update is out of place** (U-A), not a windowed write (U-B): U-B owes four implemented refusals — `ExternalValueWritten`, `MultipleWriters`, the absent proof about untouched bytes, and `ForbiddenAlias` — plus the one U-A gets free, since a post-commit failure under U-B leaves the retained state partially updated with nothing to prove which bytes are new, and ADR 0033 is explicit that initial transactions are out of place. **The layout stays `[8, S, 128]`**, decided on contraction locality, because the contiguous-window argument for `[S, 8, 128]` is worth nothing while the whole tensor is copied anyway. **The program boundary is L6's**, handed over with the arithmetic: one program per step costs 3,816,587,264 B of peak KV residency at B1-d against 1,976,557,568 B for per-layer programs — 1,840,029,696 B (1.714 GiB) — and the copy traffic, 1.60× the model's own weight traffic per token, is identical under both.

### The worked example and its four cases

Nine C1 executions at the real extents: prefill `T = S = 10`, decode steps at `S = 11 … 18`, with the cache in and out bytes per layer (0 → 81,920 at prefill, 139,264 → 147,456 at step 8), the model-wide figures, the mask and rotary shapes, and the tiled admissibility. The state after step 8 is `28 × 147,456 = 4,128,768` bytes, exactly L1's 18-position figure. **The tiled realization is admissible at one of nine executions**, `S = 16`, so eight route direct through the same artifact and the same guard.

**Incorrect position — nothing in Tiler refuses it, and the record says so rather than inventing a check.** A wrong `cos`/`sin` row is a `[1, 128]` F32 tensor with the same shape, dtype, accessible range, and launch geometry as the right one; the envelope decodes, the guard holds, `plan_dispatch`'s byte comparison passes, and the result is a plausible logit vector with a wrong argmax. The structural half of the answer — one cursor authority, and a slice that makes the *inconsistency* mode unrepresentable — is filed with its limit stated: it does not remove the wrong-cursor mode. **Stale state** is refused by the additive extent relation and by nothing else in the stack. **Partial update** leaves the state bit-identical under U-A, so the refusal is the poisoned status, not corruption. **Cross-device reuse** is undetectable by the loader — `ExecutionEnvironment` has three fields and two devices of one family classify identically — and `LiveExecutionContext` deliberately carries no device handle, so the check is necessarily the adapter's.

### One prior judgement superseded, deliberately

`scope-the-sequence-extending-tensor-family` recorded "no capability ticket — filing the additive-extent-relation gap as its own ticket would duplicate a constraint the record hands to the contract work that will need it." That was correct while nothing needed it. This rung is that consumer and it makes the gap load-bearing rather than latent, so [`admit-an-additive-extent-relation`](admit-an-additive-extent-relation.md) is filed with the stale-state case as its motivating evidence. The supersession is stated rather than silent.

### Tickets filed

Eleven, dependency-ordered, none for the flash shape, batching, speculative decoding, or recurrent state:

1. `admit-the-sequence-extension-concatenate-family` — deps `scope-the-sequence-extending-tensor-family`; scopes `implementation/ir`, `implementation/reference`, `contracts/foundation`.
2. `admit-an-additive-extent-relation` — deps 1; scopes `implementation/ir`, `contracts/foundation`.
3. `define-the-runtime-kv-state-boundary` — deps 1; scopes `contracts/integrations`, `contracts/foundation`. Carries D-15 to Tom.
4. `bind-the-kv-cache-through-the-artifact-and-runtime-interface` — deps 1, 3; scopes `implementation/artifact`, `implementation/runtime`, `implementation/build`.
5. `execute-the-stateful-prefill-path` — deps 4, `integrate-the-attention-block-into-the-runtime`.
6. `execute-the-decode-step-path` — deps 5.
7. `integrate-the-autoregressive-decode-loop` — deps 6.
8. `test-the-autoregressive-state-failure-cases` — deps 7.
9. `prove-the-c1-stateful-attention-vertical` — deps 8. **This is the rung's user-visible outcome.**
10. `scope-a-windowed-kv-append-into-retained-capacity` — `deferred`, deps 9, with U-B's exact residency trigger and its five obligations.
11. `admit-a-position-selecting-slice-for-the-rotary-table` — deps 7, carrying the incorrect-position case's structural half.

Every public boundary among them is a draft: the concatenation key's spelling, the state object's surface, and the device-scoping question are Tom's.

### Roadmap edits

The L5 ladder row moved from "none" to its delivered outcome and its trigger cell now carries the dated design-rung interpretation. Two support-matrix trigger cells gained dated notes: sequence extension names the two tickets L5 filed and states that the rung does not move because nothing is registered; sub-tensor selection gains a **third** trigger, the position-identity one, which is about correctness rather than bytes. The two prose paragraphs under the ladder that enumerate which rungs carry a record were corrected to include L5.

### Verification

`tkt lint`; `git diff --check`; `tkt guard --base 03a10ae` (verdict `ok`, with the expected non-failing shared-scope overlap on `project/tickets`); `make full`. Every local Markdown link in all fifteen changed files was resolved by a checked script that names its population and counts it — 459 links across the files `git diff --name-only 03a10ae HEAD` lists, 0 missing — and the check was proved able to fail: appending one Markdown link to a nonexistent sibling path made the script name that exact target and exit 1, and the file was restored.

### Deliberately not done

**No contract edit.** `contracts/foundation` and `contracts/integrations` are declared scopes and neither `docs/architecture.md` nor `docs/integration/**` was touched: nothing here is accepted, and a proposed record must not become the operative rule by default. **No ADR** — this ticket's scopes admit no `contracts/decisions`, and the two questions that would need one (the state object's public surface and its device scoping) are routed to Tom through ticket 3 instead. **No measurement of any kind**, so D-12's mask question, D-14's layout trade, and every latency or residency claim stay arithmetic or open. **No implementation.**
