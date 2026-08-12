---
id: discharge-the-derived-requirements-in-the-candle-metal-adapter
title: Accept or revise derived-requirement discharge in the Candle Metal adapter
status: done
priority: p2
dependencies: []
related: [check-synchronization-realization-before-the-routing-commit, carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit, prototype-candle-metal-adapter, correct-the-reversed-requirement-order-in-the-serial-sum-run-doc-comment]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`prototypes/candle-metal-adapter` refuses, by name and before the routing commit, a route whose entries require a synchronization realization this backend cannot deliver or an index arithmetic this bound device does not establish — two obligations carried by each entry's fixed `ResourceRequirements`, with no separate route-requirement row restating them.

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

Five properties, perturbed separately, each quoted in the worker report:

- deleting the discharge from `prepare_entries` → `error[E0061]: this method takes 2 arguments but 1 argument was supplied`;
- forging the witness at the call site → `error[E0423]: cannot initialize a tuple struct which contains private fields`;
- checking only the first entry's synchronization (`.take(1)`) → `every_entry_s_derived_synchronization_is_checked_and_the_refusal_names_it` fails;
- interleaving the two passes into one → `an_unrealizable_synchronization_outranks_an_undecidable_device_observation` fails, and only that one;
- substituting a fabricated `Apple9` observation for the device's → `an_undecidable_family_refuses_the_index_arithmetic_and_names_the_entry` fails.

## Unmeasured and unsupported

- **Unmeasured:** the entry walk — `entry.entry().resources()` over a real `RoutedEntry` — is exercised only by a device-bound run, which the coordination host cannot perform. `RoutedEntry`'s fields are `pub(super)` in `tiler-runtime`, so no device-free fixture in this crate can construct one. The decision it feeds is fully covered device-free.
- **Unsupported:** no artifact this workspace produces requires a subject Metal declines or an arithmetic outside `CompleteU64`, so the refusing path is unreachable from a real route today. It is a delivery-time guard against a producer that built for a different backend.
- **Not constructible:** `IndexArithmetic` has one variant, so two entries cannot differ in it and the index pass's own per-entry behaviour has no negative fixture. `minimum_gpu_family`'s wildcard-free match is what stops a widened vocabulary reaching this decision unclassified.

## Historical decision-boundary classification — corrected below

When the implementation landed, this ticket said Tom still had to accept or revise the additive `RouteRefusal::{SynchronizationUnrealizable, IndexArithmeticUnsupported}` variants and the witness-bearing `prepare_entries` seam that makes discharge precede pipeline construction. That classification ignored the binary target's private module boundary and is corrected in the dated disposition below.

**Recommendation: accept the draft as built.** The two variants retain the owning typed causes, preserve the device-free-before-device-dependent refusal order, and make deletion or forgery of the discharge step a compile failure. **Strongest counterpoint:** `RouteRefusal` is deliberately exhaustive rather than `#[non_exhaustive]`, so even an additive variant is a breaking promise to downstream matches; Tom may prefer a nested derived-requirement refusal before accepting more top-level variants.

The original packet therefore parked the ticket as `awaiting-decision`. The corrected disposition establishes that no cross-crate public promise existed and closes the already-tested implementation.

## Graph maintenance

- **Corrected outside this ticket.** [`correct-the-reversed-requirement-order-in-the-serial-sum-run-doc-comment`](correct-the-reversed-requirement-order-in-the-serial-sum-run-doc-comment.md) is `done`. `resolve_prepared_route` now states the actual live-device → direct-requirement → pipeline-preparation order and preserves the retired wording only inside its dated correction, so a grep hit is not evidence that the reversed claim remains live.
- **Corrected below.** The earlier packet called the new `RouteRefusal` variants breaking because the enum is deliberately not `#[non_exhaustive]`. That ignored Rust reachability: the enum lives under a private module of a binary-only package, so it has no downstream match surface.

## Corrected disposition — 2026-08-12

Tom accepted the implemented behavior after a current-main, source-first audit, but not the ticket's public-boundary premise. `tiler-prototype-candle` has only a binary target, `main.rs` declares every module privately, and nothing outside the package can name `RouteRefusal`. Its `pub` spelling is therefore module-sharing visibility inside this binary, not a downstream exhaustive-match promise. The `public-boundary` and `needs-tom` tags were removed and this already-delivered ticket is complete.

The user-visible and refusal prose is also imprecise where it says no artifact row carries these requirements. Each entry's fixed artifact record encodes its complete `ResourceRequirements`, including `index_arithmetic` and the synchronization presence/subject. What is deliberately absent is a second `RouteRequirement` or backend-feature row restating those already-derived facts. The implementation correctly reads the verified entry record and compares it through `tiler_metal`'s owning authorities.

The accepted implementation remains unchanged: every entry's synchronization is checked before any entry's index arithmetic; both passes precede pipeline construction; `DirectRequirementsDischarged` remains mandatory at `build_pipelines`; and the two direct, typed `RouteRefusal` variants retain the owning causes. A nested refusal adds no correctness or compatibility value in this private binary. Current-main verification ran `cargo test -p tiler-prototype-candle -- --nocapture`: 18 unit tests and the dependency-direction integration test passed.

Making this prototype's approximately 42 module-sharing `pub` items explicitly `pub(crate)` could prevent accidental exposure if a root module is later published, but that is one coherent visibility-hygiene sweep rather than unfinished work on these two variants. It is not required to close this correctness ticket and must not be inferred as authorization to create a public Candle crate.
