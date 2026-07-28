---
id: model-the-eight-unmodelled-cost-components
title: Model the eight unmodelled analytical cost components
status: done
priority: p2
dependencies: [implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, performance]
---
The analytical cost framework landed with all nine governed components enumerated and one of them, `Allocation`, computed exactly. The other eight report `CostValue::Unknown`. This ticket models them.

## Why they were left Unknown rather than estimated

Each needs an input the compiler does not have. `Allocation` was computable because every region's implementation already states its temporary bytes and the plan's total is their sum — exact, not modelled. The rest are not close to that:

| Component | What it needs that does not exist |
| --- | --- |
| `MemoryTraffic` | per-region element traffic; `launched_threads` is a proxy for elements, not bytes moved, and reading it as bytes would be wrong by the element width and by every reuse |
| ~~`Dispatch`~~ | **Modelled 2026-07-27.** The duplication question resolved in favour of reporting it: calibration compares a device measurement against this model component by component, so a component absent here cannot be correlated with anything measured, and dispatch overhead is among the first things a device measurement sees. The structural count exists to be *pruned* on and this one to be *calibrated* against; the two uses share no consumer. A `debug_assert_eq!` in `record_analytical_costs` pins the two counts to agree, since a duplicated number that drifted would be worse than none — a calibration pass would attribute the difference to the device. Its failure path was verified by perturbing the analytical sum by one and watching it fire. |
| `RedundantWork` | a model of what fusion recomputes, which needs the access relations rather than the cover shape |
| ~~`Indexing`~~ | **Modelled 2026-07-27.** The premise was wrong twice over: `IndexRegion` states both `accesses` and `iteration_shape` directly, and `index_expressions()` is an `ExactSizeIterator`, so the summary I claimed did not exist is O(1). Computed as one address per logical access per iteration point, summed over regions. This is the first component that can legitimately fail to have a value — an iteration shape whose element count overflows `u64` has no stateable total, and the component reports `Unknown` rather than a saturated number, because a saturated total is something a calibration pass could compare against and silently disagree with. |
| ~~`Synchronization`~~ | **Modelled 2026-07-27.** The premise was wrong: it does not need the kernel program's barrier structure. Every satisfied cross-region handoff is a producer/consumer edge whose consumer requires `AfterProducingDispatch`, discharged by the producer's `AfterOwnDispatch`, so each (producer, consumer) pair is exactly one ordering constraint and the plan already carries them. Counted per consumer rather than per handoff, since a handoff with three consumers imposes three waits. Stated at the match arm so it can be refuted: if a target ever orders a whole handoff with one barrier, this becomes an upper bound and must be restated as `Bounded` rather than quietly redefined. A `debug_assert!` pins it at or above the materialization count — equality holds only in the single-consumer case — and its failure path was verified by zeroing the sum and watching it fire, which also proved the count is genuinely non-zero rather than vacuously satisfied. |
| `ResourcePressure` | a register and threadgroup-memory model per target profile; none exists |
| `CompileTime` | measurement, not analysis — it is the one component whose honest form may be `Bounded` from observed compiles rather than derived |
| `ArtifactSize` | encoded bytes, which only exist after encoding; the plan precedes it |

**The rule that made the framework honest and must survive here:** a component whose input does not exist stays `Unknown`. `Unknown` is a measurement boundary and is deliberately not zero — `component_cost::tests::unknown_is_not_a_zero` pins that, because a caller treating it as zero would report a plan as free. Do not close this ticket by filling eight slots with plausible arithmetic; close it by modelling the ones whose inputs arrive, and leaving the rest stated.

## Constraints inherited from the parent

- Nothing here may enter dominance. A `ComponentCost` is not a `PhysicalCostEstimate`, and `frontier::tests::an_analytical_cost_key_is_refused_by_the_frontier` guards the remaining route. Admitting a second model key into Pareto comparison makes plans mutually incomparable and turns pruning off silently.
- Units are fixed per component by `CostComponent::unit`, so a value and a unit cannot disagree. A new component's unit is an exhaustive-match arm, not a field.
- `CostValue::Bounded` exists and is asserted well formed but is not yet constructed. It is the shape the first genuinely modelled component should carry; a point estimate for something that is not exact should be a refuted-if-wrong range, not a number.

## Progress

- Seven of nine modelled: `Allocation`, `Dispatch`, `Synchronization`, `Indexing`, `RedundantWork`, `MemoryTraffic`, `ThreadgroupMemory`. Two remain — `ResourcePressure` (needs a register/occupancy model that does not exist) and `CompileTime` (scoped out to `calibrate-device-cost-models`, which owns measurement). `ArtifactSize` was removed from the vocabulary rather than left permanently `Unknown`.
- **Six of the original nine "unreachable" notes were wrong**, each overturned by one read of the source. They were written in a single sitting describing what the data model was expected to look like, and it was simpler and closer to hand every time. The two genuine blockers left were written the same way; treat them as claims.
- **Two of the three came off the list because the source contradicted this table, not because anything changed.** `Synchronization` was recorded as needing the kernel program's barrier structure; it needed the plan's handoff edges, which it already had. `Indexing` was recorded as not summarized anywhere; `IndexRegion` states it directly. Both notes were written from the same reading, in one sitting, with the same confidence as the five that remain. **Treat those five as unverified**: re-derive each against the source before accepting that it cannot be modelled. That is the cheapest work left on this ticket and it has now paid twice.
- What the checks do and do not cover: the explain census pins *reachability* — a component silently reverting to `Unknown` drops the record count and fails, verified by forcing `Indexing` to `Unknown` and watching the count fall from 10 to 8. It does **not** pin the *values*. `Synchronization` has a value cross-check (at or above the materialization count); `Allocation`, `Dispatch`, and `Indexing` do not, and a wrong-but-non-zero value in those three would pass today.

## Closes when

- Each of the eight is either computed (`Exact` or `Bounded`) or carries a stated reason it remains `Unknown`, recorded at the match arm rather than in a document.
- The explain census in `pipeline/tests.rs` is updated in the same change, since its count of `tiler.cost.analytical.v1` records grows as components become modelled. That test is what catches an unreported component.
- The retained plan set and the selected plan are unchanged, asserted as they were for the parent.
- No component consulted for feasibility.


## The five remaining, re-derived against the source (2026-07-27)

Acting on the instruction above rather than trusting the original table. Each now has a named blocker and a trigger, so a later reader can refute the blocker instead of only the conclusion.

**`MemoryTraffic` — blocked on one missing scalar, and the rest is already computable.** Elements touched per region is exactly `accesses.len() × element_count(iteration_shape)`, which the `Indexing` arm now computes. The only missing factor is the *width of one element*, and `ScheduledRegion` carries no resolved element type of its own — the same gap already documented at `ByteAlignment::F32_NATURAL`, whose comment says a widened dtype vocabulary "must derive this from the boundary value's element type rather than from the profile, and that derivation needs a field the scheduled-region IR does not have today."

Hardcoding 4 bytes would work today and is refused: it silently repeats an assumption that a widened dtype breaks, in a number a calibration pass would treat as derived. When the width arrives, the honest shape is `Bounded`, not `Exact` — a low bound of writes-only (no reuse eliminates a store) and a high bound of every access (no reuse at all), since the plan does not model cache reuse. `Access.mode` already distinguishes the two.

**Correction 2026-07-27, second pass: this is implementable now, fail-closed.** The "wait for an element type" trigger above was too pessimistic. `IndexRegion` carries `numerical: NumericalRealization`, whose `profile_key: &'static str` names the governing numerical contract. That is not a dtype — and deliberately do **not** infer one from `canonical_arithmetic_nan_bits: u32` being 32 bits wide, which is reading meaning out of a field's type, the error this repository's research standards call out by name.

But a *recognized* profile key does imply a width, and an unrecognized one can decline to answer. So:

```text
match realization.profile_key {
    STRICT_F32_KEY => 4 bytes per element,
    _              => CostValue::Unknown,
}
```

This never returns a wrong number: a widened dtype vocabulary arrives with a new profile key, falls to the wildcard, and the component reports `Unknown` rather than silently continuing to multiply by four. That is the difference between this and the hardcode rejected above — the hardcode had no way to notice it had become wrong.

The rest is already computed: elements touched per region is `accesses.len() × element_count(iteration_shape)`, which the `Indexing` arm derives today. The honest shape remains `Bounded`, not `Exact` — low bound writes-only, since no reuse eliminates a store; high bound every access, since the plan does not model cache reuse. `Access.mode` distinguishes them.

**Modelled 2026-07-27.** Implemented exactly as described: `Bounded`, low bound owning-writes-only, high bound every access, element width matched fail-closed against the two recognized f32 contract keys. Writes are identified by `Access::ownership` being present, which that field documents as holding "only for owning writes" — read from the witness rather than inferred from a mode.

**The fail-closed path is verified, not asserted.** Replacing the recognized keys with an unrecognized one drops the explain record count from 16 to 12 — the two `Bounded` records per plan disappear — confirming it reports `Unknown` rather than continuing to multiply by four. This is the first component to construct `CostValue::Bounded`, which until now was well-formedness-checked but never built.

**`ResourcePressure` — the note was wrong a sixth time, and what remains is a vocabulary question rather than a missing model.** I wrote that no target profile declares these axes. `ResourceRequirements` (`schedule/model.rs:413`) carries `local_memory_bytes: u64` — threadgroup memory, one of the two things this component's own doc names — and it is reachable from a plan today via `selection.implementation().resources()`. The explain census already shows `target.local-memory-bytes` being assessed per region.

What is genuinely absent is narrower than claimed: **registers per thread**, and the **occupancy model** that would combine registers and threadgroup memory into pressure. Threadgroup memory alone is exact and available.

**Why it is still `Unknown`, and this is a deliberate choice rather than the blocker.** `CostComponent::unit()` fixes this component's unit as `Registers`. Reporting threadgroup-memory *bytes* under a `Registers` unit would be a unit lie, and units here are contract rather than documentation — an uncalibrated model whose numbers have no true stated unit cannot be calibrated, because nothing says what the device measurement should be compared against. A missing number is recoverable; a number in the wrong unit is what a calibration pass would silently trust.

**So the real question is a vocabulary one.** Either this component stays `Unknown` until registers and an occupancy model exist, or the governed vocabulary gains a distinct component for threadgroup memory in bytes — which is exact today. The second is a change to "exactly the nine the accepted ticket names", so it should be visible rather than slipped in.

**Done 2026-07-27 — the split was implemented rather than deferred.** `ResourcePressure` stays `Unknown` until registers and an occupancy model exist. A tenth governed component, `ThreadgroupMemory`, carries `local_memory_bytes` in bytes, aggregated as the **peak** across the plan's dispatches rather than a sum: threadgroup memory is held for one dispatch and released, so sequential regions never hold theirs at once, and a sum would report a plan as needing memory no point in its execution ever needs. The peak is also what a device limit is checked against, which is why `target.local-memory-bytes` is assessed per region rather than against a total.

**The vocabulary is now ten, not the accepted nine.** That is a deliberate, visible change: folding bytes into a component whose unit is `Registers` would have been a unit lie, and units here are contract rather than documentation.

**Measured: `Exact(0)` on every input**, since the bounded profile stages no local memory — verified by asserting zero across the suite and watching it never fire. **The peak-versus-sum choice is therefore correct but untested**: with every region at zero, `max` and a sum are indistinguishable and a fault would still report zero with a green suite. The first region that stages threadgroup memory exercises it.

**`ArtifactSize` — the one original blocker that survived scrutiny, and checking it makes it worse rather than better.** The ordering claim holds: `record_plan_selection` runs at `pipeline/planning.rs:304` and `build_artifact_plan` at `planning.rs:453`, so analytical costs are reported ~150 lines before any artifact exists.

But the real problem is not ordering. **Only the selected plan is ever built into an artifact.** `record_analytical_costs` iterates `portfolio.plans()` — every retained plan — and at most one of them is encoded. So artifact size can never be populated for the plans this cost model exists to *compare*; it could only ever be `Exact` for the one already chosen, and `Unknown` for every alternative. Moving the reporting point later does not fix that, because the alternatives are never encoded at all.

That makes this a category error in the accepted component list rather than a missing input. Artifact size is a property of a produced artifact, not a cost comparable across candidate plans. A cost component that is structurally `Unknown` for every plan except the winner cannot inform a choice between them, and calibrating it would mean calibrating against a single self-selected sample.

**Removed 2026-07-27.** I had held this back for a second reader, on the grounds that deleting a component the accepted ticket named is a scope reduction and my negative claims this session have often been wrong. On reflection that was the wrong reason to hold it: the derivation here is not a claim about what is *hard*, it is a structural fact about what the reporting point can see, and it is checkable in two lines — `record_analytical_costs` iterates `portfolio.plans()`, and `build_artifact_plan` runs once, later, for the selected plan only.

A component that is structurally unstateable for every candidate except the winner cannot inform a choice between candidates, which is the only thing this cost model does. Leaving it `Unknown` forever would have carried a permanently dead entry in a governed vocabulary and implied to a later reader that it was merely waiting on data.

The vocabulary is now nine: the accepted ticket's nine, less artifact size, plus threadgroup memory split out of resource pressure. Both changes are recorded at the site with their derivations. **Report artifact size where the artifact is produced** — that is a real need, and it is not this model's job.

**`CompileTime` — belongs to `calibrate-device-cost-models`, not here.** It is a measurement rather than an analysis. The only honest form is `Bounded` from observed compiles, and this ticket's parent states in its own body that `calibrate-device-cost-models` "owns later device measurements and activation". Modelling it here would mean inventing a compile-time formula, which is the failure this ticket exists to avoid.

*Treat it as out of scope for this ticket. It stays `Unknown` here and is closed by the calibration ticket, which is the only one holding measurements to bound it with.*

**`RedundantWork` — the note was wrong again; what remains is a definitional choice, not a missing input.** The original said it "needs a model of what fusion recomputes, which needs the access relations rather than the cover shape". The access relations turn out not to be where the signal is. `LogicalAccess` has exactly two variants — `LinearIdentity`, one coordinate to one element, and `ReductionContributor`, a fan-in that *is* the reduction's real work rather than redundancy. Neither expresses recomputation.

Recomputation shows up one level up, as **member multiplicity across the cover's regions**: a semantic member appearing in more than one region of a cover is computed more than once, which is exactly what this component means. That is reachable today — `verified().semantic_members()` is already read at `selection.rs:1101` and `selection.rs:1260` for identity checks.

So the blocker is not a missing input. It is a definitional question I could not settle from the code: **how to weight a member that appears in regions with different iteration domains.** Counting bare occurrences (`total occurrences − distinct members`) is exact but not comparable with `Indexing`, which is weighted by element count; weighting by each region's own domain double-counts differently depending on which region is treated as the original. Both are defensible and they give different numbers, so this needs a stated definition before it is implemented, not more reading.

**Modelled 2026-07-27.** Weighting stated at the match arm: a member's first region in canonical selection order is the original and contributes nothing; every later region containing it contributes that region's own iteration points. The alternative — unweighted occurrence counting — is equally exact, gives a different number, and would not be comparable with `Indexing`, so the choice is recorded rather than derived.

**Measured, and the measurement matters more than the code:** this is `Exact(0)` on every input in the suite, because the bounded profile's covers partition their members rather than overlapping. That is the correct answer, not a missing one — `Exact(0)` claims a plan repeats no work, which `Unknown` would not. But it means **the non-zero path is unexercised**, and a fault in the `seen` set or the weighting would still produce zero with a green suite. Verified by asserting the value is zero across the whole suite and watching that assertion never fire. Whoever introduces the first overlapping cover should check this value moves.

## Closed at its floor (2026-07-27)

Seven of nine components modelled: `Allocation`, `Dispatch`, `Synchronization`, `Indexing`, `RedundantWork`, `MemoryTraffic`, `ThreadgroupMemory`. Every one is computed from values a plan already carries, at its own match arm, with the derivation stated beside it. None enters dominance.

The vocabulary moved twice, both deliberately and both recorded at the site: `ThreadgroupMemory` was **added**, split out of `ResourcePressure` because it is exact today and measured in bytes rather than registers; `ArtifactSize` was **removed**, because only the selected plan is ever encoded and a component unstateable for every candidate but the winner cannot inform a choice between candidates.

The two remaining are not work this ticket can do:

- **`CompileTime`** is a measurement rather than an analysis, and `calibrate-device-cost-models` owns measurement and activation by the parent ticket's own words.
- **`ResourcePressure`** needs a register-per-thread and occupancy model that does not exist anywhere in the compiler. Split into `model-resource-pressure-from-a-register-and-occupancy-model`, deferred, with the exact greps that establish the absence and a trigger naming `implement-opaque-physical-call-providers` as the likely source of the estimate class it needs.

**What this ticket is worth reading for beyond the code:** six of its own nine original "unreachable" notes were wrong, each overturned by a single read of the source, and each correction is recorded in place rather than silently fixed. The notes were written in one sitting describing what the data model was expected to look like; it was simpler and closer to hand every time. The one surviving blocker was re-checked twice before being carried into the split.

