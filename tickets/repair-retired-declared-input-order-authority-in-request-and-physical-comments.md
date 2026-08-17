---
id: repair-retired-declared-input-order-authority-in-request-and-physical-comments
title: Repair retired declared-input ordering authority in request and physical comments
status: todo
priority: p2
dependencies: []
related: [repair-retired-input-ordinal-claims-in-compiler-pipeline-tests, repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [defect, documentation, access-ordinal]
---
## User-visible outcome

Current compiler request and physical-planning comments no longer assign declared-interface ordering authority to fieldless `TensorRole::Input` or intrinsic schedule verification. The documented owner is the compiler's retained checked request subject and exact `AccessOrdinal` projection.

## Discovery — 2026-08-16, exact main `f46ac65cc6050c6804f9376f2fb86e44430c8c31`

A complete read of `crates/tiler-compiler/src/pipeline/tests.rs` for the related pipeline-comment repair required re-reading the current request and physical association owners. It found three adjacent stale present-tense anchors outside that narrow ticket:

- `request.rs`: `pointwise access contract requires a region's declared input ordinals not to descend`;
- `physical.rs`: `own read-ordering rule refuses as two spellings`; and
- `physical.rs`: `canonical order a pointwise region requires: declared inputs by ascending ordinal`.

Public `TensorRole::Input` is fieldless. Intrinsic schedule verification sees ordered local accesses and cannot decide a declared-interface substitution. `VerifiedScheduledRegion::declared_input_at` projects an exact `AccessOrdinal` through the retained verified request subject, and `CoverAssembly::from_plan` consumes that association.

The first physical comment also justifies the current conservative `value_input == weight_input` refusal for `rms_norm(x, x)`. Exact-position machinery can represent repeated reads, so the support boundary and its true owner must be audited rather than silently rewriting only the prose.

## Required work

- Re-read the complete request normalization/recognition, physical construction/verification, checked projection, assembly, staged-family, and repeated-read paths at the exact implementation base.
- Re-audit each anchor as Fact, Inference, or stale claim. Repair the ticket first if the current support boundary differs.
- Replace retired declared-ordinal/intrinsic-verifier authority with the exact compiler-owned ordering and projection authority.
- Decide from source whether `rms_norm(x, x)` remains a deliberate fail-closed support boundary under another rule or is now an unowned removable refusal. If changing support or a public/identity boundary is required, stop and split/route that work instead of widening under this documentation ticket.
- Add a source-subject check that reaches each retired phrase; perturb each independently and quote the failure.
- Preserve behavior, identities, schemas, tests, and public surfaces unless a separately accepted dependency authorizes otherwise.

## Graph

Related to the narrow pipeline-test prose repair and the completed fieldless-role documentation repair. It must not be folded into either without its own current-base source audit.
