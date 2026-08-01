---
id: validate-metal-payload-argument-slots-against-declared-bindings
title: Validate a Metal payload's argument slots against the entry's declared bindings
status: in-progress
priority: p2
dependencies: [prototype-candle-metal-adapter]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime, artifacts]
claimed_from: todo
assignee: worker-arg-slots
lease_expires_at: 1785563995
---
## User-visible outcome

A `metallib` whose kernel takes a different set of buffer arguments than the artifact's entry declares is refused before the routing commit, naming the disagreement, rather than reaching the encoder and being bound wrongly.

## Why this is open

ADR 0090 item 8 places three obligations on a backend validating its own payload from bytes: that the bytes decode into something executable, that they name the entry symbol the artifact says they do, and **that the slots they address are the ones the entry declares**. `prototypes/candle-metal-adapter`'s `validate_payload` discharges the first two and not the third.

**Fact — the third needs reflection, which nothing in the stack currently builds.** Reading a Metal function's argument table means `newComputePipelineStateWithFunction:options:reflection:error:` and an `MTLComputePipelineReflection`. Candle's `metal::Device` wrapper (`candle-metal-kernels-0.11.0/src/metal/device.rs`) exposes only `new_compute_pipeline_state_with_function`, which discards reflection, and the adapter builds pipelines through it — so no argument table is available at any point in the route.

**Fact — the consequence today is a wrong binding rather than a refusal.** A slot the object does not address is set and ignored; a slot it addresses that the artifact does not declare is never set, and the kernel reads an unbound argument. Neither produces an error at encode time, so this is one of the few paths in the adapter that does not fail closed. It is bounded by the artifacts this profile routes being produced by Tiler's own emitter from the same ABI, which is a reason it has not bitten rather than a guarantee.

## Closes when

- The adapter obtains the prepared pipeline's reflection — through `objc2-metal` directly if Candle's wrapper still discards it — and compares the buffer arguments it reports against the entry's declared ABI bindings and their transport slots.
- The comparison runs before the routing commit, and its refusal is a typed pre-commit class distinct from an absent symbol.
- The refusal is watched failing against a real object: an artifact whose declared binding count or transport mapping is perturbed away from the object's own.
- If reflection proves unavailable or unreliable on the qualified toolchain row, that is recorded as a measurement with its exact procedure, and the obligation is restated as explicitly undischargeable rather than left implicit.
