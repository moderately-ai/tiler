---
id: correct-the-slice-normative-definition-and-recompute-compiler-identities
title: Correct the Slice normative definition and recompute compiler identities
status: in-progress
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims, pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach]
related: [admit-a-position-selecting-slice-for-the-rotary-table]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, correction]
claimed_from: todo
assignee: w-sol-slice
lease_expires_at: 1786209191
---
## Why this exists

`SLICE_F32_NORMATIVE_DEFINITION` still says the semantic layer can only carry static shapes and that no source-bearing slice reaches the index layer. The first premise is false after sourced semantic result shapes landed. Unlike nearby comments and diagnostics, this string is identity-bearing and cannot be corrected as an isolated prose edit.

## Starting evidence, stale until re-read at this ticket's base

- `crates/tiler-ir/src/semantic/slice.rs`, anchor `SLICE_F32_NORMATIVE_DEFINITION`, contains the stale bytes and registers them through `OperationDefinition::new`.
- `crates/tiler-ir/src/semantic/registry.rs`, anchor `fn encode_operation_definition`, frames the normative definition into both the reached-definition projection and the registered-operation snapshot.
- Compiler tests and pins consume semantic-program identities and the standard-registry subject. The exact affected population must be enumerated from construction sites and failures on this ticket's merged base; do not copy a count or value from this ticket.
- The current behavior to describe is narrower: `SliceAxisSelection::Window` has `offset: u64`; `decode_axis` rejects `symbolic-window` before inference; `SliceF32::infer` later calls `request.static_operand_shape(0)` for literal bound checking. General `ValueFact` shape carriage is sourced.

The worker's first deliverable is a per-Fact verdict at the exact base, including the complete ticket, `slice.rs`, registry encoding, all compiler consumers and pins, and the identity-domain census this ticket depends on. Repair this ticket before editing if the propagation graph or affected population differs.

## Outcome

Correct the normative definition so it states the actual Slice-family boundary without claiming all semantic values are static. Recompute every reached semantic identity, registry subject, compiler expectation, fixture, and pin affected by those bytes on the merged tree. Keep the identity-domain grammar unchanged unless the encoding grammar itself changes: moving a value within an existing framed normative-definition field requires recomputation, not a separator-version step.

Add or strengthen a correctness-bearing assertion that reaches the normative bytes and identity consumers. Perturb the subject, not the assertion, and record the failure text before restoring it.

## Non-goals and stop conditions

Do not add the reserved symbolic-window relation, choose attribute versus operand carriage, change Slice inference, or widen a public boundary. If correcting the text exposes a need to change the semantic encoding grammar rather than its value, stop and map that domain step separately. If an affected identity has no owning pin authority, stop and create it rather than updating an ad hoc expectation.

## Closes when

The normative bytes match current behavior; every affected identity and pin is enumerated and recomputed on the final merged base; unaffected domains are justified; subject perturbations prove the checks reach both the definition and its compiler consumers; package checks, doctests, Clippy, rustdoc with warnings denied, formatting, `tkt lint`, citations, and `tkt guard` pass.
