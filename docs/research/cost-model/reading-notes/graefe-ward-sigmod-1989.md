---
schema: "tiler-doc/v1"
id: "tiler.research.cost-model.reading-note.graefe-ward-sigmod-1989"
kind: "research"
title: "Reading note: Graefe and Ward, Dynamic Query Evaluation Plans (1989)"
topics: ["cost-model", "optimizer", "parametric-plans", "prior-art"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.optimizer"]
ticket: "close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward"
---

# Reading note: Graefe and Ward, Dynamic Query Evaluation Plans (1989)

Distilled from the full text of the byte stream pinned as `graefe-ward-sigmod-1989` in [the source record](../sources/README.md#graefe-ward-sigmod-1989), read 2026-08-07. That row carries the citation, the digest, the provenance, and the licence verdict; this note carries what the paper says and what it settles for Tiler. Nothing here is redistributed from the document beyond the quotations shown.

**Why this note exists.** [The measured-feedback tuning loop](../measured-feedback-tuning-loop.md) named its choice of the [`AvailabilityPhase`](../../../decisions/0043-use-typed-phased-target-feasibility.md) ladder over a run-time `choose-plan` operator as one of two conclusions thinner than the rest. This is the paper that introduced `choose-plan`, and until 2026-08-07 the repository knew it only through two later papers' descriptions of it.

## What the paper proposes

The problem is a query compiled once and run many times, whose predicate contains program variables unbound at compile time. **Fact.** The paper's framing: "The value of the program variables is not known at compile-time, i.e., when the query is optimized", so the optimizer guesses, and "In the case of complex queries with inequality constraints involving program variables, the resulting query evaluation plan can be far from optimal."

Its answer has two halves. The first is an optimality *test*: "a special predicate is associated with each compiled query evaluation plan", evaluated on the actual constants, returning TRUE to proceed or FALSE to re-invoke the optimizer. The second, which is the contribution the literature kept, is the *dynamic plan*: ship several plans in one access module and pick among them when the module is activated.

**The decision is precomputed, not re-derived.** This is the property that distinguishes the 1989 design from its 1994 successor. The optimizer's job is to locate break-even points at compile time and ship them: "The task of the query optimizer is to determine the break-even point. More exactly, the query optimizer must determine the formulas to find the break-even point and to compare it with the actual selectivity, and include these formulas in the query evaluation plan." For joins the same idea becomes a tree — the optimizer "must design an efficient decision procedure which can be executed when the query evaluation plan is activated. This decision procedure must have resolved the interdependence of partial decisions into a straight-forward decision tree, and include the break-even values between alternative plans." At run time, "the first step is to evaluate the decision tree."

**How break-even points are located, which matters for what later papers say about this one.** The construction is two-stage. Ranges are split at discontinuities first: "When a cost function is invoked, it first determines whether the cost increases 'smoothly' over the range of input sizes, or whether the cost function shows a significant discontinuity. If necessary, the range is split at points of discontinuity, and a choose-plan operator is introduced into the plan." Only then does search run, and only inside a smooth subrange: "If two methods with smooth cost functions show overlapping cost ranges … the optimizer invokes the cost function repeatedly to determine the break-even point of the two plans, and optimizes the subranges separately." The worked example is hash-join overflow — "The cost function of hash join may indicate a discontinuity at the point where temporary files must be used to resolve hash table overflow."

## Measurement, and what is only an estimate

**Measurement (theirs, on a Sequent Symmetry with 16 MHz Intel 80386 CPUs, 3,600 KB of file-system buffer, a 13,229-record utility-billing site table and a 77-record city table).** Two plans were compared across output sizes — B-tree scan with index nested-loops join against file scans with hash join — where cost is "the sum of the CPU time in user mode and the estimated time spent performing I/O". The paper states the extremes: "If only one account satisfies the predicate, the index strategy is superior by a factor of 10. If all or almost all accounts qualify, it is inferior by a factor of more than 3." The tabulated totals bracketing those claims are 0.429 s against 4.563 s at one tuple printed, and 36.257 s against 10.713 s at 13,229. Bounded to that prototype and that host, and to a two-plan choice.

**The run-time cost of the decision itself is asserted, not measured.** The paper estimates a compiled optimality predicate "requires probably in the range of 20 to 100 instructions", and that where a dynamic module buys nothing "the decision tree can be an empty function … and 'evaluating' the decision tree costs only one instruction." **Inference.** These are design assertions. No measurement in the paper isolates decision-procedure overhead, so the 1989 design's headline advantage over its successor — a cheaper start-up — is argued rather than demonstrated.

## What this settles for Tiler

**The candidate mechanism the design record was looking for does not exist in this paper.** The record named one way reading it could change a conclusion: if the 1989 minimal decision procedure were cheaper or more general than its successors describe, it would be a candidate for resolving a deferred cost row at a later availability phase *without carrying alternatives*. **Refuted, on the clause that mattered.**

- **Cheaper at start-up: yes.** Comparing bindings against precomputed break-even values runs no cost function at start-up. The 1994 successor moved cost evaluation into start-up and paid 5.8 s of start-up CPU for its largest dynamic plan.
- **Avoids carrying alternatives: no, and the paper says so.** "In addition to the decision tree designed by the optimizer, the access module must also contain the support functions for all possible query evaluation plans. … The physical organization of the access module must allow equally efficient execution of any of the query evaluation plans." It concedes this does not scale — "it is not possible to include all query execution plans in the access module" — and offers a heuristic subset ("plans with wide range of optimality must be selected", the criterion deferred to "a later paper") or component-level sharing, which is 1994's DAG idea in embryo.
- **More general: no, strictly less.** The 1989 paper carries **no** optimality guarantee. The guarantee that makes the line interesting — that no plan omitted from a dynamic plan could be optimal for any binding — is 1994's contribution, and it is exactly what a heuristic subset gives up.

**Consequence.** The availability-phase conclusion is strengthened rather than weakened: the origin of the choose-plan line carries the same artifact-size and retention obligation as its successor **without** the guarantee that was the obligation's only justification. **This note changes no conclusion in the design record.**

## Two secondary characterisations, checked against the primary

Both were carried in the source record as explicitly-flagged second-hand claims so they could be checked when the document arrived.

**Cole and Graefe's "left two all-important questions unanswered" — confirmed, and understated if anything.** The paper's own §7 is titled *Current Work* and lists both as open. On the compile-time optimizer: "We are currently designing and implementing modifications to the EXODUS Optimizer Generator … that enable generated optimizers to optimize queries with free variables", and "The next step will to allow designation of free variables in queries." On which decisions to delay, it states the property without a procedure — "Their advantage is that they allow delaying exactly as many choices as advisable" — and defers the selection criterion by name to a later paper. The experiment is the strongest evidence: the module was hand-built ("We constructed a multi-plan access module containing the state records and support functions needed to execute this query"), and the decision procedure was never exercised, because "we 'forced' the choose-plan operator to use a plan of our choice, disreganding the threshold amount" *(OCR damage in the source)*.

**Cole and Graefe's minimal-decision-procedure attribution — right about the goal, wrong about the mechanism.** The phrase "minimal decision procedure" appears nowhere in the 1989 paper, and neither does "inverse" or any cognate (`pdftotext -layout` over the pinned copy matches `invers` zero times). The goal attribution is sound: a decision tree of break-even values precomputed at compile time. The mechanism attribution is not. The 1994 rejection — "it requires building inverses for all cost functions", "entirely unrealistic to assume that inverses of cost functions can be provided" — refuses analytic inversion, which the 1989 paper never asks for. Its stated method is repeated *forward* evaluation. **Numerical root-finding on a forward cost function is not an inversion of it**, so the successor rejects a stronger requirement than the predecessor stated.

**Markl et al.'s "simple binary-search techniques as in [GW89] will not work" — the citation is real, the premise about the paper is wrong.** The paper does propose a binary search: "The first experiment will involve a binary search scheme similar to the one proposed for XPRS [10]." But it scopes it immediately — "only a very limited number of variables can be dealt with in this way, probably only one or two" — and it handles non-smoothness *outside* the search, by splitting ranges at discontinuities before any search runs. Markl et al.'s objection names a real gap in the wrong place: **the paper never says how the optimizer determines smoothness or locates the discontinuities**, asserting only that the cost function "first determines" this. That silence is the genuine weakness, and it is not what the quoted sentence claims.

## Recorded, not acted on

**The break-even construction is POP's, fifteen years earlier.** Split ranges at discontinuities, then locate a crossover within a smooth subrange by repeated forward evaluation, is structurally what `markl-pop-sigmod-2004` does by capped Newton-Raphson on `cost(Palt, c) – cost(Popt, c) = 0`. Neither paper cites the other for it. This changes nothing — Rule 2 declines to derive validity regions from an analytic model for reasons that apply to both constructions — but a reader tracing where derived validity regions come from should know the idea predates the paper the repository credits.

**The paper states a measured-feedback ambition.** §7.2 closes: "In a later stage, we intend to augment the selectivity estimation procedures and cost functions with observations from query evaluations. … Notice that only dynamic query evaluation plans allow incorporating cost function adjustments in existing access modules effectively, i.e., without recompilation." The acknowledgements credit Guy Lohman for "the importance of learning cost functions from run time observations". **This does not contradict the design record's position that a choose-plan *decision* is not a measured-feedback mechanism**, and the two must not be conflated: the decision remains an analytic comparison in both papers, and the claim here is only that dynamic plans are a convenient host for coefficients learned elsewhere. It is also an unimplemented 1989 prospectus whose stated benefit — updating cost functions "without recompilation" — is worth nothing to a compiler whose model is ahead-of-time recompilation.
