---
id: repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation
title: Repair fieldless TensorRole documentation after AccessOrdinal reconciliation
status: todo
priority: p1
dependencies: []
related: [reconcile-input-ordinal-region-local-and-declared-input-semantics, decide-the-source-bound-live-row-major-access-surface]
scopes: [implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [defect, documentation, access-ordinal]
---
## User-visible outcome

Live architecture and compiler documentation describe fieldless
`TensorRole::Input`, full-list `AccessOrdinal`, and compiler-private
declared-input association without attributing declared-interface ordinals to
shared schedule roles or to intrinsic schedule verification.

## Exact-base Fact audit — 2026-08-16, `98669e8ea9cafc91b3a9139ff821781560c526bd`

- **Fact — the accepted replacement is implemented.**
  [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md), anchor `The public replacement is complete`, records the landed fieldless role and full-list coordinate. `crates/tiler-ir/src/schedule/handles.rs`, anchor `The exact position in a scheduled region's complete ordered access list`, defines public `AccessOrdinal`; `crates/tiler-compiler/src/physical.rs`, anchor `Projects one local input access back to the declared program interface`, projects that position through the retained verified request subject to private `DeclaredInputOrdinal`.
- **Fact — the architecture contract contradicts that implementation.**
  `docs/architecture.md`, anchor `distinguishes inputs by ordinal and carries none`, still assigns the retired declared-ordinal payload to the shared role while explaining multi-output limits. The live source has fieldless `TensorRole::Input`; output attribution remains a program/cover concern rather than a fact supplied by an input role.
- **Fact — one program-assembly comment repeats the retired authority.**
  `crates/tiler-compiler/src/program.rs`, anchor `separates reads of`, says the shared role distinguishes declared inputs. The live loop below it instead constructs `AccessOrdinal` from the exact read position and calls `VerifiedScheduledRegion::declared_input_at`; the comment's separate statement that two `Intermediate` reads lack an edge coordinate remains true and must be preserved.
- **Fact — five request comments need reclassification against the fieldless verifier.**
  `crates/tiler-compiler/src/request.rs` contains five live references to `reads_bind_boundary_tensors_in_order` (current lines 1821, 6188, 6200, 6267, and 6587; anchors remain authoritative). The function still exists, but after fieldless roles it checks boundary categories and at most one intermediate; it cannot see declared ordinals. The `BoundaryRead` comment at anchor `states the same separation from the schedule side` correctly says access position and boundary category differ, but imprecisely says `CoverAssembly::from_plan` resolves the role: assembly now resolves an exact `AccessOrdinal` through `VerifiedScheduledRegion::declared_input_at`. The other comments at anchors `the canonical spelling`, `what reads_bind_boundary_tensors_in_order admits`, `binds the pair in one canonical order`, and `states the boundary-role rules` attribute declared-input grouping, ascent, or dense-before-mapped ordering to that intrinsic verifier. Those orders are compiler-private normalization properties of `canonical_input_reads` and `record_leaf`, not shared-IR role authority.

Reproduce:

```sh
rg -n 'distinguishes inputs by ordinal and carries none|separates reads of' docs/architecture.md crates/tiler-compiler/src/program.rs
test "$(rg -c 'reads_bind_boundary_tensors_in_order' crates/tiler-compiler/src/request.rs)" -eq 5
rg -n -C 4 'reads_bind_boundary_tensors_in_order' crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/builder.rs
rg -n 'pub struct AccessOrdinal|Projects one local input access back to the declared program interface|fn declared_input_at' crates/tiler-ir/src/schedule/handles.rs crates/tiler-compiler/src/physical.rs
```

## Required work

- Correct the one architecture paragraph without broadening or narrowing the live multi-output support claim it is explaining.
- Correct the program-assembly comment to describe exact access-position projection for inputs while retaining the independent multi-intermediate attribution refusal.
- Re-site all five request comments on compiler-private canonical read ordering, fieldless boundary-category validation, and exact `AccessOrdinal` projection. Preserve the first comment's true access-position/category separation while replacing its stale role-resolution wording. Do not rename the intrinsic helper merely to make the stale prose resolve, introduce a public declared ordinal, or move request authority into shared IR.
- Search the live `docs/` and `crates/tiler-compiler/` populations for the same retired authority, classify every hit, and add any newly found live claim to this ticket before editing it.

## Evidence and negative controls

- Existing subset, repeated-read, epilogue, and later-declared-input compiler tests remain green; this is prose repair only.
- Grep the two retired exact claims and require no live hit after repair. Separately require `reads_bind_boundary_tensors_in_order` still exists and its fieldless boundary-category documentation remains, so deletion or renaming cannot masquerade as a repair.
- Perturb one repaired request comment back to attributing declared-ordinal ascent to the intrinsic verifier and show the exact source-reading/citation check that rejects it; a grep that merely resolves the helper name is not evidence of semantic accuracy.

## Closing conditions

The architecture and compiler comments agree with the landed coordinate model, the source census is recorded, targeted compiler tests plus `tkt lint`, `make citations`, and `git diff --check` pass, and no production behavior or identity changes.
