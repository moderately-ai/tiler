---
id: represent-semi-affine-index-expressions-in-the-ir
title: Represent semi-affine index expressions in the IR
status: closed
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [indexing]
closed_reason: superseded
closed_note: Requirements consolidated into the coherent sourced-extent and semi-affine public-boundary draft.
---
## Decision needed (2026-07-28)

**The question, atomic:** how does `IndexExprView` express a *symbolic* divisor?

`FloorDiv` and `Modulo` currently expose `divisor: u64` **by value** (`crates/tiler-ir/src/index/model.rs:673` and `:680`). `IndexExprRef::class()` and `IndexExprRef::view()` are the public accessors that return them, so nothing here can be staged `pub(crate)` — the shape is public the moment it exists.

| | 1 — new variants | 2 — change `divisor` to a symbolic type |
| --- | --- | --- |
| **Enables** | `FloorDivSymbolic`/`ModuloSymbolic` beside the existing pair. Additive under `#[non_exhaustive]` (`model.rs:655`), so no out-of-crate consumer breaks. Keeps the affine subset expressible in a type that cannot carry a symbol, so a pass that only handles affine divisors rejects the symbolic case structurally. | One pair of variants, one shape, every consumer handles both cases by construction. |
| **Prevents** | Permanently doubles the divisor vocabulary: every recognizer, every lowering, every cost model matches four arms where two would do, and a reader must know which pair a given divisor lives in. | A **breaking field change**, which `#[non_exhaustive]` does not soften: a consumer reading `divisor` by value stops compiling. Turns the structural affine/semi-affine distinction into a runtime check. |

**Recommendation: option 2**, on the ground that the repository has no external consumers and `breaking-changes-are-allowed` is the standing position, so the cost option 1 exists to avoid is not a cost that is being paid — while the vocabulary it doubles is paid forever, by every future recognizer.

**Counterpoint that deserves weighing:** option 1 keeps the affine subset expressible in a type that cannot carry a symbol, which is a real property. A cost model or a lowering that only handles affine divisors can accept `FloorDiv` and reject `FloorDivSymbolic` structurally, where option 2 makes that a runtime check. If that structural distinction is wanted, option 1 is not merely the cheaper choice.

### What option 2's divisor type would actually be

The candidate spelling that first suggests itself does not work, and saying why narrows the decision.

```rust
// Rejected. `divisor: &'a IndexInteger` — the borrowed-exact-integer precedent.
FloorDiv { dividend: VerifiedIndexExprId, divisor: &'a IndexInteger }
```

The **precedent is real**: `LinearTermRef::coefficient` already returns `&'a IndexInteger` (`crates/tiler-ir/src/index/model.rs:711`), and `IndexExprView::Constant` and `LinearCombination::constant` both borrow one, so the view already speaks this shape. The **conclusion does not follow**: `IndexInteger` is `pub struct IndexInteger(pub(super) BigInt)` (`crates/tiler-ir/src/index/integer.rs:34`) — an exact signed mathematical integer, with no symbol case and no way to add one without changing what the type means everywhere it is already used. Widening the divisor to `&'a IndexInteger` buys unbounded magnitude and buys *nothing* toward a symbolic divisor. It is the wrong axis.

```rust
// The shape the job needs: constant-or-symbol, exactly two cases.
FloorDiv { dividend: VerifiedIndexExprId, divisor: SourcedDivisor<'a> }
// where SourcedDivisor is `Static(Extent) | Symbol(ShapeSymbol)`-shaped, as `SourcedExtent` already is.
```

**That type already exists, crate-internally.** `SourcedExtent` is `Static(Extent) | Symbol(ShapeSymbol)` at `crates/tiler-ir/src/index/sourced.rs:138`, and its own documentation states the property this needs: "Deliberately two cases and not an expression tree. A composed extent is a relation in the environment's constraint set, where it can be decided, rather than arithmetic the index layer would have to re-derive." It is `pub(crate)`, and publishing it is a decision owned by `promote-the-symbolic-index-profile-to-a-public-boundary` (decision 3 there). **So option 2 is coupled to that ticket**: either it publishes `SourcedExtent` and this reuses it, or this mints a second constant-or-symbol type in the index layer, which is the duplication that ticket exists to avoid. That coupling is the concrete cost of option 2 and it is not visible from the two-line summary above.

**What would shrink the counterpoint.** Option 2's structural loss is only as large as the cost of asking. A cheap `divisor.as_constant() -> Option<u64>` on whichever type is chosen turns "reject a symbolic divisor" into one `let … else` at the head of an affine-only pass, and the loss is then a missing compile-time guarantee rather than a missing capability. Deciding how much of the counterpoint is real means deciding whether an affine-only pass rejecting at run time with a typed diagnostic is acceptable where a non-matching variant would have rejected at compile time. That is the sharpest form of the question in front of the owner.

**Reserved because the ticket says so:** "Tom reviews the boundary." Both options are implementable and neither is blocked by evidence.

## Background

Split from `admit-semi-affine-index-expression-class`, which landed `ShapeEnv::proves_positive` and settled the two questions this work would otherwise have to re-litigate. Read its Outcome section first.

## What is already decided, so do not reopen it

**Divisor positivity is a semantic input constraint, never a variant guard.** `ShapeEnv::proves_positive` reads `self.constraints` and never `self.guards` (`crates/tiler-ir/src/shape/env.rs:960`), and a two-sided test pins that — the same relation proves positivity when required and does not when guarded. `x floordiv 0` has no meaning under any plan, so positivity is a condition of the expression being *defined*, which is the semantic-constraint side of the discriminator `shape/env/constraint.rs` draws.

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
