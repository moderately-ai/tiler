---
id: represent-semi-affine-index-expressions-in-the-ir
title: Represent semi-affine index expressions in the IR
status: todo
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
