---
id: state-the-rule-that-a-deterministic-budget-is-a-derivation
title: Decide and state the deterministic-budget provenance rule
status: awaiting-decision
priority: p3
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration]
scopes: [implementation/compiler, contracts/optimizer, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, conventions, decision, needs-tom]
---
## User-visible outcome

The compiler and its two governing documents tell one exact story for every deterministic budget: what it is derived from or what work-policy it caps, what exhaustion can remove, and whether the observed demand is exact or only a lower bound. No budget value or request identity moves under this ticket.

## Per-Fact audit — 2026-08-09

The original ticket was stale enough to change the work and the state of the node.

- **False — “five of fourteen are derived.”** `DeterministicBudgets::governed` documents **eight** authoring-side derivations: five program-scoped bounds (`semantic_values`, `semantic_operations`, `regions`, `host_expression_nodes`, `buffers`) and three region-shape bounds (`region_members`, `region_boundary_outputs`, `region_live_values`). The derivations are recorded when the governed profile is authored; `governed()` is a nullary `const fn`, so they are not runtime formulas over each request.
- **False — four search constants remain.** Six literal work-policy caps remain: `normalization_rewrites = 8`, `region_candidates_per_seed = 32`, `region_expansions = 10_000`, `region_covers = 1_024`, `region_cover_expansions = 100_000`, and `physical_plan_combinations = 4_096`. The original list omitted `region_candidates_per_seed` and undercounted the population.
- **False — every search-cap exhaustion merely loses an alternative while coverage survives.** Region formation emits the singleton and whole-program extremes before growth, cover enumeration retains the fully materialized and fused extremes, and normalization retains the verified input. `enumerate_combinations`, however, checks `produced >= max_combinations` before assembling the next plan; at a limit of zero it records `PlanBudgetStop` and can return an empty `SelectedPortfolio`. `SelectedPortfolio::is_empty` explicitly permits that state. No baseline complete plan is retained by this layer.
- **Verified — exact numeric provenance is absent.** Current source explains what the six literals bound and, for most, what their stop preserves. No accepted decision, retained measurement, or source derivation establishes why the values are exactly 8, 32, 10,000, 1,024, 100,000, and 4,096. They can be described honestly as uncalibrated deterministic policy literals; they cannot be relabelled as derived or measured.
- **False governing prose exists in both directions.** `docs/compiler/optimizer.md`, anchor `Most of the budgets above bound a *search*`, includes `physical_plan_combinations` in a claim that complete coverage survives. `docs/research/region-search/rewrite-search-formalism.md`, anchor `Deterministic budgets.`, calls five `region_*` fields search budgets even though three are shape bounds that can refuse a program.
- **Verified — identity is out of scope.** `canonical_explain_subject_bytes` serializes every field. No value, field, width, order, or request-subject domain may move here.

This audit supersedes the duplicate [`state-the-search-constant-provenance-the-caps-audit-found-bare`](state-the-search-constant-provenance-the-caps-audit-found-bare.md), which is closed as a duplicate carrier rather than left as competing work.

## Decision boundary

Tom decides the contract for `physical_plan_combinations` before the prose can be made normative:

1. **Permit an empty portfolio on exhaustion.** Keep the implementation and describe this row as a truncating cap that may remove every complete plan; correct both governing documents so they stop promising retained coverage at this layer.
2. **Require a retained baseline complete plan.** Change selection so a known complete plan is assembled outside the bounded combination search, analogous to region and cover extremes, then state the stronger alternative-loss guarantee.

**Recommendation: require a retained baseline.** A deterministic work cap should make optimization less complete, not turn an otherwise completable request into an empty portfolio solely because its limit is zero. **Strongest counterpoint:** unlike region and cover enumeration, global selection has no obviously privileged plan until feasibility and cross-region compatibility have been assembled; manufacturing a “baseline” may itself encode a selection policy and cost more than an honest typed refusal.

The six exact numeric values are a separate evidence gap, not a hidden third decision. This ticket may record them as uncalibrated policy literals without changing them. Choosing or changing a value requires its own measurement/decision ticket and complete identity accounting.

## Required work after the decision

- State the general rule at `DeterministicBudgets`: an authoring-side derivation names its formula and owning declaration; a literal policy cap names its work unit, stop effect, and evidence status. Do not imply runtime tracking.
- Classify all fourteen fields as the eight derived bounds and six literal policy caps above.
- Correct `docs/compiler/optimizer.md` and `docs/research/region-search/rewrite-search-formalism.md` to distinguish program bounds, region-shape bounds, alternative-preserving search caps, and the decided physical-plan behavior.
- Add a focused `physical_plan_combinations = 0` regression over a known completable cover. Under the current implementation it must demonstrate `PlanBudgetResource::Combinations`, `limit = 0`, `actual = 1`, and an empty portfolio; if Tom chooses retained baseline, perturb that baseline subject and show the test fail.
- Preserve every value and every encoded request byte.

## Explicit non-goals

No budget-value change, request-subject encoding change, identity-domain step, new caller-configurable budget set, or silent calibration claim. [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md) remains the separate public identity decision.

## Closes when

Tom answers the physical-plan stop contract; source and both governing documents classify all fourteen fields truthfully; the physical-plan stop has a subject-perturbation regression; all six literals are labelled with their actual evidence status; and no value or identity moved.
