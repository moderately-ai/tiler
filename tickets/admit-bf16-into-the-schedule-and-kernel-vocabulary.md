---
id: admit-bf16-into-the-schedule-and-kernel-vocabulary
title: Admit BF16 into the schedule, kernel, and physical-carrier vocabularies
status: in-progress
priority: p1
dependencies: [register-the-bf16-semantic-operation-signatures, evaluate-bf16-reference-semantics, derive-boundary-alignment-from-the-element-type]
related: [spike-bf16-through-the-second-dtype-seams]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, kernel-ir, schedule, physical]
claimed_from: todo
assignee: agent-bf16-vocab
lease_expires_at: 1785691167
---
## User-visible outcome

A verified BF16 kernel exists: a scheduled region whose scalar program is BF16, lowered to a structured kernel whose element type, constants, and binary operations are BF16, over a physical carrier that is two bytes wide.

## Why these vocabularies move together

**Fact, at `ef3c051`.** The four vocabularies a scalar dtype has to enter are each one enum with one float member, and each carries an identity tag:

- `KernelType::F32` is the sole float variant (`crates/tiler-ir/src/kernel/model.rs`).
- `KernelConstant::F32Bits(u32)` is the sole float constant.
- `BinaryOp::{F32Add, F32Multiply}` are the sole float arithmetic operations.
- `StorageScalar::{U8, F32}` is the physical carrier, with a `byte_width`.
- `ScalarProgram::PointwiseF32(PointwiseF32Expression)` is the scalar-program variant, and the whole `schedule/pointwise.rs` module is one dtype's expression language.

**Fact.** `crates/tiler-ir/src/schedule/builder.rs` additionally gates reductions on `accumulation != ArithmeticType::F32`, and `kernel/verify.rs` gates the write buffer's element type and the accumulator set on `KernelType::F32`.

**Inference.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) classified all of these as *legitimately F32-specific*: the operand type is part of each operation's identity and each tag is in the artifact encoding, so these are new variants beside the existing ones, never renames. They move together because a kernel type with no constant, or a constant with no operation, is not a state the verifier can accept — the vocabulary is only coherent as a set.

## Implementation keys

- New variants with **new** tags. Every existing tag keeps its value; the artifact encoding depends on them and an existing artifact must not change meaning.
- `PointwiseF32Expression` is one dtype's expression language, and there are two honest options: a second BF16 expression type, or one expression type parameterized by the arithmetic type. Choose with the elimination stated — the parameterized form risks admitting a mixed-dtype expression that no operation signature allows, and whichever is chosen must make that unrepresentable rather than merely unreached.
- `StorageScalar` gains a two-byte BF16 carrier. `byte_width` is exhaustive and derives from the variant, so no second width table appears.
- The reduction accumulation gate and the kernel write-type gates become derivations from the region's own dtype rather than equality against `F32`. Each must still refuse a mismatch — widening a gate to accept anything is the failure mode here.
- Lowering from the BF16 scalar program to the BF16 kernel mirrors the F32 path, and the NaN canonicalization is the BF16 one rather than `CanonicalizeF32Nan`.

## Required evidence

- A BF16 pointwise program verifies, lowers to a `VerifiedKernel`, and its interpreted result agrees **bit for bit** with `tiler-reference`'s independent evaluation, including a negative zero, both least subnormals, a canonicalized non-canonical NaN, and both infinities.
- A kernel mixing BF16 and F32 values is refused by name at verification.
- A BF16 constant carrying a 32-bit payload is refused.
- Every existing F32 tag is unchanged, pinned by the existing identity goldens; the F32 kernel identity is byte-identical.
- Deleting the BF16 NaN canonicalization from the lowering makes the reference comparison fail on exactly the NaN element and no other — the perturbation the CPU vertical uses, applied here.

## Closes when

A BF16 kernel is verified and agrees bit-for-bit with the reference oracle, the mixed-dtype and payload-width refusals are observed failing, the canonicalization perturbation is observed failing, F32 identities are unchanged, and the `Kernel vocabulary` and `Physical carrier and encoding` cells for BF16 move in `docs/dtype-support.md`.

## Graph maintenance

- Depends on the operation signatures (nothing to schedule otherwise), on the reference oracle (nothing to compare against otherwise), and on the boundary-alignment derivation (the physical carrier cannot state its alignment otherwise).
- Gates `carry-bf16-through-the-artifact-encoding-and-identity` and `lower-bf16-to-metal`.
- Optimizer legality is **not** in scope. A BF16 rewrite rule needs its own legality argument, because the reassociation and contraction permissions are per-arithmetic-type and BF16's are not F32's.
- This ticket moves identity tags in `tiler-ir`. Every downstream golden and every spike citing a kernel or program identity may drift; recompute on the merged tree rather than picking a side.
