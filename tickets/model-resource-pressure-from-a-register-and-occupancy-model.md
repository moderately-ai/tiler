---
id: model-resource-pressure-from-a-register-and-occupancy-model
title: Model resource pressure once a register and occupancy model exists
status: deferred
priority: p3
dependencies: []
related: [implement-opaque-physical-call-providers, calibrate-device-cost-models]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, target-profiles]
---
Split from `model-the-eight-unmodelled-cost-components`, which modelled seven of nine components and reached its floor. `ResourcePressure` is the one component with a genuine, checked blocker.

## Why it is blocked rather than merely unwritten

`CostComponent::ResourcePressure` means "register and threadgroup-memory pressure, and the occupancy it implies". Threadgroup memory was split into its own exact component and is done. What remains is **registers per thread** and the **occupancy model** that would combine the two into pressure. **A register-and-occupancy vocabulary now exists, and it is the wrong class of evidence for this component.** `PressureDimension` (`crates/tiler-compiler/src/estimate.rs:96-103`) declares `Registers` (`:98`), `Occupancy` (`:100`), and `SourceBytes` (`:102`) — so the names are no longer absent, and the earlier claim that "neither exists anywhere in the compiler" is no longer the accurate statement of the blockage. What is absent is what the component actually needs: a *target profile* declaring register-per-thread and occupancy axes, and a model deriving pressure from them. An estimate carrying a register count does not tell a cost model what the device's limit is, and `crate::estimate`'s module header (`estimate.rs:22-27`) records why the two cannot be bridged by conversion — an estimate is for ranking and reporting, a requirement is for deciding, and the absence of any `TryFrom` between them "is the enforcement".

*The check, stated so it can be reproduced or refuted in one line:* `ResourceRequirements` carries buffer bindings, threads per workgroup, local memory, device-memory use, and four numerical fields — no register count. And `rg -n "ResourceEstimate|PressureDimension|EstimateProvenance" crates -g '*.rs' -g '!crates/tiler-compiler/src/estimate.rs'` finds only references to the estimate vocabulary rather than a target register-limit declaration. So the estimate vocabulary supplies no device limit this component could compare against even when a producer supplies a demand. The current target capability census names buffer bindings, device-memory availability, grid extent, operation-complete unsigned-64 index arithmetic, local-memory bytes, threads per workgroup, and the numerical dimensions; synchronization and device-address width are absent for current regions, and there is no register axis.

This is a missing **model**, not a missing summary. That distinction earned its emphasis: six of the nine original "unreachable" notes on the parent ticket turned out to be missing summaries that were in fact one read away, and this one was re-checked twice before being written down.

## The constraint that must survive

Do not repair this by widening `CostComponent::unit`. Reporting some other quantity under a `Registers` unit would be a unit lie, and units here are contract rather than documentation — an uncalibrated model whose numbers have no true stated unit cannot be calibrated, because nothing says what the device measurement should be compared against. A missing number is recoverable; a number in the wrong unit is what a calibration pass silently trusts. That reasoning is why threadgroup memory became its own component instead of being folded in here.

## Closes when

- A target profile declares register-per-thread and occupancy axes, typed like the existing capability axes.
- `ResourcePressure` is computed from them, in `Registers`, with `Bounded` rather than `Exact` unless the derivation is genuinely exact.
- The explain census in `pipeline/tests.rs` is updated in the same change; its `tiler.cost.analytical.v1` count grows as components become modelled, and that test is what catches an unreported one.
- The retained plan set and the selected plan are unchanged, as for every other component: nothing here enters dominance.

## Trigger for reconsideration

Any work that adds register or occupancy information to a **target profile**. `implement-opaque-physical-call-providers` was the nearest candidate — its body calls for "uncertain `ResourceEstimate`-class pressure estimates with provenance and an explicit `Unknown` state, including registers, occupancy, and source size", which is the same vocabulary this component needs.

## Trigger fired 2026-07-28; still deferred, and here is why

**The trigger fired.** `implement-opaque-physical-call-providers` is `status: done` and landed exactly the estimate class the trigger named: `PressureDimension::{Registers, Occupancy, SourceBytes}`, with provenance and an explicit `Unknown` that is deliberately not zero.

**The check the trigger asks for was run, and the answer is no.** This does not become computable from it, for two independent reasons either of which suffices. First, the estimate is the wrong evidence class: `crate::estimate`'s header states there is deliberately no conversion from a `ResourceEstimate` into anything feasibility or a requirement consults, and that the absence is the enforcement rather than an omission — so a pressure model built on an estimate would either lie about its class or route around a decided boundary. Second, and more simply, **an estimate states a demand and this component needs a limit**. A call reporting 48 registers per thread says nothing about pressure until a target profile says how many the device has. No profile declares that axis.

**The parent ticket reached the identical conclusion and it never reached here.** `implement-opaque-physical-call-providers.md:64` says in terms: "that ticket is deferred waiting for exactly this vocabulary. `PressureDimension::Registers` and `Occupancy` now exist. What is still missing there is a *target profile* declaring the axes — an estimate carrying a register count does not tell the cost model what the device's limit is. Check before assuming it is unblocked." Recording it on the parent changed nothing a reader of *this* ticket consults, which is why it is restated here rather than cross-referenced.

**Next trigger, narrowed so it cannot fire spuriously.** Reopen this when a **target profile** gains a register-per-thread or occupancy axis — that is, when `PrototypeTargetProfile` or its successor declares a device *limit* on either dimension, typed like the existing quantitative bounds. Whoever lands that change owns reopening this ticket. **A `crate::estimate` producer landing is explicitly *not* the trigger**: that would give the compiler more demands to report and still no limit to compare them against, and treating it as the trigger is the mistake this section exists to prevent from being made a second time.
