---
id: admit-a-caller-declared-target-profile
title: Admit a caller-declared target profile
status: todo
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [express-metal-honourability-in-the-shared-form, prototype-public-compiler-api, report-per-target-compilation-outcomes, admit-a-dtype-dispatchability-capability-axis]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, feasibility, identity]
---
## User-visible outcome

An out-of-crate caller can build an immutable, validated target profile and submit one or more profiles to compilation without manufacturing target facts, illegal numerical dimension/behaviour pairs, or silently incomplete dtype support. Compilation returns an outer request/coordination result and an ordered outcome for every admitted target profile, while an omitted or unknown fact fails closed.

## Why the old decision was removed

**Fact:** the former option table said `DimensionBehaviour` had two variants. The current source has five variants over eleven governed scalar-arithmetic dimensions, and a usable declaration also needs `ArithmeticType`, `DeclaredBehaviour`, `RelaxationRequirement`, availability phase, authority, validity, and provenance. Promoting three enums could not construct the promised row.

**Fact:** raw dimension/behaviour rows admit combinations that `NumericalDimension::admits` must reject. `SupportedWithExactEmulation` is treated as proven by feasibility, but a public row currently cannot identify a compiler-selected, costed emulation implementation. Exposing that spelling would let a caller assert a proof the compiler does not possess.

**Fact:** dispatchability is qualitative and keyed by the complete canonical resolved semantic type. It is not a `u64` quantitative `CapabilityAxis`, a bare `TypeKey` that loses parameters and encoded components, or an enum duplicated in the target layer.

**Inference:** none of the former public-enum, pair-builder, or quantitative-axis options survives correctness. The remaining work is an implementation draft followed by review of the exact public boundary, not a product decision.

## Implementation keys

- Replace `PrototypeTargetProfile` with an immutable checked `TargetProfile` produced by a consuming `TargetProfileBuilder`; validate and canonicalize once, sort once, and keep reads allocation-free.
- Use scalar-specific resolved declaration types whose constructors make invalid dimension/behaviour and relaxation pairings unrepresentable. Initially admit exact support, explicit unsupported facts, and conditional support only where the condition is validated. Admit exact emulation only after a compiler-selected provider token identifies and costs the implementation.
- Key the first versioned policy subject by `ScalarArithmetic` plus canonical full resolved-type identity. Reserve an outer versioned subject-family seam for future integer, boolean, complex, decimal, quantized, MX, conversion, and owner-defined contracts; do not force them through scalar semantics.
- Model dtype dispatchability as a target-family qualitative fact/predicate family keyed by canonical full resolved-type identity. Unknown `(target family, resolved type)` pairs reject. Keep quantitative capability axes unchanged.
- Carry structured producer provenance from `carry-the-honourability-fact-provenance-into-the-artifact-record`; the compiler validates and selects facts but does not become the measured fact's authority.
- Keep `TargetProfileKey` owned, validated, and present in request and artifact identity. Remove residual `&'static str` key bindings and centralize profile validation.
- Make sparse omission resolve `Unknown`, never satisfied. Validate quantitative bounds and reject duplicate keys and empty target sets with typed invalid-request errors.
- Design request cardinality and results together: accept an ordered non-empty profile collection, return one ordered per-target outcome for every admitted profile, and keep request-wide structural/coordination failure outside those outcomes.
- Preserve the governed profile's allocation-free construction where practical without exposing caller-declared ABI facts or weakening validation.

## Required evidence

- An external caller constructs a valid profile and receives a target result.
- Illegal dimension/behaviour and relaxation pairings are unconstructible or reject before compilation.
- Omitted scalar dimensions and unmeasured dtype/target-family pairs reject as `Unknown`.
- A caller cannot assert exact emulation without a compiler-selected provider.
- Nominal, parameterized, encoded, quantized, boolean, and integer identities cannot collide or inherit a neighbouring scalar fact.
- Two target profiles produce two ordered outcomes even when one target rejects, while malformed request structure returns the outer error.
- Canonical profile descriptor bytes and artifact identity change only through the documented rebaseline procedure.
- Every new validator is perturbed once and observed failing; targeted compiler tests, per-package Clippy, and `make full` pass.

## Public review boundary

Before acceptance Tom reviews the exact `TargetProfile`, builder, scalar-policy declaration, qualitative dispatchability fact, request collection, per-target outcome, and typed error signatures together with representative call sites. This ticket may produce a tested concrete draft; it must not treat compilation as implicit acceptance of those public types.

## Graph maintenance

- This ticket follows structured target-fact provenance, then unblocks `express-metal-honourability-in-the-shared-form`.
- `report-per-target-compilation-outcomes` and `admit-a-dtype-dispatchability-capability-axis` are superseded by this coherent boundary because their signatures and invariants cannot be reviewed independently.
- If the first public request deliberately remains single-profile, split multi-target cardinality into a deferred ticket with its activation trigger; do not leave a structurally plural private request behind a singular public result.
- A future policy family is filed only with a named producer, consumer, validation schema, identity consequence, and lowering or runtime evidence.
