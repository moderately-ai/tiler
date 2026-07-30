---
id: admit-measured-compile-profile-sources-across-fact-families
title: Admit measured CompileProfile sources across target fact families
status: in-progress
priority: p0
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, carry-the-honourability-fact-provenance-into-the-artifact-record, decide-per-dtype-dispatchability-as-a-target-capability]
scopes: [implementation/compiler, implementation/build, contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, correctness]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785450295
---
## User-visible outcome

A profile producer can attach checked `CompileProfile`/`MeasuredProfile`/`MeasuredEnvironment` provenance to quantitative capabilities, exact dtype dispatchability, and every measured numerical dimension without relabelling empirical evidence as a portable external guarantee or gaining an unrestricted phase/authority constructor.

## Facts and measurement boundary

**Fact:** `TargetCompileProfileMeasurementSource` fixes the correct phase, authority, validity, producer, compiler-build set, and execution-environment contexts, but the public builder accepts it only for the two complete measured subnormal helpers. Quantitative declarations, exact dtype dispatchability, and every other numerical declaration accept `TargetFactSource`; its only public compile-profile constructor is `external_guarantee`, whose validity is `PortableProfile`.

**Inference:** the first authoritative Metal profile cannot truthfully encode measured F32 dispatchability or any measured non-subnormal row through the current public types. A cast, conversion to the unrestricted source type, caller-selected authority tuple, or external-guarantee label would erase the distinction this provenance schema exists to preserve.

**Measurement boundary:** this ticket provides representation and validation, not evidence that any Metal fact is true. Each consumer still supplies exact retained measurement contexts, and omitted rows remain `Unknown`.

## Implementation keys

Add narrow producer operations or a reviewed measured-source capability that can populate quantitative, exact resolved-type dispatchability, and dimension-safe numerical declarations while fixing phase, authority, and validity in the type. Preserve transactional insertion, duplicate/conflict diagnostics, canonical source encoding, descriptor identity, bounded compiler/environment sets, and exact resolved-type matching. Do not expose a constructor that accepts arbitrary `AvailabilityPhase`, `FactAuthority`, or `FactValidityScope`. Tom must review the consequential public boundary before acceptance.

## Required evidence

Tests must prove that each supported fact family records `CompileProfile`/`MeasuredProfile`/`MeasuredEnvironment`, rejects empty or malformed contexts, rejects duplicate and contradictory rows atomically, leaves omitted rows `Unknown`, and changes the canonical profile descriptor when producer identity, compiler build, execution environment, fact value, or source context changes. Compile-fail or visibility evidence must show downstream callers cannot construct an arbitrary phase/authority combination.

## Closes when

The authoritative Metal profile can express every genuinely measured compile-profile row with exact structured provenance, no empirical row is represented as `ExternalProfile`/`PortableProfile`, all focused tests and `make check` pass, and Tom has reviewed the public construction boundary.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep it related to `carry-the-honourability-fact-provenance-into-the-artifact-record` and `decide-per-dtype-dispatchability-as-a-target-capability`; those established the schema and placement, while this ticket widens only the checked measured construction surface.
