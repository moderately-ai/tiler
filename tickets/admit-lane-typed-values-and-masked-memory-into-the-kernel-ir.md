---
id: admit-lane-typed-values-and-masked-memory-into-the-kernel-ir
title: Admit lane-typed values and masked memory into the structured kernel IR
status: blocked
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary]
related: [design-the-cpu-vector-lane-tier, admit-subgroup-typed-values-and-collectives-into-the-kernel-ir]
scopes: [implementation/ir, implementation/artifact, implementation/metal, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, ir, cpu, simd, public-boundary, decision, needs-tom]
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
- **`KernelType` (and similarly `UnaryOp`) is a total-map vocabulary deliberately not `#[non_exhaustive]`**, so widening it is a build error at every encoder and every target-support match. That is the mechanism; expect and keep the breakage rather than adding a wildcard arm. **`Builtin` is `#[non_exhaustive]`** (ADR 0074 convention 5a): a lane-index builtin lands additively with an appended tag and does not force out-of-crate exhaustiveness or move prior identity bytes.
- **Identity encoding is append-only.** New type and op tags and fields only; no previously encodable kernel's bytes move. Do not step the kernel identity domain (`tiler.kernel.v7\0` at writing) unless a field lands inside a fixed record that forces a domain bump — the same discipline `Bf16` and `LocalInvocationIndex` used.
- **Mechanical `KernelType` total-map / refusal arms in `tiler-artifact` and `tiler-metal` are in scope** as build breakage for the non-exhaustive-free map (`element_type_tag` / reverse; `msl_type` / `UnsupportedValueType`). Emitting lane programs from those crates remains a non-goal; only the compile-or-named-refusal seams required so widening cannot silently default are required here.

## Required failure-path evidence

A mask passed where a block predicate is required and the reverse; a fixed-shape operation consuming a scalable value; a masked load whose mask is not dominated by the enclosing predicate; a masked store whose ownership witness does not cover its active lanes; a lane operation whose element type disagrees with its buffer's; an identity-encoding round trip proving no previously encodable kernel's bytes moved.

## Non-goals

Any emission target. Any backend. Schedule vocabulary (its own ticket, and this one's dependency). Performance.

## Decision packet — 2026-08-10

ADR 0093 accepted the vector-tier model only, not the exact public Rust spellings of kernel-IR lane constructs. Tom must accept the public shapes under ADR 0075 before implementation lands. Recommendation: fixed and scalable lane types as distinct `KernelType` variants (not one shape with an optional count), a mask type distinct from `Bool`, masked load/store that name the mask beside the existing witnesses, a lane-index builtin, element-wise arithmetic widening of the existing op families rather than a parallel vector family, and deliberate absence of a horizontal-reduce kernel op (ordered horizontal accumulate stays `SerialLoop` / `ReductionTopology::Serial`). Exact nested-vs-flat variant spellings and whether mask-generating compare is a new op vs a builtin-shaped form remain open within that shape set.

**Fact repair — 2026-08-10.** An earlier Implementation key claimed both `KernelType` and `Builtin` were deliberately not `#[non_exhaustive]`. Only `KernelType` (and `UnaryOp`) carry that total-map discipline; `Builtin` is `#[non_exhaustive]`. Reproduce: attributes on `pub enum KernelType` and `pub enum Builtin` in `crates/tiler-ir/src/kernel/model.rs`.

## Board release path

While [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) is open, this ticket stays `blocked`. When that dependency completes, move this ticket to `awaiting-decision` (not `todo` / `ready`) until Tom accepts the public kernel-IR shapes under ADR 0075 — matching the schedule-vocabulary and target-profile siblings.

## Closes when

The constructs are admitted, the kernel verifier proves the lane obligations, every check above is observed failing against an accepted neighbour, and the lowering from a lane-bound scheduled region produces a kernel whose predication is visible in its own body.
