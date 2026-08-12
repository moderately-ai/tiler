---
id: derive-artifact-binding-alignment-from-selected-access-requirements
title: Derive artifact binding alignment from selected access requirements
status: blocked
priority: p1
dependencies: [admit-typed-byte-alignment-and-effective-program-view-guarantees, carry-complete-access-alignment-requirements-on-physical-proposals]
related: [package-selected-physical-implementation-provenance-in-artifact-identity, carry-the-binding-offset-through-the-runtime-route]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [alignment, artifact, abi, build, identity, correctness]
---
## User-visible outcome

Every artifact binding states the selected implementation's exact required bound-address alignment, and artifact construction refuses a program view that cannot provide it.

## Facts at filing base `f199b26376612e4b39c35569b084dda4c67490ce`

- **Verified.** Artifact `BindingData` and the manifest codec already encode an `alignment: u32`; public and decoded accessors document it as the byte alignment the bound storage must satisfy.
- **Verified.** `ArtifactProgramBuilder::check_bindings` currently populates the field from `value.alignment()` while separately adopting the stage view's accessible offset. It neither consumes selected implementation requirements nor derives the effective guarantee of `value + offset`.
- **Verified.** The existing field is already folded into artifact identity and manifest bytes. Correcting its producer requires no new row, tag, field, or manifest schema.

## Required delivery

- Extend the neutral artifact entry-binding construction input with one required `AlignmentRequirement` per exact slot. `tiler-build` fills it only from the compiler-minted selected entry/slot projection; no default, value-alignment inference, target-row inference, or backend constant is accepted.
- In artifact construction, resolve the program stage/view and prove its effective `AlignmentGuarantee` satisfies the selected requirement. Distinguish wrong entry, slot, view, selected population, and insufficient guarantee with typed errors.
- Encode the selected requirement in the existing binding alignment field. Keep value/allocation guarantees in the program subject; do not duplicate them into the artifact binding.
- Keep artifact schema `16.0` and its binding grammar only if exact-base perturbation proves the field's documented meaning and framing are unchanged. Existing valid natural-alignment artifacts must retain the same binding bytes; a stronger selected requirement intentionally changes artifact identity and bytes.
- Cross-check the complete selected physical provenance population when that accepted carrier is present. An unused provider never changes a binding requirement; changing the selected proposal's requirement does.

## Required evidence

- Package whole and partial F32 views with natural requirements; prove correct offsets retain current bytes.
- Package a 16-byte selected requirement over a 16-byte-effective view and refuse the same requirement over a view weakened to 4 bytes.
- Perturb entry, slot, requirement, view offset, selected proposal, and unused offered provider independently.
- Decode/re-encode the stronger requirement byte-identically and prove an old reader already interprets the same field as a requirement rather than as a guarantee.
- Show by unchanged manifest field census that no schema/version step was hidden.

## Non-goals

Runtime address observation, allocator guarantees, target profile rows, provider selection, vector operation requirements, or packaging the entire offered provider environment.

## Closes when

The existing artifact binding field is derived from exact selected access requirements, checked against effective program views, identity-bearing, and schema-neutral for the stated reason.
