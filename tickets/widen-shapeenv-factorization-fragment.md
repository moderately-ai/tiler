---
id: widen-shapeenv-factorization-fragment
title: Widen the ShapeEnv fragment to nonlinear split-axis factorizations
status: awaiting-decision
priority: p2
dependencies: []
related: [implement-shapeenv-constraints, implement-shapeenv-index-bindings]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, shapes, indexing, mature-product]
claimed_from: todo
assignee: agent-ir2
lease_expires_at: 1784999471
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

## Outcome

**Research done; the decision is Tom's and the ticket is `awaiting-decision`.** The ticket states the choice "is not correctness-derived" and that the alternatives "encode different priorities", which is exactly the case AGENTS.md reserves for an atomic question rather than an autonomous call. No code changed. What follows is the evidence that makes the question decidable, including two findings that change the option set.

### Finding 1 — the ticket's enumeration candidate has no bound to appeal to

The ticket offers "bounded-domain enumeration justified by a real bound on extents". There is no such bound. `crates/tiler-ir/src/shape.rs` bounds only rank — `const MAX_SHAPE_RANK: usize = 4_096` — and a `Dim` is a full `u64` with no magnitude limit; reproducible as `grep -n "const MAX" crates/tiler-ir/src/shape.rs`, which returns only the rank. `constraint.rs` accordingly solves over `MAX_EXTENT = 2^64 - 1`. Enumeration over that domain is not a candidate, and introducing an extent bound to make it one would be a far larger contract change than this ticket contemplates.

### Finding 2 — a complete procedure with a resource budget *is* the third-status option

This collapses the option set from three to two, and it is the most important thing here.

Deciding `p == f0 * f1` with all three terms free, under interval and congruence bounds, is nonlinear integer arithmetic. With `p` determined and the factors free it is exactly "does `p` have a divisor in `[a, b]`" — bounded-divisor search over a 64-bit integer. With `p` free too, and several relations sharing symbols, it is a general nonlinear system. Bit-blasting to SAT decides it, at thousands of clauses per 64-bit multiplier and with cost that is not a function of the program's size.

Every other authority in this crate is resource-bounded, and this solver runs on the compile path. A complete procedure would therefore need a budget — and exceeding the budget yields neither *satisfiable* nor *contradictory*. So **option 1 does not avoid option 2's third status; it adds a solver in front of it.** The real question is not "complete procedure or third status" but "is the solver worth building, given that the third status has to exist either way".

That reframing matters because option 2's stated cost — "making every consumer handle a third case" — is unavoidable under option 1 as well, and is only avoided under option 3.

### Finding 3 — the environment already carries the fact that would make many real cases decidable

The ticket's option 3 says binding a factor "makes the tile size a compile-time parameter of the region rather than a runtime one". The environment already models that distinction and does not currently use it for fragment membership.

`crates/tiler-ir/src/shape/env.rs` gives every `RootBinding` an `AvailabilityPhase` alongside its `BindingSource`, validated against `BindingSource::earliest_phase()`. `BindingSource::CallerParameter` is documented as "A value supplied by the caller at compilation **or** launch" — one source class spanning both sides of the specialization boundary. Determination in `check_fragment` reads only `Resolved::Known`, which comes from a literal, an `Equal` against a literal, or `BindingSource::StaticValue`. A caller parameter that is in fact supplied at compile time is treated exactly like one supplied at launch.

So `128 == outer * tile` where `tile` is a compile-time caller parameter is refused today, and is arithmetically identical to the in-fragment `128 == 8 * outer` once the value is substituted. Closing that gap needs no nonlinear reasoning at all — it needs a specialization step that substitutes compile-time-available bindings before deciding, and it makes the substituted values part of artifact identity, which is the consequence option 3 already says must be stated.

This is not a fourth option so much as the mechanism option 3 was missing. It also bounds how much option 1 or 2 would actually buy: they are only needed for factorizations with two or more terms that stay unknown until *launch*.

### The question

**Should a factorization with two or more launch-dynamic terms be refused, or admitted with an explicit undecided status?**

A concrete program. A caller splits a dynamic axis where neither the tile size nor the outer count is known before launch:

```text
input:  x with shape [p]            p  from InputMetadata, LaunchPreflight
params: outer, tile                 both CallerParameter, LaunchPreflight
constraint: p == outer * tile
```

- **Refuse (extend today's rule).** `build` rejects with `UnderdeterminedFactorization`. Every environment that builds is decided, and `satisfiable` keeps meaning what it means now. The frontend must make one of `outer` or `tile` compile-time available, which specializes the artifact on that value. Enables: one meaning of "decided", no third case in any consumer, no solver on the compile path. Prevents: a single artifact serving a fully dynamic split; the caller pays an artifact per tile size.
- **Admit as undecided.** `build` succeeds and reports a third outcome distinct from satisfiable and contradictory. Enables: a fully dynamic split reaches the compiler, which may still reject it later on other grounds. Prevents: the current guarantee that a built environment is decided — every consumer must handle the third case, and the contract's normative "contradictory semantic constraints reject the graph" becomes "reject when we could tell", which is a weaker promise that has to be written down as such.

**Recommendation: refuse, and separately close the phase gap from Finding 3.** The evidence is that the refusal's cost is much smaller than the ticket assumes once compile-time caller parameters are treated as determined — that change alone admits the split-with-static-tile case that motivates most of layer 0's factorization constraints, with a procedure that is still provably complete. Paying either a nonlinear solver or a workspace-wide third status to also serve the fully-launch-dynamic split is a large, irreversible widening of what "decided" means, bought for a case no current frontend requirement names. Specializing on tile size is also normal for kernel compilation and usually wanted for performance.

**Counterpoint, stated plainly.** If Tiler is meant to ship one artifact that serves arbitrary runtime shapes without recompilation — a reasonable product goal that no accepted decision currently rules out — then the refusal is a real limit and the third status is the honest way to represent it. Choosing refusal now does not preclude the third status later, but it does mean consumers get written against a two-outcome contract and would all need revisiting.

**If refusal is chosen, this splits into:** a ticket making compile-time-available bindings determined for fragment membership (with the artifact-identity consequence recorded), and no change to the arithmetic fragment at all.
