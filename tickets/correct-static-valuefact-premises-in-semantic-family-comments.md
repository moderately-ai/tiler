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

Two correctness-bearing IR comments outside the research/navigation repair
still generalize a literal-only operation-family boundary into a false claim
that no semantic value fact can carry a symbol.

## Starting evidence, stale until re-read at this ticket's base

- `crates/tiler-ir/src/semantic/concatenate.rs`, anchor `Every extent a
  semantic occurrence can carry`, says every occurrence is static.
  `ValueFact` is now source-bearing, while Concatenate itself still asks
  `static_operand_shape` for each operand before deriving its exact sum.
- `crates/tiler-ir/src/semantic/contraction/tests.rs`, anchor `The unresolved
  outcome remains unreachable`, says `ValueFact` carries a static `Extent` and
  no fact can carry a symbol. The unresolved equality outcome remains
  unreachable for this family because `StrictTensorContractionF32::infer`
  narrows through `static_operand_shape` before binding and comparing indices;
  no symbolic equality/unresolved-requirement rule is defined there yet.
- `crates/tiler-ir/src/semantic/slice.rs` has a third stale implementation
  record inside `SLICE_F32_NORMATIVE_DEFINITION`. It is deliberately excluded:
  [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md)
  owns its identity-bearing bytes and all downstream compiler pin
  recomputation.

The first two sites are the implementation half of the six-remainder census
reviewed clean on `repair-the-shape-records-after-sourced-semantic-result-shapes`
at `cb21e6a640872083feb418d7f73ee84b8800f8ce`. Treat that as discovery evidence:
read both complete files, current inference/construction sites, relevant tests,
and the full predecessor ticket before editing; repair this ticket first if the
facts or population have moved.

## Outcome

- Correct the two non-identity comments so each states its family-specific
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
