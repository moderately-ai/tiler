---
id: decide-whether-the-implementation-frontier-owes-a-retention-budget
title: Decide whether the implementation frontier owes a retention budget
status: done
priority: p2
dependencies: []
related: [record-the-four-surface-optimizer-invariant]
scopes: [contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, contracts, budgets]
---
## User-visible outcome

The per-region implementation frontier has one accepted bounded-work contract: caller-installed providers emit raw proposals and declines through a host-bounded channel, provider count is preflighted separately, and exceeding either limit refuses atomically rather than compiling a partial frontier.

## Why this is its own ticket

**Fact, corrected at exact base `f3364b126d18544d694860fc2cb4de9bbef0e75c`.** `ImplementationFrontier` (`crates/tiler-compiler/src/frontier.rs`) retains every admitted implementation. `ImplementationFrontier::non_dominated` allocates a borrowed Pareto *view* over that complete population; it does not own or reduce the retained population. Neither storage nor the view has a count bound, and `DeterministicBudgets` (`crates/tiler-compiler/src/request.rs`) has no field for either population — the source-safe anchors are `pub(crate) struct ImplementationFrontier`, `pub(crate) fn non_dominated`, and `pub(crate) struct DeterministicBudgets`.

**Fact.** Plan selection multiplies over the full `frontier.admitted()` slice, not `non_dominated()`. `bind_region_frontiers` in `crates/tiler-compiler/src/selection.rs` binds `admitted: entry.frontier.admitted()` into cover plan combinations; `ImplementationFrontier::non_dominated` is a pure Pareto view exercised in frontier unit tests and contract prose, not the compile-path bind filter. Complete-plan combination growth is already capped by `DeterministicBudgets::physical_plan_combinations` (governed default 4096). A retention-budget decision must name which population it bounds: admitted proposals, the Pareto view, plan combinations, or more than one.

**Fact.** `docs/compiler/optimizer.md` carried "8 nondominated implementations per region" as a forward-looking budget whose stated activation condition was the physical-implementation-frontier stage landing. That stage landed (`enumerate_frontier` is called from `crates/tiler-compiler/src/pipeline/planning.rs`), and the budget did not follow. `record-the-four-surface-optimizer-invariant` replaced the stale sentence with the current state and this pointer rather than inventing either a field name or a decision.

**Fact — the activation trigger fired on 2026-08-08.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), source anchor `item 2 landed`, records `InstalledPhysicalProviders` and `CompileRequest::with_physical_providers` on the ordinary compile path. The optimizer contract independently records the same boundary under `The condition that made that bound self-limiting expired on 2026-08-08`. A caller can now add providers, so the retained population is no longer bounded by this build's single governed provider. This decision closes that fired trigger; the linked delivery tickets own the canonical budget and contract changes.

## Decision — bound raw provider output and refuse atomically

**Accepted by Tom on 2026-08-11 in the Codex coordination thread.** Tom accepted the coordinator's ranked recommendation and delegated choices within the stated repository constraints. The exact decision is:

- Replace the public complete-`Vec` offer protocol with a host-owned bounded emission sink or equivalent pull protocol. Proposals and typed declines charge the same raw provider-output population before Tiler accepts or verifies them.
- Preflight an explicit provider-count limit. A provider that emits nothing still consumes one invocation, so an outcome limit alone is not a host-work bound.
- Exceeding either limit atomically returns typed `BudgetExhausted` at the first excess, with lower-bound demand `limit + 1`. No prefix becomes a frontier; Tiler does not retain the governed implementation, first entries, cheapest entries, structural-Pareto entries, or a claimed baseline.
- Keep `physical_plan_combinations` as the separate downstream Cartesian-product work bound. The frontier bound owns provider invocations and raw proposal/decline outcomes before proposal assessment; the plan-combination bound owns complete-plan assembly attempts after verified frontiers exist.
- Treat installed providers as trusted native code. The sink bounds Tiler-owned accepted outcome storage and subsequent verification; it cannot preempt arbitrary computation or allocation performed inside a provider before an emission. A stronger untrusted-provider guarantee would require isolation or a separately bounded construction vocabulary.
- Put the exact limits in the complete deterministic budget policy and canonical request/evidence subject. This changes the request-subject domain and explain qualifier. It does **not** directly change artifact or cache identity; those move only if selected packaged content changes, and atomic exhaustion produces no artifact.

**Fact — dominance is not a cardinality bound.** `PhysicalCostEstimate::dominates` requires at least one strict improvement. Distinct provider identities offering equal-boundary, equal-cost implementations therefore produce an arbitrarily large incomparable set; the existing additive-provider tests already exercise the two-provider case. The former dominance-only branch is refuted rather than retained as a co-equal option.

**Correctness boundary.** Selection deliberately ranges over every retained valid plan, including structurally dominated plans that a measured target cost row may prefer. A local first-N, cheapest-N, governed-only, or Pareto-only truncation can remove the only globally compatible plan or the measured winner. No safe retention order or baseline exists today, so the narrow first pass has one exhaustion disposition: refuse.

**Implementation graph.** [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](calibrate-the-physical-frontier-provider-and-outcome-budgets.md) owns the first values and workload census. [`replace-provider-offer-with-a-host-bounded-frontier-sink`](replace-provider-offer-with-a-host-bounded-frontier-sink.md) owns the source-breaking public seam, request identity, typed stop, and compile-path tests. A later caller-selectable exhaustion disposition remains owned by [`design-explicit-caller-selected-budget-exhaustion-policies`](design-explicit-caller-selected-budget-exhaustion-policies.md); this decision does not silently add one.

## Outcome

The architecture decision is closed: raw provider output is bounded before Tiler accepts it, provider count is bounded separately, and exhaustion refuses atomically. The linked calibration and implementation tickets own delivery. No worker may infer the retired value eight, a Pareto-prefix policy, or a governed fallback from this record.

## Measured input from the region-general provider

**Measurement, 2026-08-04, governed five-operation program (`crates/tiler-compiler/src/hot_path.rs`'s `program(4, 3)`), governed strict contract, prototype target-neutral baseline profile.** The [minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md)'s `Q-MPR-02` predicted that a provider offering for every region rather than for three "multiplies the frontier population by the cover's region count". It did not, and the reason is the shape of the generalization rather than luck: the fourteen newly-answered region subjects are answered with *declines*, which enter `ImplementationFrontier::rejections` and never `admitted`, so `non_dominated` still ranks the same three admitted implementations it ranked before. The frontier enumeration count is unchanged at seventeen (`one_compile_enumerates_each_distinct_region_subject_once`), the admitted population per subject is unchanged (one each for the pointwise, reduction, and whole-program subjects; zero for the other fourteen), and what grew is the rejection list and the explain trace — 34 frontier records, 16 declines, and 14 `selection.region-coverage.v1` records whose blocked-cover counts sum to 38 (cover, region) pairs, where earlier censuses had 8, 2, and 0. **Correction — 2026-08-10.** The third axis is explain *records* (14, pinned by `every_wired_authority_emits_its_typed_explain_records`), not the historical per-pair rejection count (38); the pair total remains true as the sum of `blocked-covers` facts. **Measurement boundary:** one program, one contract, one target profile, one installed provider. It says nothing about the ADR 0090 item 2 registry case this ticket names as its activation trigger, which is where several callers' providers each propose for the same region and the *admitted* set is what could grow.

## Graph maintenance

- Not a blocker for the four-surface invariant, which cites the current state rather than depending on the answer.
- ADR 0090 item 2's provider registry landed on 2026-08-08. The trigger fired and Tom chose the bounded raw-output/atomic-refusal contract on 2026-08-11; no implementation worker should infer the answer from the old forward-looking value of eight.
