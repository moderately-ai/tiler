---
id: repair-retired-declared-input-order-authority-in-request-and-physical-comments
title: Repair retired declared-input ordering authority in request and physical comments
status: in-progress
priority: p2
dependencies: []
related: [repair-retired-input-ordinal-claims-in-compiler-pipeline-tests, repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation, decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [defect, documentation, access-ordinal]
claimed_from: todo
assignee: worker-declared-input-prose
lease_expires_at: 1786966365
---
## User-visible outcome

Current compiler request and physical-planning comments no longer assign declared-interface ordering authority to fieldless `TensorRole::Input` or intrinsic schedule verification. The documented owner is the compiler's retained checked request subject and exact `AccessOrdinal` projection.

## Discovery — 2026-08-16, exact main `f46ac65cc6050c6804f9376f2fb86e44430c8c31`

A complete read of `crates/tiler-compiler/src/pipeline/tests.rs` for the related pipeline-comment repair required re-reading the current request and physical association owners. The initial discovery named three adjacent stale present-tense anchors outside that narrow ticket:

- `request.rs`: `pointwise access contract requires a region's declared input ordinals not to descend`;
- `physical.rs`: `own read-ordering rule refuses as two spellings`; and
- `physical.rs`: `canonical order a pointwise region requires: declared inputs by ascending ordinal`.

Public `TensorRole::Input` is fieldless. Intrinsic schedule verification sees ordered local accesses and cannot decide a declared-interface substitution. `VerifiedScheduledRegion::declared_input_at` projects an exact `AccessOrdinal` through the retained verified request subject, and `CoverAssembly::from_plan` consumes that association.

The first physical comment also justifies the current conservative `value_input == weight_input` refusal for `rms_norm(x, x)`. Exact-position machinery can represent repeated reads, so the support boundary and its true owner must be audited rather than silently rewriting only the prose.

## Exact-current read-only repair audit — 2026-08-17, `e8141d7decbb8204e7930421d0b1acedef9b4dd5`

- **Verified — the cited source population did not drift.** `request.rs` and
  `physical.rs` are byte-identical from discovery base `f46ac65c` through this
  audit base.
- **False as a complete census — three additional present-tense blocks make the
  exact population six.** Besides the three anchors above, repair the
  `BoundaryRead` claim `whole-program elementwise region never had to
  distinguish them`, the `RegionSpellingKind::Epilogue` claim `a pointwise
  region's reads are the declared inputs in declaration order`, and
  `elementwise_region`'s claim `A whole-program or prologue region reads every
  declared input in declaration order`.
- **Verified — intrinsic schedule verification owns none of those interface
  associations.** `TensorRole::Input` is fieldless;
  `reads_bind_boundary_tensors_in_order` checks local boundary categories;
  compiler normalization retains the ordered declared association; and
  `VerifiedScheduledRegion::declared_input_at(AccessOrdinal)` projects one exact
  access through the checked request subject before `CoverAssembly` constructs
  `AssemblyBinding::Input`.
- **False — the current RMS refusal is justified by an intrinsic two-spelling
  rule.** The schedule vocabulary can retain two fieldless input reads and the
  checked request subject can project both positions to one declaration.
  However, removing the refusal is not authorized here: two operand-position
  accesses and one coalesced access are distinct schedule/kernel/identity
  spellings, and no accepted contract chooses between them.

[`decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands`](decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands.md)
owns that consequential support decision. This ticket keeps `rms_norm(x, x)`
fail-closed and states only the truthful unsupported boundary.

## Required work

- Re-read the complete request normalization/recognition, physical construction/verification, checked projection, assembly, staged-family, and repeated-read paths at the exact implementation base.
- Re-audit each anchor as Fact, Inference, or stale claim. Repair the ticket first if the current support boundary differs.
- Repair all six audited comment blocks. Replace retired declared-ordinal/intrinsic-verifier authority with `canonical_input_reads`, `recognize_epilogue`, the retained checked request subject, exact `AccessOrdinal` projection, and `CoverAssembly` as appropriate.
- Preserve the `value_input == weight_input` refusal. Describe it only as an unsupported coincident-operand staged-pass spelling whose canonical access/identity choice belongs to the linked decision ticket; do not claim the intrinsic verifier rejects it.
- Add a source-subject check that reaches each of the six retired phrases; perturb each independently and quote the failure.
- Preserve behavior, identities, schemas, tests, and public surfaces unless a separately accepted dependency authorizes otherwise.

## Graph

Related to the narrow pipeline-test prose repair and the completed fieldless-role documentation repair. It must not be folded into either without its own current-base source audit.
