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
