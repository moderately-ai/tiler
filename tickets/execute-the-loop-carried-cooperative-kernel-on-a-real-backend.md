---
id: execute-the-loop-carried-cooperative-kernel-on-a-real-backend
title: Execute the loop-carried cooperative kernel on a real backend
status: review
priority: p1
dependencies: [lower-a-loop-carried-cooperative-body]
related: [implement-the-single-workgroup-synchronized-reduction-strategy, share-one-structured-kernel-interpreter, promote-the-bounded-scalar-cpu-vertical-into-a-production-backend]
scopes: [implementation/ir, implementation/reference, implementation/metal, implementation/conformance, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, synchronization, conformance, metal, cpu]
claimed_from: todo
assignee: worker-execute-loop-carried
lease_expires_at: 1786643165
---
## User-visible outcome

The first multi-round cooperative kernel is compiled and executed by an actual eligible backend and agrees bit-for-bit with an independent reference at its declared grouping. The former host-simulator result is retained only as historical bounded evidence, not the implementation guarantee.

## Facts and boundary

- **Verified.** `lower-a-loop-carried-cooperative-body`, anchor `Measurement boundary`, states that its `[2, 6] -> [2]`, three-participant, two-round result was executed only by a host interpreter; nothing was compiled or dispatched on a device. Re-read at this base: the paragraph still says the fixture was "executed by an interpreter on the host. Nothing here was compiled or dispatched on a device, no Metal emission of a multi-round body exists". `crates/tiler-ir/src/kernel/tests.rs` still owns `the_loop_carried_body_matches_the_reference_at_its_declared_order` and the local `cooperative_reference` helper. `crates/tiler-metal/goldens/` still has only the single-round `cooperative_workgroup_reduction.metal`.
- **Verified.** Its structural tests already prove the canonical peeled round, barrier placement, synchronization census, anti-dependency, and verifier refusals. Those remain layer-local and are not duplicated here. The names that still exist: `a_loop_carried_tile_lowers_to_a_peeled_round_body`, `each_loop_carried_synchronization_rule_refuses_its_own_defect`, `the_barrier_convergence_rule_admits_only_the_nesting_a_tile_authorizes`, and the schedule-layer `a_loop_carried_rewrite_with_no_round_boundary_is_refused`.
- **Verified, with a precision note.** The *accepted* bounded scalar CPU production profile refuses barriers and concurrency ([CPU backend](../docs/backends/cpu.md): "Every unsupported value type, operation, address space, vector or packed value, barrier, cooperative scope, thread model, and numerical realization is a typed refusal"). The production crates are still spike-only (`promote-the-bounded-scalar-cpu-vertical-into-a-production-backend` is todo). The spike implements the refusal as "a barrier, which has no participants in a scalar execution model". The first execution therefore used Metal. A later threaded CPU realization must run the same property matrix before claiming support; it is not inferred from the Metal result.

## Fact audit at `b19762f0383d1789ad9c1ad853cd49ce1cfab852`

Re-read at this worker's base before any edit. No Fact was false. The CPU-profile Fact was tightened from "the production scalar CPU profile" to the accepted-but-still-spike-only profile named above, because there is not yet a production `tiler-cpu*` crate. That tightening does not change what the ticket is for.

## Work

- Build the exact verified multi-round schedule/KIR through the real backend emitter and artifact/runtime path available at implementation time.
- Supply launch geometry from the scheduled program, not staging allocation or fixture constants.
- Derive the expected value through an independent reference-owned representation of the declared participant/round/contributor grouping. Do not add another local `cooperative_reference` helper.
- Use both a contributor-set-sensitive input and a grouping-sensitive input; pin the populations each can and cannot distinguish.
- Execute an accepted single-round neighbour and the multi-round subject. Perturb round contribution arithmetic, barrier placement, launch width, and grouping independently and retain each named refusal or wrong-result comparison.
- Report unavailable on an ineligible host. Never count source emission or successful compilation as execution.

## Acceptance

The real backend returns the exact independently derived bits, every property perturbation fails for its own reason, the structural tests remain intact, and the measurement claim names the host/profile/backend and does not generalize to a threaded CPU realization that has not run.

## Outcome — worker, awaiting review

Delivered on this branch. The first multi-round cooperative kernel is compiled and executed by Metal on an eligible host and agrees bit-for-bit with `tiler_reference::cooperative_grouped_sum` at the grouping the scheduled program declared. The accepted single-round neighbour was executed beside it. The host-simulator result remains historical evidence.

**Measurement, 2026-08-13.** Host macOS 27.0 build `26A5406e` on `arm64`; device Apple M4 Max, GPU family Apple9; offline `Apple metal version 32023.921` / `AIR-LLD 32023.921`; SDK `macosx 27.0` build `26A5388f`; profile `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`; AOT `air64-apple-macos26.0` under `metal4.0`; backend Metal; launch `6×3` read from the scheduled program. Not a threaded CPU realization.

Public draft surface in `tiler-reference`: `CooperativeCellLayout`, `CooperativeGrouping`, `cooperative_grouped_sum`, `cooperative_grouped_sum_under`. Labelled draft under ADR 0075.
