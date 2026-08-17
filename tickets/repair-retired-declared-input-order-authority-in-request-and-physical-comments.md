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

## Exact implementation-base Fact audit — 2026-08-17, `3969f46cc94ad296bba46885b2688f8a6124bb55`

- **Verified — the two edited Rust sources did not drift from the complete
  read-only audit.** `git diff --quiet e8141d7d 3969f46c --
  crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/physical.rs`
  exits zero. The complete files and their current construction, verification,
  projection, assembly, staged-family, and repeated-read owners were re-read at
  this checkout before source edits.
- **Verified — the exact stale population remains six blocks.** Each of the six
  shortest source anchors named by the read-only audit occurs once; no seventh
  live request/physical block assigns declared-interface authority to the
  fieldless role or intrinsic verifier. Historical past-tense accounts remain
  accurate and are excluded.
- **Verified — the replacement authority remains unique.** `canonical_input_reads`
  owns whole-program/prologue declared-input grouping and dense-before-mapped
  order; `recognize_epilogue` prepends its one staged read;
  `VerifiedScheduledRegion::declared_input_at` projects an exact
  `AccessOrdinal` through the retained `VerifiedRequestSubject`; and
  `CoverAssembly::from_plan` consumes that checked projection.
- **Verified — the RMS boundary still requires no behavior change here.** The
  equality guard and all of its construction and consumption paths are
  unchanged. The linked P1 still owns the unresolved one-access versus
  two-access canonical spelling, so this repair preserves the guard and changes
  only its false intrinsic-verifier explanation.

## Required work

- Re-read the complete request normalization/recognition, physical construction/verification, checked projection, assembly, staged-family, and repeated-read paths at the exact implementation base.
- Re-audit each anchor as Fact, Inference, or stale claim. Repair the ticket first if the current support boundary differs.
- Repair all six audited comment blocks. Replace retired declared-ordinal/intrinsic-verifier authority with `canonical_input_reads`, `recognize_epilogue`, the retained checked request subject, exact `AccessOrdinal` projection, and `CoverAssembly` as appropriate.
- Preserve the `value_input == weight_input` refusal. Describe it only as an unsupported coincident-operand staged-pass spelling whose canonical access/identity choice belongs to the linked decision ticket; do not claim the intrinsic verifier rejects it.
- Add a source-subject check that reaches each of the six retired phrases; perturb each independently and quote the failure.
- Preserve behavior, identities, schemas, tests, and public surfaces unless a separately accepted dependency authorizes otherwise.

## Graph

Related to the narrow pipeline-test prose repair and the completed fieldless-role documentation repair. It must not be folded into either without its own current-base source audit.

## Implementation record — 2026-08-17

- Repaired exactly the six audited comment blocks. Whole-program/prologue and
  epilogue read ordering now name compiler normalization; physical construction
  and program assembly name the retained checked request subject and exact
  access projection. The `value_input == weight_input` guard remains unchanged
  and its comment links the P1 that owns coincident-operand support.
- No executable Rust, test, fixture, assertion, public surface, identity,
  schema, pin, or supported population changed.
- The final source check rejects all six retired anchors and positively requires
  `canonical_input_reads`'s live normalization anchor, `recognize_epilogue`'s
  staged-read construction, `declared_input_at`'s projection documentation, and
  CoverAssembly's call to that accessor. Restoring each retired source phrase
  independently produced, respectively:

  ```text
  crates/tiler-compiler/src/request.rs:1843:/// whole-program elementwise region never had to distinguish them.**
  ERROR: retired declared-input authority remains
  crates/tiler-compiler/src/request.rs:7170:/// tidiness.** The pointwise access contract requires a region's declared input ordinals not to descend.
  ERROR: retired declared-input authority remains
  crates/tiler-compiler/src/physical.rs:282:/// with that same declaration; the schedule's own read-ordering rule refuses as two spellings.
  ERROR: retired declared-input authority remains
  crates/tiler-compiler/src/physical.rs:338:    // The reads in the canonical order a pointwise region requires.
  ERROR: retired declared-input authority remains
  crates/tiler-compiler/src/physical.rs:602:    /// different facts: a pointwise region's reads are the declared inputs in declaration order.
  ERROR: retired declared-input authority remains
  crates/tiler-compiler/src/physical.rs:1498:/// between the two elementwise regions this profile builds.** A whole-program or prologue region reads every declared input in declaration order.
  ERROR: retired declared-input authority remains
  ```

  Each perturbation was restored before the next; the same check then returned
  zero retired hits and all four live anchors.
- The eight named request, physical, conformance, pipeline, and IR controls each
  ran one test and passed. Compiler nextest ran 941 tests: 941 passed and one was
  skipped. Compiler all-target check and Clippy with warnings denied, rustdoc
  with warnings denied, and all 16 compiler doctests passed. Final formatting,
  ticket lint, citations, diff check, and exact-base scope guard are the closing
  gates recorded with the worker handoff.

Source check:

```sh
matches=$(rg -n -e 'whole-program elementwise region never had to distinguish them' -e "pointwise access contract requires a region's declared input ordinals not to descend" -e 'own read-ordering rule refuses as two spellings' -e 'canonical order a pointwise region requires' -e "a pointwise region's reads are the declared inputs in declaration order" -e 'A whole-program or prologue region reads every declared input in declaration order' crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/physical.rs || true)
if [[ -n "$matches" ]]; then printf '%s\n' "$matches"; printf '%s\n' 'ERROR: retired declared-input authority remains'; exit 1; fi
for anchor in "compiler normalization's canonical spelling" 'The staged read, then whichever declared inputs' 'Projects one local input access back to the declared program interface' 'declared_input_at(access)'; do
  if ! rg -n -F "$anchor" crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/program.rs; then printf 'ERROR: live authority anchor missing: %s\n' "$anchor"; exit 1; fi
done
```
