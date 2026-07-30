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

A pure policy and a platform observer independently establish whether the current macOS host may offer the exact authoritative Metal `TargetProfileRef`. The runtime cannot earn eligibility from a `Compilation`, an artifact under validation, or equality with producer-owned bytes.

## Facts and measurement boundary

**Fact:** the prototype currently builds `ExecutionEnvironment.target_profile` directly from `Compilation.target_profile_key()` and `target_profile_descriptor()`. This proves equality with the compiler declaration and says nothing about whether the live host satisfies its applicability predicates.

**Fact:** the intended first profile is bounded by Apple9 hardware, exact macOS version/build and architecture, and compiler/environment facts established by the unified MSL 4 measurement. The offline compiler belongs to artifact provenance; the runtime/pipeline compiler belongs to the execution environment and moves with the OS.

**Inference:** reading a profile reference from the artifact or rebuilding it from a local compilation is a tautology. Device name alone is insufficient; registry ID is not a stable hardware identity and differs across retained runs on the same named M4 Max.

**Measurement boundary:** this ticket consumes the exact predicates produced by `measure-macos-apple9-f32-under-unified-msl4-profile`. It does not broaden them to another OS build, Apple family, compiler build, physical iOS device, or dtype.

## Implementation keys

Define a deterministic pure applicability policy over normalized observations and a platform adapter that observes OS family/version/build, architecture, supported GPU family, stable hardware class, and the runtime compiler/environment identities the measurement requires. The policy returns an exact eligible profile reference or a typed reason for refusal. Keep observation separate from decision so all positive and negative cases run on non-Apple CI. Offer the reference only after policy success, before routing, and preserve live-device and prepared-pipeline checks as distinct later obligations.

## Required evidence

Tests must accept the exact qualified observation and reject wrong platform, architecture, Apple family, OS version, OS build, compiler build, missing observations, key mismatch, and same-key/different-descriptor mismatch. A test must prove neither `Compilation` nor decoded artifact bytes are inputs to eligibility. An eligible-host integration run must offer the exact measured reference; an unavailable host must report the precise predicate it cannot satisfy.

## Closes when

The runner no longer calls `host_environment(&Compilation)`, the pure policy and observer independently earn the profile reference from exact measured predicates, runtime compiler provenance remains environment-owned, all refusals occur before routing commit, and focused tests plus `make check` pass.

## Graph maintenance

This ticket depends on `measure-macos-apple9-f32-under-unified-msl4-profile` and blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep `record-metal-runtime-compiler-provenance-gap`, `prototype-metal-runtime-proof`, and `restore-replayable-apple-compatibility-evidence` related; they establish adjacent provenance and preflight contracts but do not implement host eligibility.
