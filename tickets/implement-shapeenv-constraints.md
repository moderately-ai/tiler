---
id: implement-shapeenv-constraints
title: Implement the ShapeEnv constraint environment and contradiction check
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, shapes, indexing, mature-product]
---
`implement-shapeenv-core` landed the scoped symbol and typed root-binding half of the ShapeEnv authority. This is the other half the contract names, split out rather than stubbed.

**Fact — `docs/ir.md`, constraint and proof context.** "Semantic and index lowering share a typed `ShapeEnv` containing scoped symbol declarations, source bindings, and a constraint environment containing extent equalities, divisibility, nonnegativity, intervals, and factorization relationships." The first two are implemented; the constraint environment is not.

**Fact — the same section makes contradiction rejection normative.** "Contradictory semantic constraints reject the graph." A constraint set that stores relations without deciding contradiction would be a type-system reservation wearing the name of an implemented authority, which is why it was not stubbed alongside the symbol half.

**Fact — identity already excludes derived state and must keep doing so.** "Canonical identity includes symbol declarations, root-binding provenance, and semantic constraints but excludes derived solver caches." `ShapeEnv`'s current encoder covers declarations and bindings; constraints must fold in and nothing derived from them may.

**Fact — the contract deliberately leaves the solver open.** "The solver algorithm and exact supported arithmetic fragment remain implementation choices." So the decidable fragment is this ticket's to choose and to state, not to inherit.

## Scope

Add the five relation kinds over declared symbols, each carrying the `FactProvenance` the symbol half already defines. Decide and state the supported arithmetic fragment and what happens outside it — the contract's discipline requires an explicit rejection rather than a silently weaker answer, so an unsupported relation is refused rather than ignored.

Decide explicitly whether contradiction detection runs at `build` or as a separate checked step, and state the reason. Running it at `build` keeps the ADR 0071 rule that a verified product is trustworthy without a second pass; deferring it would mean a `ShapeEnv` exists that the contract calls invalid.

Preserve the distinction the contract draws between a **semantic input constraint** — required for the expression to be defined, whose failure is an invalid-input diagnostic — and a **variant guard**, required only for a particular optimization, whose failure selects another plan. They are explicitly "not interchangeable", and a constraint environment that stored both in one list would erase that at the point it matters most.

Also preserve: "inferred or proven facts may not silently become additional frontend-required semantics." A `StaticallyProven` fact must not be recorded as `FrontendRequired` by any path.

## Closes when

The five relation kinds are representable over declared symbols with provenance, contradiction is decided with its timing stated, the supported fragment is stated and unsupported relations reject explicitly, constraints participate in canonical identity while derived state does not, semantic input constraints stay distinguishable from variant guards, and `uv run --locked python scripts/check_repository.py` passes.
