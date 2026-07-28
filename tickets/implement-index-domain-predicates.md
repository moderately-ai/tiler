---
id: implement-index-domain-predicates
title: Implement typed index-domain predicates and proof exchange
status: awaiting-decision
priority: p1
dependencies: [implement-shapeenv-index-bindings]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof, mature-product]
---
Add the accepted bounded typed predicate language, semantic obligations, durable proof evidence, and sound Unknown outcomes to verified index regions. Extend bounds and write-ownership proving beyond the static structural and finite fallback profile without converting semantic predicates into physical guards.

## Decision needed (2026-07-28)

**The question, atomic:** what expression class may a typed index-domain obligation be stated in?

**Why this is a decision and not research.** The 2026-07-27 split below says of step 1 that "the accepted bounded typed predicate language" has no accepted design linked from here or findable under `docs/decisions/`, and instructs a worker to "locate or write it before starting". Locating it has now been attempted and it does not exist: `grep -rn '^title:' docs/decisions/*.md | grep -i predicate` returns nothing and exits 1. So the instruction resolves to "write it", which is an architectural choice about what the compiler can be asked to prove — not something a worker should settle inside an implementation ticket.

**This is urgent for a graph reason, not only a design one.** `tkt ready` currently returns **exactly this ticket and nothing else**. A worker taking the only ready work in the graph reaches step 1, finds no accepted language, and stalls. Whatever the answer, it unblocks the whole frontier.

**A concrete program the profile cannot prove today.** Take a rank-1 value of extent `M` bound by `ShapeEnv`, and a region reading it at `i floordiv d` for a symbolic divisor `d` with `d >= 1` proved from semantic constraints. Bounds proving needs `i floordiv d < M` for every `i` in the domain. `interval_verdict` compares each symbolic axis against the side of its own interval that makes each answer sound; with `d` symbolic it can neither prove nor refute, so both flags stay false — a correctly computed `Unknown`. Today that `Unknown` is collapsed to a rejection at the boundary. The predicate language is what would let the region verify while *carrying* `i floordiv d < M` as a stated obligation instead.

| Option | Enables | Prevents |
| --- | --- | --- |
| **(a) Closed conjunctions of affine inequalities over sourced extents.** Obligations are `Σ cᵢ·xᵢ + k ≥ 0` conjunctions where every symbol is a `ShapeEnv`-sourced extent. | The smallest vocabulary that states the common obligation, and every obligation in it is decidable by construction — `docs/research/shapes/constraint-prover-boundary.md:188-204` puts affine inequalities and constant div/mod in the Presburger lane's decidable region. Discharge is total: an obligation is proved or refuted, never resource-limited on its own form. | Cannot state the example above at all. A symbolic divisor is not affine, so the region that motivated the ticket is still rejected rather than carried — the vocabulary excludes the case where the `Unknown` matters most. Widening it later is a public change to the obligation type and therefore to identity. |
| **(b) The semi-affine class `admit-semi-affine-index-expression-class` already admitted.** Affine, plus constant-divisor quasi-affine, plus guarded semi-affine with symbolic coefficients and proven-positive symbolic divisors. | Exactly matches what the IR can represent, so no obligation is unstateable that the index vocabulary can express — no second boundary to keep in sync. States the example. ADR 0046 already governs the class, and `proves_positive` already exists as its guard. | Not decidable: that ticket's own outcome records that a symbolic divisor is classed **nonlinear** for the Presburger lane, and that proving positivity establishes *definedness only*, not analyzability. So a discharge attempt may legitimately answer neither proved nor refuted, and the discharge protocol (step 4) must handle a third outcome — an obligation that is well-formed and undecidable — rather than two. |
| **(c) A quantifier-free Presburger fragment.** Boolean combinations, including disjunction and negation, of affine constraints; constant div/mod admitted. | Strictly more expressive than (a) at the same decidability. Disjunction is what states a piecewise obligation, and negation is what lets a refutation be an obligation in its own right rather than a separate channel. | Decidable but not cheaply so; the same reference notes a production implementation "may return `ResourceLimit` rather than permit pathological compile time". That reintroduces a budget stop *inside* discharge — the exact flattening step 3 exists to remove, arriving one layer down. Still cannot state the symbolic-divisor example. |

**Elimination run explicitly, since two of these look like they survive.** (c) is eliminated: it buys expressiveness this profile has no demonstrated obligation for, and it pays with a resource-limited discharge outcome, which recreates the budget-refusal-collapses-to-rejection problem the ticket is chartered to fix. Adopting a fragment whose decision procedure can time out, in order to remove a timeout-driven rejection, is circular. (a) survives on decidability and fails on coverage: it is a real, safe vocabulary, but it cannot state the obligation class the IR can already represent, so the compiler would be able to *build* an index expression it cannot *talk about* — and that gap is where an ad hoc obligation gets stated, which step 33 below identifies as the physical-guard failure in a different spelling.

**Recommendation: (b), with the third discharge outcome made explicit rather than hidden.** The evidence is that the IR's expressible class and the obligation's stateable class must be the same set, or the difference becomes unstated. `admit-semi-affine-index-expression-class` is `status: done` and already decided what that set is, under ADR 0046, with `proves_positive` as the positivity guard. **The counterpoint is the one that matters and Tom should weigh it:** (b) makes "undecidable" a first-class discharge result, so the protocol carries `SoundProof`, exhaustive finite evidence, empirical evidence, `Unknown`, and now an obligation that is *permanently* undischargeable by any lane. If those collapse into one another under implementation pressure, the ticket's own invariant — four distinct evidence classes, never a confidence scalar — is lost. Choosing (a) avoids that by construction, at the price of leaving the symbolic-divisor region rejected; that is a coherent, defensible narrower scope, and it is the fallback if Tom judges the fifth outcome too much for a first language.

**Not part of this decision.** That an `Unknown` must never become a silently inserted physical guard is settled, binds every option identically, and is restated below.

## Closes when — step 1 only (2026-07-28)

The `## Closes when` for this ticket covers the predicate language alone. Steps 2 through 4 close on their own criteria and should be split into their own tickets once step 1 lands.

1. **The obligation vocabulary is a typed, closed enumeration**, not an open expression tree, and a construct outside it is a build error at the construction site rather than a runtime refusal.
2. **Every obligation the IR's admitted index-expression class can produce is stateable in it**, and a test enumerates that correspondence rather than asserting it in prose — the two sets must be pinned against each other, because a divergence is silent.
3. **No physical guard is inserted for any unproved region, and a test asserts it.** The pin already stated below: a region refused today for exceeding `MAX_EXHAUSTIVE_PROOF_CELLS` / `MAX_EXHAUSTIVE_PROOF_BYTES` must instead verify while carrying an explicit obligation, and the test must confirm the emitted physical program contains no bounds check attributable to that obligation. Confirm this test can fail before relying on it.
4. **The four evidence classes stay four.** `SoundProof`, exhaustive finite evidence, empirical evidence, and `Unknown` remain distinct in the type, with no ordering and no collapse to a confidence scalar. If the chosen language adds a fifth outcome, it is added as a named variant, not folded into `Unknown`.
5. **The accepted language is written down where the next worker will find it** — an ADR under `docs/decisions/`, since the check that found nothing was `grep -rn '^title:' docs/decisions/*.md | grep -i predicate`, and `make full` passes.

**Status.** Frontmatter is not this record's to change; the request to move `todo` to `awaiting-decision` is left for the coordinator, and it is the urgent one — this is the sole `tkt ready` ticket and its blocker is the decision above.

## Scoped — split at the Unknown boundary (2026-07-27)

Thirteen lines naming four deliverables at once. Reading `crates/tiler-ir/src/index/builder/proof.rs` (691 lines) shows the four are not peers, and that one of them is largely built.

**The three-valued logic already exists and is already sound.** `interval_verdict` returns `IntervalVerdict { interval_proved, definitely_outside }`, and its own documentation states the invariant this ticket would otherwise have to establish from scratch: "the two answers are independent and neither implies the other's negation: an interval that overlaps a boundary proves nothing either way, while one lying wholly outside refutes the access." A symbolic axis is compared "against the side of its own interval that makes each answer sound" — lower bound to prove, upper bound to refute. Where the environment bounds an axis nowhere, both flags stay false: "nothing about it is provable and nothing about it is refutable either." That is `Unknown`, computed correctly, today.

**What is absent is durability, not the logic.** An unproved obligation currently has nowhere to live. When the exhaustive fallback exceeds `MAX_EXHAUSTIVE_PROOF_CELLS` / `MAX_EXHAUSTIVE_PROOF_BYTES`, the budget refusal is pushed as a *diagnostic* (`proof.rs:336`) — the region is refused rather than verified-with-an-obligation. So `Unknown` is computed internally and then collapsed to rejection at the boundary. The four deliverables reduce to: a predicate language able to *state* an obligation, a place in the verified region to *carry* it, and a discharge protocol; the sound `Unknown` outcome is mostly the existing verdict stopping being flattened.

**The constraint that orders the split, and it is the whole reason the ticket exists.** The body says the extension must not convert semantic predicates into physical guards. That is precisely the tempting resolution: a region whose bounds cannot be proved could be admitted with a runtime bounds check, and it would work. It would also move a semantic obligation into the physical layer where the optimizer can no longer reason about it, and make "unproved" indistinguishable from "proved, then guarded anyway". An `Unknown` must remain an obligation the compiler carries and reports, never a guard it silently inserts.

**Suggested split, in dependency order.** Each is landable alone and the first is the only one that needs a design decision this ticket cannot supply from the code:

1. **The bounded typed predicate language.** The vocabulary an obligation is stated in. Everything below is expressed in it, so it comes first. *This is the piece the ticket's phrase "the accepted bounded typed predicate language" refers to, and no accepted design is linked from here or findable under `docs/decisions/`; locate or write it before starting.*
2. **Durable proof evidence.** Where a discharged obligation's evidence lives in the verified region and how it enters identity. Must keep `SoundProof`, exhaustive finite evidence, empirical evidence, and `Unknown` as distinct classes rather than a confidence scalar.
3. **Sound `Unknown` outcomes.** Stop flattening the budget refusal and the unbounded-axis case into diagnostics; carry them as stated obligations. The pin: a region that is refused today for exceeding the proof budget must instead verify carrying an explicit obligation, and a test must confirm no physical guard was inserted for it.
4. **Semantic obligations and discharge.** Who discharges a carried obligation, when, and what happens if nobody does — which must be an explainable refusal at a named stage, not an admitted program.

Do not start at 3 or 4. Without 1 there is no vocabulary to state an obligation in, and an obligation stated ad hoc is the physical-guard failure in a different spelling.
