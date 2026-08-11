---
id: admit-guarded-output-tails-for-cooperative-contraction
title: Admit guarded output tails for the cooperative contraction
status: awaiting-decision
priority: p1
dependencies: [admit-a-cooperative-tile-over-shared-operands]
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, realize-the-strict-contraction-on-metal]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, public-boundary, decision, needs-tom]
---
## User-visible outcome

A cooperative contraction can represent a partial output block without reading outside either operand or writing outside the output, while every launched participant still reaches every required synchronization point.

## Facts at filing — 2026-08-11

**Fact — the retained tiled kernel needs this boundary.** [`contract_tiled`](../spikes/scheduling/metal_contraction_vertical/kernels.metal) launches a 16×16 workgroup at `M = 1` and `M = 10`, substitutes `0.0` for out-of-range operand loads, keeps all 256 participants convergent at both barriers, and predicates the owning write with `m < M && n < N`. The six-cell record therefore does not satisfy the exact-divisible bijection accepted for [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md).

**Fact — this is more than a wider commit range.** [`TailPolicy`](../crates/tiler-ir/src/schedule/model.rs) admits only `Exact`; intrinsic verification requires launched work items, grid threads, and iteration elements to agree; contraction operand proofs assume every iteration coordinate is in range; and the current commit guard represents a prefix rather than a two-dimensional boundary predicate. Relaxing any one of those alone would leave another layer authorizing an out-of-range effect.

## Decision still required

Tom must choose the exact typed relation joining all of these facts:

1. padded launch coverage and the logical output domain;
2. the derived active-output predicate over workgroup and local coordinates;
3. guarded operand loads and the value written to staging when the operand coordinate is inactive;
4. the subset of participants permitted to perform the owning write;
5. the ownership proof that every real output has exactly one writer and every inactive invocation has none; and
6. identity, KIR guard, preflight, and backend-consumption consequences.

There is no default from exact to guarded and no fallback from tiled to direct. A caller must request an admitted approach; an unsupported guarded relation refuses before executable admission.

## Non-goals

Contracted-axis padding, a padding-neutrality proof, cost selection, Metal emission, or a general masked-kernel language. `K` remains exact and separately refused.

## Activation and closure

This decision follows the exact-divisible relation so its tail semantics extend one settled execution binding instead of designing both simultaneously. It closes only when the complete predicate and proof population is accepted, every inactive read/write case is independently perturbation-proved, barrier convergence remains uniform, and the exact-divisible relation retains its own fail-closed path unchanged.
