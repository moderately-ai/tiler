---
id: discharge-the-derived-requirements-in-the-candle-metal-adapter
title: Discharge the derived requirements in the candle metal adapter
status: awaiting-decision
priority: p2
dependencies: []
related: [check-synchronization-realization-before-the-routing-commit, carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit, prototype-candle-metal-adapter]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786187249
---
## User-visible outcome

`prototypes/candle-metal-adapter` refuses, by name and before the routing commit, a route whose entries require a synchronization realization this backend cannot deliver or an index arithmetic this bound device does not establish — the two obligations the verified program derives and no artifact row carries.

## Fact audit

**The ticket was dispatched with an empty body: frontmatter and nothing else.** There were no Facts to verify. What follows audits the claims in the dispatch brief instead, each re-read in full at base `1438c8670d090372dec20a9859ea5ade2e0db287`. Citations are searchable anchors.

**Fact (verified) — this adapter discharged no derived requirement.** `grep -rn "resources()\|synchronization\|index_arithmetic\|direct_requirement\|Discharg" prototypes/candle-metal-adapter/` returned nothing at that base. Neither `evaluate_synchronization` nor `evaluate_index_arithmetic` was reachable from any route the adapter took, and `RuntimeAdapter::prepare_entries` went straight to pipeline creation.

**Fact (verified) — `tiler_metal` owns both comparisons and both are reachable.** `pub fn evaluate_synchronization` in `crates/tiler-metal/src/synchronization_requirement.rs` takes `Option<SynchronizationSubject>` alone; `pub fn evaluate_index_arithmetic` in `crates/tiler-metal/src/direct_requirement.rs` takes the arithmetic and an `Option<MetalGpuFamilySupport>`.

**Fact (verified) — the witness pattern is what the brief described.** In `prototypes/serial-sum-run/src/proof.rs`, `pub(super) struct DirectRequirementsDischarged(())` lives in `mod discharge` whose only function is `check_direct_requirements`, and `fn prepare_pipelines` takes one by value as `_discharged`. Deleting the check is a compile error.

**Fact (FALSE as stated) — "one stage earlier than a live-device resolution".** The brief and its parent ticket (`check-synchronization-realization-before-the-routing-commit`, "which `resolve_prepared_route` already runs *one stage earlier* than the live-device requirement resolution") both have the stage order backwards. `fn resolve_prepared_route` calls `qualify_live_device` — which is `resolve_live_device_requirements` — **first**, then `check_direct_requirements`, then `prepare_pipelines`, then `resolve_target_properties`. The discharge stage is one stage earlier than *pipeline preparation*, not than the live-device rows.

The correction does not change what to build: `route_with_adapter` calls `prepare_entries` after `resolve_live_device_requirements` too, so placing the discharge at the head of `prepare_entries` reproduces the landed order exactly. It does change what may be *claimed* about it.

**Fact (imprecise) — "because the answer needs no device".** True of `evaluate_synchronization` and false of `evaluate_index_arithmetic`, which takes an Apple-family observation. Only the synchronization half is device-free, which is why the two are separate passes here rather than one.

**Fact (verified) — the name collision is real.** `tiler.metal.route-requirement.minimum-gpu-family` is a `RouteRequirement::BackendFeature` key this adapter answers in `observe_live_device`. `tiler_metal::direct_requirement::minimum_gpu_family` maps an `IndexArithmetic` to an `AppleFamilyFloor`, in a module headed `# Why this is not a route requirement`. Different obligations, answered from the same device observation.

**Fact (verified) — `prototypes/` is excluded from the style gate only.** The `Makefile`'s `lint` target passes `--exclude tiler-prototype-candle`; `build` (`cargo check --workspace --all-targets`) and `test` (`cargo nextest run --workspace`) do not. A Clippy or dead-code warning here is invisible to `make full`; a compile error and a failing test are not.

## What was built

`RuntimeAdapter::prepare_entries` now runs `discharge::check_direct_requirements` over every routed entry before `CandleMetalAdapter::build_pipelines`, which takes the resulting `DirectRequirementsDischarged` by value. The decision itself is `derived_requirements_hold`, split from the route the same way `binding_fits` is split from the device: the walk supplies the population, the decision supplies the verdict, and the gate watches every refusal fail without hardware and without an artifact.

**Two passes, not one.** Every entry's synchronization is checked before any entry's index arithmetic. A subject Metal has no construct for is refused before the Apple-family observation is consulted, because no device change repairs it and reporting the device-dependent refusal first would send a reader to change hardware for a program no Metal device runs.

**The observation is not an `Option` here.** `MetalIndexArithmeticRefusal::Unobserved` is reachable in `prototypes/serial-sum-run` because the `metal` 0.33 binding cannot name every governed enumerator. `objc2-metal` models `MTLGPUFamily` as a newtype over `NSInteger`, so `observed_apple_family` asks about every family and there is no unasked case. The signature states that instead of leaving a bare `Some` at the call site.

Two `RouteRefusal` variants carry the outcome — `SynchronizationUnrealizable` and `IndexArithmeticUnsupported` — each holding the owning comparison's cause whole rather than flattened, under a new `candle-metal.derived-requirement` stage prefix.

## Evidence

Four properties, perturbed separately, each quoted in the worker report:

- deleting the discharge from `prepare_entries` → `error[E0061]: this method takes 2 arguments but 1 argument was supplied`;
- forging the witness at the call site → `error[E0423]: cannot initialize a tuple struct which contains private fields`;
- checking only the first entry's synchronization (`.take(1)`) → `every_entry_s_derived_synchronization_is_checked_and_the_refusal_names_it` fails;
- interleaving the two passes into one → `an_unrealizable_synchronization_outranks_an_undecidable_device_observation` fails, and only that one;
- substituting a fabricated `Apple9` observation for the device's → `an_undecidable_family_refuses_the_index_arithmetic_and_names_the_entry` fails.

## Unmeasured and unsupported

- **Unmeasured:** the entry walk — `entry.entry().resources()` over a real `RoutedEntry` — is exercised only by a device-bound run, which the coordination host cannot perform. `RoutedEntry`'s fields are `pub(super)` in `tiler-runtime`, so no device-free fixture in this crate can construct one. The decision it feeds is fully covered device-free.
- **Unsupported:** no artifact this workspace produces requires a subject Metal declines or an arithmetic outside `CompleteU64`, so the refusing path is unreachable from a real route today. It is a delivery-time guard against a producer that built for a different backend.
- **Not constructible:** `IndexArithmetic` has one variant, so two entries cannot differ in it and the index pass's own per-entry behaviour has no negative fixture. `minimum_gpu_family`'s wildcard-free match is what stops a widened vocabulary reaching this decision unclassified.

## Graph maintenance

- **Out of scope, needs a ticket:** `resolve_prepared_route`'s doc comment in `prototypes/serial-sum-run/src/proof.rs` states "the requirements the verified program itself derived, which need only the bound device; then the live-device rows the artifact carried" — the reverse of what the function does. That file is `implementation/runtime`, which this ticket does not hold. It is the source of the false Fact audited above and should be corrected there.
- The new `RouteRefusal` variants are an additive-to-the-crate but **breaking** change to an enum deliberately not `#[non_exhaustive]`; nothing outside `refusal.rs` matches it exhaustively, so no other site changed.
