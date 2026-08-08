---
id: repair-downstream-records-after-sourced-semantic-results
title: Repair downstream records after sourced semantic results
status: done
priority: p1
dependencies: [repair-the-shape-records-after-sourced-semantic-result-shapes]
related: [repair-the-records-the-sourced-semantic-shape-falsifies]
scopes: [research/program-planning, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, correction]
---

Three research records outside the shape-record ticket's scopes retain the
former general boundary that `ValueFact` stored a fixed `Shape`. Repair them as
dated corrections while preserving each record's family-specific or
artifact-specific conclusion where it still follows.

## Starting evidence, stale until re-read at this ticket's base

- `docs/research/program-planning/flash-class-capability-set.md`, anchor `the
  growing context axis is a symbolic extent`, says a semantic `ValueFact`
  carries a static extent and the three-link delivery chain is entirely
  `todo`. The reviewed shape repair found `ValueFact` source-bearing and the
  resolve ticket `awaiting-decision`; planning still has later boundaries.
- `docs/research/program-planning/complete-model-ingestion-and-execution.md`,
  anchors `The count survives because the C1 row's shapes are inferred
  results` and `still thirteen as of 2026-08-08`, retain fixed `ValueFact`, the
  removed `SymbolicOperandUnsupported` name, and stale delivery status. Its
  separate broadcast-attribute and graph/artifact-identity argument must be
  re-derived rather than discarded with those premises.
- `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md`
  contains the literal test-name anchor
  `the_reduced_extent_is_always_literal_so_no_symbolic_refusal_can_fire`. A
  source-bearing semantic result is constructible generally, while the current
  Softmax family still refuses a sourced operand through its own
  `static_operand_shape` path.

These three records are the research half of the six-remainder census reviewed
clean on `repair-the-shape-records-after-sourced-semantic-result-shapes` at
`cb21e6a640872083feb418d7f73ee84b8800f8ce`. That census is discovery evidence,
not authority: the worker must read all three records, the complete current
construction/refusal sites, and the relevant tickets before editing, and must
repair this ticket first if any characterization or population is false.

## Per-Fact audit at `7f0a73bfd3289bbb61336b2433fc87b495ffa737`

- **Flash premise — false.** Its anchor `the growing context axis is a symbolic
  extent` says `ValueFact` is static and every delivery link is `todo`.
  `pub(super) shape: SourcedShape` in `semantic/operation.rs` and the
  `symbols and all` comment in `SemanticProgramBuilder::push_operation` refute
  the former; `resolve-semantic-shape-inference-over-symbolic-extents` is
  `awaiting-decision`, while the frontend-construction and compiler-request
  tickets are `todo`.
- **Complete-model premise — false in its mechanism; conclusion re-derived.**
  Its anchor `The count survives because the C1 row's shapes are inferred
  results` retains fixed `ValueFact`, `BuildError::SymbolicOperandUnsupported`,
  and `in-progress`. The cited conclusion remains conditional on the actual
  family and frontend boundaries: `static_operand_shape` in Contraction,
  Softmax, and Broadcast, together with literal broadcast attributes; identity
  writes both input and result `SourcedShape`s through `shape.encode(&mut
  bytes);`.
- **Numerics test-name premise — false as a current-source claim.** The old
  `the_reduced_extent_is_always_literal_so_no_symbolic_refusal_can_fire` text
  remains retired prose, but the source test is now
  `a_symbolic_reduced_extent_is_refused_and_every_literal_one_infers` and
  exercises Softmax's `static_operand_shape` refusal. Its two further live
  premises, `The growing symbolic extent was *not* reached` and `it turned out
  to be unreachable rather than hard`, are the same false general-carriage
  claim and are counted separately below.
- **Census — verified.** The completed shape repair names these three records
  as the research half of its six-record remainder census.

## Live correction population

Five sites need dated correction: the Flash growing-context paragraph; the
Complete-model C1-count correction; and the Numerics symbolic-refusal block,
capability-table row, and ladder conclusion. Historical premises remain as
dated text; the preserved conclusions must name their family, frontend,
broadcast, artifact, or identity ground rather than general result carriage.

## Outcome

- Date each retired premise beside the claim that used it. State that
  `ValueFact` and `push_operation` preserve `SourcedShape`, and locate remaining
  refusal or non-delivery at the actual family, frontend, compiler, artifact,
  or identity boundary.
- Preserve valid conclusions only after re-deriving them. In particular, do not
  infer that the C1 artifact count moved merely because general result carriage
  did; re-read the broadcast attribute/identity argument and every family in
  the path.
- Correct stale ticket statuses from the work graph rather than carrying the
  reviewed branch's snapshot.
- Make no implementation, identity, schema, public-surface, or support-rung
  change.

## What closes this

Every live fixed-`ValueFact`/general-symbolic-refusal premise in the three full
records is counted and corrected or supported; conclusions are separately
classified; all source anchors are literal fixed-string fragments; `make
citations`, `tkt lint`, `git diff --check`, and exact-base `tkt guard` pass.
