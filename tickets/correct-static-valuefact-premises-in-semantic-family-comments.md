---
id: correct-static-valuefact-premises-in-semantic-family-comments
title: Correct static-ValueFact premises in semantic family comments
status: in-progress
priority: p2
dependencies: [repair-the-shape-records-after-sourced-semantic-result-shapes]
related: [correct-the-slice-normative-definition-and-recompute-compiler-identities]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, correction]
claimed_from: todo
assignee: w-terra-static-valuefact
lease_expires_at: 1786211727
---

Three correctness-bearing, non-identity IR comment clusters outside the
research/navigation repair still generalize a literal-only operation-family
boundary into a false claim that no semantic value fact can carry a symbol.

## Starting evidence, stale until re-read at this ticket's base

- `crates/tiler-ir/src/semantic/concatenate.rs`, module anchors `occurrences still carry static` and `Every extent a semantic occurrence can carry` say every occurrence is static. Its function documentation at
  `Extent agreement runs through the accepted three-outcome path` repeats the
  premise. `ValueFact` is now source-bearing, while Concatenate itself still
  requires every operand to pass `static_operand_shape`; the first sourced
  operand refuses before the exact-sum helper runs.
- `crates/tiler-ir/src/semantic/contraction/tests.rs`, anchor `The unresolved
  outcome remains unreachable`, says `ValueFact` carries a static `Extent` and
  no fact can carry a symbol. The unresolved equality outcome remains
  unreachable for this family because `impl OperationInferencer for StrictTensorContractionF32`
  narrows each operand through `static_operand_shape` before that operand's
  extents reach binding or equality; no symbolic equality/unresolved-requirement
  rule is defined there yet.
- The prior Slice record is deliberately excluded. Its named
  `SLICE_F32_NORMATIVE_DEFINITION` is identity-bearing and belongs to
  [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md),
  which owns any change to those bytes and downstream compiler-pin
  recomputation; re-read whether it remains stale at this base.

The three owned comment clusters are the implementation follow-up
reviewed clean on `repair-the-shape-records-after-sourced-semantic-result-shapes`
at `cb21e6a640872083feb418d7f73ee84b8800f8ce`. Treat that as discovery evidence:
read both complete files, current inference/construction sites, relevant tests,
and the full predecessor ticket before editing; repair this ticket first if the
facts or population have moved.

## Per-Fact audit at base `2e82d9a7179ce8b880147c22da598db2d2c8e1a7`, before any edit

| Starting Fact | Verdict | Evidence read at this base |
| --- | --- | --- |
| The ticket described two owned comments, with Concatenate named only by `Every extent a semantic occurrence can carry`. | **false as a population count; verified as drift.** The full file has two Concatenate clusters: the module cluster also says `occurrences still carry static`, and the `concatenate_result_shape` documentation repeats the premise after `Extent agreement runs through the accepted three-outcome path`. | `crates/tiler-ir/src/semantic/concatenate.rs`, complete file; anchors `occurrences still carry static`, `Every extent a semantic occurrence can carry`, and `Extent agreement runs through the accepted three-outcome path`. |
| `ValueFact` is source-bearing, while Concatenate asks `static_operand_shape` before deriving its exact sum. | **verified.** `ValueFact::shape` is `SourcedShape`; the inferencer requires every operand to pass the host literal-only accessor, and its collection returns the first refusal before `concatenate_result_shape`. | `crates/tiler-ir/src/semantic/operation.rs`, anchors `pub struct ValueFact` and `pub fn static_operand_shape`; `crates/tiler-ir/src/semantic/concatenate.rs`, anchor `let shapes: Vec<&Shape>`. |
| The Contraction test at `The unresolved outcome remains unreachable` says a fact carries a static `Extent` and no fact can carry a symbol. | **verified as drift.** The comment is live, but `impl OperationInferencer for StrictTensorContractionF32` narrows each operand before that operand's extents reach binding or equality. | `crates/tiler-ir/src/semantic/contraction/tests.rs`, complete file and anchor `The unresolved outcome remains unreachable`; `crates/tiler-ir/src/semantic/contraction.rs`, anchors `impl OperationInferencer for StrictTensorContractionF32` and `let shape = request.static_operand_shape(position)?`. |
| Slice has a third stale record inside `SLICE_F32_NORMATIVE_DEFINITION` and is excluded because its bytes are identity-bearing. | **false as a live-drift claim; verified as an exclusion.** The identity-bearing constant now says `semantic value facts generally can carry sourced extents` and identifies Slice's literal grammar and `static operand shape` boundary. This ticket must not modify it. | `crates/tiler-ir/src/semantic/slice.rs`, anchors `SLICE_F32_NORMATIVE_DEFINITION`, `semantic value facts generally can carry sourced extents`, and `NormativeDefinitionRef::new(SLICE_F32_NORMATIVE_DEFINITION)`. |

The audit does not change this ticket's purpose. Its complete owned population
is **three live non-identity comment clusters**: the two Concatenate clusters
above and the one Contraction-test cluster. The two sentences in the
Concatenate module cluster are one contiguous comment cluster, so they are
corrected together. Other uses of `static` in the two complete files are either
literal-only family explanations, specific workload values, or
identity-bearing strings; none repeats the false global `ValueFact` premise.

The owned edits do not flow into `OperationDefinition` identity: Concatenate's
identity-bearing values are `CONCATENATE_F32_NORMATIVE_DEFINITION` and
`concatenate_facts()`, passed to `NormativeDefinitionRef::new` and
`OperationDefinitionFacts::new` respectively; neither is edited. The remaining
Concatenate changes are Rust documentation comments, and the Contraction change
is documentation attached to a `#[test]`; neither is encoded into an operation
definition.

## Outcome

- Correct the three non-identity comment clusters so each states its family-specific
  literal-only boundary without denying source-bearing semantic facts.
- Preserve Concatenate's exact literal sum and Contraction's currently
  unreachable unresolved-equality outcome; change their stated cause, not
  behavior.
- Re-audit nearby module/test comments for the same live premise and name the
  population checked. Do not edit Slice normative bytes on this branch.
- Do not change inference, schemas, identities, public surfaces, or tests except
  a correctness-bearing comment assertion if a current one directly consumes
  the corrected prose.

## What closes this

The complete owned population is corrected or supported with literal
source-safe anchors. Package check, nextest, doctests, Clippy and rustdoc with
warnings denied, formatting, `make citations`, `tkt lint`, `git diff --check`,
and exact-base `tkt guard` pass. If any edited string reaches identity, stop and
map it instead of recomputing pins under this ticket.
