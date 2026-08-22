---
id: design-explicit-caller-selected-budget-exhaustion-policies
title: Design explicit caller-selected budget-exhaustion policies
status: deferred
priority: p2
dependencies: [state-the-rule-that-a-deterministic-budget-is-a-derivation, replace-provider-offer-with-a-host-bounded-frontier-sink]
related: [decide-whether-a-derived-budget-belongs-in-the-request-subject]
scopes: [contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, policy, public-boundary, deferred]
---
## User-visible outcome

A caller that needs a budget policy other than the governed one explicitly selects one complete typed policy before compilation. The request says both how much work may be performed and what exhaustion means; Tiler never infers a preset from the host, silently changes policies, retries another backend, or invents a default.

## Why this is deferred

**Fact — the narrow governed contract is already explicit and fail-closed.** [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md) records Tom's 2026-08-11 decision. Complete-plan search retains every valid plan assembled before `physical_plan_combinations` fires. A non-empty truncated portfolio proceeds through ordinary selection with the stop in explain; an empty truncated portfolio returns typed `BudgetExhausted`. No target or backend fallback occurs.

**Proposal — tuned configurations may eventually need more than one policy.** Examples worth comparing include accepting the best valid plan seen under a stated cap, refusing any non-exhaustive result even when a valid plan was found, or using a separately proved constructive baseline under its own work budget. These are different compilation questions and must not share a request subject or one ambiguous word such as `fallback`.

The physical-frontier decision adds another mandatory case: raw provider-output exhaustion initially refuses atomically because no retained prefix is known complete. A future preset that selects from a partial frontier must separately prove what partial population is valid to select from; it may not inherit complete-plan search's successful-prefix rule by analogy.

No second measured policy, public budget request, or constructive-baseline authority exists today. Adding a preset abstraction now would harden unsupported choices and violate ADR 0069's one-policy discipline.

## Required decision packet when the trigger fires

- Present one closed, complete policy value containing every limit and the exhaustion disposition. Every constructor must require the choice; no `Default`, optional field, per-field fallback, environment lookup, or global setting is permitted.
- Define preflight validation for incoherent policies and unsupported combinations before search begins. Refusal is typed and names the selected policy and invalid field.
- Keep `BudgetExhausted`, `NoFeasiblePlan`, and successful-but-truncated selection distinct. A policy may select only among valid plans actually retained; it may not call a different backend, target profile, strategy family, or numerical contract.
- Encode the full policy and values into the canonical request/evidence subject and account for explain, receipt, selected-content, artifact, and cache consequences separately rather than claiming they all move together.
- Measure Tiler host runtime for the named setup, including attempted combinations, rejected joins, retained plans, and compile latency. Kernel runtime is a separate metric.
- Define compatibility and removal policy for presets before accepting the public surface. A human-readable preset name is not sufficient identity; its content must be canonical.

## Trigger

Activate when either (a) a second calibrated workload/target setup demonstrates that a different limit or exhaustion disposition is needed, (b) a caller needs to state a budget policy publicly, or (c) a constructive baseline with independently verified compatibility is available. The trigger supplies the exact workload, target/profile, current policy result, proposed policy, host-runtime measurement, and why changing only a literal is insufficient.

## Non-goals

Automatic “best effort,” silent `max(1)`, retry after failure, backend fallback, target fallback, an ambient debug/release choice, or treating truncation as proof of infeasibility.

## Trigger check log

- 2026-08-11 — **not fired.** Tom accepted the future direction while retaining the one governed policy. The existing selector already chooses among valid plans seen before exhaustion and returns typed `BudgetExhausted` only when none were retained; no second measured policy or public request exists yet.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **This condition is not mechanically checkable, and saying so is the repair.** All three arms of the trigger are acts rather than states: (a) a *second calibrated workload/target setup demonstrating* a different limit or disposition, (b) a *caller needing* to state a budget policy publicly, and (c) a constructive baseline with independently verified compatibility. Each requires a measurement and a stated need that the repository cannot hold on its own, and the trigger's own text requires the firing party to supply the exact workload, target profile, current policy result, proposed policy, and host-runtime measurement — which is the evidence, and it does not exist to be grepped. A human must read the calibration records under `docs/research/program-planning/` for a second measured setup, and the request surface for a caller asking to state a policy. The one adjacent fact that *is* checkable, and is not the trigger, is that the governed policy remains single: a second governed policy value appearing in the request vocabulary is the signal to do that reading. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
