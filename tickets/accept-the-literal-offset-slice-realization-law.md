---
id: accept-the-literal-offset-slice-realization-law
title: Accept the literal-offset slice realization law
status: awaiting-decision
priority: p2
dependencies: []
related: [admit-an-index-realization-law-for-the-literal-offset-slice]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
# Accept the literal-offset slice realization law

## Goal

Tom decides whether to accept the exact public `IndexRealizationLaw` surface
landed as a labelled draft by
[`admit-an-index-realization-law-for-the-literal-offset-slice`](admit-an-index-realization-law-for-the-literal-offset-slice.md).

**Classification.** The delta is additive growth of an existing public
`#[non_exhaustive]` type, which [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md)
places in the category a coordinator may merge after its four gates. The
repository's operative working contract separately keeps every tested public
surface a labelled draft until Tom accepts its exact included and excluded
surface. This parked node carries that acceptance; implementation, tests, and a
merge do not satisfy it.

## Work

- Review `IndexRealizationLaw::Slice { selection_attribute }`, the public
  `IndexRealizationLaw::slice_f32()` constructor, append-only tag `13`, and the
  revision-`1` standard row for `tiler::slice-f32@1` as one surface.
- Confirm the included semantics are exactly `WholeAxis -> d` and literal
  `Window { offset, .. } -> d + offset`, with identity writes and no reached
  scalar operation.
- Confirm the excluded semantics remain strided windows, source-bearing or
  symbolic offsets, view-versus-copy planning, scheduled-region vocabulary,
  physical planning, and backend realization.
- If accepted, record Tom's decision provenance and replace the draft marker
  with the repository's accepted-public-surface form. If the shape is rejected,
  return the implementation ticket's commit for revision rather than accepting a
  nearby surface by implication.

## Acceptance

Only Tom closes this ticket after accepting or rejecting the exact variant,
constructor, encoding, registration, and included/excluded semantics above.
Until then it remains `awaiting-decision` and satisfies no dependent.
