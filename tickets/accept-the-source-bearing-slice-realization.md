---
id: accept-the-source-bearing-slice-realization
title: Accept the source-bearing Slice realization
status: awaiting-decision
priority: p1
dependencies: []
related: [preserve-source-bearing-slice-offsets-through-index-refinement, accept-the-literal-offset-slice-realization-law]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the widened Slice-law interpretation so a source-bearing window is `t + C` through the accepted sourced-addend vocabulary rather than a typed refusal.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes a change to an already-accepted public law's included/excluded set to Tom. [`accept-the-literal-offset-slice-realization-law`](accept-the-literal-offset-slice-realization-law.md) accepted `IndexRealizationLaw::Slice` on 2026-08-11 with symbolic offsets **excluded**. [`preserve-source-bearing-slice-offsets-through-index-refinement`](preserve-source-bearing-slice-offsets-through-index-refinement.md) grew that interpretation at `f903da13fab2f3f51bad57b5ef8bdb98a725a642` without bumping the law or provider revision. This node is not implementation work. Only Tom closes it.

## The surface, as landed at `f903da13`

**Included, added to the 2026-08-11 acceptance.**

- A source-bearing `Window` realizes as `t + C` through `sourced_linear_combination(C, [(1, t)])`. The canonical relation stores constant `0` and the terms `C * 1` and `t * 1`. There is no second cursor input and no resolved-value rewrite.
- Law and governed lowering attach the subject's exact `ShapeEnv` only when `SliceSelection::names_a_symbol()` is true. Literal windows keep the environment-free `d + offset` path, so their region identities do not move.
- `SliceSelection::names_a_symbol` is an additive query on the existing labelled-draft selection type.
- Total-access verification discharges the bound from that retained environment. An unproved interval remains `InsufficientFacts` and does not mint a verified receipt.

**Unchanged from the 2026-08-11 acceptance.**

- `IndexRealizationLaw::Slice { selection_attribute }`, `slice_f32()`, append-only tag `13`, slice law revision 1, and the standard row for `tiler::slice-f32@1`.
- Literal `Window { offset, .. } -> d + offset` and `WholeAxis -> d`.
- Identity output writes and no scalar operation.

**Still excluded.**

- Strided windows.
- View-versus-copy planning, scheduled-region vocabulary, physical planning, and backend realization.
- A live-extent payload carrier. A verified source-bearing region remains non-executable (`compile.unsupported.strategy.operation-set`).
- A law or provider revision bump. Previously admitted literal subjects keep their bytes.
- Self-acceptance.

## Recommendation

Accept as drafted. The encoding did not change; only previously refused source-bearing subjects now realize, and they do so through the already-accepted `SourcedIndexInteger` addend vocabulary. **Strongest counterpoint:** the 2026-08-11 acceptance named symbolic offsets as excluded, so growing the interpretation at revision 1 means two compilers that share tag 13 / revision 1 now disagree on whether those subjects realize.

## Closes when

Tom accepts, accepts with named exclusions, or revises. Do not treat the implementation merge as an accepted surface on this packet alone.
