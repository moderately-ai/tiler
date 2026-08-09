---
id: declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles
title: Declare the BF16 iOS family answers on authoritative iOS profiles
status: blocked
priority: p2
dependencies: [first-authoritative-ios-metal-compile-declaration]
related: [declare-the-bf16-rows-on-the-authoritative-metal-profile, measure-apple-numerics-on-physical-ios-device, measure-macos-apple9-bf16-under-unified-msl4-profile]
scopes: [implementation/build, implementation/metal]
shared_scopes: [project/tickets]
tags: [implementation, dtype, bf16, target-profiles, metal, apple-targets, ios]
paths: []
---
## User-visible outcome

The authoritative iOS-Simulator profile states that BF16 is `Unsupported`, with the exact measured diagnostic, and the iOS-device profile states nothing about BF16 at all — which resolves `Unknown`, because nobody asked. A BF16 program routed at an iOS family is then refused before the routing commit rather than failing at pipeline creation after it.

## Why this is a separate ticket

Split out of [`declare-the-bf16-rows-on-the-authoritative-metal-profile`](declare-the-bf16-rows-on-the-authoritative-metal-profile.md) at integration on 2026-08-02. That ticket's evidence bar required a three-family matrix — macOS `Dispatchable`, simulator `Unsupported`, device `Unknown` — plus `f32` `Dispatchable` on all three so that no refusal sits on a dead profile.

**Fact — neither iOS profile exists, and its prerequisite is parked.** [`first-authoritative-ios-metal-compile-declaration`](first-authoritative-ios-metal-compile-declaration.md) is `deferred`, because Tom deprioritized iOS on 2026-08-01. A `deferred` dependency satisfies no dependent, so a ticket requiring the three-family matrix could never reach `ready` regardless of what BF16 evidence arrived. Keeping the macOS half hostage to that is the permanent-unreachability shape [`re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed`](re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed.md) was filed to repair elsewhere in this graph, so the halves were separated instead.

**Until this lands, both iOS families answer `Unknown` by absence, and that is correct rather than a gap.** With no iOS profile at all there is no row to read, and `Unknown` is precisely what a profile that has not spoken should resolve. Nothing is silently claiming iOS support in the meantime.

## The measured evidence this ticket transcribes, and the problem it inherits

**Measurement, from `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv`** — Apple M4 Max, macOS 27.0 build 26A5388g, Metal 32023.883, Xcode 26.6. The iOS Simulator compiles and links every `bfloat` module and then fails pipeline creation with `XPC_ERROR_CONNECTION_INTERRUPTED`. The arithmetic-free `materialize_bf16` is refused too, so the refusal is about the **format**, not one operation. The iOS device was never asked.

**Fact — that record is not admissible against an MSL 4.0 profile, and this ticket inherits the same obstacle the macOS half hit.** Its `probe.fixed_flags` is `-std=metal3.1` and its simulator target is `air64-apple-ios16.0-simulator`. `TargetCompileProfileMeasurementSource` holds compiler builds and an execution environment but carries **neither the language standard nor the target triple**, so a row sourced from it would be indistinguishable from one measured under the profile's own compilation. Whatever authoritative iOS declaration lands must therefore either be measured under its own standard, or [`record-the-compilation-selection-in-target-measurement-provenance`](record-the-compilation-selection-in-target-measurement-provenance.md) must land first so a second source can be labelled honestly. Do not transcribe the MSL 3.1 simulator row into an MSL 4.0-sourced profile.

## Required evidence

- BF16 resolves `Unsupported` on the authoritative iOS-Simulator profile at `AvailabilityPhase::CompileProfile`, from a measured source carrying the exact diagnostic.
- BF16 resolves `Unknown` on the iOS-device profile — **absent, not `Unsupported`.** The distinction is the whole point: nobody asked the device.
- `f32` resolves `Dispatchable` on both iOS profiles, so neither refusal sits on a dead profile.
- `f16` still resolves `Unknown` on both, so a measured BF16 row did not fill a neighbour's omission.
- Each answer watched failing under perturbation, not merely asserted.

## Explicit non-goals

The macOS rows, which the parent ticket owns. Re-measuring — [`measure-apple-numerics-on-physical-ios-device`](measure-apple-numerics-on-physical-ios-device.md) is the only route to closing the device `Unknown` and stays `deferred`; it must **not** become a dependency, because `deferred` never satisfies a dependent and this ticket must not inherit a second permanent block.

## Closes when

Both iOS families carry their answers on authoritative profiles, the device row is absent rather than negative, the source's language standard and target triple are attributable rather than assumed, every refusal has been watched failing, and any moved identity is enumerated.

## Graph maintenance

- A differing physical-iOS measurement would reopen `declare-metal-numerical-honourability`; say so rather than assuming the family agrees with macOS.
- If `record-the-compilation-selection-in-target-measurement-provenance` lands first, re-read whether an honestly-labelled MSL 3.1 source becomes admissible here — that changes this ticket's evidence route, not its outcome.
