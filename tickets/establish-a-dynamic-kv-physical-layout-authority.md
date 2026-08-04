---
id: establish-a-dynamic-kv-physical-layout-authority
title: Establish a dynamic KV physical-layout authority
status: ready
priority: p1
dependencies: []
related: [design-autoregressive-state-and-kv-cache, define-the-runtime-kv-state-boundary, bind-the-kv-cache-through-the-artifact-and-runtime-interface, evaluate-retained-shape-relations-before-routing-commit]
scopes: [research/runtime, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, runtime, artifact, abi, kv-cache, correctness]
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

## Closes when

The current source facts and any measurements are reproducible; every required family is eliminated or survives with a refutable correctness/performance/maintainability derivation; one coherent representation and addressing authority is recorded, or one atomic Tom decision is pending between multiple valid survivors; identity/schema/public-boundary consequences and unsupported cases are explicit; deliberate negatives are accounted for; and correctly scoped carrier tickets and dependency edges prevent either blocked ticket from accepting or implementing the invalid capacity-strided spelling first.
