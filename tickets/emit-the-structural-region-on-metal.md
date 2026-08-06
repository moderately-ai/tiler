---
id: emit-the-structural-region-on-metal
title: Emit the structural region on Metal
status: in-progress
priority: p1
dependencies: []
related: [reach-a-verified-kernel-through-the-structural-families, admit-the-structural-families-into-the-scheduled-region-vocabulary, own-operation-family-support-matrix]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, structural, backend]
claimed_from: todo
assignee: agent-metal-structural
lease_expires_at: 1785994006
---
## User-visible outcome

A structural region — a widening broadcast read, or a reindexed operand feeding a pointwise neighbour — reaches emitted Metal source, so the structural row's R6 rung rests on a backend that emits it rather than on a vocabulary that could.

## Why this is separate from the compiler-side vertical

**Fact — the vertical stops one crate short, and the residual is small and named.** [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md) carried both families from `compile()` to a `VerifiedKernel` whose result is the reference evaluator's bit for bit. What it could not carry is emission: `tiler-metal` depends on `tiler-ir` and `tiler-artifact` and *not* on `tiler-compiler`, so its fixtures build regions by hand and no test in this repository puts a compiler-derived structural region through `emit`. That is `implementation/metal` scope, which the compiler-side ticket does not hold.

**Fact — one construct is definitively unemittable, and nothing catches it.** `emit_binary` in `crates/tiler-metal/src/emit.rs` maps `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo`, `I32Subtract`, the `f32`/`bf16` arithmetic, and `F32Maximum`, and falls through every other tag to `MetalEmitError::UnsupportedOperation { family: Binary }`. `BinaryOp::IndexSubtract` — appended at tag `0x0c` by [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) — has no arm. It is emitted by exactly one producer, `emit_offset` in `crates/tiler-ir/src/kernel/lower.rs:2093`, for a reindex mirror's `extent - 1 - c`, so the `reverse-axis` form verifies as a kernel and refuses at the backend. Reproduce:

```sh
rg -n 'IndexSubtract' crates/
```

Four hits, none of them under `crates/tiler-metal/`.

**Fact — every *other* ingredient is already emitted, which is what makes this bounded.** `crates/tiler-metal/goldens/reduction_multi_axis.metal` emits `/`, `%`, `*`, and `+` over `uint64_t` structured index arithmetic (lines 59–65), which is exactly the construct set a broadcast replication decode and every non-mirrored reindex decode produce. `crates/tiler-metal/goldens/contraction_strict_tensor.metal` declares two read buffers at *different* element counts (8 and 12) beside one write, which is the signature shape a widening broadcast needs. So the residual is the region never having been run through the emitter, plus the one missing arm — not a vocabulary redesign.

## Required delivery

- **An `IndexSubtract` arm in `emit_binary`**, with the unsigned-wrap question answered rather than inherited: the KIR operation's contract is that its result is *proven* non-negative (the compiler's own test interpreter asserts `lhs >= rhs` rather than wrapping), and `uint64_t` subtraction in MSL wraps silently. Decide, and record, whether the emitter asserts, emits a widened signed form, or rests on the producer's proof — and say which in the emitted comment block if it rests on a proof.
- **At least one golden covering a structural region**, added to `crates/tiler-metal/src/golden_compilation.rs`'s complete-directory list so it compiles through the offline driver. A broadcast replication and a mirrored reindex are the two shapes worth pinning; the mirror is the one the new arm exists for.
- **A totality check over `BinaryOp` reachable from lowering**, or an explicit statement of why one is not wanted. Today a tag appended in `tiler-ir` reaches the backend's wildcard arm with nothing red, which is how this gap arrived.

## Non-goals

Device dispatch (that is R7 and belongs with the runtime integration), any new `LogicalAccess` relation, and the compiler-side recognition — all three are delivered or separately owned.

## Closes when

A structural region emits Metal source that compiles through the offline driver, the `reverse-axis` form among it, and the structural row's rung records what a backend now actually emits.
