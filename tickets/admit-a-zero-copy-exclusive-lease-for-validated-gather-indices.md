---
id: admit-a-zero-copy-exclusive-lease-for-validated-gather-indices
title: Admit a zero-copy exclusive lease for validated gather indices
status: deferred
priority: p3
dependencies: [admit-an-invocation-scoped-gather-index-validation-receipt]
related: [validate-device-resident-gather-indices-before-dispatch]
scopes: [research/runtime, implementation/runtime, implementation/frontend, implementation/artifact, contracts/integrations, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, runtime, gather, validation, performance]
---
## Question

Can an exclusive, asynchronously held storage lease prove that the host-visible index bytes checked during preflight are exactly the bytes later dispatched, without copying them into the receipt-owned immutable snapshot?

## Trigger

Activate only when retained measurement shows the snapshot copy is a material Tiler runtime cost for a selected workload, or a named integration cannot supply immutable snapshot ownership. The lease must prevent mutation and aliasing through completion, compose with cancellation and one-way commit, and fail closed when exclusivity cannot be established.

## Non-goals

A caller promise, a shared mutable slice, post-hoc mutation detection, content-hash caching, or fallback to unchecked execution.

## Trigger check log

- 2026-08-11 — **not fired**. No measurement shows the narrow host-visible snapshot copy is material, and the immutable copy is the simpler correctness boundary.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **This condition is not mechanically checkable, and saying so is the repair.** Both arms are evidence rather than code: *retained measurement* showing the snapshot copy is a material Tiler runtime cost for a selected workload, or *a named integration* that cannot supply immutable snapshot ownership. Neither can be read out of `crates/`, and a census of the snapshot lane would report the same answer in both worlds, which is precisely the unfireable shape this repair removes. A human must read the retained runtime measurements under `spikes/` for a snapshot-copy cost attributed to a selected workload, and the integration set for one that cannot hand over immutable ownership. Note the asymmetry the ticket records: the immutable copy is the simpler correctness boundary, so absence of evidence is a reason to stay deferred rather than a gap to close. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
