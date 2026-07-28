---
id: emit-typed-opaque-call-frontier-rejection-records
title: Emit typed opaque-call frontier rejection records
status: todo
priority: p2
dependencies: []
related: [emit-analytical-costs-through-the-typed-cost-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, explain, opaque-calls]
---
## User-visible outcome

A reader of the explain trace can distinguish an unregistered opaque call, a malformed named binding, a numerical-contract mismatch, an underivable boundary contract, unresolvable work scaling, and target infeasibility. Each refusal retains the opaque-call identity, provider attribution, and its typed or stable reason rather than collapsing into `frontier.rejected-count`.

## Why this is separate from analytical cost reporting

These records are refusals, not costs. `ExplainEvent::CostAssessment` has no rejection subject or reason and widening it would make a calibration record assert a frontier disposition it never computes.

The existing rejection events also cannot represent the complete set truthfully. `UnregisteredCall` and `MalformedBinding` can be intrinsic checked refusals, and `numerical-contract-mismatch` can be a numerical-legality refusal, but `CallNotAdmissible("target-infeasible")` retains only a reason string. `ExplainEvent::Feasibility` requires typed `required` and `available` quantities and validates their relation, so constructing it from the current frontier would require inventing data. Labelling the same refusal intrinsic would put a target verdict at the wrong stage. The construction site is `enumerate_frontier` in `crates/tiler-compiler/src/frontier.rs`: `assess_resources` returns the `PhysicalError`, but the closure currently reduces every failure to `"target-infeasible"` before storing the rejection.

## Implementation keys

- Preserve enough typed detail in `FrontierRejection::CallNotAdmissible` to reconstruct target feasibility without parsing strings. Do not make the reason string a second authority over the typed error.
- Emit one typed explain record per opaque-call rejection from `record_frontier`, with the exact call and provider as subjects/provenance.
- Keep frontier counts only if they remain useful as a summary; they cannot substitute for the records.
- Do not route any rejection through `CostAssessment`, and do not let explain reporting alter frontier admission, dominance, or retained plans.
- Exercise every binding fault and all four current call-admission refusal paths, including a target-infeasible fixture whose required/available relation is validated.

## Closes when

- Every opaque-call `FrontierRejection` variant reaches explain with a typed stable reason and exact subject/provenance.
- The target-infeasible record carries the quantities produced by the feasibility authority.
- A perturbation that drops or misclassifies each refusal makes its check fail.

## Graph maintenance

- Update the frontier explain census once, naming whether per-rejection records replace or accompany the summary count.
- Remove the split-remainder note from `emit-analytical-costs-through-the-typed-cost-vocabulary` when this ticket closes.
