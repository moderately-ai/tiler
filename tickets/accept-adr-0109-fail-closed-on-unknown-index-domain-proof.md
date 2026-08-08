---
id: accept-adr-0109-fail-closed-on-unknown-index-domain-proof
title: Accept ADR 0109 fail closed on Unknown index-domain proof
status: done
priority: p1
dependencies: []
related: [reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement, repair-adr-0078s-budget-stop-and-unknown-gap-evidence]
scopes: [contracts/decisions, contracts/navigation, contracts/foundation, contracts/optimizer, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [decision, correctness]
---
## The decision

[ADR 0109](../docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md) narrowly supersedes [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md)'s historical `Ok` and “the plan stands” requirement for an exhausted index-domain proof. ResourceLimit remains `Unknown`, neither disproof nor admission; compilation fails closed before executable planning and coverage; an assessed `Disproved` claim takes precedence; and every produced assessment remains explainable. No public analysis-only result, executable pending coverage, artifact/cache/runtime fallback, or numerical change is accepted.

This node records an acceptance that happened before the record was drafted, following the accepted-before-dispatch carrier precedent of [`accept-adr-0105-retire-the-scalar-lowering-seam`](accept-adr-0105-retire-the-scalar-lowering-seam.md). It is `done` because the decision has been taken; a ticket conditional on the decision depends on this acceptance node rather than on the reconciliation work that assembled its evidence.

## Decided — accepted

**Accepted by Tom on 2026-08-08 in the current Codex session**, relayed to the author by the coordinator from Tom's message, “yes you may make the correct decision and accpet the change”. The accepted packet was the narrow supersession recommended by the historical/current derivation: retain ResourceLimit as Unknown, refuse before executable coverage, preserve Disproved precedence and complete assessment explanation, and widen no public, identity, artifact, cache, runtime, or numerical boundary.

## Sweep

The same change lands ADR 0109 as accepted, records the item-scoped supersession on ADR 0078, aligns the operation-extension contract's stale sole-diagnostic sentence, cites the accepted authority from the optimizer and runtime-execution contracts, adds both hand-maintained catalog rows, and hardens the IR ledger tests. Production executable behavior and every identity domain remain unchanged.
