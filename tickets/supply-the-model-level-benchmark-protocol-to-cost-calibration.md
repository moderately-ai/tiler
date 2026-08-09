---
id: supply-the-model-level-benchmark-protocol-to-cost-calibration
title: Supply the benchmark protocol cost calibration is waiting on
status: todo
priority: p3
dependencies: [build-the-model-level-measurement-harness, reclassify-language-model-work-as-a-conformance-track]
related: [calibrate-device-cost-models, implement-analytical-component-cost-model, design-model-level-qualification-and-optimization]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [research, cost-model, measurement, performance, calibration, language-model, class-performance-study]
---
## User-visible outcome

[`calibrate-device-cost-models`](calibrate-device-cost-models.md) has the one activation input it still names as missing — a reproducible benchmark protocol against a selected workload — so its deferral is a scheduling question rather than a missing artefact.

## Why this exists

**Fact.** That ticket is `deferred` behind four inputs, and its own trigger 3 records that [`define-first-metal-lm-workload`](define-first-metal-lm-workload.md) "supplies three of the four activation inputs at once — representative kernels, the exact target profile, and the device — leaving only the benchmark protocol, which is this ticket's own work." That workload selection has since delivered.

**Inference — the protocol is a property of the workload's shape and its host discipline; the fitting, provenance, uncertainty, and drift policy are properties of the model.** Splitting them this way keeps the calibration ticket's own subject intact and lets the protocol be reviewed against the workload rather than against the cost model.

## Required work

- Write the protocol against the L8 measurement harness rather than beside it: the same bench host, the same interleaved-A/B and settled-minimum procedure, and the same amendment that a model-level A/B interleaves whole forward passes and never shares one weight allocation between variants.
- Map each of the analytical model's nine components to what the model-level workload can and cannot supply. **Fact correction:** `crates/tiler-compiler/src/component_cost.rs`, anchor `CostComponent::ResourcePressure | CostComponent::CompileTime => CostValue::Unknown`, leaves only those two components unknown in the governed analytical model. Allocation, dispatch, synchronization, indexing, redundant work, memory traffic, and threadgroup memory already have exact, bounded, or conditional analytical answers. For each component, state whether this workload produces a usable observation, a bounded observation, or none; distinguish validating an existing analytical answer from fitting a missing one, and where the workload supplies none, say what would.
- **Require that the candidates stay separately costed.** The prefill decomposition alternatives, the final-position projection, the quantized weight-decode alternatives, and the per-execution variant guard over `S` are four places where collapsing to a presumed winner would be selecting on an unmeasured assumption.
- Preserve the boundary the cost plan already fixes: compile time and artifact size are separate objectives with their own budgets and are never converted into GPU nanoseconds; hard feasibility never enters the cost; and an estimate may never rank two plans that resolved different numerical contracts.
- State what a calibration run may not claim while `ResourcePressure` and `CompileTime` remain `Unknown` and while the other seven components have only their stated exact, bounded, or conditional authority, so that the first measured number does not become a device-optimal claim by proximity.

## Explicit non-goals

No coefficient fitting, no activation of the deferred ticket, and no change to the analytical model. Reopening the calibration ticket belongs to whoever writes the first device measurement, exactly as that ticket states.

## Closes when

The protocol exists, every one of the nine cost components is classified against what this workload supplies, the four separately-costed candidate sets are named, and the calibration ticket's activation paragraph can cite it as its fourth input.
