---
id: name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set
title: Decide the stable diagnostic key for a materialized reduction prologue
status: done
priority: p3
dependencies: []
related: [admit-a-recognized-chain-more-than-one-materialization-boundary-deep, admit-a-materialized-producer-in-a-serial-reduction-contributor]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

A caller receives a stable rule that truthfully classifies the materialized producer a reduction prologue cannot retain, without pretending the compiler has distinguished a producer kind it discarded.

## Current-base Fact audit — 2026-08-12, base `0a67f558`

- **Verified:** `recognize_elementwise` has one production caller, `recognize_reduction`; output recognition invokes `plan_elementwise` directly and consumes `Folded(ValueId)` by constructing an epilogue.
- **Verified:** `impl From<ElementwiseRefusal> for RequestError`, anchored by `Flattens a discovered materialization boundary into the rule a caller`, is the only flattening path. It discards the value and previously returned `operation-set`.
- **Verified:** `materializes_its_result` includes strict serial reduction, strict tensor contraction, and every family whose registered law realizes a region sequence. `Folded` retains no producer-family class.
- **Verified:** `NormalizedSerialSum` retains an optional pointwise prologue but no producer relation. The missing fact is therefore that the reduction contributor crosses a materialization boundary, not that the producer belongs to one particular family or necessarily forms a chain.
- **Verified:** `CompileFailureClass::UnsupportedCapability { rule }` exposes `rule` as a stable caller-visible diagnostic key. Changing the key is an intentional public observed-value change.
- **Verified:** the affected production regression population includes the nested-reduction fixture in `request.rs`, nested reduction and contraction fixtures in `materialized_intermediate_epilogue_wall.rs`, and another nested fold/contraction fixture in `contraction_direct_path.rs`. A registered staged-family contributor reaches the same flattening arm and was missing as a direct control.
- **Verified:** the same fold over a declared-input expression remains accepted, while a genuinely unrecognized operation remains `operation-set`; those are distinct neighboring subjects.
- **Verified:** the rule is not encoded into request, schedule, KIR, artifact, cache, or executable identity. No schema or identity-domain step is owed.

## Decision — accepted 2026-08-12

Tom accepted one stable rule named `reduction-contributor-materialization`. It names the exact failed relation: the serial-reduction contributor walk reached a recognized value that must cross a materialization boundary, while `NormalizedSerialSum` has no producer relation to retain it.

The rule deliberately does not name the producer family. A nested reduction, contraction, and registered staged family are separate subjects exposing the same missing relation. Producer-specific keys would mix cause with subject and force every future materializing family to widen the public reason vocabulary. If callers later need the producer identity, that belongs in structured diagnostic or explain subject data, not in the stable cause key.

## Included implementation

- Replace only the flattened `ElementwiseRefusal::Folded` mapping with `reduction-contributor-materialization`.
- Update the complete affected regression population and add the missing staged-family control.
- State the distinction from `operation-set` in the optimizer contract.
- Keep the declared-input neighbor admitted and genuine vocabulary gaps classified as `operation-set`.
- File admission of the missing producer relation separately.

## Excluded

No program becomes admissible. This change adds no producer field, schedule form, fallback, request field, schema, identity input, or producer-specific reason key.

## Outcome — 2026-08-12

`reduction-contributor-materialization` now classifies every serial-reduction contributor walk that reaches a recognized materialized producer. Nested reduction, contraction, and staged-family subjects share the key; the declared-input neighbor remains admitted; genuine vocabulary gaps remain `operation-set`. The separate admission ticket owns representing the missing producer relation.

## Verification — 2026-08-12

`cargo nextest run -p tiler-compiler --test materialized_intermediate_epilogue_wall --test contraction_direct_path --test recognized_chain_depth_boundary` passed 15 tests. `cargo nextest run -p tiler-compiler every_refusal_names_its_unrecognized_property` and `cargo nextest run -p tiler-compiler gather_is_absent_from_the_real_request_recognition_operation_set` each passed their exact unit subject. `cargo check -p tiler-compiler`, package Clippy with warnings denied, and package rustdoc with warnings denied passed.

Two independent subject perturbations were observed failing. Replacing the production rule with `PROBE-reduction-contributor-materialization` failed the nested-reduction assertion with `left: "PROBE-reduction-contributor-materialization", right: "reduction-contributor-materialization"`. Replacing the refused fixture's `folded_prologue(true)` with its accepted `folded_prologue(false)` neighbor failed before string comparison because recognition returned `Ok(SerialSum(...))`. Both perturbations were reverted before the green runs.
