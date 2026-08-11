---
id: replace-provider-offer-with-a-host-bounded-frontier-sink
title: Replace ProviderOffer vectors with a host-bounded frontier sink
status: todo
priority: p1
dependencies: [decide-whether-the-implementation-frontier-owes-a-retention-budget, calibrate-the-physical-frontier-provider-and-outcome-budgets]
related: [accept-the-installed-physical-provider-public-surface, design-explicit-caller-selected-budget-exhaustion-policies]
scopes: [implementation/compiler, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, backend-providers, public-boundary, identity]
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
