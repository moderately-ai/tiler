---
id: publish-an-l3-contraction-cell-through-the-accepted-route
title: Publish an L3 contraction cell through the accepted route and compare its retained digest
status: todo
priority: p2
dependencies: [raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells]
related: [integrate-the-contraction-vertical-into-the-runtime, correct-the-four-thread-grid-rationales-the-measured-row-falsified, realize-the-tiled-contraction-schedule-and-its-metal-emission]
scopes: [implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, contraction, runtime, conformance]
---
## User-visible outcome

One L3 correctness cell executes through the accepted AOT and runtime route and its bytes are compared against the realization probe's retained `result_sha256` for that cell, so that digest becomes the independent cross-check it was retained to be rather than an unavailable predicate.

## Why this is a separate ticket

**Fact — the compile-phase half is already delivered, and it is the half that was blocked.** [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md) moved the grid-axis row to a measured 268,435,456 and proved by compiling that all six L3 cells now reach a selected physical plan (`tiler_build::metal_plan::tests::the_measured_grid_axis_admits_every_l3_contraction_cell`). The `target.grid-axis` / `Rejected("target-infeasible")` / `required: Threads(1024), available: Threads(4)` refusal that [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) recorded no longer occurs.

**Fact — what remains is a scope boundary, not a feasibility one.** Publishing and dispatching a cell needs `prototypes/serial-sum-compile` (`implementation/metal-aot`) and `prototypes/serial-sum-run` (`implementation/runtime`). `crates/tiler-build` reaches neither: it declares no `tiler-runtime` edge and creates no device, so it can establish that a plan composes and nothing about executed bits.

## Implementation keys

**Add a member; do not move the published one.** The existing `contraction` member is `2x2x3`, and its shape is load-bearing: a result with more than one row *and* more than one column is what makes the two operand access relations — `(t, o, d) -> (t, d)` never mentioning `o`, and `(t, o, d) -> (o, d)` never mentioning `t` — separately observable, and it is what let the `operands[0]`-for-`operands[ordinal]` perturbation fail while every one-input member passed. Every L3 correctness cell is `M=1` or has `M != N`; `w_decode_kv` is exactly the `1 x N` shape the producer's own comment records as unable to separate the two relations. Repointing the existing member at a cell would trade a discriminating shape and five adversarial numerical cases for a non-discriminating one, so the cell arrives as a second member.

That second member forces the runner change this ticket exists for: `prove_contraction` opens exactly one path, `proof_member(base, CONTRACTION_CLASS, "selected")`. It already derives `(m, n, k)` from the artifact's declared interface rather than from its own constants, so the shape handling itself needs nothing.

**The operands are the probe's, or the digest means nothing.** `crates/tiler-compiler/src/governed/contraction_conformance.rs` already carries the probe's seeding — `WORKLOAD_SEED = 0x5445_524D`, the `RIGHT_SEED_MASK` derivation, `splitmix64`, and the `m * 2^-24` value rule that makes every operand exactly representable. Read it there rather than re-deriving it.

**State which comparison is being made.** Digesting the sidecar's reference expectation and digesting the *executed* bytes are different claims. The deliverable is the second; a producer-side assertion that the embedded reference expectation hashes to the retained value is a useful validity condition for the fixture and is not a substitute for it.

**Watch the digest check fail before trusting it.** A comparison against a 64-character constant passes trivially if the bytes never reach it.

## Non-goals

The tiled realization, owned by [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md); any timing or performance comparison; the remaining five cells, which follow the first at no architectural cost.

## Boundary this inherits

The grid-axis row is `MeasuredEnvironment`-valid on the macOS 27.0 `26A5388g` / Apple M4 Max / Xcode 26.6 `17F113` / SDK 26.5 `25F70` row alone, and the retained `result_sha256` values are a measurement of that same host. A run on any other host row is a different claim and must announce the difference and decline to compare, not compare anyway.

## Closes when

One L3 correctness cell is published through `prototypes/serial-sum-compile` and dispatched through `prototypes/serial-sum-run` with exact `MTLCommandBufferStatusCompleted` before readback, and the SHA-256 of its executed result bytes is compared against the retained value for that cell on a matching host row — reported as a match, or as a correctness finding with full evidence if it is not.
