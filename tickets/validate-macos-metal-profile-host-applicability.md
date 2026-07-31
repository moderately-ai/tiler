---
id: validate-macos-metal-profile-host-applicability
title: Validate macOS Metal profile host applicability independently
status: todo
priority: p0
dependencies: [measure-macos-apple9-f32-under-unified-msl4-profile]
related: [record-metal-runtime-compiler-provenance-gap, prototype-metal-runtime-proof, restore-replayable-apple-compatibility-evidence]
scopes: [implementation/build, implementation/runtime, implementation/metal, contracts/artifacts, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, target-profiles, provenance]
---
## User-visible outcome

A pure policy and a platform observer independently establish whether the current macOS host satisfies the measured applicability predicates required by the first Metal profile. The result is a checked eligibility receipt; the dependent profile-construction ticket binds that receipt to the exact `TargetProfileRef` it owns. The runtime cannot earn eligibility from a `Compilation`, an artifact under validation, or equality with producer-owned bytes.

## Facts and measurement boundary

**Fact:** the prototype currently builds `ExecutionEnvironment.target_profile` directly from `Compilation.target_profile_key()` and `target_profile_descriptor()`. This proves equality with the compiler declaration and says nothing about whether the live host satisfies its applicability predicates.

**Fact:** the intended first profile is bounded by Apple9 hardware, exact macOS version/build and architecture, and compiler/environment facts established by the unified MSL 4 measurement. The offline compiler belongs to artifact provenance; the runtime/pipeline compiler belongs to the execution environment and moves with the OS.

**Inference:** reading a profile reference from the artifact or rebuilding it from a local compilation is a tautology. Device name alone is insufficient; registry ID is not a stable hardware identity and differs across retained runs on the same named M4 Max.

**Measurement boundary:** this ticket consumes the exact predicates produced by `measure-macos-apple9-f32-under-unified-msl4-profile`. It does not broaden them to another OS build, Apple family, compiler build, physical iOS device, or dtype.

## Implementation keys

Define a deterministic pure applicability policy over normalized observations and a platform adapter that observes only predicates established by the retained measurement: OS family/version/build, architecture, exact reported device name, supported GPU family, and runtime compiler/environment identity. Registry ID is correlation evidence and no unmeasured “stable hardware class” may be invented.

The policy returns a non-forgeable eligibility receipt scoped to its versioned policy and exact normalized observation, or a typed reason for refusal. It does not return or contain a target-profile key or descriptor, because `construct-and-bind-the-first-authoritative-metal-compile-profile` owns that declaration and currently depends on this ticket. The parent consumes a successful receipt and binds it to the exact profile it constructs; this removes the circular requirement for a dependency to return a value its dependent creates.

Keep observation separate from decision so all positive and negative cases run on non-Apple CI. Keep Metal device observation out of device-free `tiler-runtime`; the current prototype may host the first adapter, while any reusable Metal runtime adapter requires its own reviewed ownership boundary. Preserve live-device and prepared-pipeline checks as distinct later obligations.

## Required evidence

Tests must accept the exact qualified observation and reject wrong platform, architecture, Apple family, device name, OS version, OS build, runtime compiler build, and missing observations. A test must prove neither `Compilation`, `TargetProfileRef`, nor decoded artifact bytes are inputs to eligibility. The parent ticket owns key mismatch, same-key/different-descriptor mismatch, and the final eligible-host offer of the exact measured reference. An unavailable host must report the precise predicate it cannot satisfy.

## Closes when

The pure policy and observer independently produce a checked policy-scoped eligibility receipt from exact measured predicates; no profile reference or producer-owned bytes enter that decision; runtime compiler provenance remains environment-owned; all refusals occur before routing commit; the parent has an explicit typed input it can bind to its profile; and focused tests plus `make check` pass. Removing `host_environment(&Compilation)` and offering the final exact profile remain parent integration work rather than a circular closing condition here.

## Graph maintenance

This ticket depends on `measure-macos-apple9-f32-under-unified-msl4-profile` and blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep `record-metal-runtime-compiler-provenance-gap`, `prototype-metal-runtime-proof`, and `restore-replayable-apple-compatibility-evidence` related; they establish adjacent provenance and preflight contracts but do not implement host eligibility.
