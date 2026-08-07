---
id: design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature
title: Design the measured-feedback tuning loop against the autotuning and adaptive-execution literature
status: done
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

## Why this exists, and why it was deferred until 2026-08-07

> **The Fact below was false when written and is struck. Corrected 2026-08-07.** It read: "The analytic cost model does not exist yet (the 2026-08-05 audit: no cost/estimate/ranking type in `crates/`)". That audit rested on a grep anchored `^(pub )?`, which cannot match a `pub(crate)` declaration — and every cost and estimate type in this workspace is `pub(crate)`. `calibrate-device-cost-models` already cited `crates/tiler-compiler/src/component_cost.rs` as of **2026-07-28**, a week before the audit that reported the file's contents absent. The deferral was therefore correct in its *conclusion* — the tuning loop still had nothing settled to refine — and wrong in the reason it gave.

**Fact, 2026-08-07 — the cost authority exists and is already consuming measurement.** `crates/tiler-compiler/src/component_cost.rs:76` declares `ANALYTICAL_MODEL_KEY = "tiler.cost.analytical.v1"`, with `CostComponent`, `CostUnit`, `CostValue`, `ComponentCost` and `AnalyticalPlanCost` beside it; the key is consumed in `frontier.rs` and pinned in `pipeline/tests.rs`. `cost-model-bootstrap` and `implement-analytical-component-cost-model` both read `status: done`. A **measured** selector landed alongside it — `crates/tiler-compiler/src/measured_cost.rs`, key `tiler.cost.measured-fold-steps.v1` — which ranges over retained valid plans and can prefer a structurally dominated one.

**Inference.** So the authority this loop would refine is not merely present but already taking device measurement into selection, which is this ticket's own subject matter. The design questions are now askable against something real rather than against a guess.

## The literature-survey obligation

Preserve primary sources per the source-record discipline: **AutoTVM** and **Ansor** (hierarchical search + learned cost refinement), **OpenTuner**, the **Halide autoscheduler** papers, and — the deliberately-drawn database parallel — **adaptive query execution** literature (mid-execution replanning on observed cardinalities) and **parameterized plan caching**, whose transfer questions (when does a cached plan survive a parameter change?) are this system's shape-family transfer questions with twenty years of prior art. Statistical discipline for noisy GPU timings (warm-up, repetition, outlier policy) is part of the survey, and every measurement obligation routes to the designated measurement host per the coordination record.

## What the record must decide or defer

Where measurement enters (offline calibration vs in-loop tuning vs post-hoc refinement); the tuning store's identity discipline — it is NOT the expansion cache (results carry environment rows and noise, entries are refutable rather than immutable) and the design must say exactly which of the cache's invariants (validation on hit, atomic publication) transfer and which invert; the transfer model across shapes; and the fail-closed story when measurements are unavailable (the analytic model answers, never a stale measurement silently).

## Non-goals

Any implementation; any timing collected before the trigger fires; committing a learned cost model (a later question the survey may scope).

## Trigger — fired, retained for provenance

The trigger was: the first analytic cost-model landing in `crates/` (the `bootstrap-cost-model` thread reaching implementation), or `calibrate-device-cost-models` firing — whichever arrives first. **The first disjunct fired on 2026-08-07** and the ticket is `todo`; the log entry below records the verification and the corrected recheck command. Nothing here is still waiting on a condition.

## Trigger check log

- 2026-08-05 — **not fired.** No cost, estimate, or ranking type exists in `crates/` (the audit's grep stands), and `calibrate-device-cost-models` is `deferred` with its own unfired log. Recheck: `grep -rnE "^(pub )?(struct|enum) [A-Za-z]*(Cost|Estimate|Rank|Score)" crates/ --include='*.rs'` returning more than the shape-rank marker, or `grep -m1 '^status:' tickets/calibrate-device-cost-models.md` printing other than `status: deferred`.
- 2026-08-07 — **FIRED on the first disjunct.** The trigger is a disjunction — "the first analytic cost-model landing in `crates/` … **or** `calibrate-device-cost-models` firing, whichever arrives first" — and the first has arrived. Verified independently by the coordinator: `crates/tiler-compiler/src/component_cost.rs:76` declares `ANALYTICAL_MODEL_KEY = "tiler.cost.analytical.v1"`, consumed at `frontier.rs` and pinned in `pipeline/tests.rs`, with `CostComponent`, `CostUnit`, `CostValue`, `ComponentCost` and `AnalyticalPlanCost` beside it. On the graph, `cost-model-bootstrap` and `implement-analytical-component-cost-model` both read `status: done`. A **measured** selector has since landed alongside it (`crates/tiler-compiler/src/measured_cost.rs`, key `tiler.cost.measured-fold-steps.v1`), so the cost authority this ticket exists to refine is not merely present but already consuming device measurement — which is this ticket's own subject.

  **This ticket's recheck command is broken and could never have reported the firing.** `grep -rnE "^(pub )?(struct|enum) [A-Za-z]*(Cost|Estimate|Rank|Score)" crates/ --include='*.rs'` returns exactly one line — a shape-rank marker — because **every cost and estimate type is declared `pub(crate)`, which the `^(pub )?` anchor cannot match.** Re-running the documented command today still reads as not-fired. That is a check which cannot say *yes*: the inverse of the failure this repository usually guards against, and it would have kept this ticket parked indefinitely.

  **Its stated Fact was also already false when written**: "The analytic cost model does not exist yet (the 2026-08-05 audit: no cost/estimate/ranking type in `crates/`)" — but `calibrate-device-cost-models` cites `crates/tiler-compiler/src/component_cost.rs` as of 2026-07-28, a week earlier. Both the Fact and the command must be corrected before this is briefed. Working recheck: `grep -n 'ANALYTICAL_MODEL_KEY' crates/tiler-compiler/src/component_cost.rs && grep -m1 '^status:' tickets/cost-model-bootstrap.md`.

## Outcome — done, 2026-08-07

Landed at **`923f7703`**: `docs/research/cost-model/measured-feedback-tuning-loop.md` (245 lines) and `docs/research/cost-model/sources/README.md` (129 lines).

**The finding that reframes the record, verified independently by the coordinator.** No measurement enters compilation at all: `grep -rn 'EstimateProvenance::Measured' crates/ --include='*.rs'` returns **nothing** — the variant has zero producers — and what `measured_cost` consumes is a `u64` read off a target profile that is a **literal**, `saturated_parallel_fold_steps: Some(1_056)` at `crates/tiler-build/src/metal_declaration.rs:337`. So "measured" in the compiler today names the *provenance of a hand-transcribed constant*, not a loop. This ticket's premise was true in letter and misleading in weight; the record accordingly decides whether the transcription path should ever *become* a loop, and concludes it is the right design stated as a rule rather than left an accident.

**Decisions taken:** measurement enters by offline calibration only, with in-loop tuning deferred behind a two-part trigger (Ansor's 1,000 trials per test case would have to run inside a proc-macro under the no-runtime-JIT constraint; and Halide 2016→2019 puts ~75% on the model for only ~1.34x more from hours of autotuning). The tuning store's five cache properties are taken one at a time: complete identity transfers and must widen; validation splits, with semantic validation **inverting** because a cache hit is self-proving and a tuning hit only unrefuted; immutability **inverts one level down** — observations immutable and append-only, verdict derived and replaceable. Transfer across shapes is machine-parameter fitting only, refusing a per-shape winner table on three independent grounds. Fail-closed gives six typed resolutions and **no "last known good" measurement**.

**Honest findings retained rather than smoothed:** the repository's own separation rule fails Hoefler & Belli Rule 6 (no distributional check recorded behind standard errors of medians) — recorded as an inherited weakness affecting the two held-out misses, with the nonparametric repair left to the protocol's owner. Eleven primary sources retrieved, SHA-256'd and read as text, with WebFetch PDF summaries discarded as unverifiable paraphrase after one produced non-existent Ansor "quotes". Three sources unreachable, each with reference, attempt, and the decision it would have informed; no claim rests on them.

Three follow-on tickets filed: the catalog carrier (the record lands uncatalogued because `docs/research/README.md` is `contracts/navigation`), source vendoring pending licence reads, and acquisition of the three unreachable sources.
