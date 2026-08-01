---
id: admit-lane-typed-values-and-masked-memory-into-the-kernel-ir
title: Admit lane-typed values and masked memory into the structured kernel IR
status: blocked
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary]
related: [design-the-cpu-vector-lane-tier]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, ir, cpu, simd, public-boundary]
---
## User-visible outcome

A CPU emission target consumes a `VerifiedKernel` whose lane structure and lane predication are *in the kernel*, so the kernel verifier's existing proofs — predicate dominance of every effect, bounds-witness provenance, exactly-once output coverage — cover the lane program rather than a scalar shadow of it.

## Why a lane-annotated schedule is not enough

**Fact.** The kernel-IR module states its own purpose: "The layer exists so a backend never reconstructs graph-specific semantics… Iteration-domain guarding is an explicit `OperationView::Predicated` region, so tail behaviour is visible rather than implied by a launch geometry." A design in which the lane mask lives on the schedule and not in the body would leave the verifier proving predicate dominance and bounds provenance over a body whose real predication it cannot see, and would land the fault-suppression obligation of a masked load in a backend with nothing checking it was met. The elimination is derived in [the CPU vector-lane tier](../docs/research/scheduling/cpu-vector-lane-tier.md).

## Implementation keys

- **`KernelType` gains a fixed lane shape (element plus literal lane count) and a scalable lane shape (element alone).** Two shapes, not one with an optional count: an optional count would let a fixed-width operation consume a scalable value, and the accepted design's whole point is that those are different programs.
- **A lane-mask type distinct from `Bool`.** `Bool` is the whole-block control predicate `OperationView::Predicated` consumes; a mask is per-lane and is consumed by memory operations. They must not be interchangeable.
- **Masked load and masked store**, each naming its mask beside the buffer, offset, and `BoundsWitnessId` or `OwnershipWitnessId` it already names.
- **A mask-generating comparison of the lane index against a bound**, plus a lane broadcast. Without the first, the scalable binding's predicated covering relation is not expressible in a body; without the second, a loop-invariant scalar cannot enter a lane computation.
- **Element-wise widening of the existing `BinaryOp`/`UnaryOp` families to lane-shaped operands, not a parallel vector family.** The rounding rule is unchanged and a parallel family would duplicate every numerical obligation into a second set that can drift.
- **No horizontal-reduce operation.** By the accepted decision's item 1, an order-preserving horizontal accumulate is below the schedule boundary and its kernel-IR spelling already exists: the `SerialLoop` with a typed loop-carried accumulator that `ReductionTopology::Serial` already lowers to. Adding one would create a second unchecked spelling of one fold.
- **`KernelType` and `Builtin` are deliberately not `#[non_exhaustive]`**, so widening them is a build error at every encoder and every target-support match. That is the mechanism; expect and keep the breakage rather than adding a wildcard arm.

## Required failure-path evidence

A mask passed where a block predicate is required and the reverse; a fixed-shape operation consuming a scalable value; a masked load whose mask is not dominated by the enclosing predicate; a masked store whose ownership witness does not cover its active lanes; a lane operation whose element type disagrees with its buffer's; an identity-encoding round trip proving no previously encodable kernel's bytes moved.

## Non-goals

Any emission target. Any backend. Schedule vocabulary (its own ticket, and this one's dependency). Performance.

## Closes when

The constructs are admitted, the kernel verifier proves the lane obligations, every check above is observed failing against an accepted neighbour, and the lowering from a lane-bound scheduled region produces a kernel whose predication is visible in its own body.
