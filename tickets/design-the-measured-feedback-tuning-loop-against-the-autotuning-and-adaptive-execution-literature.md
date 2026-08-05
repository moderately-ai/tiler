---
id: design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature
title: Design the measured-feedback tuning loop against the autotuning and adaptive-execution literature
status: deferred
priority: p2
dependencies: []
related: [calibrate-device-cost-models]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The architecture of search-guided-by-measurement — where on-device timing enters compilation, how tuning results are stored with full environment identity and noise statistics, and what transfers across shapes — exists as a designed seam before anyone needs it, grounded in the autotuning literature and its database ancestor, adaptive query execution.

## Why this exists, and why it is deferred

**Fact.** The analytic cost model does not exist yet (the 2026-08-05 audit: no cost/estimate/ranking type in `crates/`), and `calibrate-device-cost-models` sits deferred. A tuning loop refines a cost authority; designing the refinement before the authority exists would design against a guess. **Inference.** The design questions are nonetheless nameable now, and the literature survey loses nothing by waiting — which is why this files at `deferred` rather than `todo`.

## The literature-survey obligation (when fired)

Preserve primary sources per the source-record discipline: **AutoTVM** and **Ansor** (hierarchical search + learned cost refinement), **OpenTuner**, the **Halide autoscheduler** papers, and — the deliberately-drawn database parallel — **adaptive query execution** literature (mid-execution replanning on observed cardinalities) and **parameterized plan caching**, whose transfer questions (when does a cached plan survive a parameter change?) are this system's shape-family transfer questions with twenty years of prior art. Statistical discipline for noisy GPU timings (warm-up, repetition, outlier policy) is part of the survey, and every measurement obligation routes to the designated measurement host per the coordination record.

## What the record must decide or defer (when fired)

Where measurement enters (offline calibration vs in-loop tuning vs post-hoc refinement); the tuning store's identity discipline — it is NOT the expansion cache (results carry environment rows and noise, entries are refutable rather than immutable) and the design must say exactly which of the cache's invariants (validation on hit, atomic publication) transfer and which invert; the transfer model across shapes; and the fail-closed story when measurements are unavailable (the analytic model answers, never a stale measurement silently).

## Non-goals

Any implementation; any timing collected before the trigger fires; committing a learned cost model (a later question the survey may scope).

## Trigger

The first analytic cost-model landing in `crates/` (the `bootstrap-cost-model` thread reaching implementation), or `calibrate-device-cost-models` firing — whichever arrives first.

## Trigger check log

- 2026-08-05 — **not fired.** No cost, estimate, or ranking type exists in `crates/` (the audit's grep stands), and `calibrate-device-cost-models` is `deferred` with its own unfired log. Recheck: `grep -rnE "^(pub )?(struct|enum) [A-Za-z]*(Cost|Estimate|Rank|Score)" crates/ --include='*.rs'` returning more than the shape-rank marker, or `grep -m1 '^status:' tickets/calibrate-device-cost-models.md` printing other than `status: deferred`.
