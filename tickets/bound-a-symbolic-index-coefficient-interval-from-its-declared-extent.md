---
id: bound-a-symbolic-index-coefficient-interval-from-its-declared-extent
title: Bound a symbolic index coefficient interval from its declared extent
status: in-progress
priority: p2
dependencies: []
related: [admit-symbolic-index-expression-coefficients]
scopes: [implementation/ir, implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [indexing, proofs, decision]
claimed_from: todo
assignee: agent-interval
lease_expires_at: 1786121747
---
## User-visible outcome

An access whose index expression carries a symbolic coefficient can be proved in-bounds from the symbol's own declared extent interval, instead of falling through to an unproved obligation whenever a coefficient is not a literal.

## Why this exists

**Fact — the capability was available and was deliberately declined.** [`admit-symbolic-index-expression-coefficients`](admit-symbolic-index-expression-coefficients.md) made `interval_linear` return `None` for any symbolic coefficient, so the bound is retained as `InsufficientFacts`. Its worker recorded that `ExtentSources::interval` could soundly bound such a term — a coefficient declared `B ∈ [1, 64]` yields a real interval — and that the enumeration fallback could use `determined()` exactly as `plan_divisors` already does for symbolic *divisors*. The reasoning is written into `interval_linear`'s rustdoc. This is a capability left on the table, not an oversight.

**Why it was declined, and why that is a decision rather than a default.** Deriving an interval from the environment makes an expression's cached interval — and therefore *which accesses verify* — a function of the binding rather than of the program. That is the same graph-versus-specialized identity distinction the sourced boundary exists to keep, and the one the coefficient's normalization rule already declines to cross. Crossing it for proofs while declining it for normalization would be a coherent position, but it is a different position, and it is Tom's rather than a worker's.

**The asymmetry that makes this worth asking.** The symbolic *divisor* path already reaches `determined()` through `plan_divisors`, so the crate does not treat environment-derived facts as universally off-limits in proofs. Whether coefficients should follow divisors here, or divisors should be reconsidered against the coefficient's stricter line, is the question — and answering it one way for both is likely better than leaving the two halves of one admitted vocabulary on different rules.

## What this ticket owes

- The decision stated: whether a proof may consult the environment for a bound the program alone does not carry, and if so under what retained evidence, so the answer covers divisors and coefficients together rather than only the half that prompted it.
- If admitted: `interval_linear` bounds a symbolic-coefficient term from `ExtentSources::interval`, the enumeration fallback uses `determined()` as `plan_divisors` does, and an access provable only through the environment is distinguishable in its retained evidence from one provable from the program alone. A caller must be able to tell which it got.
- If declined: `interval_linear`'s rustdoc states the decline as settled with its ground, and the divisor path's use of `determined()` is reconciled against it or explicitly carved out with a reason.

## Explicit non-goals

Not a change to normalization — folding a symbolic coefficient stays refused whatever this decides, because that rewrite changes the program's identity rather than what is known about it. Not a widening of `SourcedIndexInteger` or of any surface in [`accept-the-symbolic-index-coefficient-surface`](accept-the-symbolic-index-coefficient-surface.md).

## Closes when

The environment-in-proofs question is answered for coefficients and divisors together, the answer is implemented or the decline is recorded as settled, and no path derives a bound from a binding without its retained evidence saying so.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the producing ticket, from a remainder its worker named and reasoned about rather than silently omitted. Kept separate because it is a design expansion under a reserved boundary, not the completion of admitted work.

**Scopes `implementation/build` and `research/target-profiles` added 2026-08-07 by the worker, as scheduling metadata rather than an outcome expansion.** Retaining a fact source on each discharged index-domain assessment moves the canonical region encoding to `tiler.index-region.v11`, so every region carrying a discharged predicate re-encodes — including the wholly static standard Metal path, which names no symbol and whose every new tag therefore reads `Program`. That path's three published identities are pinned in `crates/tiler-build/src/metal_plan.rs` (`implementation/build`) and mirrored in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` (`research/target-profiles`), and both state the values as current fact. Leaving either stale would make the gate red and the ledger false, so moving them is part of landing the encoding rather than separable work. Neither scope had a live claim when it was added; the three live claims were `agent-ordinal`, `agent-crate-record`, and this one.

**These are shared pins and the coordinator must recompute them on the merged tree.** They moved from base `73ac63f4`, `main` had already advanced to `709db244` when this branch ran, and the values below are this branch's rather than the merge's.
