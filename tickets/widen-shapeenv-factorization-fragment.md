---
id: widen-shapeenv-factorization-fragment
title: Widen the ShapeEnv fragment to nonlinear split-axis factorizations
status: todo
priority: p2
dependencies: []
related: [implement-shapeenv-constraints, implement-shapeenv-index-bindings]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, shapes, indexing, mature-product]
---
`implement-shapeenv-constraints` landed the constraint environment with a stated decidable fragment. This is the one case that fragment refuses and that a mature product will need.

**Fact — the boundary that landed.** `crates/tiler-ir/src/shape/env/constraint.rs` admits a `Factorization` relation when at most one of its terms is undetermined, where determined means the term's equality class holds a constant from a literal, an `Equal` against a literal, or a `BindingSource::StaticValue` root binding. Two or more undetermined terms is `FragmentViolation::UnderdeterminedFactorization` and rejects the environment.

**Fact — why it was drawn there.** `docs/ir.md` leaves "the solver algorithm and exact supported arithmetic fragment" an implementation choice but makes contradiction rejection normative: "contradictory semantic constraints reject the graph." A procedure that missed contradictions would answer *satisfiable* for a set the contract calls invalid. The fragment was therefore narrowed until the interval-congruence propagation is provably complete on it, and `p == a * b` with both factors dynamic is nonlinear integer arithmetic that no such propagation decides.

**Fact — the case this excludes is a real one.** `docs/ir.md` layer 0 requires that "composed axes have factorization constraints". A split axis whose tile size is static is in-fragment today: `128 == 8 * outer` solves to `outer == 16`. A split whose outer count *and* tile size are both caller parameters is not, and rejects.

## Scope

Decide whether the fragment widens and how. The alternatives encode different priorities and the choice is not correctness-derived, so this is research before it is implementation:

- **Widen with a complete procedure.** Bounded nonlinear integer constraints over extents are decidable in principle. Establish what algorithm decides the actual relation shapes — products of two or three symbols under interval and congruence bounds — and at what cost. Bit-blasting to a small SAT core, or a bounded-domain enumeration justified by a real bound on extents, are both candidates.
- **Widen the representation but not the decision, with an explicit typed status.** Admit the relation and make the environment report a third outcome distinct from satisfiable and contradictory. This preserves the rule that unknown never masquerades as decided, at the cost of making every consumer handle a third case.
- **Keep the refusal and require the frontend to bind a factor.** The narrowest option; it makes the tile size a compile-time parameter of the region rather than a runtime one, which has consequences for artifact identity and specialization that must be stated rather than assumed.

Whichever is chosen, record it against the contract: the fragment is named in the module documentation and in `implement-shapeenv-constraints`'s outcome, and widening it changes what "the environment decided" means to every downstream consumer.

## Closes when

The choice is made with its evidence, the contract text that names the fragment agrees with the implementation, any newly admitted relation is decided rather than approximated or is reported through an explicit third status that consumers must handle, and the repository gate passes.
