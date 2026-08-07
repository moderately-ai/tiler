---
id: admit-symbolic-index-expression-coefficients
title: Admit symbolic coefficients to index expressions
status: done
priority: p1
dependencies: []
related: [admit-live-extent-operands-to-payload-indexing, promote-the-symbolic-index-profile-to-a-public-boundary, admit-semi-affine-index-expression-class]
scopes: [implementation/ir, contracts/foundation, implementation/reference, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, shapes, public-boundary]
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

## Outcome — delivered 2026-08-07 at `8326ce93`, closed late

**This ticket's work landed on 2026-08-07 and the coordinator failed to close it at the time** — the acceptance node was written and closed, and this implementation ticket was left `in-progress` with a live claim for several hours. Found by the worker on `repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them`, which hit it as a live `contracts/foundation` collision, checked whether it was concurrent editing, and correctly diagnosed stale ticket state instead: the deliverables were already ancestors of its base and the `docs/ir.md` closing condition was already satisfied. Recorded rather than quietly re-statused, because a ticket reading `in-progress` over landed work is exactly the drift that makes the board untrustworthy.

**What landed.** A declared `ShapeSymbol` may be an index expression's coefficient or addend. `SourcedIndexInteger` and `IndexRegionBuilder::sourced_linear_combination` were added; `LinearTermRef::coefficient` widened from `&IndexInteger` to `&SourcedIndexInteger` — the only widening. The identity domain stepped `tiler.index-region.v9 → v10`, with five law-chain pins and the standard Metal identity, cache subject and byte count recomputed and ledgered.

**Two decisions worth re-reading before building on this.** Normalization **declines every fold** over a symbolic coefficient — never merged, dropped at a pinned zero, distributed, or unwrapped at a pinned one — because folding when the environment happens to pin a value would make canonicalization a function of the binding. And a symbolic **addend rides as the term `symbol * 1`** rather than occupying the constant slot, which must stay exact because literal constants from any operand fold into it; so `LinearCombination { constant }` did **not** widen, and the accepted surface is narrower than this ticket's text implies.

**Accepted by Tom on 2026-08-07** under [`accept-the-symbolic-index-coefficient-surface`](accept-the-symbolic-index-coefficient-surface.md), conditional on the deferred capability being captured on the board — discharged by [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md), which has since **landed and refuted the ground this ticket recorded for declining it**: a `ShapeEnv` holds no values, so an environment-derived bound is a fact about the region rather than about a binding. The decline was an unexamined exception, not a principled line, and the exception is now closed.

**Residual drift this ticket's landing caused, now owned elsewhere:** six documentation sites still say a bound symbol cannot be an index coefficient or addend. Filed as [`correct-the-symbolic-coefficient-era-index-vocabulary-claims`](correct-the-symbolic-coefficient-era-index-vocabulary-claims.md), which names the trap — the literal wording survives for `SourcedExtent` while the claim it supports does not, so find-and-replace produces true-but-misleading sentences.
