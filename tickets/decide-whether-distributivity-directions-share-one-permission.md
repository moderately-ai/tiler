---
id: decide-whether-distributivity-directions-share-one-permission
title: Decide whether factoring and expansion share one permission
status: deferred
priority: p3
dependencies: [decide-whether-to-admit-a-distributivity-permission]
related: [settle-contraction-chain-distributivity-permission]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision]
---
Activate only if Tiler admits a distributivity permission.

Factoring `sum(x * c)` into `sum(x) * c` and expanding it back have the same
algebraic identity but different structural preconditions and error behavior.
Determine from concrete rewrite and numerical evidence whether one permission
honestly grants both directions or whether each direction needs a distinct
caller authorization.

## Closes when

The accepted numerical contract states one or two permissions with the evidence
that distinguishes them, and every admitted rewrite checks the corresponding
direction explicitly.

## The parent decided, and it declined (2026-08-01)

**Fact.** [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) closed on 2026-08-01 with [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md): Tom declined, no distributivity permission is admitted, and the contraction-chain regroup stays explicitly rejected. **So this ticket's opening line is now precise rather than conditional** — Tiler did not admit a distributivity permission, this question does not arise, and nothing here is dispatchable.

**Why this stays `deferred` rather than closing.** `deferred` is a parked state `tkt ready` excludes and that never satisfies a dependent, which is exactly the semantics wanted: the question is not abandoned, it is not-arising, and the condition that would make it arise is written above and is unchanged. **Its activation condition is now precisely ADR 0095's reopening trigger** — the first workload whose *natural spelling* is a directly regroupable contraction chain, one where the regrouping that consumes distributivity is what the workload asks for rather than one an optimizer might speculatively want. This ticket cannot fire on its own: it fires only if that trigger reopens the parent and the parent then admits a permission.

**A divergence from the parent's own plan, stated rather than absorbed.** The parent's 2026-08-01 section listed "close [this ticket], whose question does not arise under a decline" among the remaining work. `closed` is a terminal state that does not satisfy dependents, and it would be a defensible reading; `deferred` was chosen instead because it keeps the conditional framing that makes the reopening path a single status change rather than a re-filing, and because [`sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired`](sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired.md) carries this pair as its worked example of a fired trigger holding a dependent parked — a sweep that needs the node to still exist in a parked state to be worth anything. Nothing about the decision changes either way.

## Trigger check log

- 2026-08-04 — **not fired.** ADR 0095 stands and the parent [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) has not been reopened, so the question still does not arise. This pair is the sweep's worked example and the parking is deliberate: `deferred` keeps the node reachable for the reopening path. Recheck: the parent's status.
- 2026-08-09 — **not fired.** The parent is `done` on the accepted decline and has not been reopened; ADR 0095 remains accepted. No permission exists whose two directions need one-or-two identity treatment, so this conditional question remains deliberately parked. Recheck `tkt show decide-whether-to-admit-a-distributivity-permission` and the anchor `decision_status: "accepted"` in `docs/decisions/0095-decline-a-distributivity-permission.md`.
