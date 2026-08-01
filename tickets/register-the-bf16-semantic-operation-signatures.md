---
id: register-the-bf16-semantic-operation-signatures
title: Register the pure-BF16 constant, multiply, and add operation signatures
status: in-progress
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams, register-the-accepted-built-in-dtype-catalog, own-operation-family-support-matrix, design-the-bf16-computation-and-accumulator-contract]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, semantics, operations]
claimed_from: todo
assignee: worker-bf16-sigs
lease_expires_at: 1785569171
---
## User-visible outcome

A semantic program can name a pure-BF16 constant, multiply, and add. Today `tiler::bf16@1` is a recognized identity that no operation signature admits, so a BF16 tensor can be described and nothing can be done to it.

## Why this is the second root

**Fact, at `ef3c051`.** `register_builtin_dtype_catalog` registers `tiler::bf16@1` with a complete structural descriptor, and its module documentation states that a row "creates no operation signature, reference evaluator, storage carrier, kernel type, target dispatch fact, or backend lowering". The registered operation keys are all F32-specific: `constant_f32_op`, `multiply_f32_op`, `add_f32_op`.

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) confirmed `SemanticRegistryProvider::register_operation` is public and dtype-neutral, and derived the exact semantics these keys must carry.

**Inference.** These are new operation keys, not widened ones. Operand type is part of an operation's identity under ADR 0026, so `tiler::multiply-bf16@1` sits beside `tiler::multiply-f32@1` rather than replacing it, and nothing may make an F32 operation accept a BF16 operand.

## Implementation keys

- Three keys — `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, `tiler::add-bf16@1` — each with an inferencer refusing any operand that is not `tiler::bf16@1`.
- The constant's payload attribute is exact BF16 bits, validated against the registered descriptor's width, in the same shape `F32_CONSTANT_BITS_ATTRIBUTE` uses for binary32. A binary32 payload on a BF16 constant is refused.
- Each operation's canonical facts state, separately and explicitly: computation type, accumulator type, intermediate-materialization type, and result type. All four are BF16 here, and **stating them separately is the point** — a future F32 accumulator must be an explicit change to a fact, not the removal of an assumption. The facts also state round-to-nearest-ties-to-even at every observable materialization and the canonical arithmetic NaN payload.
- The normative definition names the ratified RISC-V BF16 operand format and the preserved source id, matching the catalog row. Do not restate the format table.
- **No FMA, no contraction, no reassociation, no mixed precision, no implicit promotion.** Each rejects by typed reason. `design-the-bf16-computation-and-accumulator-contract` owns whether any of them is ever admitted.
- The typed facade (`F32Constant`'s peer) may be added for BF16, but the marker binding and the operation keys are the deliverable; a facade with no registry behind it is not.

## Required evidence

- A program applying `tiler::multiply-bf16@1` to two BF16 values verifies; the same operation applied to an F32 value is refused by name, and to a mixed pair is refused by name.
- An F32 operation applied to a BF16 operand is refused, so registration did not weaken the existing signatures.
- A constant carrying a binary32-width payload is refused.
- The four type facts are readable from the registered operation and are all `tiler::bf16@1`, asserted individually rather than as a group.
- Registering these keys does not make any BF16 program compilable, reference-evaluable, or dispatchable; a test asserts each of those still fails closed.

## Closes when

The three keys are registered with complete facts, every refusal above is observed failing, the operation-family matrix row in `docs/roadmap.md` moves from R1 to R3 for BF16 arithmetic with its evidence stated, `docs/dtype-support.md`'s BF16 `Semantic operation signatures` cell moves off `absent/unsupported`, and no other cell moves.

## Graph maintenance

- Gates `evaluate-bf16-reference-semantics` and `admit-bf16-into-the-schedule-and-kernel-vocabulary`. Independent of the target-profile children.
- Do not re-register the identity; `register-the-accepted-built-in-dtype-catalog` owns it and is `done`.
- The `Cast and convert` row of the operation-family matrix states that admitting any second dtype into a profile forces an explicit conversion operation. This ticket does **not** discharge that; it deliberately admits no BF16/F32 conversion, and the first program needing one blocks on the conversion row rather than acquiring an implicit promotion here.
