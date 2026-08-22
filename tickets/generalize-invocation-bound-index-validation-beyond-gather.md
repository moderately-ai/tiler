---
id: generalize-invocation-bound-index-validation-beyond-gather
title: Generalize invocation-bound index validation beyond gather
status: deferred
priority: p3
dependencies: [admit-an-invocation-scoped-gather-index-validation-receipt]
related: [scope-the-scatter-and-indexed-update-family, validate-device-resident-gather-indices-before-dispatch]
scopes: [research/indexing, contracts/foundation, contracts/decisions, implementation/ir, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, indexing, validation, architecture]
---
## Question

Which parts of the gather-specific invocation receipt are genuinely shared by a second indirect operation, and which must remain operation-owned semantic validation?

## Trigger

Activate only after the gather receipt lands and a second admitted operation requires invocation-bound index-domain validation. Derive the shared population from both complete contracts; do not generalize from gather alone or add a callback/schema registry in anticipation.

## Trigger check log

- 2026-08-11 — **not fired**. Gather is the only admitted semantic family requiring this validation; scatter remains separately deferred.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Checkable half.** The trigger is *a second admitted operation* requiring invocation-bound index-domain validation, and the admitted-family population is exactly what the governed key census reports: `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` reports the **19** registered operation-key constructors, and `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` reports **50 unique governed keys** — unique keys through `sort -u`, not lines of output, among which `tiler::gather-f32@1` is the only index-consuming family. A second such family joining that census is the changed answer, and the check is one-directional — a new family key is necessary for the trigger and does not by itself establish that the family *requires* this validation. **This condition is not mechanically checkable, and saying so is the repair.** That second part is a reading of the new family's complete contract, and the trigger's own instruction is to derive the shared population from *both* complete contracts rather than generalizing from gather alone. A human must read the new family's index-domain obligations. Scatter, the nearest candidate, remains separately deferred at [`scope-the-scatter-and-indexed-update-family`](scope-the-scatter-and-indexed-update-family.md). Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
