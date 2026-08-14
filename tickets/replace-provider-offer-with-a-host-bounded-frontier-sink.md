---
id: replace-provider-offer-with-a-host-bounded-frontier-sink
title: Replace ProviderOffer vectors with a host-bounded frontier sink
status: in-progress
priority: p1
dependencies: [decide-whether-the-implementation-frontier-owes-a-retention-budget, calibrate-the-physical-frontier-provider-and-outcome-budgets]
related: [accept-the-installed-physical-provider-public-surface, design-explicit-caller-selected-budget-exhaustion-policies, accept-the-host-bounded-physical-frontier-sink]
scopes: [implementation/compiler, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, backend-providers, public-boundary, identity]
claimed_from: todo
assignee: worker-frontier-sink
lease_expires_at: 1786668147
---
## User-visible outcome

An installed physical provider cannot hand Tiler an unbounded proposal/decline collection. It emits through a host-owned bounded channel; exceeding the declared policy returns typed `BudgetExhausted` before proposal verification, selection, or artifact construction, with no partial frontier or silent fallback.

## Required delivery

- Replace `PhysicalImplementationProvider::propose -> ProviderOffer` and the complete `Vec` carrier with an object-safe host-owned bounded sink or equivalent pull protocol. Charge every proposal and every `DeclinedStrategy` before accepting it.
- Preflight the complete provider population, including the governed provider. Bound zero-outcome provider invocations independently from emitted outcomes.
- Latch overflow even if provider code ignores an individual insertion result. Refuse the whole compilation at `limit + 1`; do not retain a prefix, reserve a governed entry, or retry another backend, target, provider set, numerical contract, or budget policy.
- Add the calibrated limits to the complete typed deterministic budget value, `BudgetResource` population, internal mappings, canonical request/evidence encoding, explain records, qualifier pins, and domain/version ledger. Keep artifact/cache consequences separate.
- Preserve additive/provider-order-independent semantics. The same provider set and outputs must either produce the same complete frontier or the same typed exhaustion regardless of installation order.
- Correct every contract and module comment that presently calls the unbounded stored admission set or borrowed Pareto view bounded.

## Public-boundary rule

This deliberately revises a recently accepted public trait and `ProviderOffer` surface. The implementation remains a labelled draft until Tom accepts the exact included/excluded signature at its reviewed commit. Do not retain a deprecated complete-`Vec` alias in this pre-production tree.

## Required negative controls

Perturb independently: provider count, proposal count, decline count, ignored sink refusal, installation order, request-subject field encoding, budget-resource enumeration, and the first-over-limit demand. Each assertion remains unchanged and must fail on its production subject.

## Unsupported guarantee

The provider is trusted in-process native code. This channel bounds compiler-owned accepted outcomes and subsequent verification; it does not sandbox a provider that loops, allocates before emission, or constructs an oversized raw proposal. File isolation or bounded proposal-construction work separately if a real untrusted-provider requirement appears.

## Closes when

The complete compile path uses the bounded channel, calibrated limits are canonical, overflow is an atomic typed refusal, full provider/public-boundary tests and identity censuses pass, and an independent exact-commit review finds no fallback or unbounded retained-output path.

## Fact audit at `403db7d712d01523d368fa36fa6705e8d624b574`

- **Verified.** `PhysicalImplementationProvider::propose` returns `ProviderOffer` at this base. Anchor: `fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer`.
- **Verified.** `ProviderOffer` is a complete `Vec` carrier: `proposals: Vec<ImplementationProposal>` and `declined: Vec<DeclinedStrategy>`.
- **Verified.** Eight is not a live physical-frontier `DeterministicBudgets` field. `governed()` has no provider-count or raw-outcome field. The live `8` is `normalization_rewrites: 8`, a different budget.
- **Verified.** `BudgetResource::ALL` is sized by `variant_count` and `refusal()` is the three-way authority (`ExactDemand` / `PlanningUpperBound` / `SearchLowerBound`). The public payload field is `reported`.
- **Verified.** First calibrated limits are provider count 32 (headroom 32) and raw outcomes 256 (headroom 256), from `docs/research/program-planning/physical-frontier-budget-calibration.md` (2026-08-13).
- **Verified.** Module and contract comments still call the stored admission set or Pareto view bounded. Anchors: `The bounded local implementation frontier`, `Enumerates the bounded implementation frontier`, `retained on a bounded Pareto frontier`, optimizer `bounded implementation frontiers`.
- **Imprecise in the parent decision, not this ticket.** `ImplementationFrontier::non_dominated` remains a borrowed Pareto view with no count bound after this work; the new bounds are raw output and provider count, not a retention cap on the admitted set.

No Fact change altered what this ticket is for.

## Negative-control perturbations

Each assertion was left unchanged. The production subject was perturbed, the failure text quoted, and the subject restored.

- **Provider count.** Loosened `reported > limit + 10` in the preflight check. `thirty_three_providers_refuse_the_compilation` failed: `33 providers including governed exceed the governed limit of 32: ImplementationFrontier {...}`. Restored.
- **Proposal count.** Skipped `charge()` in `PhysicalFrontierSink::propose`. `three_proposals_against_a_limit_of_two_must_refuse` failed: `three proposals against a limit of two must refuse: ImplementationFrontier`. Restored.
- **Decline count.** Skipped `charge()` in `PhysicalFrontierSink::decline`. `three_declines_against_a_limit_of_two_must_refuse` failed: `three declines against a limit of two must refuse: ImplementationFrontier`. Restored.
- **Ignored sink refusal.** Removed the post-`propose` overflow latch. `ignored_sink_refusal_still_latches_at_the_first_excess` failed: `an ignored sink refusal cannot retain a prefix: ImplementationFrontier` with a two-item `StrategyDeclined` prefix. Restored.
- **First-over-limit demand.** Reported `limit + 2` instead of `limit + 1`. `ignored_sink_refusal_still_latches_at_the_first_excess` failed: `assertion left == right failed left: 4 right: 3`. Restored.
- **Budget-resource enumeration.** Omitted `PhysicalProviders` from `BudgetResource::ALL`. Build failed: `expected an array with a size of 15, found one with a size of 14`. Restored.
- **Request-subject field encoding.** Omitted `self.budgets.physical_providers` from the encoder. `physical_frontier_budget_fields_participate_in_the_request_subject` failed: `changing physical_providers must move the request subject`. Restored.
- **Installation order.** Reset the request-scoped outcome budget per provider inside `enumerate_frontier_with_outcomes`. `exhaustion_is_independent_of_installation_order` failed: `four declines against a limit of three must refuse: ImplementationFrontier` with four retained `StrategyDeclined` rejections. Restored.
