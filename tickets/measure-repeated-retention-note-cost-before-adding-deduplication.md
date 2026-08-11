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

## Trigger check log

- 2026-08-11 — **not fired.** No retained rust-analyzer measurement shows repeated retention notes materially affecting Tiler expansion time or usability. Reconsider when a reproducible session records expansion count, repeated artifact/retention identity, total bytes written, and frontend wall time with and without reporting.

## Required decision if fired

- Compare ungated output with bounded per-artifact/retention deduplication and an explicit user policy.
- Any deduplication key must name the exact artifact/retention subject, not merely “some note was printed”.
- State a finite memory bound. On overflow or an unnameable subject, fail open by reporting; never silently drop an unknown diagnostic.
- Do not change compilation success, artifact/cache identity, retention storage, backend selection, or runtime execution.

## Non-goals before the trigger

No once-per-process flag, user setting, hashing/global lock on the expansion path, or suppression behavior.
