---
id: decide-the-producer-derived-logical-group-public-boundary
title: Decide the result-binding and producer-derived logical-group public boundary
status: awaiting-decision
priority: p1
dependencies: []
related: [group-internal-compound-materializations-by-logical-value, admit-strict-affine-quantize-physical-candidate, accept-the-partitioned-result-binding-boundary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity-domain, compound-values]
---

## Decision requested

**Only Tom resolves this ticket.** Choose the exact public, proof-derived boundary by which `tiler_ir::program` can bind a complete internal compound materialization to the semantic operation result that produced it. The decision must name the included and excluded `tiler_ir::program` surface, the artifact projection, the stage-accounting shape, and every identity/schema domain that moves. No implementation ticket may fill those gaps by minting a caller-chosen identifier or reconstructing a group from roles, shapes, or slot order.

This is a P1 prerequisite because the current tree contains the proof before the program boundary, but not across it:

- [`IndexRefinementReceipt::result_bindings`](../crates/tiler-ir/src/index/refinement.rs) retains the checked ordered semantic-result-to-output-root relation. Its source anchor is `Returns ordered result-to-output bindings.`
- That relation does not yet admit a compound result. [`bind_results`](../crates/tiler-ir/src/index/refinement.rs), anchored by `the region's distinct output *tensors*`, requires one distinct output tensor per semantic result. `ResultBinding` carries result ordinal, output tensor, write access, and written scalar value, but no component role; `two_distinct_output_tensors_still_disagree_with_one_result` holds the present refusal. Partition members work only because several roots write the same tensor, so their accepted grouping rule cannot be reused as component grouping by implication.
- [`CoveredOccurrence::from_receipt`](../crates/tiler-ir/src/program/model.rs) retains only the graph identity, occurrence, and reached-only executable-coverage identity. Its source anchor is `graph: receipt.graph().clone(),`; no result binding remains available to the program builder or its readers.
- [`MaterializedComponentSpec`](../crates/tiler-ir/src/program/model.rs) declares a component role and physical facts, but no producer relation. [`KernelProgramBuilder::check_origin`](../crates/tiler-ir/src/program/builder.rs) therefore reaches `UngroupedInternalComponent { role }` for every internal temporary component.
- [`MaterializedValueData`](../crates/tiler-ir/src/program/model.rs), artifact [`BindingData`](../crates/tiler-artifact/src/program/model.rs), and the decoded binding view carry a component role but no logical-group identity. The artifact builder's `BindingTargetData::Internal` projection cannot derive the missing semantic relation.

## Included surface to decide

The accepted answer must fix all of these together, because choosing one constrains the others:

- the opaque proof-derived record, handle, or other checked association representing one semantic operation result and its complete internal logical materialization;
- the `ResultBinding` cardinality and role vocabulary required to bind several distinct component output tensors to one semantic result without weakening the accepted partition-member meaning, followed by the minting path from a completed `IndexRefinementReceipt` into the program;
- how the semantic result's complete `ResolvedValueType`, logical shape, ordered component declarations, and parameter-map contracts reach that verifier and the program builder without becoming caller declarations;
- the minimum public constructors and borrowed readers needed by `tiler-compiler` to build the program and by `tiler-artifact` to project the verified program, with an explicit list of fields and methods that stay private;
- the cardinality and completeness rules for one result with several component roles, including the typed refusals for a missing, duplicate, extra, swapped, wrong-type, wrong-shape, or cross-result component;
- whether a compound result is written by one widened kernel stage or by several component-writing stages, and the exact program declaration that accounts for the chosen topology;
- the allocation, view, access, lifetime, output, and artifact-binding readers that retain the group while continuing to treat component storage independently;
- canonical ordering, population limits, and the program/artifact/codec identity grammar, including the required `PROGRAM_DOMAIN`, `ARTIFACT_DOMAIN`, and `MANIFEST_SCHEMA` steps and affected pins.

The stage-accounting choice cannot be split into a sound independent implementation ticket yet. [`verify_signature`](../crates/tiler-ir/src/kernel/verify.rs) currently derives exactly one `write_buffer` with the anchor `let (write_buffer, read_buffers) = data`, while [`verify_stage_accounts`](../crates/tiler-ir/src/program/verify.rs) admits exactly three declared reasons for an empty coverage set. Giving one occurrence to several component-writing stages would otherwise collide with occurrence coverage, and leaving later component writers uncovered would require a fourth account. Whether to widen the kernel signature or introduce a checked multi-stage result declaration depends on the very result-to-logical-group association this ticket decides.

## Explicit exclusions

- No caller-supplied integer, handle payload, resolved type, role list, or parameter map may create or complete a logical group.
- No association may be inferred from component order, equal shape, equal type, adjacent allocation, or the presence of familiar roles.
- The artifact and codec remain projections of the verified program, never a second semantic authority.
- The accepted shape must be generic over ordered encoded-component declarations; it may not embed a fixed code/scale/zero-point struct or otherwise make strict affine the universal vocabulary.
- This ticket does not implement physical `Quantize`, choose its kernel topology, broaden parameter-map support, or reopen the already-working compound interface-input/output path.
- Process-local `ValueId` and materialized arena ordinals are lookup capabilities, not sufficient serialized identity by themselves.

## Evidence and alternatives to compare

The decision packet must compare at least these families against correctness, maintainability, and identity size:

1. widen `ResultBinding` with an explicit component role/cardinality rule, extend `CoveredOccurrence` with a sealed result-binding projection, and derive logical groups from it;
2. widen refinement with a separate sealed compound-result binding and mint a logical-result materialization receipt from the same completed authority;
3. retain the necessary checked association in a compiler-owned assembly product while exposing only proof-derived program-builder operations, with an explicit account of why that does not duplicate IR-owned semantic authority.

For each, prove that the artifact can read everything it must encode without gaining construction authority, and that a new result-affecting field cannot be omitted silently from either independent encoder. Reuse the accepted one-binding-per-output-root meaning in [`accept-the-partitioned-result-binding-boundary`](accept-the-partitioned-result-binding-boundary.md); do not reinterpret `ResultBinding::result` or infer a component role that the current result binding does not carry.

## Closes when

Tom accepts, accepts with named exclusions, or rejects one exact boundary. Record who, date, venue, and the complete included/excluded surface. On acceptance, update [`group-internal-compound-materializations-by-logical-value`](group-internal-compound-materializations-by-logical-value.md) with the chosen derivation, public draft/acceptance handling, stage-accounting rule, and identity/schema steps before moving that ticket back to dispatchable work.

## Source audit — 2026-08-08, base `8259cef4a8962c5f42ae41bf79a4fe53d2a70238`

**Verified.** The sources named above were read in full, together with [`builder.rs`](../crates/tiler-ir/src/program/builder.rs), [`verify.rs`](../crates/tiler-ir/src/program/verify.rs), the artifact program builder/model/codec files, and accepted ADRs [0030](../docs/decisions/0030-first-class-quantized-values.md), [0070](../docs/decisions/0070-own-shared-compiler-ir-in-tiler-ir.md), [0071](../docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md), [0072](../docs/decisions/0072-separate-semantic-meaning-from-provider-provenance.md), [0074](../docs/decisions/0074-use-explicit-public-api-conventions.md), and [0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). The missing compound-result binding, receipt-to-program relation, public-boundary choice, stage topology, and identity/schema consequences are jointly real; none can be chosen mechanically from current slot order.
