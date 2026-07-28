---
id: implement-analytical-component-cost-model
title: Implement an analytical component cost model
status: todo
priority: p1
dependencies: [implement-boundary-property-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, performance]
---
Implement deterministic symbolic component costs for memory traffic,
allocation, dispatch, redundant work, indexing, synchronization, resource
pressure/occupancy, compile time, and artifact size. Preserve units,
assumptions, uncertainty, target-profile subjects, and typed explain; hard
feasibility remains separate. This is explicitly analytical and uncalibrated.
`calibrate-device-cost-models` owns later device measurements and activation.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Not startable as written — no stated outcome (2026-07-27)

**This ticket has no `## Closes when` and no sections.** It names the model's subject and none of its acceptance. A cost model is only meaningful against a decision it changes, so "implemented" has to mean a specific plan choice moves, measurably, against a stated baseline — not that a formula exists.

**What it needs before it is claimable.** Which selection decision the model is supposed to change; what the current structural cost gets wrong that it would get right; and the measurement that would show it, on the M3, against `hot_path`'s existing numbers. Without that a cost model can be written, be plausible, and be unfalsifiable.

**One constraint already recorded elsewhere and worth carrying in:** `AGENTS.md` requires hard feasibility to stay separate from estimated cost — an infeasible plan is rejected with an explainable reason, never hidden behind an infinite cost. Whatever slice is stated must not blur that.
