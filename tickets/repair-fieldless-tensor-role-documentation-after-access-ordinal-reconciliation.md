---
id: repair-fieldless-tensor-role-documentation-after-access-ordinal-reconciliation
title: Repair fieldless TensorRole documentation after AccessOrdinal reconciliation
status: in-progress
priority: p1
dependencies: []
related: [reconcile-input-ordinal-region-local-and-declared-input-semantics, decide-the-source-bound-live-row-major-access-surface, repair-retired-input-ordinal-claims-in-compiler-pipeline-tests, refresh-multi-output-correctness-row-after-access-ordinal-reconciliation]
scopes: [implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [defect, documentation, access-ordinal]
claimed_from: todo
assignee: root
lease_expires_at: 1786921522
---
## User-visible outcome

The live architecture overview, program-assembly comment, and request
normalization comments describe fieldless `TensorRole::Input`, full-list
`AccessOrdinal`, and compiler-private declared-input association without
attributing declared-interface ordinals to shared schedule roles or to intrinsic
schedule verification. Two larger records discovered by the census remain
explicitly owned by the related follow-ups below.

## Exact-base Fact audit — 2026-08-16, `52aa389eebf817376bc2aa2984e91b3faecd12bd`

The three edited source files were read in full at this exact claimed base before
any source edit. `docs/architecture.md` and `crates/tiler-compiler/src/program.rs`
are byte-identical to the earlier `98669e8e` audit. `request.rs` differs by the
orthogonal structured-lowering-capability migration only: the diff replaces a
flat capability key with `LoweringCapabilitySubject` and does not touch any
access-coordinate comment or owner below. The verdicts here are therefore
current-source verdicts, not inherited line citations.

- **Verified — the accepted replacement is implemented.**
  [`reconcile-input-ordinal-region-local-and-declared-input-semantics`](reconcile-input-ordinal-region-local-and-declared-input-semantics.md), anchor `The public replacement is complete`, records the landed fieldless role and full-list coordinate. `crates/tiler-ir/src/schedule/handles.rs`, anchor `The exact position in a scheduled region's complete ordered access list`, defines public `AccessOrdinal`; `crates/tiler-compiler/src/physical.rs`, anchor `Projects one local input access back to the declared program interface`, projects that position through the retained verified request subject to private `DeclaredInputOrdinal`.
- **Verified — the architecture contract contradicts that implementation.**
  `docs/architecture.md`, anchor `distinguishes inputs by ordinal and carries none`, still assigns the retired declared-ordinal payload to the shared role while explaining multi-output limits. The live source has fieldless `TensorRole::Input`; output attribution remains a program/cover concern rather than a fact supplied by an input role.
- **Verified — one program-assembly comment repeats the retired authority.**
  `crates/tiler-compiler/src/program.rs`, anchor `separates reads of`, says the shared role distinguishes declared inputs. The live loop below it instead constructs `AccessOrdinal` from the exact read position and calls `VerifiedScheduledRegion::declared_input_at`; the comment's separate statement that two `Intermediate` reads lack an edge coordinate remains true and must be preserved.
- **Verified but incomplete as originally written — five request comments name the fieldless verifier, and three further compiler comments repeat the same retired authority.**
  `crates/tiler-compiler/src/request.rs` contains five live references to `reads_bind_boundary_tensors_in_order` (current lines 1821, 6188, 6200, 6267, and 6587; anchors remain authoritative). The function still exists, but after fieldless roles it checks boundary categories and at most one intermediate; it cannot see declared ordinals. The `BoundaryRead` comment at anchor `states the same separation from the schedule side` correctly says access position and boundary category differ, but imprecisely says `CoverAssembly::from_plan` resolves the role: assembly now resolves an exact `AccessOrdinal` through `VerifiedScheduledRegion::declared_input_at`. The other comments at anchors `the canonical spelling`, `what reads_bind_boundary_tensors_in_order admits`, `binds the pair in one canonical order`, and `states the boundary-role rules` attribute declared-input grouping, ascent, or dense-before-mapped ordering to that intrinsic verifier. Those orders are compiler-private normalization properties of `canonical_input_reads` and `record_leaf`, not shared-IR role authority.
  The required live-population census found three more present-tense compiler
  comments outside that helper-name count: `request.rs`, anchor
  ``[`TensorRole::Input`] binds at that ordinal``, attributes an opaque-call
  work count to the fieldless role rather than to the checked access projection;
  `pipeline/tests.rs`, anchor ``the `TensorRole::Input` ordinal equal``, describes
  the retired coordinate instead of the exact leaf/access-position projection;
  and the neighbouring anchor `declared input ordinals to ascend strictly`
  assigns compiler-private canonical order to the shared schedule contract.
  That 9,411-line test module entered the population only after source edits had
  begun, so its two corrections are split to
  [`repair-retired-input-ordinal-claims-in-compiler-pipeline-tests`](repair-retired-input-ordinal-claims-in-compiler-pipeline-tests.md), whose first obligation is a complete exact-base file read. This branch restored its tentative comment edits before any gate rather than claiming the reading obligation was satisfied by a partial inspection.
  Historical audit reports and the explicitly superseded ABI/research paragraphs
  remain truthful dated evidence; comments about fieldless
  `TensorRole::Intermediate` lacking an edge coordinate remain current and are
  not this defect.
  The broader documentation census also found a separate stale support-matrix
  paragraph in `docs/correctness-and-testing.md`, anchor `The multi-output row is now positive`: it still calls later-input folds unsupported and names dense declared ordinals as a contraction invariant. [`refresh-multi-output-correctness-row-after-access-ordinal-reconciliation`](refresh-multi-output-correctness-row-after-access-ordinal-reconciliation.md) owns the required complete-document audit and correction so dated measurements in that large record are not rewritten opportunistically here.

Reproduce:

```sh
rg -n 'distinguishes inputs by ordinal and carries none|separates reads of' docs/architecture.md crates/tiler-compiler/src/program.rs
test "$(git grep -c 'reads_bind_boundary_tensors_in_order' 52aa389eebf817376bc2aa2984e91b3faecd12bd -- crates/tiler-compiler/src/request.rs | awk -F: '{print $NF}')" -eq 5
rg -n -C 4 'reads_bind_boundary_tensors_in_order' crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/builder.rs
rg -n 'pub struct AccessOrdinal|Projects one local input access back to the declared program interface|fn declared_input_at' crates/tiler-ir/src/schedule/handles.rs crates/tiler-compiler/src/physical.rs
rg -n 'the `TensorRole::Input` ordinal equal|declared input ordinals to ascend strictly|\[`TensorRole::Input`\] binds at that ordinal' crates/tiler-compiler/src/pipeline/tests.rs crates/tiler-compiler/src/request.rs
test "$(rg -c 'reads_bind_boundary_tensors_in_order' crates/tiler-compiler/src/request.rs)" -eq 1
rg -n 'fn reads_bind_boundary_tensors_in_order|checks boundary categories' crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/builder.rs
if rg -n 'distinguishes inputs by ordinal and carries none|separates reads of \*declared inputs\* by ordinal|\[`TensorRole::Input`\] binds at that ordinal|reads_bind_boundary_tensors_in_order.*(canonical|admits|binds the pair|boundary-role)' docs/architecture.md crates/tiler-compiler/src/program.rs crates/tiler-compiler/src/request.rs; then exit 1; fi
```

## Required work

- Correct the one architecture paragraph without broadening or narrowing the live multi-output support claim it is explaining.
- Correct the program-assembly comment to describe exact access-position projection for inputs while retaining the independent multi-intermediate attribution refusal.
- Re-site all five request comments on compiler-private canonical read ordering, fieldless boundary-category validation, and exact `AccessOrdinal` projection. Preserve the first comment's true access-position/category separation while replacing its stale role-resolution wording. Do not rename the intrinsic helper merely to make the stale prose resolve, introduce a public declared ordinal, or move request authority into shared IR.
- Correct the opaque-call work-count comment. The two pipeline-test comments
  found by the current-source census are owned by the linked follow-up, which
  must preserve their behavioral claims while attributing declared-interface
  association and ordering to the retained checked request subject and exact
  access projection.
- Search the live `docs/` and `crates/tiler-compiler/` populations for the same retired authority, classify every hit, and add any newly found live claim to this ticket before editing it.

## Evidence and negative controls

- Existing subset, repeated-read, epilogue, and later-declared-input compiler tests remain green; this is prose repair only.
- Grep the two retired exact claims and require no live hit after repair. Separately require `reads_bind_boundary_tensors_in_order` still exists and its fieldless boundary-category documentation remains, so deletion or renaming cannot masquerade as a repair.
- Perturb one repaired request comment back to attributing declared-ordinal ascent to the intrinsic verifier and show the exact source-reading/citation check that rejects it; a grep that merely resolves the helper name is not evidence of semantic accuracy.

## Closing conditions

The named architecture overview, program-assembly comment, and request comments agree with the landed coordinate model; the broader census and its two independent remainders are recorded; targeted compiler tests plus `tkt lint`, `make citations`, and `git diff --check` pass; and no production behavior or identity changes.

## Implementation record — 2026-08-16

- Repaired the architecture, program-assembly, five request-helper, and opaque-call work-count comments against the exact fieldless-role and access-projection owners. No executable Rust changed.
- The required census found two stale comments in the 9,411-line pipeline test module and one separate stale support-matrix paragraph. Tentative pipeline comment edits were restored once the late population expansion exposed that its complete-file reading obligation had not preceded them. The two linked follow-ups preserve both discoveries with exact read-first obligations rather than silently narrowing the census.
- The final-state commands above require the intrinsic helper and its fieldless-category documentation to survive, require exactly one truthful request-file reference, and reject the retired authority phrases in this ticket's three edited live-source files. Reintroducing ``spelling `reads_bind_boundary_tensors_in_order` admits`` into `request.rs` made the final `if rg ...; then exit 1; fi` command exit 1 with `request.rs:6185`; the subject was restored and the same command returned zero.
