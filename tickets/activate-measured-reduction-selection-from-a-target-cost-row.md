---
id: activate-measured-reduction-selection-from-a-target-cost-row
title: Activate measured reduction selection from a target cost row
status: todo
priority: p1
dependencies: [calibrate-and-activate-parallel-reduction-selection]
related: [calibrate-device-cost-models, implement-parallel-reduction-strategies]
scopes: [implementation/compiler, implementation/build, contracts/optimizer, research/target-profiles]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, cost-model, target-profiles, selection]
---
## User-visible outcome

Physical selection prefers a parallel reduction where the qualified profile's own measured evidence says it is faster, and the serial fold where it says the opposite, with explain output naming the term that decided.

## Why this is parked rather than todo

**Tom decides two things this ticket cannot execute without.** It adds a `pub` `TargetProfileBuilder` declaration for a quantity no target currently carries, and moving that row moves the canonical descriptor, which moves every pinned artifact identity and cache subject derived from it. Both are reserved: consequential public boundaries and identity-domain steps are his, and this one is a *new kind* of row rather than another instance of an existing kind.

It is `awaiting-decision` rather than `deferred` because nothing is missing. The measurement exists, the model exists, the held-out score exists, and the design below is complete enough to land in one commit.

## The evidence this rests on

**Measurement, 2026-08-07** — [`spikes/program-planning/reduction-dispatch-crossover`](../spikes/program-planning/reduction-dispatch-crossover/README.md), retained at `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`, on a host matching the [authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s offline and execution rows in every field.

- The serial fold costs up to **50.7x** the best parallel plan (4 rows of 8,192 contributors) and as little as **0.56x** it (16,384 rows of 4). Both ends are far outside the noise.
- The two parallel strategies are inside each other's noise almost everywhere, so **the decision is binary**: parallelize or not.
- A three-parameter work-span model — `sum over stages of ( encoder + max(work / parallel_threads, depth) * step )` — fitted on perfect-square contributor counts agrees with the measured verdict on **24 of the 26 held-out cells whose verdict is separated**, worst measured penalty **1.81x**.
- **Only `parallel_threads` moves a decision.** Scaling `encoder` by twenty or `step` by a tenth leaves every predicted winner unchanged; scaling `parallel_threads` by a quarter drops held-out agreement to 20 of 26 and the worst penalty to 3.04x.

So the row this ticket needs is **one number**: the fold steps the device retires at once when saturated. Fitted at 1.056e3 on that host, and determined to roughly a factor of four — quadrupling it left fit-set agreement unchanged and *improved* the held-out worst penalty to 1.20x.

## The design problem to settle first, because it is not the row's value

**A cost row is not a capability key, and the profile vocabulary is built for capability keys.** [`docs/research/program-planning/flash-class-capability-set.md`](../docs/research/program-planning/flash-class-capability-set.md) already eliminated putting a bandwidth or clock number on a target profile, and the argument applies unchanged here: every `CapabilityAxis` variant is a *hard bound*, silence about one is `Unknown`, and `Unknown` never reaches an executable frontier. A cost row declared the same way would make silence render a profile **unexecutable for a quantity no feasibility predicate reads**, which is the wrong failure direction. Silence about a cost term must mean "no preference", not "no plan".

Second, `crates/tiler-compiler/src/component_cost.rs` records why a second cost-model key cannot simply join the first: `PlanStructuralCost::dominates` returns `false` across differing model keys, so plans carrying different keys never dominate each other, the non-dominated set silently becomes the whole set, and Pareto pruning goes dark with nothing reporting it.

Third, selection is a Pareto relation over exact structural counts with a canonical-identity tie break, deliberately **not** a scalar latency total order (`crates/tiler-compiler/src/pipeline/planning.rs`, `select_non_dominated`). A measured cost term that decides between mutually non-dominated plans is a change to what selection *is*, not a new dimension in what it already does.

## Implementation keys

Settle the three above and then, in one commit:

- declare the term on `TargetProfile` through a `declare_*` / `declare_measured_*` pair whose measured constructor carries the same `TargetCompileProfileMeasurementSource` the grid-axis row uses, so its validity stays `MeasuredEnvironment` and cannot widen to a portable claim;
- have `BoundMetalCompileDeclaration::first_macos_apple9` declare it from the retained 2026-08-07 measurement, citing that spike;
- thread it to the point where reduction alternatives are compared, and make the comparison explain itself — the winning alternative's explain row must name the term and both sides of the `max`, not merely report `selected`;
- recompute the canonical descriptor length, the standard Metal artifact identity, and its cache subject on the merged tree, enumerating each moved pin;
- keep hard feasibility untouched: no infeasible plan may become an expensive one, and a profile that declares no cost row must select exactly as it does today.

Do not widen `PlanStructuralCost` with a latency dimension as a shortcut. Its four dimensions are exact counts a plan carries; a fitted quantity beside them would make a Pareto relation over measured and counted quantities at once, and a profile without the row would then dominate differently from one with it.

## Required evidence

The selection change is mutation-proved on the term: perturbing the declared value changes the selected alternative on a named shape, or refuses. A profile declaring no term selects bit-identically to today, proved by an unchanged golden. The explain report names the deciding term. Every moved identity pin is enumerated with its before and after. The measured shapes the new selection prefers are the ones the retained sweep measured faster, checked against the retained TSV rather than re-argued.

## Closes when

Selection consults the declared term, the qualified profile declares it from retained measurement, explain names why the winner won, no infeasible plan is represented as expensive, identity moves completely in one commit, and Tom has accepted the exact public surface.

## Graph maintenance

- Keep after `calibrate-and-activate-parallel-reduction-selection`, which supplies the measurement and the model this ticket activates.
- `calibrate-device-cost-models` remains the owner of general analytical-cost calibration. This ticket is one term for one decision and must not be widened into that.
- If the design problem above resolves against a profile row, file the alternative carrier and close this rather than reshaping it silently.

## Accepted — 2026-08-07

**Tom approved the direction on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, on the basis presented: a measured cost row is admitted, **declared as a distinct kind from a capability axis**, with silence about it meaning *no preference* rather than *no plan*.

**The acceptance came with a standing instruction, and it governs how this ticket is executed:** do not cut scope or decisions for short-term gain. Performance, correctness, long-term maintainability, code quality, and compatibility are all to be weighed — a cheaper shape that defers one of them is not a saving. Nothing in the list below may be dropped, narrowed, or split off without saying so explicitly and giving the reason.

### What the acceptance settles

- The **direction**: selection may consult a measured term where the qualified profile declares one.
- The **carrier kind**: a cost row is *not* a `CapabilityAxis`. Declaring it as one would make silence render a profile unexecutable for a quantity no feasibility predicate reads, which is the wrong failure direction. The flash-class capability record already eliminated that shape for a bandwidth number and the argument transfers unchanged.
- The **silence rule**, which is testable rather than aspirational: **a profile declaring no cost row must select bit-identically to today, proved by an unchanged golden.**

### What the acceptance does not settle, and must not be quietly assumed

The exact public spelling of the `declare_*` / `declare_measured_*` pair remains a public boundary under ADR 0075 and comes back to Tom with the built surface. Acceptance of the model is not acceptance of its spelling.

### Obligations carried forward in full

Every item below was already in this ticket and is restated because the acceptance instruction forbids trimming them:

1. Declare the term through a `declare_*` / `declare_measured_*` pair whose measured constructor carries the same `TargetCompileProfileMeasurementSource` the grid-axis row uses, so its validity stays `MeasuredEnvironment` and cannot widen into a portable claim.
2. `BoundMetalCompileDeclaration::first_macos_apple9` declares it from the retained 2026-08-07 measurement, citing that spike.
3. Thread it to the point where reduction alternatives are compared, and **make the comparison explain itself**: the winning alternative's explain row names the term and both sides of the `max`, not merely `selected`.
4. Recompute the canonical descriptor length, the standard Metal artifact identity, and its cache subject **on the merged tree**, enumerating each moved pin. Two branches moved these same three pins on 2026-08-07 from different bases and neither's values survived; the current values are artifact identity `23c46a19…`, cache subject `e89c4d82…`, fixed content 64,542 bytes.
5. Keep hard feasibility untouched: no infeasible plan may become merely an expensive one.
6. **Do not widen `PlanStructuralCost` with a latency dimension as a shortcut.** Its four dimensions are exact counts a plan carries; a fitted quantity beside them would make a Pareto relation over measured and counted quantities at once, and a profile without the row would then dominate differently from one with it.
7. Two constraints that are correctness rather than taste, and that the implementation must answer rather than route around: `PlanStructuralCost::dominates` returns `false` across differing model keys, so plans carrying different keys never dominate each other, the non-dominated set silently becomes the whole set, and **Pareto pruning goes dark with nothing reporting it**. And selection today is a Pareto relation over exact structural counts with a canonical-identity tie break, deliberately *not* a scalar latency total order — a measured term that decides between mutually non-dominated plans is a change to what selection **is**, not a new dimension in what it already does.

### Evidence required, unchanged

The selection change is mutation-proved on the term: perturbing the declared value changes the selected alternative on a named shape, or refuses. A profile declaring no term selects bit-identically to today. The explain report names the deciding term. Every moved identity pin is enumerated with before and after. The shapes the new selection prefers are checked against the retained TSV rather than re-argued.

### If the design resolves against a profile row

This ticket's own instruction stands: file the alternative carrier and close this rather than reshaping it silently.
