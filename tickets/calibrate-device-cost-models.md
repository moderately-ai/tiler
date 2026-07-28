---
id: calibrate-device-cost-models
title: Calibrate analytical costs for selected device profiles
status: deferred
priority: p2
dependencies: [implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler, research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, measurement, cost-model, deferred]
---
Activate only after representative kernels, exact target profiles, devices,
and a reproducible benchmark protocol are selected. Fit and validate component
parameters with held-out measurements, provenance, uncertainty, drift policy,
and an explicit activation threshold. Until then the analytical model remains
uncalibrated and must not claim device-optimal latency.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## What the source has assigned this ticket (2026-07-28)

**Two sites in the compiler name this ticket as the owner of work they decline to do**, so it is load-bearing while deferred rather than merely queued.

- `crates/tiler-compiler/src/component_cost.rs:42` — of the two components the analytical model leaves unmodelled, compile time "is a measurement rather than an analysis and belongs to `calibrate-device-cost-models`".
- `crates/tiler-compiler/src/estimate.rs:59` — `EstimateProvenance::Measured` is documented as reserved, "no measurement path reaches this module yet", and "`calibrate-device-cost-models` owns device measurement and activation".

**What the model reports meanwhile is honest and checkable in one line.** `grep -n 'CostComponent::ResourcePressure | CostComponent::CompileTime' crates/tiler-compiler/src/component_cost.rs` prints `567`, the single arm where both components evaluate to `CostValue::Unknown` for every plan. That is the shape the repository's evidence rules require — an unmeasured component is `Unknown`, not a fabricated number — and it also means the two components cannot currently break a tie between candidates, because they are constant across them.

**Only prose holds this ticket shut.** Its one dependency, `implement-analytical-component-cost-model`, is **`done`**. There is no unmet graph edge; what keeps it deferred is the activation paragraph above — representative kernels, exact target profiles, devices, and a reproducible benchmark protocol — none of which exists yet.

**Trigger — whichever of these three arrives first.**

1. **A measurement path reaches the compiler.** Anything that times a dispatch on a named device and feeds the number back into an estimate has to construct `EstimateProvenance::Measured`, and the moment that variant has a producer, this ticket owns the fitting, provenance, uncertainty, and drift policy around it rather than leaving a raw number in the model.
2. **A plan choice turns on an `Unknown` component.** Today's two `Unknown`s are constant across candidates, so they cannot mis-rank anything. The first target profile or plan class where resource pressure or compile time genuinely differs between candidates makes the constant a wrong answer rather than an absent one, and the model must be calibrated or must decline the comparison explicitly.
3. **`define-first-metal-lm-workload` selects a workload.** That ticket supplies three of the four activation inputs at once — representative kernels, the exact target profile, and the device — leaving only the benchmark protocol, which is this ticket's own work.

**Who acts.** Whoever writes the first device measurement owns reopening this ticket; it is not a coordinator decision and does not wait for a sweep. **Until it is reopened and closed, nothing may describe the analytical model as calibrated or claim device-optimal latency from it** — the model's own documentation is the current authority on what it does and does not state, and it says two of nine components are `Unknown`.
