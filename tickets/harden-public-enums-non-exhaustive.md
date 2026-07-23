---
id: harden-public-enums-non-exhaustive
title: Mark growth-expecting public enums and output records non-exhaustive
status: todo
priority: p2
dependencies: []
related: [prototype-apple-aot-driver, prototype-scheduled-region-ir]
scopes: [implementation/ir, implementation/metal-aot, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening]
---
The newly merged `tiler_ir::schedule` and `tiler-metal-aot` public surfaces expose
enums that are documented single-variant or bounded-profile placeholders for the
wide-operation future, plus output records that will gain identity dimensions. As
written, every future operation/dtype/family variant and the deferred artifact
content digest is a breaking change to a downstream `match` or struct literal.

Mark the growth-expecting types `#[non_exhaustive]` so those additions land
additively:

- `tiler_ir::schedule`: `ScalarProgram`, `LogicalAccess`, `ReductionTopology`,
  `BoundsProofKind`, `OwnershipProofKind`, `SubnormalMode`, `NumericalPermission`,
  `ExecutionBinding`, `TailPolicy`, `ContributorOrder`.
- `tiler-metal-aot`: `AppleSdk` (reserves Mac Catalyst), `MslVersion`,
  `OptimizationLevel`, and the output records `ArtifactProvenance` and
  `CompiledArtifact` (which gain the deferred content digest). Leave the input
  structs (`CompileRequest`, `MetalTarget`, `NumericalRealization`) exhaustive:
  callers construct them and their growth is a `new()`-signature change regardless.

Marking `ScalarProgram` non-exhaustive forces `tiler-compiler`'s recognizer
(`verify_region_subject_binding`, `verify_access_and_semantics`) to grow an
explicit reject-unknown arm — this is the intended fail-closed posture, not a
regression. There are no external consumers of either surface yet, so this is
timing-free now and expensive to retrofit after the first consumer lands.

Reviewed as a fast-follow to the two prototype merges (Tom saw the surfaces).
This ticket does not change any semantics; it only reserves additive growth.
