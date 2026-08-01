---
id: correct-the-subgroup-threads-route-dimension-meaning
title: Correct what RouteResourceDimension::SubgroupThreads means
status: done
priority: p2
dependencies: []
related: [design-the-subgroup-execution-tier]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, subgroup, defect, public-boundary]
---
## User-visible outcome

The one live-device route dimension the artifact vocabulary carries names a property a device can actually be asked about, compared by a relation that is sound for the routes that state it.

## The defect, in two separable halves

**Fact.** `RouteResourceDimension::SubgroupThreads` (`crates/tiler-artifact/src/program/requirement.rs`) is documented as "Threads one subgroup must execute in lockstep for the route to be correct", and its satisfaction test is `is_satisfied_by(observed) = self.minimum <= observed` — a floor.

**Half one: the stated property is not one current GPU families provide.** **Fact — CUDA Programming Guide.** "In GPUs of compute capability 7.0 and later, *independent thread scheduling* allows full concurrency between threads, regardless of warp", and "*Warp-synchronous* code assumes that threads in the same warp execute in lockstep at every instruction, but the ability for threads to diverge and reconverge at sub-warp granularity makes such assumptions invalid." A floor over "threads that execute in lockstep" therefore bounds a quantity no adapter can soundly observe. That every implemented adapter answers `Unrecognized` is consistent with this, though they answer it for the different reason that Metal publishes no device-scoped width.

**Half two: a floor is the wrong relation for the route that would state it.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md) §3 derives that a width-`W` shuffle tree is sound on a wider device only if lanes `0..W` of each subgroup are all active, and that conjunct is exactly what a floor does not carry. What such a route needs is an equality on the width together with a full-participation obligation. This is the same shape as the argument [CPU vector realization facts](../docs/research/target-profiles/cpu-vector-realization-facts.md) makes for why a lane width "looks quantitative and is not".

**This is a live defect in landed public vocabulary and is independent of whether the subgroup tier is accepted.** No route states the row today, so nothing is currently wrong at run time — but the vocabulary is what a future route would reach for, and it is deliberately not `#[non_exhaustive]` precisely so that changing it is a build error at every adapter.

## What to decide

- Whether the fix is a corrected doc comment plus an equality relation, a renamed dimension, or removing the dimension until a route actually states one — noting that removal is cheapest now and most expensive later, and that the family's own module doc argues the dimension is "the one that survives" a derivation.
- What an adapter is being asked to observe, stated so that an adapter can answer it rather than answering `Unrecognized` — which on Metal means confronting that `threadExecutionWidth` is a prepared-pipeline property and not a device one.
- Whether the full-participation obligation belongs in this vocabulary at all, or stays a schedule-side intrinsic obligation with only the width crossing the artifact boundary.

## Public boundary

`RouteResourceDimension` is `pub` in `tiler-artifact` and is deliberately not `#[non_exhaustive]`. Any change to its variants, its comparison relation, or its wire tag is Tom's, and a wire-tag change is an artifact identity step.

## Non-goals

Implementing a subgroup route. Declaring a Metal subgroup width. Anything in `tiler-ir`'s schedule vocabulary.

## Closes when

The dimension's documented meaning matches a property an adapter can observe, its comparison relation is sound for the routes that would state it or the dimension is removed with the reason recorded, and the decision on the public boundary is Tom's rather than self-accepted.

## Outcome

**The dimension is kept, its documented property is corrected, and its relation is now an equality the dimension itself fixes.** Base `c81e5c2`.

### The elimination, with one survivor

*Removal until a route states one* is eliminated on cost and on the reservation discipline. `SubgroupThreads` is the **only** variant, so removing it empties `RouteResourceDimension` and takes the whole neutral quantitative half with it: `RouteResourceFloor`, `RouteRequirement::ResourceFloor`, wire kind tag `0x01`, `RouteRequirementSubject::Resource`, `LiveDeviceObservation::Quantity`, `TagSubject::RouteResourceDimension`, and the contract's two-kind row table. That is a wire-tag removal — an artifact identity step (v14 → v15, manifest 12.0 → 13.0) — followed by a second identity step to re-add it, on a vocabulary the module's own exhaustive derivation calls "the one dimension that survives". It would also delete a reservation the contract already labels at the correct maturity ("a typed reservation with one implemented backend answer of *cannot decide*, not a tested guarantee"), which is the labelling AGENTS.md asks for rather than a defect to clear.

*A renamed dimension* is eliminated as insufficient rather than wrong. Renaming `SubgroupThreads` addresses neither half: the relation would still be a floor, and this ticket's "closes when" requires the relation to be sound. It survives only as an addition to the corrected-doc shape, and it is not needed there — the name `SubgroupThreads` is accurate once the doc stops claiming lockstep, because the row does name a count of threads in a subgroup.

*A corrected doc plus an equality relation* **survives**, and it is the only survivor.

### The identity case: no step, and here is the check

**No identity step.** The relation is not on the wire. `RouteRequirement::canonical_bytes` and `encode_route_requirements` emit `kind tag 0x01 || dimension tag 0x01 || u64 big-endian required quantity`, and `parse_route_requirements` reads exactly those three; no byte encodes a comparison. `is_satisfied_by` is evaluated host-side at load time from the decoded row, at the single call site `crates/tiler-runtime/src/load/route.rs:454`. Changing the relation therefore changes no encoded byte, adds and removes no variant, and moves no tag — `tiler.artifact-program.v14` and manifest 12.0 are untouched, and no ledger, pin, or enumeration entry moves. Reproduce: `grep -rn "is_satisfied_by" --include="*.rs" .` returns four sites, of which one is this type's definition and one is its only caller.

### What changed

- **`RouteResourceDimension::SubgroupThreads`** — the doc no longer claims lockstep. It states the observable property (threads the device executes one subgroup at), why the relation is an equality (a width-`W` tree's combine steps are its content, so a wider device satisfies a floor while running lane arithmetic nothing verified), that lockstep is a guarantee CUDA's independent thread scheduling withdraws from compute capability 7.0, that full participation is a **schedule-side intrinsic obligation** no target declares and only the width crosses this boundary, and what an adapter is asked for — Vulkan `subgroupSize` and CUDA `warpSize` are device-scoped, Metal's `threadExecutionWidth` is a prepared-pipeline property so a Metal adapter's `Unrecognized` is correct rather than a gap.
- **The relation moved onto the dimension.** `is_satisfied_by` is an exhaustive `match` on the dimension, so a dimension added later must choose its own relation and omitting one is a build error. A producer still cannot choose it, which is what the original floor-only shape bought.
- **In-crate names that stated the removed relation were corrected:** field and accessor `minimum`/`minimum()` → `required`/`required()`, and `RouteRequirementError::VacuousFloor` → `ZeroResourceQuantity` with a Display that no longer says "asserts no capability" (under an equality a zero row is unsatisfiable, not vacuous; it is refused under either relation and the doc says so).
- **`docs/artifact-abi.md`** — the row-kind table, the `RouteResourceDimension` contract paragraph (with a new **Fact** stating the equality, its derivation, and the identity non-move), the qualitative-half sentence that said "equal floors", the refusal-vocabulary sentence naming a "vacuous floor", and the loader-comparison sentence that said "the loader compares floors itself" and "dimension and minimum".

### The third decide-bullet, settled by reading rather than re-derived

Full participation stays **schedule-side** and does not enter this vocabulary. `docs/research/scheduling/subgroup-execution-tier.md` §2 derives source-lane activity as "a requirement on the *program* rather than a guarantee from the machine, and no target declares it", to be carried "as an evidence class the verifier re-derives"; its drafted decision item 3 states it as "an intrinsic program obligation no target declares", discharged by deriving a launch geometry with no partially populated trailing subgroup. Only the width is a fact about the device, so only the width crosses the artifact boundary.

### Authority provenance, stated rather than borrowed

The dispatch brief cited "ADR 0094 (the subgroup tier), ACCEPTED". **No such file exists.** Exact check: `ls docs/decisions/ | grep -E '^009[4-9]'` returns nothing at `c81e5c2`, the highest ADR is `0093`, and the tier record still carries `disposition: "pending"` with its ADR body in a "Drafted ADR body, written to be landed verbatim" section owned by `land-the-subgroup-execution-tier-adr`. The coordinator confirmed the correction: Tom **accepted the tier design's content** at the live review of 2026-08-01, and the ADR-landing execution is queued. This work therefore cites the review and the record's §3 derivation, and `docs/artifact-abi.md` says "Accepted at Tom's review of 2026-08-01 … the ADR landing that record is queued" rather than naming a decision number that does not exist. The rollback is one paragraph and one doc comment if that acceptance is ever revised.

### Public boundary

The elimination has one survivor, so this proceeds under Tom's standing approval for non-major decisions (2026-08-01) as a semantics correction following accepted content. Two public names could **not** be corrected and were filed rather than left silent: `RouteResourceFloor` and `RouteRequirement::ResourceFloor` are additionally named by `crates/tiler-runtime`, `prototypes/serial-sum-run`, `prototypes/candle-metal-adapter`, `spikes/runtime/inline-dispatch`, `docs/research/runtime/backend-scoped-route-requirement-answers.md`, and **accepted ADR 0092's text** — four scopes this ticket does not hold. Filed as [`rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`](rename-the-route-resource-floor-vocabulary-for-its-corrected-relation.md), and the retained name is flagged in the type's own doc comment so a reader is not misled in the interval.

### Every changed check watched failing, both directions

`program::tests::a_route_resource_row_is_satisfied_only_by_an_exactly_equal_observation` names its population from `RouteResourceDimension::ALL` and asserts the count, so a dimension added without a relation lands in it.

| Perturbation | Result |
| --- | --- |
| Relation restored to the floor `required <= observed` | **FAIL** at `tests.rs:3587` — "subgroup-threads must refuse a wider device, which a floor accepted". This is the load-bearing direction: the equality refuses exactly what the floor admitted. |
| Relation weakened to `required < observed` | **FAIL** at `tests.rs:3579` — "subgroup-threads must accept the width the route was verified at". Proves the test does not pass merely by refusing broadly. |

### Measurement boundary

Nothing was executed on a GPU, and no adapter was changed. Every implemented adapter still answers `Unrecognized` for this row, so the equality has no runtime exercise — it is checked at the vocabulary. No route states the row today, which is why the correction moves no behaviour beyond the type's own contract.
