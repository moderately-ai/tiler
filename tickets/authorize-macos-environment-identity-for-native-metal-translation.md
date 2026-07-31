---
id: authorize-macos-environment-identity-for-native-metal-translation
title: Authorize macOS environment identity for native Metal translation
status: todo
priority: p0
dependencies: [prove-an-aot-compatible-metal-runtime-compiler-observer]
related: [validate-macos-metal-profile-host-applicability]
scopes: [contracts/artifacts, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The first native Metal AOT profile has an explicit, accepted applicability authority for device translation performed during pipeline creation even though the private translator/compiler build cannot be attributed. The policy either qualifies the exact retained execution-environment row without pretending that row is a compiler identity, or remains unavailable and blocks the profile.

## Facts and eliminated shortcuts

**Fact:** Apple describes a metallib as GPU-independent Metal IR and pipeline creation as the stage that may translate it to device-specific machine code. ADR 0043 preserves this required device translation inside the AOT boundary; “AOT” does not mean the offline metallib is final machine code.

**Measurement:** `prove-an-aot-compatible-metal-runtime-compiler-observer` found two GPUCompiler-related dyld images already present before the native route and no image-population change through library, function, or pipeline preparation. Direct scans of their dyld-cache paths were unavailable. Loaded-image membership and deltas therefore cannot attribute the private translator/compiler build.

**Eliminated:** `metalfe-32023.921` identifies the separate `newLibraryWithSource` comparison path and is not evidence for native preparation. Xcode, the offline compiler, OS build, a framework on disk, a loaded image, registry ID, or a hard-coded path cannot be relabelled as the private compiler identity. Each shortcut would silently certify a relationship its source does not establish.

## Decision needed

Whether a bounded native execution measurement may use its exact OS family/version/build, architecture, reported device name, and Apple-family support as the validity scope for pipeline translation while the private translator/compiler identity remains explicitly `Unknown`.

### Option 1 — authorize exact execution-environment scope (recommended)

The applicability receipt names the exact measured macOS build, architecture, device name, and Apple family. It states that private translator/compiler identity is unavailable and makes no compiler-build claim. This enables the first profile only on the retained native-execution row and requires a new measurement for any changed row. It does not provide cryptographic host attestation or authorize inference to another OS/device row.

**Point:** the numerical result was measured end to end through native metallib pipeline preparation on exactly that environment. Qualifying the observation by the environment that delivered it is truthful even when an internal implementation component is opaque, and it follows the repository's bounded-measurement evidence class without inventing provenance.

**Counterpoint:** two hosts reporting the same public row are assumed equivalent for this bounded policy even though the private component cannot be compared directly. A modified system volume or an Apple servicing change that preserves every observed public field would escape the predicate; closing that threat would require stronger host attestation, not a compiler-name guess.

### Option 2 — require attributable private translation identity

No positive applicability receipt exists until Apple exposes a stable responsible-component identity or a separately accepted observer can attribute it. This prevents any silent equivalence assumption and blocks the first authoritative profile indefinitely on current APIs.

**Point:** it preserves component-exact provenance and refuses every host whose translator cannot be named.

**Counterpoint:** it demands a stronger identity than the direct end-to-end measurement itself needed and provides no known path to implementation. It also does not solve host authenticity; a named component still needs trustworthy observation.

## Recommendation

Choose Option 1. Correctness comes from the direct native result and its deliberately narrow validity scope, not from guessing the private component's name. Performance is unaffected because the policy reads only preflight host/device facts and performs no source JIT or binary scanning. Long-term maintainability is stronger because the distinction is explicit: offline compiler provenance remains artifact-owned, private native translation identity remains `Unknown`, and a changed public execution row requires fresh evidence. Reconsider if Apple publishes a stable translator identity API, if the same reported row produces a numerical or compatibility divergence, or if Tiler adopts a host-attestation threat model.

## Required evidence

- Primary Apple documentation and the local contract agree that pipeline creation may perform device translation of metallib IR.
- The AOT observer's exact negative result and scan-unavailability boundary are cited without turning absence of attribution into absence of consumption.
- The accepted policy says what it authenticates and what it does not; source-JIT compiler identity, offline compiler identity, loaded images, and registry ID cannot satisfy the host predicate.
- The dependent host-applicability ticket carries the chosen authority and a typed refusal for every missing or mismatched public environment predicate.

## Closes when

Tom selects one surviving authority, the Metal and numerical provenance contracts record it as a Proposal or accepted decision at the correct status, the host-applicability ticket depends on and incorporates it, and `tkt lint` plus `git diff --check` pass. If Option 2 is selected, the ticket closes with the first profile explicitly blocked on a newly named observable/authority rather than with a fabricated implementation task.

## Graph maintenance

- Block `validate-macos-metal-profile-host-applicability` and therefore `construct-and-bind-the-first-authoritative-metal-compile-profile` until this decision is recorded.
- Keep `prove-an-aot-compatible-metal-runtime-compiler-observer`, `record-metal-runtime-compiler-provenance-gap`, and `measure-macos-apple9-f32-under-unified-msl4-profile` related as the evidence chain.
- If Option 1 is accepted, the host policy owns exact environment matching while artifact identity continues to own the offline compiler and linker.
- If Option 2 is accepted, file only a bounded observer/authority ticket with a concrete new evidence source; do not create an open-ended search for private symbols.
