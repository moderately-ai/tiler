---
id: admit-a-caller-declared-target-profile
title: Admit a caller-declared target profile
status: done
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [express-metal-honourability-in-the-shared-form, prototype-public-compiler-api, report-per-target-compilation-outcomes, admit-a-dtype-dispatchability-capability-axis, spike-bf16-through-the-second-dtype-seams, recheck-target-dtype-dispatch-after-semantic-rewrites]
scopes: [implementation/compiler, contracts/navigation, implementation/build, implementation/runtime, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, feasibility, identity]
---
## User-visible outcome

An out-of-crate caller can build an immutable, validated target profile and submit one or more profiles to compilation without forging compiler-verified attribution, illegal numerical dimension/behaviour pairs, or silently incomplete dtype support. Compilation returns an outer request/coordination result and an ordered outcome for every admitted target profile, while an omitted or unknown fact fails closed. Caller-declared measurements remain claims by their named producer; structural validation does not turn them into compiler-authenticated truth.

## Why the old decision was removed

**Fact:** the former option table said `DimensionBehaviour` had two variants. The current source has five variants over eleven governed scalar-arithmetic dimensions, and a usable declaration also needs `ArithmeticType`, `DeclaredBehaviour`, `RelaxationRequirement`, availability phase, authority, validity, and provenance. Promoting three enums could not construct the promised row.

**Fact:** raw dimension/behaviour rows admit combinations that `NumericalDimension::admits` must reject. `SupportedWithExactEmulation` is treated as proven by feasibility, but a public row currently cannot identify a compiler-selected, costed emulation implementation. Exposing that spelling would let a caller assert a proof the compiler does not possess.

**Fact:** dispatchability is qualitative and keyed by the complete canonical resolved semantic type. It is not a `u64` quantitative `CapabilityAxis`, a bare `TypeKey` that loses parameters and encoded components, or an enum duplicated in the target layer.

**Inference:** none of the former public-enum, pair-builder, or quantitative-axis options survives correctness. The remaining work is an implementation draft followed by review of the exact public boundary, not a product decision.

## Implementation keys

- Replace `PrototypeTargetProfile` with an immutable checked `TargetProfile` produced by a consuming `TargetProfileBuilder`. Each public declaration method returns `Result<(), TargetProfileBuildError>` and rejects an invalid or duplicate row before insertion, so failure leaves the builder unchanged and there is no recovery protocol. `build` returns `Result<TargetProfile, TargetProfileBuildError>`, canonicalizes the retained rows once before checked-profile and descriptor construction, and reports `DescriptorTooLong { actual, max }` directly.
- Use scalar-specific resolved declaration types whose constructors make invalid dimension/behaviour pairings unrepresentable. The delivered public surface admits only exact support and explicit unsupported facts. Defer public conditional support until the public numerical contract can name and validate the same complete scalar subject on the required relaxation; defer exact emulation until a compiler-selected provider token identifies and costs the implementation.
- Key the first versioned policy subject by `ScalarArithmetic` plus canonical full resolved-type identity. Reserve an outer versioned subject-family seam for future integer, boolean, complex, decimal, quantized, MX, conversion, and owner-defined contracts; do not force them through scalar semantics.
- Treat one target profile as one versioned target-family authority and model dtype dispatchability as a qualitative fact/predicate family keyed by canonical full resolved-type identity inside that envelope. Unknown `(target family, resolved type)` pairs reject and facts do not inherit across nominal, parameterized, encoded, or compound identities. Keep quantitative capability axes unchanged.
- Carry structured producer provenance from `carry-the-honourability-fact-provenance-into-the-artifact-record`; the compiler validates and selects facts but does not become the measured fact's authority. Public provenance fields and collection cardinalities are bounded, collection stops at the first excess item, and errors identify the exact field or set plus the observed and admitted bound.
- Keep `TargetProfileKey` owned, validated, and present in request and artifact identity. Remove residual `&'static str` key bindings and centralize profile validation.
- Make sparse omission resolve `Unknown`, never satisfied. A declared numerical refusal exposes its exact scalar subject, declared means, optional dimension-safe honoured behaviour, and declaring profile. Validate quantitative bounds and reject duplicate keys and empty target sets with typed invalid-request errors.
- Design request cardinality and results together: accept an ordered non-empty profile collection, return one ordered per-target outcome for every admitted profile, and keep request-wide structural/coordination failure outside those outcomes. Numerical preferences admit at most three unique contracts, preserve caller order, and collect at most the public maximum plus one before returning a typed invalid-request failure.
- Resolve each target's numerical contract independently, group targets by equal resolved contract for contract-dependent normalization and exploration, then restore caller order. A rejecting target must not erase candidates or products for another target.
- Keep malformed programs, request structure, profile-set coordination, and compiler invariant failures outside per-target outcomes. Keep target-specific contract, feasibility, and no-plan refusals inside the corresponding outcome.
- Preserve the governed profile as one lazy checked allocation followed by allocation-free clones and reads without exposing caller-declared ABI facts or weakening validation.
- Land the typed dispatchability mechanism with a governed compile-profile F32 fact for the target-neutral prototype only. This is not a Metal or production-device claim; a named adapter must provide those facts. `express-metal-honourability-in-the-shared-form` remains the first production Metal numerical-fact producer.

## Required evidence

- An external caller constructs a valid profile and receives a target result.
- Illegal dimension/behaviour and relaxation pairings are unconstructible or reject before compilation.
- Omitted scalar dimensions and unmeasured dtype/target-family pairs reject as `Unknown`.
- A caller cannot assert exact emulation without a compiler-selected provider.
- Nominal, parameterized, encoded, quantized, boolean, and integer identities cannot collide or inherit a neighbouring scalar fact.
- Two target profiles produce two ordered outcomes even when one target rejects, while malformed request structure returns the outer error.
- Two targets with different resolvable contracts are normalized and explored through their respective contract groups; a rejection or candidate-readmission failure in one group cannot erase another group's result.
- Canonical profile descriptor bytes and artifact identity change only through the documented rebaseline procedure.
- Every new validator is perturbed once and observed failing; targeted compiler tests, per-package Clippy, and `make full` pass.

## Public review boundary

Before acceptance Tom reviews the exact `TargetProfile`, builder, scalar-policy declaration, qualitative dispatchability fact, request collection, per-target outcome, and typed error signatures together with representative call sites. This ticket may produce a tested concrete draft; it must not treat compilation as implicit acceptance of those public types.

## Public boundary acceptance

Tom ratified the exact public boundary at commit `4ad5a2e` on 2026-07-30 after two independent detached reviews and their correction passes. The accepted boundary includes immutable checked `TargetProfile` construction, dimension-specific scalar declarations, full-`ResolvedValueType` dispatchability facts, the ordered sixteen-profile request bound, outer coordination failure versus per-target outcomes, structured refusal detail, and the public typed error signatures.

## Graph maintenance

- This ticket follows structured target-fact provenance, then unblocks `express-metal-honourability-in-the-shared-form`.
- `report-per-target-compilation-outcomes` and `admit-a-dtype-dispatchability-capability-axis` are superseded by this coherent boundary because their signatures and invariants cannot be reviewed independently.
- If the first public request deliberately remains single-profile, split multi-target cardinality into a deferred ticket with its activation trigger; do not leave a structurally plural private request behind a singular public result.
- A future policy family is filed only with a named producer, consumer, validation schema, identity consequence, and lowering or runtime evidence.
- `spike-bf16-through-the-second-dtype-seams` is the first non-F32 consumer of the complete scalar subject and exact dispatchability seams; its macOS success and iOS-Simulator refusal must reuse this profile vocabulary rather than add a dtype list to the compiler request or backend.
- `recheck-target-dtype-dispatch-after-semantic-rewrites` activates when a rewrite can change a value's exact resolved type. Current admitted rewrites preserve `tiler::f32@1`, so request admission is the only distinct dtype set today; the follow-up prevents that bounded fact from becoming an ambient assumption.
