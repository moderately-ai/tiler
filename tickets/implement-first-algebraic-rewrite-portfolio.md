---
id: implement-first-algebraic-rewrite-portfolio
title: Implement the first algebraic rewrite portfolio
status: todo
priority: p1
dependencies: [implement-transactional-rewrite-engine, implement-first-profile-numerical-policies]
related: []
scopes: [implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites, numerics]
---
## User-visible outcome

The optimizer can offer *legal algebraic alternatives* — each a named, versioned rule with stated semantic and numerical preconditions, oracle-compared, individually disableable, and explained whether it fired or declined. Today the engine drives exactly one rule (CSE), so "portfolio" is a capability the machinery has and no rule exercises.

Add the first separately reviewed algebraic alternatives with named rules,
explicit semantic and numerical preconditions, reference-oracle comparison,
positive/negative tests, stable explain, and bounded search. Do not fold this
portfolio into canonical normalization or fusion-region formation.

## Closes when (2026-07-28)

1. **Each rule is separately named and separately identified.** Every alternative carries its own `RewriteRuleIdentity` — a rule set, a rule key, and a revision — and that identity, not a hand-written string, is what reaches explain output. A portfolio of rules sharing one identity is not a portfolio.
2. **Each rule states its semantic precondition and its numerical precondition, separately.** The two are different refusal classes: a shape or dtype mismatch is a semantic decline, and a freedom the request's numerical contract has not granted is a numerical decline. A rule that conflates them cannot explain why it did not fire.
3. **Each rule is compared against the reference oracle on the inputs it claims to preserve**, with the comparison's tolerance derived from the numerical dimension the rule consumes rather than chosen to make the test pass. A rule authorized only under a relaxed contract is compared under that contract; a rule claiming bit-exactness is compared bit-exactly.
4. **Each rule has a positive test and a negative test, and the negative test is confirmed to fail before its guard is added.** Write the case that must be refused, run it, watch it be *accepted*, then add the precondition and watch it be refused. A negative test written after its guard proves the guard compiles, not that it can say no. This is the same failure the workspace already records for a dirty-check that could not report dirt and for a `trybuild` glob that stopped matching.
5. **A test pins that the rules are separately disableable**, and that disabling one leaves the others firing. A portfolio whose members can only be enabled as a block is one rule wearing several names, and it makes the per-rule explain output unverifiable.
6. **Explain output is stable and names both outcomes.** For every rule, on every compilation, the record says whether it was accepted or rejected and for which reason, in a deterministic order. A rejected rewrite that leaves no record is indistinguishable from a rule that never ran.
7. **Search stays bounded, and the bound is observable.** The portfolio does not make the alternative count unbounded in program size; a compilation that hits the budget reports it as a typed stop rather than silently returning the alternatives it happened to reach. `make full` passes.

Do not fold any of this into canonical normalization or fusion-region formation — a rule that is always applied is normalization, and moving it here would make the portfolio's bounded-search and separately-disableable criteria untestable.

## Dependency note — the permission vocabulary already exists, uncommitted (2026-07-28)

`implement-first-profile-numerical-policies` is `status: in-progress` with completed but **uncommitted** work in the harness worktree `.claude/worktrees/agent-ad2893b1fba4d7f5b`. Its `crates/tiler-compiler/src/policy.rs` defines `NumericalPolicyPreset` with three members — `Strict` (`:409`), `FlushSubnormalsToZero` (`:421`), and `Relaxed` (`:441`) — and widens `crate::honourability::NumericalDimension` from four dimensions to eleven. Two of those eleven are directly this ticket's subject: **reciprocal transform** and **approximate intrinsics**, the latter resolving to a governed `ApproximationEnvelope` rather than a boolean, because the normative contract requires a maximum accuracy envelope and `Permitted` would state no bound.

**Do not invent a parallel permission vocabulary here.** A rewrite in this portfolio asks whether the request's contract grants a named dimension; that question already has a type and a preset vocabulary answering it. A second spelling of "may I reassociate" would give two authorities for one fact, and the failure mode is the one `declare-metal-numerical-honourability` already recorded: two checkpoints reading one declaration cannot diverge, and two declarations can. Wait for that work to land, or build against its types, but do not restate them.

## Graph maintenance — read this first: you are "the second rule"

Half the rewrite machinery was deliberately left idle until a second rule exists, and this ticket is that rule. The map of what you inherit and what you must wire is in `route-the-compile-path-through-the-rewrite-engine` under "Assessment corrections"; do not re-derive it. Concretely:

- **The stage currently rejects more than one alternative.** `normalize_semantics` returns `InvalidRewrite { rule: "multiple-alternatives" }` when the engine adopts two — a deliberate refusal because choosing among alternatives is a cost decision with no cost model in scope at that stage. Your first structural task is deciding where multiple alternatives are consumed (the pipeline's alternative path, not a silent first-wins), and that refusal is your tripwire: your first two-rule program will hit it.
- **Wire the idle seam as you go**: `readmit_alternatives` (per-alternative readmission — then delete the inline duplicate in `pipeline.rs`, which duplicates its fault policy with nothing keeping them in sync), `group_by_resolved_contract` (alternatives resolving to different numerical contracts are not comparable on cost — the guard exists, unconsumed), `record_adopted_alternatives` (survivor-only emission), and `RewriteRuleIdentity` into explain records (today the emitted rule keys are the stage's own constants and the provider identity reaches no record — your criterion 1 is unsatisfiable until you wire it).
- **The budget is already per-alternative** (fixed 2026-07-28, two-rule regression test exists) — do not re-add summing.
- **When the first second rule lands**: update the seam reasons (`grep -rn 'second-rule seam' crates/tiler-compiler/src` lists them), widen `NormalizeError` or its mapping so `EngineFailure::Revalidation`'s rule identity stops being discarded (the drop is annotated at the mapping site), and note on `generalize-the-normalize-transaction-to-alternatives` that its "what remains" is consumed.
- **File, don't fold**: if a rule needs a numerical dimension the realization does not carry, that is `implement-first-profile-numerical-policies`' vocabulary (see `rebase-and-land-the-stranded-numerical-policies-worktree` — it may land eleven dimensions before you start; check).
