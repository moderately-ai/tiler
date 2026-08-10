---
id: decide-whether-the-implementation-frontier-owes-a-retention-budget
title: Decide whether the implementation frontier owes a retention budget
status: awaiting-decision
priority: p2
dependencies: []
related: [record-the-four-surface-optimizer-invariant]
scopes: [contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, contracts, budgets]
---
## User-visible outcome

The per-region implementation frontier either carries a declared retention bound inside the canonical request subject, or the optimizer contract records why an unbounded Pareto filter is the correct answer — so a reader of the deterministic-budget list is not left with a ninth budget that is neither implemented nor refused.

## Why this is its own ticket

**Fact.** `ImplementationFrontier::non_dominated` (`crates/tiler-compiler/src/frontier.rs`) retains every admitted implementation no other admitted implementation dominates, and applies no count bound. `DeterministicBudgets` (`crates/tiler-compiler/src/request.rs`) has no field for it — the check is `grep -n "implementations_per_region\|nondominated" crates/tiler-compiler/src/request.rs`, which returns nothing.

**Fact.** Plan selection multiplies over the full `frontier.admitted()` slice, not `non_dominated()`. `bind_region_frontiers` in `crates/tiler-compiler/src/selection.rs` binds `admitted: entry.frontier.admitted()` into cover plan combinations; `ImplementationFrontier::non_dominated` is a pure Pareto view exercised in frontier unit tests and contract prose, not the compile-path bind filter. Complete-plan combination growth is already capped by `DeterministicBudgets::physical_plan_combinations` (governed default 4096). A retention-budget decision must name which population it bounds: admitted proposals, the Pareto view, plan combinations, or more than one.

**Fact.** `docs/compiler/optimizer.md` carried "8 nondominated implementations per region" as a forward-looking budget whose stated activation condition was the physical-implementation-frontier stage landing. That stage landed (`enumerate_frontier` is called from `crates/tiler-compiler/src/pipeline/planning.rs`), and the budget did not follow. `record-the-four-surface-optimizer-invariant` replaced the stale sentence with the current state and this pointer rather than inventing either a field name or a decision.

**Fact — the activation trigger fired on 2026-08-08.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), source anchor `item 2 landed`, records `InstalledPhysicalProviders` and `CompileRequest::with_physical_providers` on the ordinary compile path. The optimizer contract independently records the same boundary under `The condition that made that bound self-limiting expired on 2026-08-08`. A caller can now add providers, so the retained population is no longer bounded by this build's single governed provider. The remaining answer changes either the canonical deterministic-budget subject or the optimizer contract and therefore belongs in `awaiting-decision`, not the executable ready queue.

## The decision

Two answers are defensible and they are not the same claim:

- **Declare a retention budget.** It joins the canonical request subject like every other `DeterministicBudgets` field, which is what makes it artifact-identity-bearing. The decision must also state the retention order, the baseline that cannot be lost, and the typed stop semantics; this ticket does not assume that truncating an incomparable frontier automatically preserves a complete plan. The cost is one more calibration input and one more identity byte sequence.
- **Record that retention is deliberately unbounded.** The Pareto filter's output is bounded by the number of *incomparable* proposals rather than by the number offered, and the accepted contract's "bounded Pareto frontier" language may already be discharged by dominance itself. This answer must state what stops a provider set from producing an unbounded incomparable set, and it must correct the contract sentence at `docs/compiler/optimizer.md`'s boundary-requirements section that calls the frontier bounded.

Deciding requires knowing whether an incomparable set can grow without bound under the boundary-property subsumption relation, which is a question about `AdmittedImplementation::dominates` and the boundary contract rather than about provider count.

## Closes when

Either a retention budget exists as a `DeterministicBudgets` field with a typed budget-stop and its default recorded as a calibration input, or the optimizer contract states that retention is bounded by dominance alone with the argument for why; in both cases the deterministic-budget list and the boundary-requirements section agree, and this ticket's pointer in `docs/compiler/optimizer.md` is replaced by the answer.

## Measured input from the region-general provider

**Measurement, 2026-08-04, governed five-operation program (`crates/tiler-compiler/src/hot_path.rs`'s `program(4, 3)`), governed strict contract, prototype target-neutral baseline profile.** The [minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md)'s `Q-MPR-02` predicted that a provider offering for every region rather than for three "multiplies the frontier population by the cover's region count". It did not, and the reason is the shape of the generalization rather than luck: the fourteen newly-answered region subjects are answered with *declines*, which enter `ImplementationFrontier::rejections` and never `admitted`, so `non_dominated` still ranks the same three admitted implementations it ranked before. The frontier enumeration count is unchanged at seventeen (`one_compile_enumerates_each_distinct_region_subject_once`), the admitted population per subject is unchanged (one each for the pointwise, reduction, and whole-program subjects; zero for the other fourteen), and what grew is the rejection list and the explain trace — 34 frontier records, 16 declines, and 14 `selection.region-coverage.v1` records whose blocked-cover counts sum to 38 (cover, region) pairs, where earlier censuses had 8, 2, and 0. **Correction — 2026-08-10.** The third axis is explain *records* (14, pinned by `every_wired_authority_emits_its_typed_explain_records`), not the historical per-pair rejection count (38); the pair total remains true as the sum of `blocked-covers` facts. **Measurement boundary:** one program, one contract, one target profile, one installed provider. It says nothing about the ADR 0090 item 2 registry case this ticket names as its activation trigger, which is where several callers' providers each propose for the same region and the *admitted* set is what could grow.

## Graph maintenance

- Not a blocker for the four-surface invariant, which cites the current state rather than depending on the answer.
- ADR 0090 item 2's provider registry landed on 2026-08-08. The trigger is fired and this node stays in `awaiting-decision` until Tom chooses the retention contract; no implementation worker should infer the answer from the old forward-looking value of eight.
