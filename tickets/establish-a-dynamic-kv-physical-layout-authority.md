---
id: establish-a-dynamic-kv-physical-layout-authority
title: Establish a dynamic KV physical-layout authority
status: review
priority: p1
dependencies: []
related: [design-autoregressive-state-and-kv-cache, define-the-runtime-kv-state-boundary, bind-the-kv-cache-through-the-artifact-and-runtime-interface, evaluate-retained-shape-relations-before-routing-commit, admit-live-extent-operands-to-payload-indexing]
scopes: [research/runtime, contracts/artifacts, contracts/integrations, research/program-planning, contracts/foundation, research/numerics]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [research, design, runtime, artifact, abi, kv-cache, correctness]
claimed_from: ready
assignee: agent-kv-layout
lease_expires_at: 1785863038
---
## User-visible outcome

One KV artifact and prepared pipeline address every decode step's retained storage correctly even when physical capacity exceeds the live logical extent — without silently treating a capacity-strided tensor as dense or compiling per cursor.

## Exact blocker

**Fact.** The rejected/provisional dense cache candidate is physical F32 `[8, capacity, 128]`, so its head stride is `capacity × 128 × 4`. The semantic input/output views are `[8, C, 128]` and `[8, S, 128]`; their ordinary dense head strides are `C × 128 × 4` and `S × 128 × 4`. These differ whenever the live extent is below capacity.

**Fact.** The current artifact/runtime ABI exposes evaluated accessible byte offset and byte count for a routed binding but no dynamic physical-stride or layout root a payload can consume. Capacity is deliberately absent from artifact identity and specialization values.

**Inference.** A future descriptor or runtime observation could diagnose that bound storage has a capacity-derived stride, but diagnosis alone cannot make payload indexing consume that stride. No such governed descriptor is landed or treated as authority by this ticket.

**Inference.** Implementing the current capacity-strided spelling can read or write the wrong head while returning a plausible tensor. This is a correctness blocker for both `define-the-runtime-kv-state-boundary` and `bind-the-kv-cache-through-the-artifact-and-runtime-interface`, not an adapter detail.

`research/runtime` owns the retained-state design record or reproducible spike. `contracts/artifacts` and `contracts/integrations` are required because any survivor must state its artifact/ABI and public runtime consequences together. These mapped scopes declare already-required research output; they do not authorize a production public boundary or schema step.

## Required research

Evaluate at least these design families against correctness first, then one-artifact/one-pipeline performance, then long-term maintainability and support:

1. Explicit dynamic physical-layout authority: governed stride/layout roots encoded and validated through artifact/ABI, bound from runtime state, and consumed by payload indexing without specializing on `C`, `S`, capacity, or another cursor-derived value.
2. A representation whose live logical rows are genuinely contiguous and has no hidden capacity stride. Include per-head or componentized allocations/bindings, copying or materialization, K/V and multi-layer population consequences, allocation/dispatch overhead, alias verification, retention, validation readback, and atomic publication.
3. Any bounded specialization alternative. Prove whether a finite non-cursor layout family can preserve one artifact identity and one prepared pipeline across all decode steps; reject per-`C`, per-`S`, or capacity-derived pipeline specialization rather than disguising it as a bounded profile.

Inspect the exact current source and accepted contracts rather than reasoning from field names. Trace binding layout facts from artifact construction through codec, loader evaluation, routed binding, backend payload indexing, storage observation, validation, and cache/pipeline identity. Preserve the distinction between a descriptor that diagnoses storage and authority that changes payload addressing.

For each candidate, identify:

- the exact identity and artifact-schema consequences, including whether a whole version-domain step and recomputed pins would be required;
- every consequential public type or method and its owning crate;
- runtime binding, preflight, feasibility, alias, retention, and publication consequences;
- unsupported layouts, ranks, batching, raggedness, capacity policies, and backend capabilities;
- an executable bounded experiment when source/specification evidence cannot decide payload expressibility or cost, with exact environment, inputs, oracle, stop condition, and retained fixture; and
- the downstream implementation scopes and migration/removal consequences, without implementing production code in this ticket.

Run the AGENTS.md elimination process explicitly. If one candidate survives correctness, performance, and maintainability, record it as the derived proposal and file any carrier/identity-step work. Ask Tom one atomic question only if multiple genuinely valid survivors remain after research; acceptance of a consequential public or schema boundary remains his.

## Deliberate negatives

The result must make these fail closed:

- **Only if the capacity-strided alternative survives:** at `capacity = 18, C = 14`, head 1 begins at byte `9,216`, not the dense-logical byte `7,168`, and the corresponding replacement at `S = 15` uses the same governed capacity stride without minting another artifact or pipeline;
- an artifact or adapter omits, misorders, or lies about a required layout/stride fact; and
- a proposed optimization reintroduces cursor/capacity specialization into artifact, library, or pipeline identity.

Every evaluated alternative must additionally provide its own exact address/layout negative oracle. The oracle names one concrete wrong address, stride, segmentation, resource selection, or equivalent layout fact for that representation and demonstrates that it refuses before program work or poisons after commit, as appropriate. Passing the capacity-stride oracle is neither required nor sufficient for an alternative that does not use capacity-strided storage.

## Graph maintenance

Keep both blocked dependents linked until the durable authority and its exact handoff are recorded. Split any independently implementable schema/identity step, runtime binding, backend realization, or experiment into narrow tickets with dependency order and scopes. A conditional capability belongs at `deferred` with a real activation trigger; work required to solve this blocker remains active.

## Outcome — 2026-08-04

[Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md) records the complete source trace, corrected elimination, and bounded Metal measurement at exact base `b4e3478d42ce21ed68e23f772b643c6370d36498`. `contracts/navigation` was added as a shared scope for the hand-maintained catalogs; `research/program-planning` and `contracts/foundation` cover the current-authority corrections across L1/L5/L6/L8 and the glossary. `research/numerics` was added because the selected physical reservation makes the quantized profile's current “survivor still unknown” statement false; correcting that statement does not broaden the authorized product outcome. The pre-cleanup Markdown population under `docs/` and `tickets/` counted 1,034 files; after deleting the two obsolete carrier tickets, the complete 1,032-file current population was searched in one invocation. Historical spans remain labelled historical rather than silently rewritten.

One alternative survives: exact-live head-major payloads packed at `C*128` and `S*128` inside two alternating capacity-sized pool buffers per logical K or V member. Allocation length is not payload layout. At `capacity=18, C=14, S=15`, old and replacement `(1,0,0)` addresses are bytes 7,168 and 7,680 and their live spans are 57,344 and 61,440 bytes inside separate 73,728-byte buffers. Capacity never reaches artifact, payload, or pipeline identity.

The retained Apple M4 Max/Apple9 spike rotates four rows over five rounds and proves each wrong-address oracle can fail: compact exact-live, the selected capacity-sized exact-live representation, capacity-strided head-major, and sequence-major. At B1-first, median-of-round GPU medians are 750.500 us for both exact-live rows, 750.500 us capacity-strided, and 779.208 us sequence-major; B1-last is 761.708, 761.708, 761.958, and 791.375 us. Pooled exact-live therefore has no measured input-access disadvantage. Its exact pinned C1 `S=10…18` and B1 allocation-lifecycle medians are 13.250 and 17.333 us, versus 14.458/17.083 for capacity-strided and 72.042/1,675.958 for fresh compact allocation. Thus the capacity-root candidate has no reuse or measured access advantage, requires larger accessible bounding spans, and adds a physical-root/schema surface; both copy kernels still transfer the same live payload bytes. The retained arithmetic oracle distinguishes payload transfer, accessible bounding span, and the common two-bank pool reservation. Sequence-major is 3.8–3.9% slower on both B1 copy cells with no correctness, pooling, identity, or resource advantage. Per-head resources and specialization remain structurally dominated.

Fixed-point review corrected an earlier conflation of those three quantities. Model-wide live payload transfer is identical for exact-live and capacity-strided copies: 2,293,760 bytes at C1 prefill, 8,028,160 at C1 final, and 3,816,587,264 at B1 final. Capacity-strided routing instead requires accessible bounding spans of 3,899,392, 8,228,864, and 3,816,787,968 bytes at those cells. The shared two-bank reservations are separately 8,257,536 bytes for C1 and 3,816,816,640 for B1. The retained checker derives all values from heads, width, element size, member population, extents, and capacity, and its injected pool-as-payload confusion fails.

The anterior live-extent gap remains real and [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) owns it. It is the only generic payload-address prerequisite for [`bind-the-kv-cache-through-the-artifact-and-runtime-interface`](bind-the-kv-cache-through-the-artifact-and-runtime-interface.md). The two physical-root carriers filed by the first elimination were deleted as obsolete before dispatch: their candidate did not survive, and leaving them active would manufacture an artifact/schema boundary the selected representation does not need. No Tom decision remains on layout. Tom still owns the live-extent carrier's tested consequential public/schema spelling when that implementation reaches review.

## Closes when

The current source facts and any measurements are reproducible; every required family is eliminated or survives with a refutable correctness/performance/maintainability derivation; one coherent representation and addressing authority is recorded, or one atomic Tom decision is pending between multiple valid survivors; identity/schema/public-boundary consequences and unsupported cases are explicit; deliberate negatives are accounted for; and correctly scoped carrier tickets and dependency edges prevent either blocked ticket from accepting or implementing the invalid capacity-strided spelling first.
