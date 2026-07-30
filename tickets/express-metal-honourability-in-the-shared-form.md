---
id: express-metal-honourability-in-the-shared-form
title: Express the Metal subnormal fact as a per-dimension honourability declaration
status: todo
priority: p0
dependencies: [compose-numerical-honourability-and-retire-the-strict-boolean, prototype-public-compiler-api, admit-a-caller-declared-target-profile]
related: [declare-metal-numerical-honourability, draft-target-honourable-numerical-contract-adr]
scopes: [implementation/metal, implementation/compiler, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, feasibility]
---
The remaining half of `declare-metal-numerical-honourability`, split out when its two settled questions landed. ADR 0076 item 3.

This ticket becomes startable after the checked caller-profile boundary lands. The former three-way ownership question was eliminated by the current dependency graph and project scope: semantic IR must not own Metal target facts, `tiler-build` already depends on both `tiler-compiler` and `tiler-metal`, and a new crate has no independent second consumer.

`declare-metal-numerical-honourability` settled the two questions that did not depend on the shared honourability form: the backend-local conformance step survives alongside the profile declaration with a stated reason, and the four golden fixtures stay governed under the strict declared realization. What it could not do is the piece that gives the ticket its name — express `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form, so `feasibility` can assess it as a peer of `CheckedTargetProfile` *before* emission rather than discovering it during.

## What is true today

**Fact — `crates/tiler-metal/src/target.rs`.** `MetalSubnormalArithmetic::{FlushesToZero { zero_sign }, PreservesSubnormals}` is a required caller-stated field of `MetalTargetFacts`, with its measurement recorded on the type. It is consulted in exactly one place, `emit::subnormal_gap`, and only during emission.

**Fact — the old strict-f32 boolean has been retired.** The compiler now has a
private per-dimension honourability form, but no measured Metal fact reaches it.

**Fact — the two crates cannot see each other, restated from the current edges.** `grep -n 'tiler-' crates/tiler-metal/Cargo.toml crates/tiler-compiler/Cargo.toml` at `01264be`: `tiler-metal` depends on `tiler-artifact` (`:16`), `tiler-ir` (`:17`), and **`tiler-metal-aot`** (`:20`); `tiler-compiler` depends on `tiler-ir` (`:16`) and **`tiler-reference`** (`:19`). Neither depends on the other, so the conclusion this ticket rests on survives, and `AGENTS.md` still requires the compiler core to stay independent of Metal types. The re-pin matters because both edge sets have grown since the original reading at `94fb26e`, and "neither depends on the other" is a claim about a graph that moves.

## User-visible outcome

Make measured Metal numerical behavior available to compiler feasibility before
emission, keyed by both numerical dimension and arithmetic dtype, while
retaining one authoritative declaration and preserving backend
re-verification.

## Ownership derivation

The checked adapter belongs in `tiler-build`, the existing orchestrator that can see both the Metal authority and the compiler declaration boundary. Putting the declaration in `tiler-ir` would widen semantic IR into target vocabulary and violate the compiler/backend separation. Creating a third crate before a second orchestrator consumer exists would add a boundary without reducing coupling. An unchecked consumer-owned translation is invalid because it cannot prove exhaustive, faithful coverage.

The adapter reads the single authoritative `MetalTargetFacts`, constructs the compiler profile through its checked builder, and is exhaustive over every relevant source and destination variant. Metal backend re-verification reads the same declaration and stays in place. Record this ownership correction in ADR 0076 when the implementation lands.

## Constraints inherited, not up for renegotiation

- **Honourability is a stated target fact, never a probed one.** Under `-fmetal-math-mode=relaxed` a `scale 1.0, bias +0.0` kernel returns subnormal operands unchanged, which looks like preservation and is not: `x * 1.0` folds to a copy under every math mode, the surviving `+0.0` fadd is the operation that flushes, and `relaxed` deletes it. Do not close any part of this by observing a compiled kernel. `docs/backends/metal.md` records the trap.
- **Do not close a gap by widening a rule.** A mismatched zero sign must stay a rejection. Letting a program that asked for positive-zero flushing run on a sign-preserving target returns `0x80000000` where it asked for `0x00000000` — a wrong answer, not a relaxed one.
- **Keep the measurements on the declaring types.** `MetalTargetFacts` documents its measured basis on the type itself; preserve that wherever the declaration moves.
- **The backend-local conformance step stays.** `MetalNumericalGap` and `require_declared_realization` are not retired by this ticket; `declare-metal-numerical-honourability` recorded why, and `crates/tiler-metal/src/record.rs` carries the reasoning. This ticket adds the earlier checkpoint, it does not remove the later one.

## Closes when

The Metal subnormal fact is expressed as a per-dimension honourability declaration in the shared form; `feasibility` assesses it before emission and rejects with the shape `compose-numerical-honourability-and-retire-the-strict-boolean` defines — naming the dimension, the required behaviour, the declared target behaviour, and the declaring profile's versioned identity; the siting is recorded in ADR 0076; and `make full` passes.

## Graph maintenance

- Follow `admit-a-caller-declared-target-profile`; do not invent a parallel declaration surface in `tiler-build`.
- Make `redesign-the-delivered-realization-record-from-typed-evidence` depend on this producer path before its fixtures claim compiler-selected Metal evidence.
- If a second non-build orchestrator needs the same exhaustive adaptation, file the evidence for extracting a shared adapter crate then; do not create one speculatively.
