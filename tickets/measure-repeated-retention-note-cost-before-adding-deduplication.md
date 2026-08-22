---
id: measure-repeated-retention-note-cost-before-adding-deduplication
title: Measure repeated retention-note cost before adding deduplication
status: deferred
priority: p3
dependencies: [preserve-retained-tool-bytes-in-macro-read-back]
related: [accept-the-retention-read-back-s-caller-visible-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [frontend, diagnostics, performance, deferred]
---
## User-visible outcome

Only measured repeated rust-analyzer retention output can authorize a bounded per-artifact deduplication policy; until then every distinct expansion remains reportable.

## Why deferred

**Fact — each expansion is bounded and the healthy case is silent.** A retention carries at most 16 runs of 16 KiB; the current quiet Metal publication carries two empty runs and writes nothing.

**Inference — cumulative defect-case output may still dominate frontend work.** A cached compilation carrying a warning is read on every rust-analyzer expansion request. The repository has no retained measurement of request frequency, repeated byte volume, formatting time, lock contention, or developer impact. Adding a process-global set now would spend complexity on an unmeasured problem and create a new way to suppress evidence.

## Required decision if fired

- Compare ungated output with bounded per-artifact/retention deduplication and an explicit user policy.
- Any deduplication key must name the exact artifact/retention subject, not merely “some note was printed”.
- State a finite memory bound. On overflow or an unnameable subject, fail open by reporting; never silently drop an unknown diagnostic.
- Do not change compilation success, artifact/cache identity, retention storage, backend selection, or runtime execution.

## Non-goals before the trigger

No once-per-process flag, user setting, hashing/global lock on the expansion path, or suppression behavior.

## Trigger check log

- 2026-08-11 — **not fired.** No retained rust-analyzer measurement shows repeated retention notes materially affecting Tiler expansion time or usability. Reconsider when a reproducible session records expansion count, repeated artifact/retention identity, total bytes written, and frontend wall time with and without reporting.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **This condition is not mechanically checkable, and saying so is the repair.** The reconsideration condition is a *reproducible rust-analyzer session* recording expansion count, repeated artifact and retention identity, total bytes written, and frontend wall time with and without reporting. That is a measurement taken on a developer's editor, not a state of this repository, and the entry above says as much: no such retained measurement exists. There is deliberately no proxy command here, because every candidate — counting retention sites, or measuring expansion bytes — would measure the mechanism rather than the cost, and the ticket's own `## Why deferred` already records that the mechanism is bounded and the healthy case silent. A human must look for a retained session record carrying those four quantities; until one exists the condition is unevaluable rather than not fired. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
