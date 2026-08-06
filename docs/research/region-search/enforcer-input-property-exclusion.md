---
schema: "tiler-doc/v1"
id: "tiler.research.region-search.enforcer-input-property-exclusion"
kind: "research"
title: "Enforcer input-property exclusion"
topics: ["optimizer", "search", "enforcers", "boundary-properties"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
depends_on: ["tiler.research.region-search.rewrite-search-formalism"]
ticket: "close-the-enforcer-input-property-exclusion-gap"
---

# Enforcer input-property exclusion

## What this record decides

[The rewrite-search formalism record](rewrite-search-formalism.md) mapped four of Volcano's five structures onto Tiler's and left the fifth open: the **excluding physical property vector**, which stops an enforcer's input search from re-deriving the property the enforcer was about to supply. Its Part 1 called the difference between that and the optimizer contract's "enforcer insertion is cycle-checked" a real gap. This record closes it, and the conclusion is stated first so a reader can attack the derivation rather than the verdict:

**The redundancy is unreachable in the current planner, and it is unreachable at four independent levels rather than one. The deepest of them is not that enforcers are unimplemented — it is that the excluding vector is the second property parameter of a goal-directed search whose *first* property parameter Tiler does not construct. Tiler enumerates producers bottom-up and independently of any consumer requirement, then joins and checks; there is no "enforcer's input search" for an exclusion vector to be a parameter of. If it ever becomes reachable, exclusion belongs beside the goal in the enumeration call — never as a third vector in the boundary-property system, which it would corrupt, and never in dominance, which already covers the half of the problem that has correctness content.**

The gap is therefore a deferral with a trigger, not work. The trigger and its reproducing command are in the last section, and are mirrored in [the ticket's own trigger check log](../../../tickets/close-the-enforcer-input-property-exclusion-gap.md).

Every claim about Tiler's code below was read in the tree at commit `d7b8604d` and states the command that reproduces it.

## The source, and a note on how it was read

**Fact — the Volcano paper is `metadata-only` in this survey's source record, so reading it required re-acquisition, and the re-acquired bytes are provably the ones the survey read.** [`sources/README.md`](sources/README.md#volcano-icde-1993) retains no bytes for `volcano-icde-1993` (IEEE copyright, no redistribution grant) and instead records a retrieval URL and a digest, on the stated principle that "the recorded digest pins the exact byte stream that was read when the survey's claims about that document were checked, and then discarded". That fingerprint was exercised here rather than trusted: the document was re-fetched from the recorded course-mirror URL into a temporary directory outside this repository, and its SHA-256 was `77a4930474ee3caf2e774c72d1b842190e299fd4f492ea3577f8307972cc3f5f`, which is `expected-sources.tsv`'s recorded digest exactly. **Measurement — 2026-08-06, 1 257 723 bytes, matching the recorded size.** The bytes were read and discarded; nothing is vendored, and the licence verdict is unchanged.

This is worth recording beyond its use here, because it is the first time a `metadata-only` row in this directory has been re-acquired and checked against its own fingerprint. The retrieval-fingerprint class did the job it claims to do.

## What Volcano's excluding vector actually is

**Fact — the mechanism, quoted from the retrieved text (pp. 213–214).** The paper describes an enforcer move as re-optimizing the *same* logical expression under a relaxed property vector, and then adds the exclusion:

> "During optimization with the modified physical property vector, algorithms that already applied before relaxing the physical properties must not be explored again. For example, if a join result is required sorted on the join column, merge-join (an algorithm) and sort (an enforcer) will apply. When optimizing the sort input, i.e., the join expression without the sort requirement, hybrid hash join should apply but merge-join should not. To ensure this, FindBestPlan uses an additional parameter, not shown in Figure 2, called the excluding physical property vector that is used only when inputs to enforcers are optimized. In the example, the excluding physical property vector would contain the sort condition, and since merge-join is able to satisfy the excluding properties, it would not be considered a suitable algorithm for the sort input."

**Fact — four things that passage establishes, each of which the ticket's summary compresses.**

- It is a **parameter of the search procedure**, not a component of a plan: `FindBestPlan(LogExpr, PhysProp, Limit)` plus a fourth argument the paper's Figure 2 does not show. Its sibling arguments are the goal property vector and the branch-and-bound cost limit, and the cost limit is unambiguously search state that never lands in a plan.
- It applies **only** when an enforcer's input is optimized — "used only when inputs to enforcers are optimized" — so it is scoped to one recursive call, not carried by the expression.
- Its test is that a candidate *is able to satisfy* the excluded properties, and the consequence is that the candidate "would not be considered". That is **generation-time suppression**: the plan is never built.
- Its stated purpose is that algorithms "must not be explored again" — **re-exploration avoidance**. The paper never claims the excluded plan is illegal, and it is not: `sort(merge-join)` computes the right answer. It is a duplicate that costs strictly more.

**Inference — the excluding vector is a search-efficiency device, not a correctness device.** It removes a plan that is redundant by construction from a search whose shape would otherwise generate it twice — once as `merge-join` serving the sorted goal directly, once as `sort(merge-join)` serving it through an enforcer. The paper puts its other pruning mechanism in exactly this register, saying of the cost limit that "it is important (for optimization speed, not for correctness) that a relatively good plan be found fast". Nothing in the passage makes the excluded plan wrong, and reading it as a correctness mechanism is the error this record's verdict most depends on avoiding.

## The three preconditions the redundancy requires

**Inference — decomposing the mechanism above, the redundancy exists only where all three of these hold.**

- **P1 — a goal.** The search is parameterized by a required physical property placed on an expression's *output*, handed down from the consumer. Volcano's `PhysProp`.
- **P2 — speculative enforcer insertion.** An enforcer that supplies the goal's property is generated as a *move* from the goal itself, independently of what the input can already deliver.
- **P3 — the enforcer's input searched against the relaxed goal**, by the same producer enumeration that would have served the unrelaxed goal, so that enumeration can independently rediscover a producer already delivering the property.

P3 is what makes it a redundancy rather than a mistake. The input search is not wrong to find merge-join; it is asked a question — "implement this join" — whose answer set legitimately includes a sorted producer. The exclusion is how the caller tells it that this particular answer has already been counted.

## Tiler today, read at source

### P2 is absent: no enforcer exists in code

**Fact.** `rg -n -i "enforcer" crates/ -g '*.rs'` returns 21 lines, and every one is prose: a `///` or `//!` doc comment, an `#[allow(reason = "…")]` string, or an assertion message. `crates/tiler-compiler/src/frontier.rs:585` is representative — "A second affinity is what a target profile would declare, and it is what makes transfer enforcers reachable" — a comment about a future, in a constant declaring the bounded profile's single affinity.

**Fact — the stronger form of the same claim, because a substring search over prose is weak evidence about code.** `rg -n -i "fn [a-z_]*enforc|struct [A-Za-z]*Enforc|enum [A-Za-z]*Enforc|::Enforc" crates/` returns four hits, all unrelated test-function names containing the verb *enforces* (`the_decoding_constructor_enforces_the_governed_alphabet` and three siblings). There is no enforcer function, type, enum, or variant anywhere in `crates/`.

**Fact.** [`implement-boundary-property-enforcers`](../../../tickets/implement-boundary-property-enforcers.md) is `status: deferred`, and its own body derives why: in the bounded profile every boundary contract's guarantee discharges its requirement, so there is no mismatch for an enforcer to reconcile.

**This is a contract gap and an implementation gap at once, and they are different claims.** The optimizer contract [names the enforcer family](../../compiler/optimizer.md#enforcers) — contiguous materialization, layout conversion, encoding repacking — and states that "enforcer insertion is cycle-checked". That sentence describes a mechanism the compiler does not yet have. Reading it as a description of live behaviour is what would make this gap look reachable.

### P1 is absent: the enumeration takes no property goal

This is the load-bearing one, and it survives P2 landing.

**Fact — the enumeration entry point has no property parameter.** `enumerate_frontier` (`crates/tiler-compiler/src/frontier.rs:2082`) takes `&VerifiedTargetRequest`, `&FrontierRegionSubject`, `&[&dyn PhysicalImplementationProvider]`, and `&OpaqueCallRegistry`. There is no required-property argument. Volcano's `PhysProp` has no counterpart in the signature.

**Fact — nor does the subject or the provider's context.** `FrontierRegionSubject` (`frontier.rs:1282`) carries a presentation role, the exact semantic members, the element counts of the cover-materialized intermediates the region reads, and the tensor its owning write targets. `ImplementationContext` (`frontier.rs:1079`) is exactly `{request, subject}` and is documented as "the read-only context a provider receives to propose implementations". **A provider is never told what any consumer requires of the value it will produce.**

**Fact — Tiler's `RequiredProperties` is not Volcano's `PhysProp`, and conflating them is what makes the gap look larger than it is.** `BoundaryContract`'s two sides are "*derived* from the verified region — never taken from the provider" (`frontier.rs:502-505`), and the property derivation restates it: "The typed properties are derived and never declared" (`frontier.rs:608`). So a `BoundaryRequirement` is what *this implementation needs of its own inputs*, computed from its own schedule — Volcano's "the algorithm's applicability function determines the physical property vectors for the algorithm's inputs". It is a fact derived upward, not a goal handed downward.

**Fact — the one function that would introduce a goal has no non-test caller.** `boundary::derive_child_requirements(goal, implementation)` (`crates/tiler-compiler/src/boundary.rs:1905`) is documented as "the accepted rule interface's `child_requirements`" and derives "what a region must require of one input, given the goal placed on its output". `rg -n "derive_child_requirements" crates/` returns five lines: the declaration at `boundary.rs:1905`, a test-module import at `:2004`, and three call sites at `:2627`, `:2667`, and `:2677` — all inside the `#[cfg(test)] mod tests` that opens at `boundary.rs:1994`. Nothing outside that file mentions it at all.

**Fact — the crate says so about itself.** The module-level `#[allow]` reason at `boundary.rs:3` states that what stays unconstructed is "the goal-directed surface no bottom-up enumeration reaches — `derive_child_requirements`, the standalone `encode_property_identity` the accepted memo contract's optimization key would use, and the reserved property values that remain unconstructed … which only a top-down property search or a second execution profile can produce". The source and this derivation agree, and the source was read first.

### P3 is absent: composition is a bottom-up join with a typed rejection

**Fact — the two enumerations are independent by design.** `crates/tiler-compiler/src/selection.rs:4-13` states that complete-cover enumeration "is a strictly *global* legality authority … choosing no implementation", that the per-region frontier "is a strictly *local* authority … proving no global coverage", and — the sentence that settles the shape — "**Neither depends on the other.** This module is the first authority allowed to *join* them: it takes one independently verified legal cover plus one already-enumerated implementation frontier per region".

**Fact — the join's failure mode is a rejection, not a re-search.** `satisfy_edge` (`selection.rs:1461`) binds one materialization edge to its producer guarantee and consumer requirements, calls `boundary::unsatisfied_properties` at `:1500`, and on the first undischarged property returns `BoundaryDisagreement::UndischargedHandoff` (`:1504-1510`). It does not re-enumerate the producer, and there is nothing to insert. That call is the only non-test call site of `unsatisfied_properties` anywhere: `rg -n "unsatisfied_properties" crates/` returns 31 lines, of which `selection.rs:1500` is the sole compile-path invocation, the rest being declarations, imports, doc comments, and test assertions.

**Inference.** There is no "enforcer's input search" in Tiler, and there is no input search of any kind that a required property parameterizes. The producers were enumerated before any consumer requirement was consulted, and the requirement is consulted exactly once, to accept or reject a pairing.

### P2 landing does not by itself create P3, and this is the fourth level

**Inference — the enforcer shape Tiler is heading for is *reactive*, and a reactive enforcer is structurally incapable of the redundancy.** [`implement-boundary-property-enforcers`](../../../tickets/implement-boundary-property-enforcers.md)'s restart condition, restated three times in its body and finally put in the graph on 2026-08-02, is "a compile-path provider proposes an opaque call whose contract the composing consumer refuses" — the enforcer's first case is an already-detected `UndischargedHandoff`. An enforcer inserted only where `unsatisfied_properties` returned a non-empty result cannot be inserted ahead of a producer that already guarantees the property, **because a non-empty result is the proof that the producer does not guarantee it.** The condition for insertion is the exact negation of the condition for redundancy.

Volcano's insertion is speculative: an enforcer is a move generated from the goal, offered whenever the goal names a property, and the search then discovers what the input could have done anyway. Tiler's would be a repair for a mismatch it has already measured. **The redundancy is a hazard of speculative insertion only**, which is why "enforcers land" is a necessary but insufficient trigger, and why a trigger stated as "enforcers land" would fire early and waste the reader it woke.

## Would dominance suppress the redundant plan anyway?

The ticket asks this as an alternative home for exclusion. The answer separates cleanly into two halves, and the separation is what decides the recommendation.

**Fact — plan-level dominance would suppress it from the answer.** `PlanStructuralCost::dominates` (`selection.rs:175`) is a Pareto relation over four exact structural counts: `dispatch_count`, `launched_threads`, `temporary_bytes`, and `materialization_count`. A plan that runs an enforcer ahead of a producer already guaranteeing the property is no better on any of the four and strictly worse on at least the dispatch count, so the direct plan strictly dominates it and `SelectedPortfolio::non_dominated` (`selection.rs:563`) drops it.

**Fact — but dominance is a view, so the plan is still built.** The selection module states it explicitly: "the portfolio retains every valid complete plan, and structural dominance (`SelectedPortfolio::non_dominated`) is a pure *view* that only prunes a plan another plan beats on the exact structural dimensions" (`selection.rs:36-38`). The same discipline holds one level down, where `CoverEnumeration::non_dominated` "is a pure view that prunes nothing from the retained set" (`cover.rs:36-37`). A dominated plan is enumerated, verified, given an identity, and retained in `plans()`; only the view hides it.

**Fact — region-level dominance does not apply at all.** `AdmittedImplementation::dominates` (`frontier.rs:1615`) compares implementations *of one region* — `self.boundary.subsumes(&other.boundary) && self.cost.dominates(&other.cost)` — and an enforcer step is not another implementation of the same region. It never sees the pair.

**Inference.** Dominance already guarantees the redundant plan cannot be *selected*; it cannot stop the plan being *built*, because it runs on plans that have been built. Volcano's exclusion stops it being built and says nothing about selection. The two mechanisms answer different questions, they do not overlap, and only the second — search cost — is open. That is the whole content of this gap: it has no correctness half.

## Where exclusion would belong, if it ever became reachable

The ticket frames the choice as "a third vector beside requirement and guarantee, or dominance". **Both are wrong, and the first is wrong in a way worth recording, because it would land as a contract change and then be hard to reverse.**

### Not a third boundary-property vector

**Inference — it fails [the contract's admission rule](../../compiler/optimizer.md#boundary-requirements-and-guarantees) on its face.** That rule admits a dimension "only when all of these are stated: its requirement space, its guarantee space, the satisfaction or subsumption rule between them, how a child boundary derives it, its dominance behaviour, its identity encoding, its maturity by the classes above, and the boundary at which a value-preserving enforcer may discharge it rather than the plan being refused". An excluding vector has no guarantee space, because no producer *guarantees* an exclusion. It has no satisfaction relation, because its test is that a producer **does** satisfy something — the inverse of satisfaction, used to reject rather than admit. And it has no enforcer that discharges it, because it is a constraint *on* enforcer insertion rather than a property an enforcer supplies. The contract's own closing sentence is the verdict: "A dimension without a satisfaction rule is a label."

**Inference — and it would corrupt identity, which is the consequence the ticket asks for by name.** Both property sets are folded into plan identity today: `BoundaryContract::encode` (`frontier.rs:564`) writes requirements and guarantees into the implementation proposal's identity, and `boundary::encode_property_identity` (`boundary.rs:1970`) is the standalone encoder "the accepted 'Possible memo contract' would key an optimization entry on". A third vector on the boundary would enter both. Two plans that are *the same plan* — same regions, same implementations, same handoffs, same costs — reached by searches that happened to exclude different things would then carry **different identities**. Identity would stop being a function of the plan and become a function of its derivation history, which breaks the property `verify_selected_plan` rests on: it re-derives the whole plan and must reproduce the identity exactly, and it has no access to a search history. A memo keyed on such an identity would also miss legitimate hits, since the same subproblem reached under two different exclusions would occupy two entries. **An excluding vector is search state; plan identity must not encode search state.**

**Inference — and Volcano does not put it there either.** The excluding vector is a parameter of `FindBestPlan`, sitting beside `PhysProp` and `Limit`. `Limit` is the branch-and-bound cost bound, which no one would propose storing on a plan. The ticket's "third vector beside requirement and guarantee" reads Tiler's requirement/guarantee pair as the analogue of Volcano's `PhysProp`, and the section above shows it is not: `PhysProp` is a goal handed down, while `RequiredProperties` is a fact derived up from a verified region. **Correcting that mapping is what dissolves the question rather than answering it.**

### Not dominance

**Inference.** Dominance covers the correctness half completely and already does, and it cannot be extended to cover the efficiency half, because it operates on plans that already exist. Moving exclusion into dominance would mean either building the redundant plan and hiding it — which is what happens today, at full enumeration cost — or making dominance run before construction, which is not dominance.

### The correct home, stated so the future change is cheap

**Proposal.** If the gap ever becomes reachable, exclusion belongs as a **parameter of the goal-directed enumeration call, introduced in the same change that introduces the goal** — which is exactly where Volcano put it, and the only place where it neither enters identity nor pretends to be a boundary property. Concretely: whichever change gives `boundary::derive_child_requirements` a compile-path caller, or gives `enumerate_frontier`/`ImplementationContext` a required-property argument, is the change that should carry the exclusion beside that argument. It is not a contract amendment to the boundary-property list, and it does not need one; the property *vocabulary* it excludes over is the existing one.

**This record proposes nothing be built now.** Implementing exclusion is a non-goal of the ticket behind it, and the derivation above says there is nothing to exclude from.

## The deferral and its trigger

**The gap is deferred with a conjunctive trigger. Both conjuncts are necessary and neither is sufficient**, which is the finding that makes the trigger worth stating precisely rather than as "when enforcers land".

- **T1 — the compile path constructs a boundary-property goal and drives producer enumeration from it.** That is `boundary::derive_child_requirements` gaining a non-test caller, or `enumerate_frontier`/`ImplementationContext` gaining a required-property parameter. This is the discriminating conjunct: nothing currently on the board is driving toward it, and the memo contract that would need it is reserved rather than committed.
- **T2 — enforcer insertion exists and is speculative rather than reactive**, generating an enforcer from a required property rather than only from an already-detected `UndischargedHandoff`. Tracked by [`implement-boundary-property-enforcers`](../../../tickets/implement-boundary-property-enforcers.md) leaving `deferred`, *plus* the insertion being speculative — a reactive enforcer landing does **not** fire this.

**T2 without T1 does not fire the gap**: a reactive enforcer at a detected mismatch cannot re-derive the property, because the mismatch is the proof the producer lacked it. **T1 without T2 does not fire it either**: a goal-directed search with no enforcer move has no enforcer input to exclude from.

**The reproducing command for T1, and it was watched saying both answers.** Not fired is:

```sh
rg -n "derive_child_requirements" crates/ | grep -v "^crates/tiler-compiler/src/boundary.rs"
```

Empty output means every mention is still inside the declaring file, whose only call sites are in its `#[cfg(test)] mod tests`. At `d7b8604d` the output is empty. **The check was proved able to say yes rather than only having been seen to say no**: the identical command shape over `unsatisfied_properties` — the sibling relation in the same module that *does* have a compile-path caller — returns eleven lines across `selection.rs`, `frontier.rs`, and `call_declaration.rs`, so an empty result distinguishes "no caller escaped the file" from "the command matched nothing".

## What this record does not establish

**Boundary.** This is a derivation over the compiler at one commit and over one paper's mechanism. It establishes that the redundancy cannot arise on today's compile path and that its two enabling conditions are absent; it does **not** establish that a future goal-directed search will want Volcano's exclusion in the form Volcano states it. A goal-directed search that memoized on `(expression, goal)` might suppress the duplicate through the memo instead, which is how Cascades' pattern memory handles a related re-expansion problem — [the formalism record's](rewrite-search-formalism.md) Part 1 records that mechanism, and this record deliberately does not choose between them, because choosing would be designing a search that does not exist.

It also establishes nothing about *how much* search the exclusion would save. Volcano offers no measurement of it, and Tiler's vocabulary — [three registered rewrite rules and a stage-3 candidate set of at most three whole programs](rewrite-search-formalism.md#part-0--what-tilers-search-actually-is-today) — is far too small for the question to be measurable here. A trigger firing is the point at which that becomes a bounded experiment rather than a guess.
