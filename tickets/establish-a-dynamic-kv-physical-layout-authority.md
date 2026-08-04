---
id: establish-a-dynamic-kv-physical-layout-authority
title: Establish a dynamic KV physical-layout authority
status: review
priority: p1
dependencies: []
related: [design-autoregressive-state-and-kv-cache, define-the-runtime-kv-state-boundary, bind-the-kv-cache-through-the-artifact-and-runtime-interface, evaluate-retained-shape-relations-before-routing-commit, admit-live-extent-operands-to-payload-indexing]
scopes: [research/runtime, contracts/artifacts, contracts/integrations, research/program-planning, contracts/foundation]
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

[Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md) records the complete source trace and elimination at exact base `b4e3478d42ce21ed68e23f772b643c6370d36498`. `contracts/navigation` was added as a shared scope because adding that governed research record requires the hand-maintained research catalog entry. `research/program-planning` and `contracts/foundation` were added because selecting the resource population makes L1/L6/L8's “eventual survivor” statements and the glossary row stale; those edits replace only the now-known physical formula and preserve every historical rejected-candidate span. These are mapped corrections required by the authorized research outcome, not product expansion.

One alternative survives: a governed bounded affine layout root, initially the rank-three F32 head-major KV address `base + head × head_stride + sequence × 128 + component`, with `head_stride = capacity × 128` derived from the runtime-owned storage observation. The artifact and payload carry the typed root declaration and use but not its live value; preflight freezes the value into a separate read-only dispatch-parameter block before routing commit. Capacity, `C`, and `S` remain absent from artifact and pipeline specialization identity.

The elimination is not taste. Exact-live dense materialization still needs a live `C`/`S` stride operand. Sequence-major storage is correct but makes the fixed-head score walk advance 4,096 bytes per sequence instead of 512. Per-head resources expand 56 logical K/V members to 448 resources and each K/V tensor use from one transport to eight. Per-extent or per-capacity specialization violates the one-pipeline requirement and contaminates cache identity. The survivor preserves head-locality, one K and V resource per layer, and one payload/pipeline while keeping physical facts out of semantic meaning.

The result is a derived proposal, not acceptance of its consequential public/schema spelling. Source tracing also found the anterior fact that artifact-side input-extent evaluation does not make `C` or `S` consumable by payload body indexing. [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) owns that narrow prerequisite and now blocks symbolic artifact-family delivery. [`draft-governed-affine-layout-roots-through-kernel-and-artifact`](draft-governed-affine-layout-roots-through-kernel-and-artifact.md) depends on it and owns the separately typed physical-root draft. [`route-governed-layout-roots-from-runtime-state`](route-governed-layout-roots-from-runtime-state.md) depends on the layout carrier and state boundary, and owns preflight derivation and committed backend binding. The cache artifact/runtime ticket depends transitively on all three, so it cannot implement either static-extent or implicit-dense payload indexing first. No experiment was needed to select the layout family: inspected source proves the missing consumable parameters, and the address/resource comparisons are exact arithmetic; all carriers still owe executable fail-capable fixtures before Tom accepts their public surfaces.

## Closes when

The current source facts and any measurements are reproducible; every required family is eliminated or survives with a refutable correctness/performance/maintainability derivation; one coherent representation and addressing authority is recorded, or one atomic Tom decision is pending between multiple valid survivors; identity/schema/public-boundary consequences and unsupported cases are explicit; deliberate negatives are accounted for; and correctly scoped carrier tickets and dependency edges prevent either blocked ticket from accepting or implementing the invalid capacity-strided spelling first.
