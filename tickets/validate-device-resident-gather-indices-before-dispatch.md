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
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **This condition is not mechanically checkable, and saying so is the repair.** The trigger is *a selected workload* supplying gather indices that cannot use the accepted host-visible immutable-snapshot lane, or a measurement showing host transfer dominates the relevant invocation. Workload selection is recorded in a profile record and measurement in a retained run; neither is a code state, and the entry above states the current selection plainly — the first vertical is deliberately host-visible U32 input. A human must read the selected workload profile for a device-resident index producer and the retained runtime measurements for a host-transfer figure. The adjacent checkable fact, which is not the trigger, is that `tiler::u32@1` is gather's admitted index identity in the governed key census; its presence is the *host-visible* lane, so it cannot be read as this trigger firing. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
