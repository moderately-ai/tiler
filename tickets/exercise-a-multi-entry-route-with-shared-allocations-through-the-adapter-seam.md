---
id: exercise-a-multi-entry-route-with-shared-allocations-through-the-adapter-seam
title: Exercise a multi-entry route with shared allocations through the adapter seam
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The adapter seam's multi-entry and shared-allocation paths are exercised by a fixture that reaches them, instead of only by the empty case a single-entry fixture produces.

## Why

**Fact — filed from `route-a-custom-backend-through-an-independently-selected-adapter`'s own boundary statement (2026-07-31).** That landing's fixture planner implements multi-entry routes and shared allocations, but its fixture is single-entry, so both paths are exercised only as the empty case. The Metal proof's materialized route (two dispatches, one shared allocation) is the shape a second fixture stage needs; the serial-sum artifacts already carry it.

## Work

Extend the out-of-crate fixture (`crates/tiler-runtime/tests/adapter_route/`) with a two-stage member — two entries, one shared scratch allocation — and assert the stage log, the shared-allocation lifetime through final device use, and the post-commit failure classification when the second stage halts. Perturb the shared-allocation size and the inter-stage ordering, each watched failing.

## Closes when

Both paths are reached by a passing fixture case with perturbations, and the empty-case tests are kept beside them rather than replaced.

## Outcome

**Fact.** `crates/tiler-runtime/tests/adapter_route/` gained a materialized member: `FixtureSpec::materialized()` packages the same semantic graph as two stages — a pointwise `x * 2.0 + 1.0` writing an `Internal`/`Temporary` scratch value, and a strict serial reduction reading it — joined by a `push_data_dependency` edge, from which the loader derives exactly one `SharedAllocation`. The fused member is unchanged and every one of its cases still runs.

Four new tests plus one empty-case assertion:

- `a_two_stage_route_shares_one_allocation_and_matches_the_reference` — the stage log spans both entries (`ValidatePayload` and `ObservePreparedEntry` twice, the rest once); the pairing is `(entry 0, slot 1) -> (entry 1, slot 0)`; both ends resolve to one host allocation; that allocation is read *after* the route returns and equals an independent `tiler-reference` evaluation of the pointwise prefix; the result equals the fused member's oracle bit for bit.
- `a_shared_allocation_shorter_than_the_route_publishes_refuses_before_the_commit` — pre-commit `UndersizedStorage`, fallback still permitted, no dispatch.
- `dispatching_the_two_entries_out_of_order_returns_a_wrong_answer_rather_than_a_refusal` — nothing refuses, both entries reach terminal success, the scratch is *correct*, and the result is wrong. The fail-open case `SharedAllocation`'s own docs name, made concrete.
- `a_halt_in_the_second_entry_is_a_post_commit_failure_naming_that_entry` — `Incomplete { entry: 1, executed: 1, expected: 2 }`, fallback foreclosed, with the first entry's completed output in the scratch and one of two rows in the output storage.
- `a_carried_payload_routes_through_a_selected_adapter_and_matches_the_reference` now asserts `shared_placements().is_empty()`, so the empty case is stated rather than implied.

**Fact — two fixture defects the exercise surfaced.** The planner sized shared allocations from `accessible_bytes()` alone while its own non-shared branch used `offset + bytes`; both proven implementations (`prototypes/serial-sum-run`, `prototypes/candle-metal-adapter`) use the reach. Corrected, and the caller-storage check generalized to a uniform "the allocation holds what the route published" comparison covering all three storage sources. `ExecutionFault` variants now name their entry: a two-entry route reporting only "1 of 2 invocations ran" cannot distinguish a route that failed having done nothing from one that failed having done everything earlier, and those are different states of the caller's storage.

**Measurement — every check watched failing** (macOS arm64, pinned nightly, `cargo nextest run -p tiler-runtime`):

| Neutered | Test that failed | Observed |
| --- | --- | --- |
| `UndersizeSharedAllocation` size cut | size + population tests | `UndersizeSharedAllocation must refuse` |
| `order.reverse()` | out-of-order test | `left != right` failed; both `[6.0, 4.0]` |
| `Incomplete { entry }` pinned to `0` | second-entry halt test | fault named entry 0 |
| shared pairing skipped entirely | accepted two-stage test | result `[0, 0]` against oracle `[6.0, 4.0]` |
| second deferred predicate removed | accepted two-stage test | one `ObservePreparedEntry`, not two |

The fourth row is the load-bearing one: a planner ignoring `Preflight::shared_allocations()` returns plausible zeros rather than refusing, which is the one place in this stack that fails open, and the suite now catches it.

**Inference.** No `crates/` source changed — the whole diff is under `crates/tiler-runtime/tests/`, so this adds evidence about the existing seam rather than altering it. No public boundary moved.

**Unsupported / not attempted.** Nonzero binding offsets: every window here starts at byte 0, so the reach correction is proven by construction and by agreement with the two prototypes, not by a fixture that exercises a partial window (`prototypes/serial-sum-run`'s `a_partial_window_route_publishes_and_plans_the_artifact_offset` covers that shape). Routes wider than two entries, more than one shared pair per route, and two entries realized by *different* payloads remain unexercised here.
