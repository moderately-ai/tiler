---
id: define-the-model-execution-state-boundary
title: Define the model execution state boundary
status: todo
priority: p1
dependencies: [assemble-the-decoder-layer-program, define-the-runtime-kv-state-boundary, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, drive-the-complete-forward-pass-over-three-artifacts, scope-a-windowed-kv-append-into-retained-capacity]
scopes: [contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [design, runtime, state, kv-cache, lifetime, public-boundary, language-model]
---
## User-visible outcome

A model in flight is one named object with one cursor, so "how many positions does this model hold, and is it usable for the next token" has a typed answer that cannot disagree with itself across 28 layers.

**It is a public boundary and therefore Tom's**; a tested implementation is a concrete draft and not implicit approval.

## Required content

Drafted from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md), which extends [rung L5's state contract](../docs/research/runtime/autoregressive-state-and-kv-cache.md) rather than replacing it.

- **Composition, not a second transaction authority.** Instantiate the generic `KvStateSet` drafted by [`define-the-runtime-kv-state-boundary`](define-the-runtime-kv-state-boundary.md) as 28 ordered K/V pairs — exactly 56 logical cached-tensor `KvState` members, one K and one V member for each layer. This boundary owns that exact model membership and token policy; it reuses the set's sole cursor/status, per-member generations, complete-route commit, poison-all, and publish-all-or-none mechanics rather than defining another mutable-state or publication authority. Each logical member uses the selected two capacity-sized pool banks with an exact-live dense payload; this boundary does not restate their address recipe.
- **The granularity rule, refined.** L5 states that "per-layer programs need per-layer cursors; one program per step needs one." What that rule protects is *observability* — that no consumer can observe a state advanced for some layers or K but not V — not program size. The 30-entry token route therefore prepares and commits the complete 56-member set once and publishes all replacements together after one observed terminal success. The forbidden combination is independently committed layer or K/V subsets with separately observable cursors.
- **The transaction boundary is the token.** The complete old active-bank population for all 56 logical members (28 K/V pairs) is retained until that one observation, while replacements occupy the alternate banks. This is what makes a post-commit failure leave the published model state bit-identical to what it was. The selected physical population is 112 capacity-sized buffers across both banks; process residency still requires measurement. **This is L6's D-16.**
- **Poisoning is model-level by instantiation.** A post-commit failure at any of the 30 route entries drops/fails the one bound set transaction, poisoning all 56 members under the transaction-minted execution identity and token ordinal. It never invents a model-only poison mechanism or leaves one layer/K/V member usable.
- **Typed refusals.** In addition to the generic set refusals, require exactly 28 ordered K/V pairs and reject a missing, extra, duplicated, or mispaired layer member before program work. Capacity, live-scope, poisoned-set, stale generation/fingerprint, and mixed-cursor refusals reuse the runtime boundary rather than being restated with a second definition.

## The question this carries to Tom

**D-16.** Whether the transaction boundary ever moves from the token to the layer. It closes only with both halves together: a measured decode-latency or process-residency result at a B1 row showing that the selected two-bank token transaction is the binding constraint, **and** a recovery contract that says what a consumer does with 28 K/V pairs (56 members) at mixed cursors. The layout representation is now established; the missing evidence is a complete-model resident-process measurement, and residency evidence alone could motivate the change but would not make it safe.

## Closes when

The runtime-state boundary has landed; the model boundary is drafted with every property and refusal above; D-16 is put to Tom only if complete-model measurement makes it live; and nothing is accepted as public without his answer.
