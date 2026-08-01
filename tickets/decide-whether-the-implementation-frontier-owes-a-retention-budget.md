---
id: decide-whether-the-implementation-frontier-owes-a-retention-budget
title: Decide whether the implementation frontier owes a retention budget
status: todo
priority: p2
dependencies: []
related: [record-the-four-surface-optimizer-invariant]
scopes: [contracts/optimizer, implementation/compiler]
shared_scopes: []
paths: []
tags: [optimizer, contracts, budgets]
---
## User-visible outcome

The per-region implementation frontier either carries a declared retention bound inside the canonical request subject, or the optimizer contract records why an unbounded Pareto filter is the correct answer — so a reader of the deterministic-budget list is not left with a ninth budget that is neither implemented nor refused.

## Why this is its own ticket

**Fact.** `ImplementationFrontier::non_dominated` (`crates/tiler-compiler/src/frontier.rs`) retains every admitted implementation no other admitted implementation dominates, and applies no count bound. `DeterministicBudgets` (`crates/tiler-compiler/src/request.rs`) has no field for it — the check is `grep -n "implementations_per_region\|nondominated" crates/tiler-compiler/src/request.rs`, which returns nothing.

**Fact.** `docs/compiler/optimizer.md` carried "8 nondominated implementations per region" as a forward-looking budget whose stated activation condition was the physical-implementation-frontier stage landing. That stage landed (`enumerate_frontier` is called from `crates/tiler-compiler/src/pipeline/planning.rs`), and the budget did not follow. `record-the-four-surface-optimizer-invariant` replaced the stale sentence with the current state and this pointer rather than inventing either a field name or a decision.

**Inference — the exposure is latent, not live.** The compile path installs one governed provider, which offers at most three proposals for a reduction, so the retained set is small by construction today. It stops being bounded by construction when [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 2's installable physical-provider registry lands and an arbitrary number of callers' providers may each propose.

## The decision

Two answers are defensible and they are not the same claim:

- **Declare a retention budget.** It joins the canonical request subject like every other `DeterministicBudgets` field, which is what makes it artifact-identity-bearing, and exhausting it becomes a typed explain budget-stop that costs an alternative while complete coverage survives. The cost is one more calibration input and one more identity byte sequence.
- **Record that retention is deliberately unbounded.** The Pareto filter's output is bounded by the number of *incomparable* proposals rather than by the number offered, and the accepted contract's "bounded Pareto frontier" language may already be discharged by dominance itself. This answer must state what stops a provider set from producing an unbounded incomparable set, and it must correct the contract sentence at `docs/compiler/optimizer.md`'s boundary-requirements section that calls the frontier bounded.

Deciding requires knowing whether an incomparable set can grow without bound under the boundary-property subsumption relation, which is a question about `AdmittedImplementation::dominates` and the boundary contract rather than about provider count.

## Closes when

Either a retention budget exists as a `DeterministicBudgets` field with a typed budget-stop and its default recorded as a calibration input, or the optimizer contract states that retention is bounded by dominance alone with the argument for why; in both cases the deterministic-budget list and the boundary-requirements section agree, and this ticket's pointer in `docs/compiler/optimizer.md` is replaced by the answer.

## Graph maintenance

- Not a blocker for the four-surface invariant, which cites the current state rather than depending on the answer.
- The activation trigger is ADR 0090 item 2's provider registry: if that lands first, this stops being latent.
