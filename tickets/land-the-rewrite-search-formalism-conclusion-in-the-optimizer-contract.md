---
id: land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract
title: Land the rewrite-search formalism conclusion in the optimizer contract
status: in-progress
priority: p2
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature]
scopes: [contracts/optimizer, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contract, optimizer, search, carrier]
claimed_from: todo
assignee: agent-formalism-carrier
lease_expires_at: 1785964902
---
## User-visible outcome

The optimizer contract stops saying "a Cascades-style memo is one possible implementation technique, not a committed architecture" and states the selected formalism, and the invariant that makes it work is written down where a reviewer will cite it: **no semantic alternative is pruned on estimated cost, at any stage.**

## Why this exists

**Fact.** [The rewrite-search formalism record](../docs/research/region-search/rewrite-search-formalism.md) landed 2026-08-05 with the elimination stated and its sources preserved. It could not edit the contract: `docs/compiler/**` is the `contracts/optimizer` scope and that survey held only `research/region-search`. This is the carrier for the transfer.

**Inference.** The record's `disposition` is `pending` precisely because this ticket has not run. Until it does, a reader of the contract's "Bounded hierarchical search" section is told the question is open when it has been answered, which is the exact stale-status hazard AGENTS.md names.

## What the edit owes

- Replace the uncommitted-memo sentence with the selected staged formalism, citing the record rather than restating its derivation.
- Add the no-cost-pruning invariant beside the four surfaces in "The four surfaces the optimizer may consult", as a review rule with the same shape as the tier/backend review obligation already there — nothing mechanical can check it, and a profitability check added at stage 3 would silently reintroduce the phase-ordering hazard the record's spike demonstrates.
- Record what a budget may and may not be, from the record's Orca comparison: a count of work performed, never a wall-clock time-out or a cost threshold, because the same request must compile to the same portfolio twice.
- Move the record's `disposition` from `pending` to `adopted` **in the same change**, and update the research catalog row that restates it. That field is in `research/region-search`, so this ticket must declare that scope too, or a second commit must carry it — decide which when claiming and say so.
- Sweep for contract sentences whose truth depended on the old status.

## Non-goals

Implementing anything; adopting an e-graph; committing a cost model; re-deriving the elimination.

## Closes when

The contract states the formalism and the invariant, no sentence in `docs/compiler/` still describes the mechanism as unchosen, the record's disposition and both catalog views agree, and `tkt lint` passes.
