---
id: validate-device-resident-gather-indices-before-dispatch
title: Validate device-resident gather indices before dispatch
status: deferred
priority: p2
dependencies: [admit-an-invocation-scoped-gather-index-validation-receipt]
related: [emit-the-indirect-gather-on-metal, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices]
scopes: [research/runtime, implementation/runtime, implementation/metal, implementation/compiler, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, runtime, gather, metal, validation]
---
## Question

How can Tiler validate a device-resident or device-produced index buffer against the exact gather extent, order that validation before gather over the same immutable storage, and surface failure before any semantically invalid gather executes?

## Trigger

Activate only when a selected workload supplies gather indices that cannot use the accepted host-visible immutable-snapshot lane, or measurement shows host transfer dominates the relevant invocation. The design must compare a validation kernel, backend-native checked primitive, and any safe host-visible mapping without treating device work as a successful routing commit or permitting fallback.

## Non-goals

Changing gather semantics, trusting a device buffer without validation, inline unchecked addressing, or reusing a receipt after mutation.

## Trigger check log

- 2026-08-11 — **not fired**. The first vertical is deliberately host-visible U32 input; no selected workload yet requires device-resident index production.
