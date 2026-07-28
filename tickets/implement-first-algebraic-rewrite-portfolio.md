---
id: implement-first-algebraic-rewrite-portfolio
title: Implement the first algebraic rewrite portfolio
status: done
priority: p1
dependencies: [implement-transactional-rewrite-engine, implement-first-profile-numerical-policies]
related: []
scopes: [implementation/compiler, implementation/reference, implementation/ir, contracts/numerics, contracts/optimizer, contracts/navigation, implementation/artifact, implementation/metal, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites, numerics]
---
## User-visible outcome

The optimizer can offer *legal algebraic alternatives* — each a named, versioned rule with stated semantic and numerical preconditions, oracle-compared, individually disableable, and explained whether it fired or declined. The unchanged canonical program remains available, and no algebraic alternative is selected before independently verified physical planning supplies comparable complete programs.

## Closes when (2026-07-28)

1. **Each rule is separately named and separately identified.** Every alternative carries its own `RewriteRuleIdentity` — a rule set, a rule key, and a revision — and that identity, not a hand-written string, is what reaches explain output. A portfolio of rules sharing one identity is not a portfolio.
2. **Each rule states its semantic precondition and its numerical precondition, separately.** The two are different refusal classes: a shape or dtype mismatch is a semantic decline, and a freedom the request's numerical contract has not granted is a numerical decline. A rule that conflates them cannot explain why it did not fire.
3. **Each rule is compared against the reference oracle on the inputs it claims to preserve**, with the comparison's tolerance derived from the numerical dimension the rule consumes rather than chosen to make the test pass. A rule authorized only under a relaxed contract is compared under that contract; a rule claiming bit-exactness is compared bit-exactly.
4. **Each rule has a positive test and a negative test, and the negative test is confirmed to fail before its guard is added.** Write the case that must be refused, run it, watch it be *accepted*, then add the precondition and watch it be refused. A negative test written after its guard proves the guard compiles, not that it can say no. This is the same failure the workspace already records for a dirty-check that could not report dirt and for a `trybuild` glob that stopped matching.
5. **A test pins that the rules are separately disableable**, and that disabling one leaves the others firing. A portfolio whose members can only be enabled as a block is one rule wearing several names, and it makes the per-rule explain output unverifiable.
6. **Explain output is stable and names both outcomes.** For every rule, on every compilation, the record says whether it was accepted or rejected and for which reason, in a deterministic order. A rejected rewrite that leaves no record is indistinguishable from a rule that never ran.
7. **Search stays bounded, and the bound is observable.** The portfolio does not make the alternative count unbounded in program size; a compilation that hits the budget reports it as a typed stop rather than silently returning the alternatives it happened to reach. `make full` passes.

Do not fold any of this into canonical normalization or fusion-region formation — a rule that is always applied is normalization, and moving it here would make the portfolio's bounded-search and separately-disableable criteria untestable.

## Delivered (2026-07-28)

- The add and multiply rules have separate `RewriteRuleIdentity` values under provider `tiler.algebraic`, with rule keys `ordered-reassociate-add-f32.v1` and `ordered-reassociate-multiply-f32.v1` and output-affecting revision 1. They operate over the frozen add and multiply definitions, each of which owns the ordered-associativity declaration its rule consumes.
- Semantic applicability, numerical permission, and per-rule configuration are independent assessments with deterministic explain records. Strict and flush-to-zero contracts decline numerically after semantic acceptance; the relaxed contract admits reassociation; disabling add leaves multiply evaluated and available.
- Each accepted rule rebuilds one right-associated three-leaf program while preserving the ordered leaf sequence, exact operation attributes, output interface, sharing observed elsewhere, and registry-inferred type and shape. Every proposal is structurally revalidated by the existing transaction.
- The exhaustive conformance oracle covers three through six leaves, enumerates every order-preserving binary grouping through the independent semantic reference evaluator, and requires the rewritten exact result bits to belong to that set. The oracle refuses an unreviewed leaf count.
- Algebraic exploration retains the canonical baseline, admits at most one proposal per registered rule, consumes the existing governed rewrite budget, and records exact limit/demand on an all-or-nothing stop.
- The compile path consumes `readmit_alternatives`, `group_by_resolved_contract`, and `record_adopted_alternatives`. Every candidate re-enters request verification; candidates are grouped by resolved contract and evaluated in caller preference order; later groups are explicitly preference-pruned; and no cost comparison crosses a contract boundary.
- Each evaluated semantic candidate runs through its own complete physical pipeline. The global portfolio verifier re-derives owner binding and alternative identity from the rule origin, semantic program, verified request, and plan before deterministic nondominated selection.
- Explain trace v3 records every semantic, numerical, configuration, budget, and preference outcome. Its composite semantic selection binds each candidate key to the exact full canonical compilation subject and rejects swapped or otherwise mismatched nested traces.

## Implemented boundary

The algebraic portfolio is semantically implemented and live, but the governed physical profile recognizes only the scale/bias/strict-serial-sum program and assembles programs specialized to that structure. An accepted three-leaf add or multiply reassociation cannot yet reach a complete physical program. That is separate capability work, not unfinished rule implementation, and is tracked by `broaden-governed-physical-support-for-reassociated-programs`.

## Graph maintenance

- The second-rule seams in `route-the-compile-path-through-the-rewrite-engine` and `generalize-the-normalize-transaction-to-alternatives` are consumed and those tickets now carry superseding outcome notes.
- `implement-first-profile-numerical-policies` delivered the eleven-dimension vocabulary this work consumes; no parallel permission type was introduced.
- Physical recognition/lowering breadth is split to `broaden-governed-physical-support-for-reassociated-programs`, which depends on this ticket and does not broaden this ticket's implementation.
- Leave this ticket `in-progress` until the final `make full` passes; then mark it done. The follow-up owns no remainder required by the seven closing criteria above.
