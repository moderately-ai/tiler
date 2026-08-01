---
id: first-authoritative-ios-metal-compile-declaration
title: Measure and bind the first authoritative iOS Metal compile declaration
status: deferred
priority: p2
dependencies: []
related: [measure-apple-numerics-on-physical-ios-device, deliver-several-artifact-families-from-one-expansion, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, measurement]
---
`tiler_build::BoundMetalCompileDeclaration` publishes one constructor, `first_macos_apple9`, and its own documentation states the rule this ticket exists to satisfy: "Widening this to another Apple family, OS row, or dtype is a new measurement rather than a new argument." `deliver ios;` and `deliver macos-and-ios;` are refused by `tiler_macros::aot::require_buildable` for exactly that reason, and the refusal now names this ticket.

## Deferral (2026-08-01)

Tom deprioritized iOS: the target devices are Metal on macOS and CPU. This ticket reactivates on the first consumer asking for an iOS artifact — until then the v13 envelope carrying one-position artifacts in practice is the accepted state, and the `#[cfg(test)]` second-family fixture remains the multi-payload exercise route.

## Why a second declaration is not constructible from the retained corpus

**Fact — the record the ledger reads has no iOS row at all.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:51` names its source as `spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`. Every case key in it is `case.macos.*` and every environment key is `environment.family.macos.*`:

```sh
R=spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv
cut -f1 "$R" | grep -o '^case\.[a-z0-9_]*' | sort -u          # -> case.macos
cut -f1 "$R" | grep -c ios                                    # -> 0
```

**Fact — the ledger refuses the inheritance move by name.** Line 133, on the F32 dispatchability row: "**Inheritance is refused in every direction.** … No iOS family, physical or simulated, gains a row from this one." That row and the seven numerical rows are the ones a declaration cannot omit: with them absent the profile resolves `Unknown`, and `NumericalContract::FlushSubnormalsToZeroF32` — the one contract `tiler_macros::aot` derives — has nothing to be honoured against.

**Fact — the iOS rows that do exist are the superseded MSL 3.1 record.** `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` carries `case.ios-device.*` and `case.ios-simulator.*`, at `probe.fixed_flags -std=metal3.1` against `air64-apple-ios16.0`. `crates/tiler-build/src/metal_declaration.rs` refuses that record for the macOS profile in the same words, and `the_declaration_does_not_carry_the_superseded_msl_3_1_record` asserts it absent: "the older MSL 3.1 / macOS 14.0 record would attribute these measurements to a compilation that did not produce them." `tiler_macros::delivery::PROFILE_MSL_VERSION` is `Metal4_0`, so an MSL 3.1-sourced declaration could not serve `deliver ios;` even if it were admissible.

**Fact — `IOsDevice` has no execution-side row at all, and `IOsSimulator`'s is the Mac's GPU.** `environment.family.ios-device.execution` reads `unavailable:no iOS device is attached to this host`. `docs/research/apple-targets/numerical-behaviour.md:219` records that the simulator's `registryID` equals the Mac's and states the consequence: "the simulator result is admissible as a simulator measurement and not as an iOS-device one". `deliver ios;` selects **both** iOS families (`DeliveredFamily::IOs::platforms()`), so the device half is required and is hardware-blocked.

**Inference — what is transferable is not enough.** Three quantitative rows are scoped to the Apple9 *GPU* family rather than to the artifact family — buffer bindings 31, local memory 32,768, `64-bit integer math` — and the device-address-space row is scoped to MSL 4.0. Those would carry. The grid-axis row is sourced from the **macOS** 26.5 SDK header and would need the `iphoneos` SDK's own. But the measured dispatchability and numerical rows are the ones that decide whether the program compiles at all, and none of them transfers.

## Closes when

An MSL 4.0 measurement exists for the iOS families under their own SDK and target triples, a second `BoundMetalCompileDeclaration` constructor is assembled from exactly those rows and no others, and the ledger gains — or is joined by — a record that names each new row's authority and validity scope with the same discipline as the macOS one. The `IOsDevice` device side is the hard part and is owned by `measure-apple-numerics-on-physical-ios-device` (deferred, hardware-blocked); an `IOsSimulator`-only declaration is a coherent narrower outcome, but it does not unblock `deliver ios;`, which requires both families.

**Do not** widen the declaration by adding an argument, by reusing the macOS measurement source under an iOS platform, or by treating compile-side byte-identity across families as a numerical measurement. `crates/tiler-build/src/metal_plan.rs`'s `second_artifact_family_fixture` is `#[cfg(test)]` precisely so that the shape of such a declaration can be exercised without any of those three becoming reachable.
