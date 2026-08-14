---
id: decide-the-truthful-public-class-for-complete-explain-capacity-refusals
title: Decide the truthful public class for complete-explain capacity refusals
status: in-progress
priority: p1
dependencies: []
related: [decide-how-explain-capacity-bounds-active-physical-provider-populations, calibrate-the-physical-frontier-provider-and-outcome-budgets, refuse-nothing-legal-on-the-explain-detail-ceiling]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [public-boundary, explain, budgets, correctness]
claimed_from: todo
assignee: worker-explain-public-class
lease_expires_at: 1786718386
---

## Outcome

Decide the exact truthful public classification for a complete-explain detail-capacity refusal reached by an otherwise valid public compile request. Keep complete-or-refused trace construction intact. Do not select a capacity value, active-provider cardinality, or full-provider support policy here; this decision is mandatory independently of whether full 32-slot activity is ever promised.

If a change survives the Pareto gate, Tom must accept the exact included and excluded public surface before implementation. Before this decision ticket closes, it must create the narrow accepted implementation ticket and its required evidence/regression ticket, then add hard dependencies from both downstream capacity tickets named under **Graph** to the implementation **and** evidence tickets. A decision-only closure is forbidden because it would satisfy the current hard dependency while the false `InvalidCompilerOutput` behavior still ships.

## Facts at discovery base `e37c05b8ec28114736648edebbbdee745f4a051b`

- **Measurement.** The retained public five-operation strict request with seven installed specialists is valid, reaches physical planning, and fails with public `CompileFailureClass::InvalidCompilerOutput`. Its complete failure trace ends in `compiler-failure:explain-detail-capacity`.
- **Fact.** `crates/tiler-compiler/src/session.rs`, anchor `InvalidCompilerOutput — **unreachable by construction from a valid call, deliberately.**`, says public reachability means Tiler shipped the defect this class reports. The enum variant's own documentation says it is always a Tiler defect, never a caller-program refusal.
- **Fact.** `ExplainWriter::push`, anchor `let exceeds = if terminal`, deliberately retains hard limits and refuses rather than truncates. [`never-truncate-the-governed-explain-trace`](never-truncate-the-governed-explain-trace.md) retained that refusal to protect combinatorial growth; it did not authorize exposing the refusal as `InvalidCompilerOutput` to a valid caller.
- **Accepted-history fact.** [`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md), anchor `A semantic program that satisfies every governed budget compiles`, established that a valid in-budget request must compile or receive a class naming the caller-facing limit. It rejected reclassification alone for its earlier population because exact duplicate rejection grounds could be aggregated at source. That decision is precedent to search for lossless aggregation, not permission to retain a false class while the search is pending.
- **Fact.** `BudgetResource` currently has thirteen variants and no explain resource. `BudgetRefusal` has the closed provenances `ExactDemand`, `PlanningUpperBound`, and `SearchLowerBound`. `ExplainError::DetailCapacity` currently carries neither which independent arm fired nor `limit`/`reported` quantities. Mapping it to `BudgetExhausted` without extending the typed cause and proving the number's provenance would invent fields.
- **Fact.** The existing explain ceilings are build constants, not `DeterministicBudgets` fields and not bytes in `canonical_explain_subject_bytes`. Turning either into a request budget would change every compiler-internal request/evidence subject and explain qualifier. A dedicated additive failure class need not do so. Neither answer directly changes plan, artifact, or cache identity; newly admitted compilation may change selected packaged content indirectly.
- **Fact.** `CompileFailureClass` and `BudgetResource` are public `#[non_exhaustive]` enums, so variants may land additively, but ADR 0075 still reserves the consequential public boundary for Tom. `BudgetRefusal` is intentionally exhaustive because consumers totally map its closed provenance vocabulary.

## Decision gate

Compare at least:

1. extend `BudgetExhausted` with one or two explain-capacity resources and the exact refusal provenance/payload needed to make `limit` and `reported` truthful;
2. add a dedicated typed explain-capacity class/resource that does not claim the refusal is a request budget, infeasible plan, unsupported capability, invalid request, or compiler defect;
3. make the public refusal unreachable by a proven complete source aggregation or sufficiently governed widening; and
4. remove both hard ceilings without silent truncation; and
5. defer the exact surface while treating current public reachability as a live defect, never as an acceptable support statement.

Eliminate `InvalidRequest`, `UnsupportedCapability`, and `NoFeasiblePlan`: the request is valid, installing a provider can increase the demand, and the compiler has not proved the plan space infeasible. Eliminate a bare rename that carries no actionable resource or provenance. Eliminate removing both hard ceilings: the installed/active population is unbounded, trace growth is combinatorial, and `never-truncate-the-governed-explain-trace` deliberately retained the hard record/byte ceiling as real host-memory protection after removing the redundant soft truncation limit. Do not let an eventual full-provider support decision stand in for this classification decision.

For every survivor state:

- caller action and the strongest statement the class makes;
- whether `reported` is exact demand or only a lower bound at the stopped record;
- completeness and failure-trace consequences;
- public enum/type and semver consequences under ADRs 0074/0075;
- request/evidence, explain, plan, artifact, and cache identity consequences;
- schema/renderer consequences;
- strongest counterargument, reversal evidence, and subject perturbations; and
- the exact implementation and regression-test dependency if Tom accepts it.

## Required perturbations

- Reproduce the seven-specialist public request and assert that the accepted class is not `InvalidCompilerOutput` while the terminal trace still names `explain-detail-capacity`.
- Independently force the record arm and byte arm one unit below their exact attempted demand; neither may be misreported as the other.
- If `BudgetExhausted` survives, perturb resource, limit, reported, and refusal provenance independently and show the unchanged public assertion fail.
- If a dedicated class survives, perturb its dimension/action payload independently.
- Keep an actual verifier-produced invalid compiler output mapped to `InvalidCompilerOutput`, proving the repair does not erase real Tiler defects.

## Graph

Both [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](decide-how-explain-capacity-bounds-active-physical-provider-populations.md) and [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](calibrate-the-physical-frontier-provider-and-outcome-budgets.md) depend on this decision. The first cannot call the current wall a supported-population policy; the second cannot land a raw budget while a smaller valid request can still surface the false defect class through an independent capacity.

## Closing conditions

- Tom accepts the exact public surface, or accepts a complete source/capacity change that makes the public refusal unreachable.
- Create `implement-<accepted-truthful-explain-capacity-outcome>` with this decision as a dependency. Do not use a generic implementation ticket whose body leaves the accepted surface implicit.
- Create the accepted regression/evidence ticket depending on that implementation. It must reproduce the seven-specialist public path, both independent capacity arms, and a genuine verifier defect that remains `InvalidCompilerOutput`.
- Add hard dependencies from **both** `decide-how-explain-capacity-bounds-active-physical-provider-populations` and `calibrate-the-physical-frontier-provider-and-outcome-budgets` to **both** the accepted implementation and evidence tickets. Verify the graph after all four edges exist.
- Only then may this decision close. If no change survives, it remains open with the live defect rather than satisfying dependents by declaration.
