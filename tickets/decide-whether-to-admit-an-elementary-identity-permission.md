---
id: decide-whether-to-admit-an-elementary-identity-permission
title: Decide whether to admit an elementary-identity permission
status: deferred
priority: p2
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, carry-the-elementary-identity-dimension-adr, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, connect-certified-rounding-error-bounds-to-rewrite-permissions, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
scopes: [contracts/decisions, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, optimizer, decision, deferred]
---
## User-visible outcome

Tom decides whether the numerical contract gains a twelfth dimension a caller can resolve to `Permitted`, so that a rewrite through an elementary function's functional equation is either reachable under a stated permission or is refused by decision rather than by absence.

## Why this exists

**Fact.** [The elementary-identity rewrite dimension record](../docs/research/numerics/elementary-identity-rewrite-dimension.md) derives the dimension, its grain, its relation to the accuracy machinery, and its cost, and deliberately makes no product choice — exactly as [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) item 4 reserved the analogous choice for distributivity rather than making it.

**Fact.** The drafted ADR's item 5 states the reservation and this ticket owns it. Admitting is a public-boundary decision under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and an identity-domain step besides.

## Why this is deferred rather than todo

**The dimension has no caller that is not already blocked elsewhere, and that is the same ground [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declined the distributivity permission on.** Its one identified caller is the online-softmax rescaling fold, which independently consumes distributivity, for which ADR 0095 declines a permission. Admitting this one alone would enable no rewrite: it would widen every contract, oblige every target profile to declare for a dimension, and step the contract-key domain at both widths, to authorize something a separate accepted decision refuses.

The record's Part 6 records the search for a caller that needs only this dimension rather than assuming there is none — a pure elementwise `exp(a) * exp(b)` fold is unstatable because no general `Exp` key is registered; log-sum-exp's shift consumes distributivity too; and the pinned workload has no chain of two square-root products. Filed `deferred` at creation for that reason, rather than filed dispatchable and parked later.

## Trigger

**Fires on either of two events, and both are single observations rather than judgements.**

1. **The distributivity reassessment resolves in the admitting direction.** [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md) is Tom's and is open; if it admits a permission, this dimension acquires a spendable caller in the same instant, and the two decisions are better taken together than sequentially. A reaffirmed decline does **not** fire this.
2. **A workload's natural spelling consumes an elementary identity without also consuming distributivity.** The shape to watch for is a product of two square roots or two exponentials with no sum between them. A workload that merely *contains* an elementary function does not fire it.

## What the decision needs in front of it

- The record's Part 5 cost accounting: the two key-domain steps, the two pinned literals, the widened injectivity check, and the obligation on every target profile to declare — with the offline Metal measurement that makes one such declaration available and the runtime gap that makes it partial.
- The record's Part 6 table of the four distributivity/identity outcomes, so the question states what admitting this one alone does and does not buy.
- Whether the numeric elementary accuracy is retrievable, since a *quantitative* admission needs it: [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md).

## Non-goals

Superseding any ADR from a worker branch; making the choice on an agent's authority; implementing any permission; presuming the distributivity reassessment's outcome.

## Trigger check log

- 2026-08-05 — **not fired.** Clause 1 cannot have fired: `reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller` is `todo` and unclaimed, so no reassessment has resolved in any direction. Clause 2 has not fired: the record's Part 6 enumerates the candidate rewrites and every one either is unstatable in the registered operation set or consumes distributivity too. Recheck with `tkt show reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller --format json | grep '"status"'`, which answers `todo` while clause 1 is unfired.

## Trigger check log

- 2026-08-06 — not fired. ADR 0101's acceptance names the dimension but is neither trigger: (1) the distributivity reassessment is unresolved — `reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller` is not `done` and ADR 0095 stands; (2) no workload spelling consuming an elementary identity without distributivity has been identified. Reproduce: `tkt show reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller --format json | grep '"status"'`.
- 2026-08-06 (second evaluation) — not fired. The distributivity reassessment resolved: Tom reaffirmed ADR 0095's decline at the live decision review, which the trigger's own text says does **not** fire this. The reaffirmation added a joint reopening condition to ADR 0095 naming this ticket's subject — both permissions considered together when a consuming rule with an instantiable bound at a schedulable fold shape exists — so the next firing check is that condition's prerequisites. Reproduce: `grep -n "Second reopening condition" docs/decisions/0095-decline-a-distributivity-permission.md`.
