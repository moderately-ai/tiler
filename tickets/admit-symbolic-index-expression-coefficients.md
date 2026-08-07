---
id: admit-symbolic-index-expression-coefficients
title: Admit symbolic coefficients to index expressions
status: in-progress
priority: p1
dependencies: []
related: [admit-live-extent-operands-to-payload-indexing, promote-the-symbolic-index-profile-to-a-public-boundary, admit-semi-affine-index-expression-class]
scopes: [implementation/ir, contracts/foundation, implementation/reference, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, shapes, public-boundary]
claimed_from: todo
assignee: agent-symbolic-coeff
lease_expires_at: 1786110590
---
## User-visible outcome

A frontend can write an index expression whose coefficient or addend is a declared `ShapeSymbol` — `i + (S - T)`, `p * B` — so the symbolic half of the admitted semi-affine vocabulary is expressible and not only the divisor half.

## Why this exists: the previous owner was superseded, and it carried both halves

**Fact at `d5c02609`.** [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md) admits "affine, constant-divisor quasi-affine, and guarded semi-affine expressions with symbolic coefficients **or** proven-positive symbolic divisors", and [`docs/ir.md`](../docs/ir.md)'s bounded initial vocabulary admits "multiplication by a parameter-only expression". Only the divisor is implemented. `IndexRegionBuilder::floor_div` and `::modulo` take a `SourcedExtent` and return `IndexExprClass::SemiAffine`; `IndexRegionBuilder::linear_combination` takes an `IndexInteger` constant and `IndexInteger` coefficients, and no other constructor admits a `SourcedExtent` into an expression. A `ShapeSymbol` therefore reaches an expression at exactly one position.

**Fact — no ticket owned this before now.** `represent-semi-affine-index-expressions-in-the-ir` carried both coefficients and divisors and was closed `superseded` into `promote-the-symbolic-index-profile-to-a-public-boundary`. That ticket's user-visible outcome names "proven-positive semi-affine coefficients and divisors", but its Implementation outcome names only the divisor, `IndexExprClass::SemiAffine`, and the sourced-extent vocabulary; it was accepted on 2026-07-31 on exactly that surface. The coefficient half was not delivered and not re-split, so it fell out of the graph. This ticket restores it, filed from the `docs/ir.md` labelling work that found the gap.

`admit-live-extent-operands-to-payload-indexing` is a different gap and not a substitute: it owns a live extent reaching a **compiled payload's** address and loop arithmetic. This one owns the IR's ability to **represent** the expression at all.

## What to do

- Admit `SourcedExtent` as a `LinearCombination` coefficient and as its constant term, reusing the crate's one constant-or-symbol vocabulary rather than minting a second. A symbolic coefficient makes the expression `IndexExprClass::SemiAffine`, which already exists and is already handled at every internal classification site.
- Decide, and state, what normalization means when a coefficient is symbolic. The current `accumulate_linear_term` folds terms by exact integer arithmetic and drops zero coefficients; neither is available for a symbol whose value the environment does not pin, and an approximation here would be a canonicalization that silently changes a program. Declining to fold is the expected answer, but it is a decision with an identity consequence and must be made deliberately.
- Interval propagation must **decline** rather than approximate, exactly as the symbolic divisor does: `interval_linear` cannot bound a term whose coefficient is unknown, and an interval nothing proved is worse than `None` — a `None` falls through to a proof that either closes another way or is retained as an explicit obligation.
- Refuse an undeclared symbol, a symbol past `EXTENT_PHASE_CEILING`, and a foreign environment through the existing `ExtentSources::admit` path, so a refused coefficient leaves the draft exactly as it was.

## Public boundary

**This is a public-boundary change and Tom reviews it.** `LinearTermRef::coefficient` returns `&'a IndexInteger` and `IndexExprView::LinearCombination` exposes the constant the same way; both are reachable through the public `IndexExprRef::view()`. Widening either to a sourced type changes an accepted surface. Note that positivity is *not* the relevant admission predicate here — a coefficient may be any sign, unlike a divisor — so `ExtentSources::proves_positive` is not the check to reuse, only `admit`.

Identity moves: an index-region domain step is required, because a coefficient would encode as a tagged `SourcedExtent` where it now writes an exact integer. Recompute pins on the merged tree.

## Required evidence

- One region expresses `i * B` and `i + S` with `B` and `S` declared symbols, is verified, and carries `IndexExprClass::SemiAffine`.
- The same region with the symbol undeclared, past the phase ceiling, or from a foreign environment refuses with its own typed cause, and each refusal is observed failing once before restoration.
- An analysis that needs an interval over a symbolic coefficient declines with a named reason rather than producing a bound.
- Two regions differing only in whether a coefficient is the literal `4` or a symbol pinned to `4` have different identities, matching the graph-versus-specialized distinction the sourced-boundary work established.
- Targeted `tiler-ir` tests, per-package Clippy, rustdoc, and `make full` pass.

## Closes when

A symbolic coefficient and addend are expressible through an accepted public surface, normalization and interval behaviour are decided and tested rather than inherited, every refusal is fail-capable, the identity domain step lands with recomputed pins, and `docs/ir.md`'s implemented-extent paragraph is updated to say the coefficient half landed.
