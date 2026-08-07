---
id: design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature
title: Design the measured-feedback tuning loop against the autotuning and adaptive-execution literature
status: todo
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

> **The Fact below was false when written and is struck. Corrected 2026-08-07.** It read: "The analytic cost model does not exist yet (the 2026-08-05 audit: no cost/estimate/ranking type in `crates/`)". That audit rested on a grep anchored `^(pub )?`, which cannot match a `pub(crate)` declaration — and every cost and estimate type in this workspace is `pub(crate)`. `calibrate-device-cost-models` already cited `crates/tiler-compiler/src/component_cost.rs` as of **2026-07-28**, a week before the audit that reported the file's contents absent. The deferral was therefore correct in its *conclusion* — the tuning loop still had nothing settled to refine — and wrong in the reason it gave.

**Fact, 2026-08-07 — the cost authority exists and is already consuming measurement.** `crates/tiler-compiler/src/component_cost.rs:76` declares `ANALYTICAL_MODEL_KEY = "tiler.cost.analytical.v1"`, with `CostComponent`, `CostUnit`, `CostValue`, `ComponentCost` and `AnalyticalPlanCost` beside it; the key is consumed in `frontier.rs` and pinned in `pipeline/tests.rs`. `cost-model-bootstrap` and `implement-analytical-component-cost-model` both read `status: done`. A **measured** selector landed alongside it — `crates/tiler-compiler/src/measured_cost.rs`, key `tiler.cost.measured-fold-steps.v1` — which ranges over retained valid plans and can prefer a structurally dominated one.

**Inference.** So the authority this loop would refine is not merely present but already taking device measurement into selection, which is this ticket's own subject matter. The design questions are now askable against something real rather than against a guess.

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
- 2026-08-07 — **FIRED on the first disjunct.** The trigger is a disjunction — "the first analytic cost-model landing in `crates/` … **or** `calibrate-device-cost-models` firing, whichever arrives first" — and the first has arrived. Verified independently by the coordinator: `crates/tiler-compiler/src/component_cost.rs:76` declares `ANALYTICAL_MODEL_KEY = "tiler.cost.analytical.v1"`, consumed at `frontier.rs` and pinned in `pipeline/tests.rs`, with `CostComponent`, `CostUnit`, `CostValue`, `ComponentCost` and `AnalyticalPlanCost` beside it. On the graph, `cost-model-bootstrap` and `implement-analytical-component-cost-model` both read `status: done`. A **measured** selector has since landed alongside it (`crates/tiler-compiler/src/measured_cost.rs`, key `tiler.cost.measured-fold-steps.v1`), so the cost authority this ticket exists to refine is not merely present but already consuming device measurement — which is this ticket's own subject.

  **This ticket's recheck command is broken and could never have reported the firing.** `grep -rnE "^(pub )?(struct|enum) [A-Za-z]*(Cost|Estimate|Rank|Score)" crates/ --include='*.rs'` returns exactly one line — a shape-rank marker — because **every cost and estimate type is declared `pub(crate)`, which the `^(pub )?` anchor cannot match.** Re-running the documented command today still reads as not-fired. That is a check which cannot say *yes*: the inverse of the failure this repository usually guards against, and it would have kept this ticket parked indefinitely.

  **Its stated Fact was also already false when written**: "The analytic cost model does not exist yet (the 2026-08-05 audit: no cost/estimate/ranking type in `crates/`)" — but `calibrate-device-cost-models` cites `crates/tiler-compiler/src/component_cost.rs` as of 2026-07-28, a week earlier. Both the Fact and the command must be corrected before this is briefed. Working recheck: `grep -n 'ANALYTICAL_MODEL_KEY' crates/tiler-compiler/src/component_cost.rs && grep -m1 '^status:' tickets/cost-model-bootstrap.md`.
