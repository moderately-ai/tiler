---
id: validate-macos-metal-profile-host-applicability
title: Validate macOS Metal profile host applicability independently
status: in-progress
priority: p0
dependencies: [measure-macos-apple9-f32-under-unified-msl4-profile, prove-an-aot-compatible-metal-runtime-compiler-observer, authorize-macos-environment-identity-for-native-metal-translation]
related: [record-metal-runtime-compiler-provenance-gap, prototype-metal-runtime-proof, restore-replayable-apple-compatibility-evidence]
scopes: [implementation/build, implementation/runtime, implementation/metal, contracts/artifacts, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, target-profiles, provenance]
claimed_from: todo
assignee: loop-validate-p0
lease_expires_at: 1785522771
---
## User-visible outcome

A pure policy and a platform observer independently establish whether the current macOS host satisfies the measured applicability predicates required by the first Metal profile. The result is a checked eligibility receipt; the dependent profile-construction ticket binds that receipt to the exact `TargetProfileRef` it owns. The runtime cannot earn eligibility from a `Compilation`, an artifact under validation, or equality with producer-owned bytes.

## Facts and measurement boundary

**Fact:** the prototype currently builds `ExecutionEnvironment.target_profile` directly from `Compilation.target_profile_key()` and `target_profile_descriptor()`. This proves equality with the compiler declaration and says nothing about whether the live host satisfies its applicability predicates.

**Fact:** the intended first profile is bounded by Apple9 hardware, exact macOS version/build and architecture, and the offline compiler/environment facts established by the unified MSL 4 measurement. The offline compiler belongs to artifact provenance. Native pipeline preparation may translate GPU-independent Metal IR through a private component; the AOT observer cannot attribute its exact identity, and the separately measured `newLibraryWithSource` compiler is not evidence for it.

**Inference:** reading a profile reference from the artifact or rebuilding it from a local compilation is a tautology. Device name alone is insufficient; registry ID is not a stable hardware identity and differs across retained runs on the same named M4 Max.

**Measurement boundary:** this ticket consumes the exact predicates produced by `measure-macos-apple9-f32-under-unified-msl4-profile`. It does not broaden them to another OS build, Apple family, compiler build, physical iOS device, or dtype.

## Implementation keys

Define a deterministic pure applicability policy over normalized observations and a platform adapter that observes only predicates established by the retained native execution: OS family/version/build, architecture, exact reported device name, and supported GPU family. The profile construction separately binds the exact offline compiler and linker to artifact provenance. Exact native translator/compiler identity remains `Unknown`; the source-JIT compiler build, OS build, loaded-image presence, and producer-owned bytes are not substitutes. Registry ID is correlation evidence and no unmeasured “stable hardware class” may be invented.

The translation authority is decided: [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) requires an attributable identity for the private translating component or exact host attestation before any positive receipt, and neither exists on current APIs. The policy therefore models the translation-authority predicate explicitly and refuses it as `Unknown` with a typed reason naming that exact predicate, even when every public environment predicate matches the measured row. The environment predicates remain worth checking and refusing precisely — they are the validity scope every future positive receipt will also need — but a positive eligibility receipt must be structurally unreachable while the translation authority is unsatisfied, not merely untested.

The policy returns a non-forgeable eligibility receipt scoped to its versioned policy and exact normalized observation, or a typed reason for refusal. It does not return or contain a target-profile key or descriptor, because `construct-and-bind-the-first-authoritative-metal-compile-profile` owns that declaration and currently depends on this ticket. The parent consumes a successful receipt and binds it to the exact profile it constructs; this removes the circular requirement for a dependency to return a value its dependent creates.

Keep observation separate from decision so positive and negative policy cases run without Metal hardware or framework access. This repository currently has no CI, so do not describe those portable unit cases as a CI guarantee. Keep Metal device observation out of device-free `tiler-runtime`; the current prototype may host the first adapter, while any reusable Metal runtime adapter requires its own reviewed ownership boundary. Preserve live-device and prepared-pipeline checks as distinct later obligations.

## Required evidence

Tests must reject wrong platform, architecture, Apple family, device name, OS version, OS build, and missing observations, each with its own typed reason, and must show that the fully matching measured observation still refuses with the ADR 0086 translation-authority predicate as the named cause. A test must prove that no admissible input combination reaches a positive receipt on current APIs, and that neither `Compilation`, `TargetProfileRef`, decoded artifact bytes, offline compiler provenance, nor a source-JIT compiler identity are inputs to host eligibility. The parent ticket owns key mismatch, same-key/different-descriptor mismatch, offline compiler provenance, and the final eligible-host offer if an ADR 0086 reconsideration trigger ever supplies the missing authority. An unavailable host must report the precise predicate it cannot satisfy.

## Closes when

The pure policy and observer independently evaluate the exact measured native-execution predicates and the ADR 0086 translation-authority predicate, producing a typed refusal that names the unsatisfied authority on every current host; the eligibility-receipt type exists and is structurally unreachable while that predicate is `Unknown`; no profile reference, producer-owned bytes, or unattributable compiler identity enters the decision; all refusals occur before routing commit; the parent has the explicit typed input and refusal it binds against; and focused tests plus `make check` pass. Removing `host_environment(&Compilation)` and any future eligible-host offer remain parent integration work rather than a circular closing condition here.

## Siting elimination

The pure policy lives in `crates/tiler-metal/src/applicability.rs`, with its device-free cases in `crates/tiler-metal/src/applicability_tests.rs`. The constraint is that both consumers must reach it with no device dependency and no new edge: `tiler-build` owns the profile declaration the parent binds, and `prototypes/serial-sum-run` hosts the first adapter. Both already depend on `tiler-metal`, whose own `[dependencies]` are `tiler-artifact` and `tiler-ir` alone.

**Eliminated — a new crate.** Crate admission is Tom's, and the ticket forbids it. Nothing here needs one: the module has no dependency the host crate does not already have.

**Eliminated — `tiler-runtime`.** It is device-free, which is the property that first suggests it, and it fails on two counts. `tiler-build` does not depend on it, so siting here means adding a build-time → load-time edge for one decision. More decisively, the crate is backend-neutral by charter ("portable across backends", `crates/tiler-runtime/src/lib.rs`), and this policy's content is `Apple9`, `Apple M4 Max`, and macOS build `26A5388g` — backend-specific facts in the one crate whose value is being backend-free. Its `load::host` module is also where a `TargetProfileRef` is compared, and the ticket forbids one as an input to this decision; the adjacency would put the forbidden input one `use` away.

**Eliminated — `tiler-compiler` and `tiler-artifact`.** Reachable from both consumers, and eliminated by the same neutrality rule: the architectural contract keeps the compiler core independent of Metal, and `tiler-artifact` owns exactly the producer-side identities ADR 0086 excludes from this decision.

**Eliminated — `tiler-metal-aot`.** It owns the offline compiler and linker provenance that ADR 0086 item 4 excludes by name, and it spawns `xcrun`; its documented value is an auditable empty-dependency shim. It is also only a *development* dependency of `tiler-metal`, so it is not reachable from the library target at all.

**Taken — `tiler-metal`, as a sibling module of `target`, not a member of it.** `crate::target` states that it holds compile-time target facts and that live-device facts are deliberately absent; a host observation is a live-host fact, so folding it in would contradict that module's own boundary. The siting also buys a structural property the alternatives do not: `Compilation`, offline compiler provenance, and the source-JIT compiler identity are *unnameable* from this crate's library target, and `applicability_tests::the_dependency_set_keeps_producer_types_unnameable` is the check that keeps it that way.

## Public boundary review packet

New public surface, all in `tiler_metal::applicability` and all a reviewed *draft* boundary under ADR 0074 §7. **Not self-accepted** — Tom's acceptance is required.

- `MetalGpuFamily` — `#[non_exhaustive]` enum, `Apple5..Apple9`, with `ALL`, `COUNT`, `as_str`, `Display`.
- `MetalGpuFamilySupport` — exhaustive (convention 5b) enum: `Highest(MetalGpuFamily)` or `NoneNamed`. Exhaustive because the answer set is closed and the adapter maps both arms; growth belongs to `MetalGpuFamily`.
- `MetalHostPredicate` — `#[non_exhaustive]` enum of the seven predicates, with `ALL` in evaluation order, `COUNT`, `as_str`, `Display`.
- `MetalHostObservation` — private fields; `unobserved()`, six `observing_*` builders, six accessors returning `Option`.
- `MetalHostApplicabilityPolicy` — private fields, **no constructor**; the single value is `FIRST_MACOS_APPLE9`, plus `id()` and six required-value accessors.
- `NativeTranslationAuthority` — public, uninhabited (see below).
- `MetalHostEligibility` — private fields; `policy()`, `observation()`. Carries no target profile key or descriptor.
- `MetalHostApplicabilityRefusal` — `#[non_exhaustive]` enum with one variant per predicate plus `Unobserved { predicate }`, with `predicate()`, `rule()`, `Display`, `Error`.
- `evaluate_metal_host_applicability(policy, &observation) -> Result<MetalHostEligibility, MetalHostApplicabilityRefusal>`.

`crates/tiler-metal/src/lib.rs` gained a paragraph saying the crate now owns this second, smaller thing, because the previous text claimed it owned source emission and target metadata only.

## Structural unreachability

`NativeTranslationAuthority`'s one field is a private empty enum (`NoAdmissibleAuthority`), so the type is uninhabited: no value exists, inside this crate or out. `MetalHostEligibility` holds one, so a positive receipt is impossible to construct rather than merely unproduced. Three compiler-checked pieces of evidence:

- `applicability::structural_unreachability::every_outcome_is_a_refusal` matches `Result<MetalHostEligibility, _>` with **no `Ok` arm** and compiles, which is only true while the `Ok` payload is uninhabited. Inhabiting the authority type makes that function stop compiling. It lives beside the policy because uninhabitedness is visible only where the private empty enum is.
- Two `compile_fail` doctests on `NativeTranslationAuthority`: `E0451` for a struct literal (the field is private) and `E0599` for a constructor that does not exist.
- Two `compile_fail,E0308` doctests on `evaluate_metal_host_applicability` proving a `tiler_artifact::program::TargetProfileRef` and raw artifact bytes are not admissible second arguments. `Compilation`, offline compiler provenance, and the source-JIT compiler identity need no doctest because `tiler-metal` cannot name them; the check for that is the manifest test named above, and the one-line reproduction is that `crates/tiler-metal/Cargo.toml`'s `[dependencies]` are exactly `tiler-artifact` and `tiler-ir`.

The registry ID is not an input and appears nowhere in the module. ADR 0086 excludes it by name, and finding "Measurement environment" of `docs/research/apple-targets/numerical-behaviour.md` records the two values (`4294968621`, `4294968452`) the retained records report for the same named Apple M4 Max.

## Adapter

`prototypes/serial-sum-run` hosts the first platform adapter, observing only the named predicates: OS family from `std::env::consts::OS`, architecture from `std::env::consts::ARCH` normalized (`aarch64` → `arm64`, everything else passed through unchanged), OS version and build from `/usr/bin/sw_vers`, device name from `MTLDevice.name`, and the highest supported Apple family from `supportsFamily:`. A `sw_vers` that does not answer leaves the predicate *unobserved* rather than supplying a placeholder. `device_facts` was refactored onto the same family observer rather than keeping a second copy of the family list.

`report_host_applicability` runs immediately after the device is opened and before every routing commit this binary makes. It **reports** rather than gates: the policy refuses on every host, so gating the value proof on it would stop the proof while proving nothing, and the eligible-host offer belongs to `construct-and-bind-the-first-authoritative-metal-compile-profile`. `host_environment(&Compilation)` is untouched — removing it is parent integration work.

## Perturbation evidence

Twelve perturbations, one per new check, each applied alone and reverted: the policy row constant, the authority being evaluated last, the receipt's structural unreachability, the GPU-family predicate, the unobserved-predicate refusal, the dependency set, the compile-fail diagnostic pin, the enumerated matrix population, the ADR citation in the rendered refusal, the architecture normalization, the `sw_vers` observation, and the device-free half's four predicates. All twelve were detected — but only after the eighth was fixed.

**The eighth did not fail on the first pass, and that is the finding worth keeping.** `no_admissible_observation_reaches_a_positive_receipt` asserted its population as `families.len() * versions.len() * …`, so cutting a domain from three values to two cut the expectation with it: 648 cases reported the same full coverage 972 did. The count is now a literal `972`, and each domain carries an explicit array length so dropping a value is a compile error as well. A count computed from the thing it counts is not a count.

## Graph maintenance

This ticket depends on `measure-macos-apple9-f32-under-unified-msl4-profile`, the AOT-compatible runtime-compiler observer spike, and `authorize-macos-environment-identity-for-native-metal-translation`, and blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep `record-metal-runtime-compiler-provenance-gap`, `prototype-metal-runtime-proof`, and `restore-replayable-apple-compatibility-evidence` related; they establish adjacent provenance and preflight contracts but do not implement host eligibility.
