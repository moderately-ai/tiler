---
id: represent-semi-affine-index-expressions-in-the-ir
title: Represent semi-affine index expressions in the IR
status: awaiting-decision
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [indexing]
---

Split from `admit-semi-affine-index-expression-class`, which landed `ShapeEnv::proves_positive` and settled the two questions this work would otherwise have to re-litigate. Read its Outcome section first.

## What is already decided, so do not reopen it

**Divisor positivity is a semantic input constraint, never a variant guard.** `ShapeEnv::proves_positive` reads `self.constraints` and never `self.guards`, and a two-sided test pins that — the same relation proves positivity when required and does not when guarded. `x floordiv 0` has no meaning under any plan, so positivity is a condition of the expression being *defined*, which is the semantic-constraint side of the discriminator `shape/env/constraint.rs` draws.

**Proving positivity does not make the expression analyzable.** A symbolic divisor is nonlinear for the Presburger lane, and ADR 0046 permits a pass to "conservatively decline semi-affine maps they cannot analyze". Positivity establishes definedness. Interval propagation over `x floordiv d` with symbolic `d` must decline rather than approximate.

**`IndexExprClass` is deliberately not `#[non_exhaustive]`.** It has no out-of-crate consumer; six matches inside `tiler-ir` are the authority. Adding `SemiAffine` therefore breaks exactly those six at compile time, which is the wanted behaviour — every site that classifies an expression is forced to say what it does with the new class. Do not add the attribute to soften that.

## What to do

- `IndexNode::LinearCombination` takes `IndexInteger` coefficients and `FloorDiv`/`Modulo` take a `u64` divisor. Admit symbolic coefficients and a symbolic divisor. ADR 0046 admits "affine, constant-divisor quasi-affine, and guarded semi-affine expressions with symbolic coefficients or proven-positive symbolic divisors".
- **This is a public-boundary change.** `IndexExprView` is `#[non_exhaustive]` and grows additively, but its `FloorDiv`/`Modulo` expose `divisor: u64` *by value*, so a symbolic divisor needs a new variant or a changed field type. `IndexExprRef::class()` and `IndexExprRef::view()` are the public accessors that must return these, so none of it can be `pub(crate)`. Tom reviews the boundary.
- Return `IndexExprClass::SemiAffine` from the classifier, and handle it at all six internal match sites rather than adding a wildcard.
- Refuse construction explicitly when a divisor's positivity is not proved from semantic constraints alone, using `ShapeEnv::proves_positive`. A typed diagnostic, never a silent approximation.
- Make at least one analysis pass decline the class rather than approximating it, and test that it declines.

## Closes when

A symbolic coefficient and a symbolic divisor are expressible; construction is refused with a typed diagnostic when positivity is not proved from semantic constraints alone; `IndexExprClass::SemiAffine` is returned and handled at every internal classification site; a pass that cannot analyze the class declines rather than approximating; and `make full` passes.

## Parked 2026-07-27 — awaiting Tom

**The question, atomic:** how does `IndexExprView` express a *symbolic* divisor?

`FloorDiv` and `Modulo` currently expose `divisor: u64` **by value**. `IndexExprRef::class()` and `IndexExprRef::view()` are the public accessors that return them, so nothing here can be staged `pub(crate)` — the shape is public the moment it exists.

Two candidates, and `#[non_exhaustive]` decides less than it looks:

1. **New variants** — `FloorDivSymbolic`/`ModuloSymbolic` beside the existing pair. Additive under `#[non_exhaustive]`, so no out-of-crate consumer breaks. It also permanently doubles the divisor vocabulary: every recognizer, every lowering, every cost model matches four arms where two would do, and a reader must know which pair a given divisor lives in.
2. **Change `divisor` to a symbolic type** — one pair of variants, one shape, every consumer handles both cases by construction. It is a **breaking field change**, which `#[non_exhaustive]` does not soften: a consumer reading `divisor` by value stops compiling.

**Recommendation: option 2**, on the ground that the repository has no external consumers and `breaking-changes-are-allowed` is the standing position, so the cost option 1 exists to avoid is not a cost that is being paid — while the vocabulary it doubles is paid forever, by every future recognizer.

**Counterpoint that deserves weighing:** option 1 keeps the affine subset expressible in a type that cannot carry a symbol, which is a real property. A cost model or a lowering that only handles affine divisors can accept `FloorDiv` and reject `FloorDivSymbolic` structurally, where option 2 makes that a runtime check. If that structural distinction is wanted, option 1 is not merely the cheaper choice.

**Reserved because the ticket says so:** "Tom reviews the boundary." Both options are implementable and neither is blocked by evidence.
