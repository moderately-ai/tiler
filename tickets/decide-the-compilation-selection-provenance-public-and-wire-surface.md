---
id: decide-the-compilation-selection-provenance-public-and-wire-surface
title: Decide the compilation-selection provenance public and wire surface
status: todo
priority: p1
dependencies: [record-the-compilation-selection-in-target-measurement-provenance, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode]
related: [carry-required-compilation-selection-identity-on-compile-profile-contexts, split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, provenance, identity, metal]
---
## User-visible outcome

Before compilation-selection provenance changes production constructors or wire bytes, Tiler has one accepted exact compile-specific context model, public constructor/accessor and error surface, Metal-owned identity grammar, and schema migration. Runtime/device evidence remains truthful and cannot be forced to invent compile flags.

## Exact-current discovery — 2026-08-17 at `e26b6a5174a2b97d8714de266a1f9007ff163a1b`

- **Verified — the semantic policy is accepted.** Every compile-profile measurement context requires one exact nonempty backend-owned selection identity. Metal includes SDK selector, requested platform/target, exact ordered compile and linker selection, and excludes source and resolved toolchain facts. There is no absent/default/inferred selection.
- **False — that policy uniquely fixes the Rust representation.** Public `MeasurementContext` and `FactEvidenceBasis::Measurement` currently serve compile and runtime/device phases. Two materially distinct correct survivors remain: a separate compile-profile context plus evidence-basis arm/tag, or an internally discriminated context with separate required compile/runtime constructors and views. The first has clearer type separation and a larger exhaustive/wire migration; the second has smaller vocabulary but makes phase-sensitive interpretation load-bearing.
- **Verified — consequential constructors are public.** A complete change must replace or alter `FactSourceProvenance::new`, `FactSourceProvenance::measured`, `TargetCompileProfileMeasurementSource::new`, or grow the public exhaustive evidence vocabulary. Exact names, ownership, invalid-state reachability, and error fields are not accepted.
- **Verified — the required linker perturbation is unreachable.** `CompileRequest::link_flags()` always returns an empty run and has no production input that can vary it. A helper-only synthetic flag would perturb test input rather than the production subject. The narrower candidate encodes the empty counted run and makes future real linker widening compile-stopping and identity-moving; adding a linker-selection input is separate public/product work.
- **Verified — the present dependency order cannot land truthfully.** The sole production compile-profile source is constructed in `tiler-build/src/metal_declaration.rs`, where one source is shared across grid, cost, dispatchability, and numerical rows even though the retained grid evidence used a different selection. Making selection mandatory before partitioning those rows forces either misattribution or a noncompiling intermediate.
- **Verified — generic layers cannot validate Metal facts against opaque bytes.** IR/compiler may check required, nonempty, bounded, framed bytes only. Metal/build ownership must derive and compare the selection against the facts it attributes before a complete descriptor or artifact is constructed.

## Required decision packet

- Re-audit the complete constructor, read-view, codec, identity, renderer, artifact, Metal AOT, and build populations at the exact packet base. Size every census from its owning type where possible.
- Apply the Pareto gate to the separate-evidence-arm and discriminated-context models, plus any materially distinct current-source survivor, status quo, bounded research, and typed deferral. Eliminate `Option`, empty/default/governed selection, phase inference, or a model that makes runtime evidence state compile flags.
- Fix exact public type/variant names, fields, visibility, exhaustiveness, constructors, accessors, ownership/copy behavior, maximum-size preallocation check, and typed errors. State whether generic malformed means only envelope/size/framing failure and keep backend-semantic validation in Metal/build.
- Fix schema-4 tags, field order/framing, schema-3 preserve-or-retire policy, and the exact Metal selection type/accessor/domain grammar. Resolve the empty production linker run explicitly; do not create a test-only synthetic authority.
- Re-derive identity migration. The known minimum is `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` 3 to 4 and unframed `DELIVERED_REALIZATION_DOMAIN` v2 to v3. Because one global provenance schema stamps governed and external sources too, enumerate all transitive target-profile, request/explain, delivered-record, artifact/envelope, and cache values that move. Do not step framed outer domains or the checked feasibility descriptor without source evidence.
- Make the production source partition and public carrier land atomically. The accepted surface must permit grid, cost, dispatchability, and numerical rows to cite only the selection that produced them, with facts-versus-selection mismatch refused in backend/build ownership.
- Compare correctness, fail-closed strictness, long-term maintainability, source compatibility, host allocation/runtime, identity/schema, and unsupported populations. Give each survivor its strongest counterargument, reversal evidence, and independent production-subject perturbations.

## Stop boundary

Decision research only. Do not edit production constructors, schema bytes, profile rows, or public APIs before an exact packet passes independent strongest-reasoning review and Tom accepts it. Do not queue a packet until it is Pareto-complete.

## Closes when

One exact public/wire surface is accepted and the implementation carrier can migrate the generic provenance, Metal authority, and truthful build-source partition in one compiling change without inventing defaults or temporarily laundering evidence.
