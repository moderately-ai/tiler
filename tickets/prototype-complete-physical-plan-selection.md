---
id: prototype-complete-physical-plan-selection
title: Select and verify complete physical plans
status: done
priority: p0
dependencies: [prototype-region-cover-enumeration, prototype-physical-implementation-frontier]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, program-selection]
---
Join independently verified legal covers with compatible per-region physical
frontiers. Verify complete occurrence/output coverage, boundary agreement,
proposed materializations and dependencies, deliberate duplication, guards, and
deterministic portfolio retention. Emit a non-forgeable checked selected-plan
or selected-portfolio receipt distinct from structured KIR and `KernelProgram`.
The P0 selector may use proved structural dominance; it must not invent
uncalibrated latency authority.

This receipt is not final executable-program authority. Only post-KIR
`KernelProgram` assembly verifies buffers, initialization, lifetimes, aliasing,
storage handoffs, ABI/launch references, executable stage coverage, and routing.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Fact.** Added `crates/tiler-compiler/src/selection.rs` (registered as a private `mod selection` in `lib.rs`), a `pub(crate)`-only draft authority mirroring the `cover`/`frontier`/`fusion_legality` convention (`#![allow(dead_code, reason=…)]`, not wired into `compile()`; wiring is a later conformance-gate ticket). It joins one independently verified legal `RegionCover` with one already-enumerated per-region `ImplementationFrontier` into complete physical plans. Entry points: `select_physical_plans(program, budgets, contract, &[CoverFrontiers], PlanBudgets) -> Result<SelectedPortfolio, SelectionError>`, `verify_selected_plan(...)`, and `verify_selected_portfolio(...)`. Inputs are `CoverFrontiers { cover, Vec<RegionFrontier> }` where `RegionFrontier { FrontierRegionSubject, ImplementationFrontier }`.

**Fact.** The cover↔frontier join re-verifies the cover from the program with `cover::verify_cover` (a foreign or stale cover fails closed), binds each cover region one-to-one to a supplied frontier by canonical semantic-member set, and cross-checks that every admitted implementation covers exactly that region and target profile. For each combination of one admitted implementation per region it reconciles boundaries: a cross-region materialized value is a `TensorRole::Intermediate` handoff, so each `MaterializationEdge` binds the producer region's unique intermediate `BoundaryGuarantee` to each consumer region's unique intermediate `BoundaryRequirement`, checking the producer's `BoundaryProduction` discharges the consumer's `BoundaryAvailability`. A per-region closure/ambiguity guard fails closed on a leaked intermediate, a dangling read, an undischarged handoff, or more than one intermediate per region boundary.

**Fact.** Hard feasibility (already decided by the frontier) stays distinct from cost. The `SelectedPortfolio` retains every valid complete plan in `plans()`; `non_dominated()` is a pure structural view that prunes only a plan another plan beats under the Pareto relation over exact structural counts (aggregate dispatch count, launched threads, temporary bytes, and cover materialization count) with a matching cost-model key. No dimension is collapsed into a scalar latency; cost never gates validity or feasibility.

**Fact.** The receipt is non-forgeable and deterministic. A `SelectedPlan`/`SelectedPortfolio` is produced only by the checked constructor; `SelectedPlanIdentity` folds the cover identity, per-region `ImplementationProposalIdentity`, satisfied handoffs, deduplicated guards, and aggregate cost in a canonical length-prefixed byte encoding over content-derived coordinates (no transient ordinals, no `HashMap` order). `verify_selected_plan` re-derives the whole plan via the same `assemble_plan` path and requires it to reproduce the receipt exactly; a tampered cost, a swapped implementation, or a foreign program each fails closed.

**Fact.** Rejection taxonomy separates valid dispositions from malformed faults. Legitimate no-plan dispositions are recorded on the portfolio as `PlanRejection` (`RegionUnimplemented`, `BoundaryDisagreement`) and never fail the enumeration; an empty `plans()` with rejections is a valid no-plan result. Malformed compiler output fails closed with `SelectionError` (`MalformedCover`, `FrontierBinding`, `InvalidComposition`, `Structure`) carrying `class`/`reason`. Nine tests cover the two-region boundary handoff, structural-dominance pruning-as-a-view, deterministic order-independent identity, forged/foreign-receipt rejection, valid no-plan vs malformed fault, binding mismatch, and boundary-disagreement detection.

**Measurement.** Full gate `uv run --locked python scripts/check_repository.py` passed (0 warnings; "complete repository validation passed"). `git diff --check` clean. `ticketsplease guard tkt/prototype-complete-physical-plan-selection`: affected scope `implementation/compiler` (+ shared `project/tickets`), inside declared scopes.

**Inference.** Within the bounded serial-sum physical profile, a boundary *disagreement* between correctly member-bound implementations is not producible end-to-end: a region's member set uniquely determines its scheduled region and hence its boundary contract, and the frontier re-verifies every implementation against the same request subject, so the cover's edges and the physical contracts are consistent by construction. The disagreement guard is therefore exercised directly against `reconcile_boundaries` with real cover occurrences and a deliberately inconsistent boundary facet; the positive composition path is exercised end-to-end on the real two-region plan. This is a genuine measurement boundary, not a gap in the guard.

**Proposal (deferred).** The receipt is deliberately pre-KIR: post-KIR `KernelProgram` assembly (buffers, initialization, lifetimes, aliasing, storage handoffs, ABI/launch references, executable-stage coverage, routing) is a later ticket and is neither performed nor pre-empted here. The `TensorRole::Intermediate`-keyed handoff binding is exact for the bounded profile; a value that is simultaneously a named output and a cross-region source, or more than one intermediate per region boundary, would need a finer edge↔tensor binding and is reserved (the guard fails closed rather than approximating it). Every `pub(crate)` boundary here remains a draft until Tom accepts the exact interface.
