---
id: calibrate-device-cost-models
title: Calibrate analytical costs for selected device profiles
status: deferred
priority: p2
dependencies: [implement-analytical-component-cost-model, emit-analytical-costs-through-the-typed-cost-vocabulary, supply-the-model-level-benchmark-protocol-to-cost-calibration]
related: [supply-the-model-level-benchmark-protocol-to-cost-calibration, define-first-metal-lm-workload]
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

**Only the measurement gate holds this ticket shut.** Both implementation dependencies are **`done`**. Representative kernels, an exact target profile, and a device were later selected; [`supply-the-model-level-benchmark-protocol-to-cost-calibration`](supply-the-model-level-benchmark-protocol-to-cost-calibration.md) remains the live prerequisite for a reproducible workload-owned protocol. There is no raw compiler measurement path to calibrate meanwhile.

**The typed analytical input now exists.** `emit-analytical-costs-through-the-typed-cost-vocabulary` moved every modelled component into `CostAssessment`: exact derivations carry `CheckedInvariant`, modelled bounds carry `Assumption`, units use the matching `Quantity` variant, and the `Reported` disposition states that none of these records entered dominance. Calibration may trust those distinctions rather than parsing `.low`/`.high` suffixes or treating every value as proven.

**Trigger — whichever of these three arrives first.**

1. **A measurement path reaches the compiler.** Anything that times a dispatch on a named device and feeds the number back into an estimate has to construct `EstimateProvenance::Measured`, and the moment that variant has a producer, this ticket owns the fitting, provenance, uncertainty, and drift policy around it rather than leaving a raw number in the model.
2. **A plan choice turns on an `Unknown` component.** Today's two `Unknown`s are constant across candidates, so they cannot mis-rank anything. The first target profile or plan class where resource pressure or compile time genuinely differs between candidates makes the constant a wrong answer rather than an absent one, and the model must be calibrated or must decline the comparison explicitly.
3. **`define-first-metal-lm-workload` selects a workload.** That ticket supplies three of the four activation inputs at once — representative kernels, the exact target profile, and the device — leaving only the benchmark protocol, which is this ticket's own work.

**Who acts.** Whoever writes the first device measurement owns reopening this ticket; it is not a coordinator decision and does not wait for a sweep. **Until it is reopened and closed, nothing may describe the analytical model as calibrated or claim device-optimal latency from it** — the model's own documentation is the current authority on what it does and does not state, and it says two of nine components are `Unknown`.

**Corrected 2026-08-01, with the graph edge added 2026-08-09 — trigger 3 has fired, and the fourth input is another ticket's rather than this one's.** [`define-first-metal-lm-workload`](define-first-metal-lm-workload.md) delivered its selection, so representative kernels, the exact target profile, and the device all exist. Trigger 3's closing clause — "leaving only the benchmark protocol, which is this ticket's own work" — is the part that is now stale: rung L8 of the language-model ladder derived the protocol's shape from the workload's own residency and host discipline, and [`supply-the-model-level-benchmark-protocol-to-cost-calibration`](supply-the-model-level-benchmark-protocol-to-cost-calibration.md) owns writing it, because the protocol is a property of the workload while the fitting, provenance, uncertainty, and drift policy remain properties of the model and stay here. It is now a dependency rather than only a related link, because calibration cannot start without it. This ticket's deferral is otherwise unchanged, and the "who acts" rule above still governs its reopening.

## Trigger check log

- 2026-08-04 — **not fired** (triggers 1 and 2; trigger 3's firing is already recorded above and its residue is [`supply-the-model-level-benchmark-protocol-to-cost-calibration`](supply-the-model-level-benchmark-protocol-to-cost-calibration.md), `todo`). No `EstimateProvenance::Measured` is constructed anywhere — `grep -rn 'EstimateProvenance::Measured' crates/ --include='*.rs'` returns nothing, so no measurement path reaches the compiler. And `ResourcePressure` and `CompileTime` still share one arm evaluating to `CostValue::Unknown` for every plan (`crates/tiler-compiler/src/component_cost.rs:618`), so neither can differ between candidates. Recheck: both greps above.
- 2026-08-06 — **not fired, and trigger 2 acquires its first named firing condition.** Both greps re-run at `95f1ffc7`: `grep -rn 'EstimateProvenance::Measured' crates/` returns nothing, and the shared arm is intact — cite `grep -n 'ResourcePressure | CostComponent::CompileTime' crates/tiler-compiler/src/component_cost.rs` rather than a line number, which has now drifted twice (`567` in the 2026-07-28 note, `618` in the entry above, `619` today). **What is new is a named condition rather than a firing.** [The flash-class capability record](../docs/research/program-planning/flash-class-capability-set.md)'s axis 4 derives that a flash-shaped implementation and the naive one are the first identified candidate pair for which `ResourcePressure` would *not* be constant, because the flash form's advantage is occupancy and traffic rather than dispatch count or temporary bytes. It records two further facts a future claimant should not re-derive: four cost models exist rather than one (`tiler.cost.partition-structural.v1`, `tiler.cost.structural.v1` at two layers, and `tiler.cost.analytical.v1`) and every one is a Pareto partial order, so two implementations of one cover that trade dimensions are incomparable **by construction** and selecting between them is what calibration would buy; and a memory-traffic simulator is eliminated rather than deferred, on the ground that `MemoryTraffic`'s `Bounded` interval is already sound and a simulator would replace an evidence-backed bound with an unfalsifiable model of an undocumented cache hierarchy. **The condition does not fire, because no such candidate pair is enumerable:** a flash-shaped region is refused earlier, at the request boundary and at fusion legality, and that record's axis 2 names both owners.
- 2026-08-09 — **not fired.** `EstimateProvenance::Measured` still has no constructor in `crates/`, and `ResourcePressure | CompileTime` still share the one `CostValue::Unknown` arm. The workload/device/profile half of trigger 3 is complete, but `supply-the-model-level-benchmark-protocol-to-cost-calibration` remains `todo`; no reproducible protocol feeds a measured row into this model. The flash-shaped candidate pair remains unenumerable, so trigger 2 has a named future subject but no competing plans to rank.
