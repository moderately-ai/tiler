---
id: accept-the-literal-offset-slice-realization-law
title: Accept the literal-offset slice realization law
status: done
priority: p2
dependencies: []
related: [admit-an-index-realization-law-for-the-literal-offset-slice]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
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

Only Tom could close this ticket by accepting or rejecting the exact variant,
constructor, encoding, registration, and included/excluded semantics above.
Before the decision it remained `awaiting-decision` and satisfied no dependent.

## Accepted — 2026-08-11

**Tom accepted the exact surface as presented** in the live coordination
session, via the message `sounds good, accept`. The acceptance includes
`IndexRealizationLaw::Slice { selection_attribute }`,
`IndexRealizationLaw::slice_f32()`, append-only tag `13`, and the revision-`1`
standard row for `tiler::slice-f32@1`, with exactly these semantics:
`WholeAxis -> d`, literal `Window { offset, .. } -> d + offset`, identity output
writes, and no scalar operation.

The accepted surface has no silent fallback. A missing law, malformed or wrong
attribute, wrong arity or binding, inconsistent result shape, or provider/law
coordinate mismatch remains a typed refusal. Strided offsets,
clamping, wrapping, view-versus-copy planning, scheduled-region vocabulary,
physical planning, and backend realization remain excluded. Acceptance changes
the public maturity label only; it adds no new implementation, runtime path,
identity byte, or compatibility promise beyond the already-landed pre-alpha
surface.

## Widened — 2026-08-13

Tom accepted the source-bearing growth at `f903da13` without a law or provider revision bump, in [`accept-the-source-bearing-slice-realization`](accept-the-source-bearing-slice-realization.md). A source-bearing `Window` now realizes as `t + C` through the accepted sourced-addend vocabulary. Literal windows, tag 13, and revision 1 are unchanged. The 2026-08-11 exclusion of source-bearing offsets is superseded for this law.
