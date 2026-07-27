---
id: admit-semi-affine-index-expression-class
title: Admit ADR 0046's semi-affine index expression class
status: done
priority: p1
dependencies: [harden-public-enums-non-exhaustive]
related: [bind-shapeenv-sources-into-tensor-boundaries-and-coefficients, harden-public-enums-non-exhaustive, implement-index-domain-predicates]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, shapes]
---
Split out of `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients`, which landed the boundary half and could not land this one. The reason is a public-API boundary, stated exactly below so the next worker does not rediscover it.

**Fact — what is still literal.** `crates/tiler-ir/src/index/model.rs` gives `IndexNode::LinearCombination` `IndexInteger` coefficients and gives `FloorDiv`/`Modulo` a `u64` divisor. [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md) admits more: "the initial expression vocabulary admits affine, constant-divisor quasi-affine, and guarded semi-affine expressions with symbolic coefficients or proven-positive symbolic divisors."

**Fact — the blocker is `IndexExprClass`, and it is worse than an ordinary additive change.** `docs/research/indexing/index-access-model.md` names four classes — `Affine`, `QuasiAffine`, `SemiAffine`, `DataDependent`. The public `tiler_ir::index::IndexExprClass` has two, and it carries **no `#[non_exhaustive]`**, so admitting the third is a *breaking* change for every out-of-crate matcher rather than an additive one. Reproduce with `grep -n -B3 "pub enum IndexExprClass" crates/tiler-ir/src/index/model.rs`. `IndexExprView` is `#[non_exhaustive]` and would grow additively, but its `FloorDiv`/`Modulo` variants expose `divisor: u64` by value, so a symbolic divisor needs either a new variant or a changed field type — also public. Neither can be expressed `pub(crate)`, because `IndexExprRef::class()` and `IndexExprRef::view()` are the public accessors that must return them.

`harden-public-enums-non-exhaustive` is the natural place to fix the attribute; this ticket should not land the third class before that, or it lands a breaking change to avoid a smaller one.

## Decided here, so the implementer inherits an answer rather than a question

The split ticket said this ticket owns "what a proven-positive symbolic divisor requires: the constraint environment can decide `d >= 1`, but a divisor is also a *guard* rather than a semantic constraint in some uses, and the two are explicitly not interchangeable."

**Decision: divisor positivity is a semantic input constraint, never a variant guard.** `crates/tiler-ir/src/shape/env/constraint.rs` states the discriminator the accepted contract draws: "a semantic input constraint is required for the expression to be defined and its failure is an invalid-input diagnostic, while a variant guard is required only for one optimization and its failure selects another plan." There is no other plan under which `x floordiv 0` has a meaning — `docs/research/shapes/shape-environment-contract.md` calls a zero divisor "a typed evaluation or statically detected construction/validation error". So positivity is a condition of the expression being defined, which is the semantic-constraint side of that line by definition.

The consequence is mechanical and load-bearing: the positivity query must read the environment's **semantic constraints only**. `ShapeEnv::extent_interval` and `ShapeEnv::proves_equal` already do — both build their relation list from `self.constraints` and never from `self.guards` — so a `proves_positive` written against either is correct by construction, and one that folded guards in would admit an expression whose definedness rests on a predicate whose failure merely selects another plan.

**Consequence to preserve, not to work around.** The shape contract states that "a symbolic divisor crosses the affine boundary and may produce a structured `Unknown` during static proof", and `docs/research/shapes/constraint-prover-boundary.md` classes a symbolic divisor as `nonlinear` for the Presburger lane. Proving `d >= 1` is therefore *not* sufficient to make an expression carrying it analyzable: it makes the expression well defined. Interval propagation over `x floordiv d` with symbolic `d` must decline rather than approximate, and ADR 0046 already permits that — "passes may conservatively decline semi-affine maps they cannot analyze."

## Also in scope: one stale pointer

`docs/ir.md` says "`ShapeEnv`-backed root bindings, semi-affine symbolic coefficients/divisors, typed index-domain predicates, and durable solver evidence are tracked by [`implement-shapeenv-index-bindings`] and [`implement-index-domain-predicates`]". The first of those has landed its half and split the rest; the semi-affine item is tracked here now. Repoint that sentence — it is why this ticket declares `contracts/foundation`.

## Closes when

`IndexExprClass` admits `SemiAffine` without a breaking public change, a symbolic coefficient and a symbolic divisor are expressible and refused explicitly when their positivity is not proved from semantic constraints alone, a pass that cannot analyze the class declines rather than approximating it, the `docs/ir.md` pointer is current, and `make full` passes.

## Outcome — the positivity query landed; the IR representation is split (2026-07-27)

### The dependency resolved differently than this ticket assumed

This ticket expected `harden-public-enums-non-exhaustive` to mark `IndexExprClass` `#[non_exhaustive]`, so that admitting a third class would be additive. **It deliberately did not**, and the reason changes this ticket's premise rather than blocking it: `IndexExprClass` has **no out-of-crate consumer at all**. It is exported through `index/mod.rs` and matched at six sites, every one inside `tiler-ir`.

So the stated blocker — "admitting the third is a *breaking* change for every out-of-crate matcher" — has no referent. There are no out-of-crate matchers. Adding `SemiAffine` breaks exactly the six internal classification sites, at compile time, which is the wanted behaviour: every authority that must classify an expression is forced to say what it does with the new class.

**Inference: the attribute is not what this ticket needs.** What it needs is the IR to be able to *represent* a semi-affine expression, because a class nothing can produce is a type-system reservation rather than an implemented seam — a distinction `AGENTS.md` draws explicitly. Adding the variant without the representation would have landed the reservation and closed the ticket on it.

### What landed: `ShapeEnv::proves_positive`

The query a proven-positive symbolic divisor needs, with the semantics this ticket already decided:

- It reads **semantic input constraints only, never variant guards**, matching `extent_interval` and `proves_equal`. Positivity is a condition of the expression being *defined* — `x floordiv 0` has no meaning under any plan — which is the semantic-constraint side of the discriminator `env/constraint.rs` draws.
- An undeclared symbol answers `false` rather than erroring: nobody told us it is positive.
- Its doc records that proving positivity is **not sufficient to make the expression analyzable**. A symbolic divisor is nonlinear for the Presburger lane, and ADR 0046 permits a pass to decline. Positivity establishes definedness, nothing more.

**The test is two-sided and its failure path is verified.** The same relation — an extent of at least one — proves positivity when required and does not when merely guarded. Mutating `proves_positive` to fold guards into its relation list makes exactly the guard assertion fail, so the test can say no rather than passing for a reason unrelated to the property.

### Also landed

`docs/ir.md` repointed: `implement-shapeenv-index-bindings` landed its half, so the semi-affine item now names this ticket and the predicate item names its own.

## Split out

`represent-semi-affine-index-expressions` carries the remainder: `IndexNode`/`IndexExprView` gaining symbolic coefficients and a symbolic divisor, `IndexExprClass::SemiAffine` returned by the classifier, explicit refusal when positivity is not proved, and a pass declining rather than approximating. It is a public-boundary change to `IndexExprView` (whose `FloorDiv`/`Modulo` expose `divisor: u64` by value) and needs `proves_positive` — which now exists — as its input.
