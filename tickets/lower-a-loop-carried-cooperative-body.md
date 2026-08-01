---
id: lower-a-loop-carried-cooperative-body
title: Lower a loop-carried cooperative body
status: todo
priority: p1
dependencies: []
related: [admit-loop-carried-cooperative-staging, realize-the-strict-contraction-on-metal, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, contraction]
---
## User-visible outcome

A cooperative tile with more than one round reaches a verified structured kernel. Until it does, the loop-carried staging vocabulary [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md) landed is representable and unlowerable: the schedule verifier admits the rewrite, derives its anti-dependencies, and requires a point for each, and `cooperative_plan` then refuses the tile by name (`crates/tiler-ir/src/kernel/lower.rs`, `KernelDiagnostic::CooperativeLoweringShape`).

## The blocker, with the exact checks

**Fact — a predicated region produces no values.** `OperationKind::Predicated { predicate, body }` (`crates/tiler-ir/src/kernel/model.rs`) carries no results, and the cooperative lowering's own comment records the consequence: "a value defined inside a guarded block cannot cross into the next one" (`lower.rs`, `emit_cooperative`).

**Fact — every boundary load must be dominated by the governed iteration guard.** `verify_effects` refuses any unguarded effect as `KernelDiagnostic::PredicateDominance` (`crates/tiler-ir/src/kernel/verify.rs`). Staged accesses are deliberately *not* effects, which is the seam this work should use.

**Fact — exactly one store, at loop depth zero.** `verify_effects` again: `stores != 1` and `effect.loop_depth != 0` are both `OutputCoverage`.

**Inference — the three compose into "no accumulator can cross a round".** A round's contribution is computed from boundary loads, so it is guarded; the accumulator must survive the round loop's back edge, so it must leave the guard; and it cannot. The one shape that escapes it puts the guarded work and the unguarded staged fold in separate regions of the same loop body — the accumulator is then updated by staged loads alone, which need no guard — and that shape is worth confirming before any KIR change is proposed.

**Inference — the seed then peels a round, and the peel realizes each point twice.** Every fold in this repository seeds at its first contributor (`start == 1`) rather than at the reduction identity, because `+0.0 + x` is not `x` for `x = -0.0`. A round loop seeded the same way runs `1..rounds` with round zero emitted ahead of it, so each declared point is realized once in the peel and once in the body — which `verify_synchronization`'s "every declared point is realized exactly once" refuses, and which `verify_edge_is_ordered` (written over a single fence position) cannot check.

## What this owns

- Whether the accumulator can be kept out of every predicate, or a value-producing predicated region (or an explicit select) is required. The second is a KIR vocabulary change and a public boundary.
- The realization rule for a peeled round: how many times a point may be realized, and what makes each realization convergent and correctly ordered. `barrier_is_convergent` (`verify.rs`) already proves the convergence half and states that matching the enclosing loop's trip count against the declared round count is this ticket's obligation.
- `verify_edge_is_ordered` and its anti-dependency counterpart, generalized from one fence position to a cyclic round body: a barrier at position `b` separates round `r`'s read at `c` from round `r + 1`'s write at `w` exactly when `b > c` or `b < w`, which is the body-level mirror of `SynchronizationPoint::discharges_anti`.
- The first multi-round strategy to emit, and its contributor arithmetic. `ReductionTopology::CooperativeWorkgroup` already defines it: participant `p` of round `r` owns the contiguous range at index `r * partitions + p`, and `verify_cooperative_loops` (`verify.rs`) needs the round dimension its single-round form does not have.

## What this does not own

The `tiled` contraction's schedule, its `K ≡ 0 (mod 16)` precondition, and its Metal emission stay with [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md). The log-depth tree needs two further capabilities this ticket does not provide — a per-access active-participant subset and a span whose stride and count depend on the round ordinal — both recorded in `crates/tiler-ir/src/schedule/cooperative.rs`.

## Closes when

A multi-round cooperative tile lowers to a body that passes the structured-kernel verifier and executes to the same bits as the reference at its declared order; every new or generalized rule has been watched refusing its own defect, including a barrier whose enclosing loop is not the round loop; and the identity consequence of whatever the body required is recorded.
