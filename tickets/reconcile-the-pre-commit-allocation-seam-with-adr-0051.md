---
id: reconcile-the-pre-commit-allocation-seam-with-adr-0051
title: Reconcile the pre-commit allocation seam with ADR 0051
status: review
priority: p2
dependencies: []
related: [re-audit-adr-implementation-status-after-the-runtime-and-metal-landings, stop-the-identity-join-producer-race]
scopes: [contracts/decisions, implementation/runtime, research/runtime, implementation/frontend, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, decisions, correctness]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
---
## User-visible outcome

The pre-commit allocation seam and ADR 0051 agree: either the landed `RuntimeAdapter` seam stops allocating program storage before the routing commit, or a superseding decision states why allocation belongs there and what a post-allocation refusal is permitted to do.

## Why this exists

**Fact — the decision places allocation after the commit.** [ADR 0051](../docs/decisions/0051-make-runtime-routing-commit-one-way.md)'s Decision reads "Only the resulting committed execution authority may allocate program resources or encode work", and its Consequences read "Program allocations and partial encodings never precede a fallback decision". Its Context gives the reason: "allocation, partial encoding, submission, validation, and publication can have observable resource or execution effects", and retrying a fallback after those stages "can duplicate work, hide device errors, or publish inconsistent results".

**Fact — the adopted research record states the same rule as a boundary between the two stages.** [`docs/research/runtime/runtime-execution-contract.md`](../docs/research/runtime/runtime-execution-contract.md) records that preparation "may allocate backend-internal library or pipeline state" but "must not allocate a program output, program temporary, validation record, private transaction result, or encode program work. This distinction permits real pipeline preflight without weakening the no-work-before-commit rule." Its transition table places "program allocation and enforcement setup" on the `routing committed to resources` row, whose fallback column reads `never`.

**Fact — the landed seam allocates before the commit and permits a fallback after it.** `RuntimeAdapter::plan_dispatch` (`crates/tiler-runtime/src/adapter.rs:371`) takes a `&Preflight` — so it runs before `Preflight::commit` — and its documented contract is that "an adapter allocates storage, honours the paired `Preflight::shared_allocations`, fills host-visible inputs, and compares each binding's required byte range against the storage it holds", with "all of that ... discarded if the route is abandoned". It returns `Self::Refusal`, and `AdapterRouteFailure::fallback_permitted` reports `Plan` as recoverable (`crates/tiler-runtime/src/adapter.rs:533`). `crates/tiler-runtime/src/load.rs:11` states the opposite rule in the same crate: "no program buffer, encoding, submission, or other irreversible program work belongs before the commit".

**Fact — a real implementation does exactly what the trait describes.** `spikes/runtime/inline-dispatch/src/adapter.rs:755` calls `device.new_buffer` for each shared intermediate and for each entry binding including the result buffer, then refuses on a workgroup wider than the pipeline admits. `docs/research/runtime/autoregressive-state-and-kv-cache.md:168` plans the same shape for the decode path: a new allocation per retained output inside `plan_dispatch`, "this is the last chance, and the capacity and range comparisons belong here".

## Work

Read ADR 0051, the runtime execution contract record, and `crates/tiler-runtime/src/adapter.rs` in full before choosing, and state the elimination rather than the conclusion. The candidates are not symmetric and should be tested against what each makes unrepresentable:

1. **Move allocation after the commit.** `plan_dispatch` splits into a pre-commit sizing and capacity check that allocates nothing, and a post-commit allocation step reached only from `RoutedDispatch`, whose failure is a `Failure` rather than a `Refusal`. This preserves the decision as written and makes an allocation failure terminal — which is what ADR 0051 argues for, and what a caller that must not duplicate device work needs.
2. **Supersede ADR 0051's allocation clause.** A new decision states that program allocation is reversible on every adapter in the initial profile, that a discarded allocation has no observable effect a retry could duplicate, and that the refusal after it is therefore safe. This is a claim about every future adapter, not only the two that exist, and it has to say what stops an adapter from acquiring a non-reversible resource in that stage.
3. **Narrow what `plan_dispatch` may do.** Keep the stage, forbid program-storage allocation in it, and let it hold only the comparisons that need no storage. This needs the capacity checks it currently performs against real allocations — `allocation_holds` compares the length a buffer actually came back with — to be restated against something a host can answer without allocating, or dropped with their loss named.

Whichever survives, the boundary is a public seam: `RuntimeAdapter` is a reviewed draft boundary under ADR 0074 §7 and its exact interface is Tom's to accept.

## Boundaries

- Do not resolve this by editing ADR 0051's Decision text in place. A change to an accepted decision is a superseding decision with its own record.
- The status record already landed: ADR 0051's `Implementation boundary` section names this as a divergence, and the runtime research record points here. Correct those two spans in whichever direction this closes.
- Nothing here reopens the one-way commit itself, which is implemented and compile-enforced. The question is only which stage owns allocation.

## Closes when

Either the seam and the decision agree in code, or an accepted superseding decision records why they do not; ADR 0051's `Implementation boundary` no longer names an unresolved divergence; and the runtime execution contract record's boundary paragraph is updated to match.

## Direction decided — candidate 1, allocation moves after the commit (2026-08-01)

**Tom chose candidate 1 at the live session on 2026-08-01, relayed here by the coordinator who witnessed it.** The elimination he was shown and endorsed: candidate 2 rests a compile-enforced invariant on a prose guarantee over every future adapter that nothing type-enforces; candidate 3 discards `allocation_holds`' observed-length evidence, and a predicted capacity is not an observed one. What survives: `plan_dispatch` splits into a pre-commit sizing-and-capacity check that allocates nothing, and a post-commit allocation step reached only from `RoutedDispatch`, whose failure is a `Failure` rather than a `Refusal`. The priced cost, accepted explicitly: a device that cannot hold the plan becomes terminal at that stage rather than recoverable, with pre-commit sizing against declared limits catching all but the allocator-rounding residue — and that residue failing loudly is a defect signal, not a routing input.

**What remains Tom's despite the direction being decided:** `RuntimeAdapter` is a public boundary under ADR 0074 §7, so the exact split interface comes back as a concrete draft before acceptance. The Boundaries section above is unchanged — the resolution is a superseding-clause-free execution of the decision as written, so no new ADR is required; ADR 0051's `Implementation boundary` divergence entry is corrected to a directed-work entry when this lands.

## Executed 2026-08-01 — the split seam, and the draft that goes to Tom

**The interface, exactly as landed, is a draft and not an acceptance.** `RuntimeAdapter` gains one method and loses none; `plan_dispatch` keeps its name and signature and its contract narrows to sizing.

```rust
/// Sizes what the route will dispatch and checks its capacity, acquiring nothing.
fn plan_dispatch(
    &mut self,
    context: &LiveExecutionContext,
    preflight: &Preflight<'_>,
) -> Result<(), Self::Refusal>;

/// Acquires and binds the program storage the **committed** route dispatches.
fn allocate_dispatch(
    &mut self,
    context: &LiveExecutionContext,
    routed: &RoutedDispatch<'_>,
) -> Result<(), Self::Failure>;
```

`AdapterRouteFailure` gains `Allocation(F)` between `Plan(R)` and `Dispatch(F)`; `fallback_permitted` answers `false` for `Allocation` and `Dispatch` and `true` for the rest, which makes the split exactly "carries `R`" versus "carries `F`". `route_with_adapter` calls `allocate_dispatch` on the line after `preflight.commit()`. No type in `crate::load` changed: `RoutedDispatch` already published `entries()` and `shared_allocations()`, which is everything an allocating stage needs.

**Two naming decisions worth refusing rather than accepting silently.** `plan_dispatch` was *kept* rather than renamed, because "plan" never implied acquisition and the stage still plans — and because the compiler forces every implementor to add the new method anyway, so no adapter can carry the old contract forward unnoticed. No default body was given to `allocate_dispatch`, deliberately: a defaulted `Ok(())` would let an adapter ported from the old shape leave its allocation in the pre-commit stage silently, which is the exact defect this ticket closes. The cost is that every implementor is a build error until it decides where its allocation goes, which is the intended pressure.

**What the pre-commit stage can still refuse**, enumerated so the priced cost is checkable: a binding whose offset and extent do not form an addressable range; a range larger than one allocation the bound context admits; a launch wider than the prepared pipeline admits; an empty launch the artifact does not permit skipping; a binding naming a program input the caller did not supply, or a target the consumer does not place; a route no slot of which binds the program output; and caller-supplied storage shorter than the route's published range — that last one because the caller's storage exists independently of the route, so comparing against it acquires nothing. What moved past the commit is exactly the allocation and the observed-length assertion over it.

**Three implementors were split the same way and their observed-length assertions moved with the allocations they inspect:** `crates/tiler-runtime/tests/adapter_route`, `spikes/runtime/inline-dispatch`, and `prototypes/candle-metal-adapter`. In the last two, `UndersizedStorage` (and, for Candle, `Allocation`) moved from `RouteRefusal` to `DispatchFailure`.

**Scopes added 2026-08-01, with the reason.** `implementation/frontend` for three `RuntimeAdapter` test doubles in `crates/tiler` that a required trait method necessarily breaks, and `implementation/candle` for `prototypes/candle-metal-adapter`, a workspace member the gate builds. Both edits are consequences of the trait change rather than separate work. `implementation/frontend` was held by the live `state-the-numerical-contract-in-the-region-grammar` at the time; file-level disjointness was verified against that worker's actual branch diff — `git diff --name-only 29a9680..tkt/state-the-numerical-contract-in-the-region-grammar -- crates/tiler/` reported nothing — and the three edits are additive method bodies inside `impl RuntimeAdapter` blocks, which a grammar ticket has no reason to touch. That check is a snapshot; integration should re-run it.

**The sibling defect was fixed in passing** because it fired here and lives in this ticket's own scope: see [`stop-the-identity-join-producer-race`](stop-the-identity-join-producer-race.md), which carries the mechanism, the eliminated candidates, the measurement, and the one closing criterion that stays unmet.
