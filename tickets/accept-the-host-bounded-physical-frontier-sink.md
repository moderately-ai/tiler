---
id: accept-the-host-bounded-physical-frontier-sink
title: Accept the host-bounded physical-frontier sink
status: awaiting-decision
priority: p1
dependencies: [replace-provider-offer-with-a-host-bounded-frontier-sink]
related: [accept-the-installed-physical-provider-public-surface]
scopes: [contracts/decisions, contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [public-boundary, needs-tom, optimizer]
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public surface of the host-owned physical-frontier sink that replaced `PhysicalImplementationProvider::propose -> ProviderOffer`.

## Decision boundary

This node is not research or implementation work. Only Tom closes it. The implementation remains a labelled draft under ADR 0075 until that acceptance.

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) still routes every concrete public surface to Tom at implementation time. [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md) accepted the previous `propose -> ProviderOffer` complete-`Vec` surface; this node is the successor for the source-breaking sink revision.

## The surface, as landed

**Included — `tiler_compiler::physical_provider`.** `PhysicalImplementationProvider` (trait, `provenance` plus `propose(&self, &ImplementationContext<'_>, &mut PhysicalFrontierSink<'_>)`); `PhysicalFrontierSink::{propose, decline}` (no public constructor); `PhysicalFrontierBudget` (opaque insertion-refusal token); the previously accepted context, baseline, subject, scheduled-kernel proposal, applicability, structural cost, provenance, installation, and decline types.

**Excluded, each by a stated reason.** A complete-`Vec` `ProviderOffer` or deprecated alias; any public constructor for the sink; any spelling that lets a provider supply the bounded channel; a public pull-protocol twin of the sink; `enumerate_frontier`, `FrontierOutcomeBudget`, `FrontierBudgetResource`, `PhysicalAuthorities`, and `GovernedPhysicalProvider`; any removal of the governed provider.

## The questions that are genuinely Tom's

1. **Is `&mut PhysicalFrontierSink` the accepted emission seam?** The host owns the sink, charges every proposal and every `DeclinedStrategy` before accepting it, and latches overflow so an ignored insertion result cannot retain a prefix.
2. **Is the two-budget split right?** Provider count is preflighted as `ExactDemand` over the complete population, governed included. Raw outcomes are a request-scoped `SearchLowerBound` cardinality of 256, sized from the expensive (proposal) side.
3. **Is atomic refusal at `limit + 1` the accepted exhaustion disposition?** No prefix, no reserved governed entry, no retry across backend, target, provider set, numerical contract, or budget policy.

## Recommendation

Accept the sink, the two-budget split, and atomic refusal as built. They implement the decision already accepted on 2026-08-11 in [`decide-whether-the-implementation-frontier-owes-a-retention-budget`](decide-whether-the-implementation-frontier-owes-a-retention-budget.md) without inventing a second public surface.

**Strongest counterpoint:** a pull protocol would make overflow a host-side loop rather than a provider-ignored `Result`, at the cost of forcing every provider to be an iterator.

## Closes when

Tom accepts or revises the exact included and excluded signature at the reviewed commit; the module documentation and governing contracts state that accepted boundary rather than a draft.
