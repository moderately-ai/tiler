---
id: implement-shapeenv-constraints
title: Implement the ShapeEnv constraint environment and contradiction check
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
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

## Outcome

**Done.** `crates/tiler-ir/src/shape/env/constraint.rs` implements the constraint environment and its decision procedure; `crates/tiler-ir/src/shape/env.rs` owns its storage, lifecycle, and identity. Full repository gate green.

**The five relation kinds, each carrying provenance.** `ExtentRelation` has exactly the five the contract names — `Equal`, `Divisible`, `NonNegativeDifference`, `Interval`, `Factorization` — over `ExtentTerm`, which is a declared symbol or a literal extent and deliberately not an expression tree. `SemanticInputConstraint` pairs a relation with the `FactProvenance` the symbol half already defined.

Nonnegativity needed a decision the contract does not make: every extent is a `u64`, so *unary* nonnegativity of an extent is a tautology and would have been a relation kind that asserts nothing. It is carried as a **difference** — `minuend - subtrahend >= 0` — which is what a slice, pad, or window precondition actually asserts and the only form of the kind that constrains anything.

**The chosen fragment, and why it is that one.** Bounded interval–congruence constraints over equality classes of declared symbols, closed under non-strict comparison, plus factorizations with at most one undetermined term. The contract leaves the fragment open, but it also makes contradiction rejection normative, and a procedure that missed contradictions would answer *satisfiable* for a set the contract calls invalid — the silently weaker answer. So the fragment was narrowed until the procedure is **complete on it**, rather than widened until it was convenient.

Completeness is by exhibited model, not by absence of refutation. Equality classes merge first; so do the strongly connected components of the `>=` graph, since a `>=` cycle forces equality. What remains is a DAG. Each class carries an interval and a modulus, both exact meets (intersection, least common multiple). Lower bounds propagate forward along the DAG and are raised to the next multiple of the class modulus. Every raise is implied, so a class whose lower bound passes its upper bound is genuinely unsatisfiable; and when none fails, assigning every class its propagated lower bound satisfies every congruence, every interval, and every edge at once. Saturating arithmetic at `2^64` is exact here rather than approximate: a bound or modulus above the extent domain admits only zero within it, which is what the unsaturated value admits too.

**What is outside, and what happens to it.** A factorization with two or more undetermined terms is nonlinear, and no interval–congruence propagation decides it. It is `ShapeEnvError::UnsupportedRelation` carrying `FragmentViolation::UnderdeterminedFactorization` — refused, not admitted and under-decided. "Determined" means the term's equality class holds a constant, from a literal, from an `Equal` against a literal, or from a `BindingSource::StaticValue` root binding; it is deliberately *not* read off a narrowed interval, so fragment membership is a syntactic, order-free property of the environment rather than a consequence of how far propagation happened to get. `128 == 8 * outer` is therefore in-fragment and solved to `outer == 16`; `n == a * b` with all three dynamic is refused.

**Contradiction is decided at `build`.** ADR 0071 makes the consuming `build` the whole-object verification point precisely so that holding a verified value is sufficient evidence. A separate checked step would reintroduce the second pass that rule exists to remove, and would leave a `ShapeEnv` in existence that the contract calls invalid; every consumer would then have to prove the step ran. Root bindings participate: a symbol bound to a static value enters the system as a constant, so a constraint contradicting a statically known extent is rejected here rather than surviving into index lowering.

**Semantic input constraints and variant guards are separate types, not one list with a flag.** The contract gives guards their own provenance vocabulary — "storage-applicability, schedule-applicability, target-compatibility, or dispatch-safety" — so `VariantGuard` carries `GuardApplicability` where `SemanticInputConstraint` carries `FactProvenance`. Neither can be passed where the other is expected. Their outcomes differ as the contract requires: one unsatisfiable relation recorded as a constraint fails `build`, and the identical relation recorded as a guard builds successfully and is reported by `ShapeEnv::unsatisfiable_guards()`. An *undecidable* guard does reject the environment, which is a different case from a failing one — a relation outside the fragment leaves the variant's selectability unknown, and treating unknown as satisfiable is the answer the contract forbids.

**Identity.** Constraints fold into the existing encoder after the entries, length-framed, in the canonical order `build` establishes. Nothing derived is stored at all — `unsatisfiable_guards()` recomputes — so no solver cache can reach identity by omission. Guards are excluded: they are not semantic constraints, and two environments describing the same program must not differ in identity because a planner recorded predicates for optimizations it was considering. The domain tag moves to `tiler.shape-env.v2` because the bytes are now a function of a larger subject; no durable reader observed `v1`, the module being `pub(crate)` and unreachable outside `tiler-ir`.

**Provenance cannot be rewritten.** There is no constructor, setter, or conversion that re-records a constraint under different provenance, and `build` deduplicates only exact `(relation, provenance)` repeats. Two assertions of one relation under different provenance stay two constraints; merging them would decide which reason survived, and whichever way it fell would rewrite one fact's provenance. That is the enforceable form of "inferred or proven facts may not silently become additional frontend-required semantics" for a module that does not yet infer facts.

**Draft status, unchanged.** Still `pub(crate)` under ADR 0074 convention 7; nothing crossed a public boundary. The module-level `dead_code` allow was widened to name the constraint half and stays accurate — `implement-shapeenv-index-bindings` is what makes index lowering read this authority.

**Evidence.** Ten new tests (twenty-one in the shape module overall), each naming the contract clause it enforces rather than the implementation. Three are specifically about the decision being a decision: `the_decision_covers_the_whole_constraint_set` builds a contradiction that belongs to no single relation — each is individually satisfiable and only the comparison chain carrying a pinned bound into a divisibility-tightened interval refutes them; `a_comparison_cycle_meets_the_facts_of_both_symbols` checks that `a >= b` with `b >= a` meets both symbols' congruences rather than treating the comparisons as independent bounds; and each contradiction test is paired with its satisfiable neighbour, so a rejection is evidence about the constraints rather than a refusal of the kind.

**Measured.** `uv run --locked python scripts/check_repository.py` passes on macOS arm64 at the pinned nightly. `cargo clippy -p tiler-ir --all-targets -- -D warnings` clean.

**Retracted during the work.** An initial reading that the ticket's base commit was not an ancestor of the worktree HEAD was wrong — the ancestry test had been run in the reverse direction, and `main` was ahead of the worktree rather than divergent from it.

**Not claimed.** This is implemented and tested at the crate-internal boundary; it is not an accepted public interface, and no consumer reads it yet. Widening the fragment to nonlinear factorizations with two dynamic factors — the split-axis case where neither the outer count nor the tile size is static — is deferred to `widen-shapeenv-factorization-fragment`, filed rather than implied.
