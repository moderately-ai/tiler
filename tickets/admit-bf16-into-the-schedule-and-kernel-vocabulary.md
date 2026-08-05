---
id: admit-bf16-into-the-schedule-and-kernel-vocabulary
title: Admit BF16 into the schedule, kernel, and physical-carrier vocabularies
status: todo
priority: p1
dependencies: [register-the-bf16-semantic-operation-signatures, evaluate-bf16-reference-semantics, derive-boundary-alignment-from-the-element-type, admit-the-bf16-type-and-carrier-into-every-total-map]
related: [spike-bf16-through-the-second-dtype-seams]
scopes: [implementation/ir, implementation/compiler, contracts/navigation]
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

## Why `scopes: [implementation/ir]` could not deliver this, and what was extracted

**Measurement, 2026-08-02, worktree at base `3990f9d`, pinned toolchain, `CARGO_TARGET_DIR=./target cargo check --workspace --all-targets`.** Adding only `KernelType::Bf16` and `StorageScalar::Bf16` — the two variants this ticket's first and fourth bullets require — stops the build at eight sites, six of them outside `crates/tiler-ir/**`. `cargo` halts at the first failing crate, so the enumeration took four rounds, each patching the previous round's sites and re-running; every patch was reverted afterwards and `git status --porcelain` was empty before the commit that carries this text.

| Site | Scope |
| --- | --- |
| `crates/tiler-ir/src/program/model.rs:543` `element_bytes` | `implementation/ir` |
| `crates/tiler-ir/src/program/model.rs:1389` `push_element_type` | `implementation/ir` |
| `crates/tiler-artifact/src/program/model.rs:1737` `element_type_tag` | `implementation/artifact` |
| `crates/tiler-artifact/src/program/model.rs:1758` `storage_scalar_tag` | `implementation/artifact` |
| `crates/tiler-artifact/src/program/codec/validate.rs:369` `check_binding_access` | `implementation/artifact` |
| `crates/tiler-compiler/src/physical.rs:2085` `index_arithmetic_requirement` | `implementation/compiler` |
| `crates/tiler-metal/src/emit.rs:812` `msl_type` | `implementation/metal` |
| `crates/tiler-compiler/src/boundary.rs:2130` alignment-representability test | `implementation/compiler` |

**Fact.** `KernelType`, `StorageScalar`, and `ScalarProgram` are each deliberately **not** `#[non_exhaustive]`, and each says so in its own doc comment: they are cross-crate total maps into artifact identity, and ADR 0074 convention 5b makes widening one a build error at every encoder that must decide the new variant's meaning. `ScalarProgram`'s doc comment additionally carries a compile-tested example asserting that an out-of-crate exhaustive match keeps compiling.

**Fact.** `ScalarProgram::PointwiseBf16` — this ticket's seventh implementation key in effect — additionally breaks `crates/tiler-compiler/src/physical.rs:1588-1738` and `crates/tiler-compiler/src/frontier.rs:856-871`. The second of those is the map from a scalar program to its `StorageScalar`, which *is* the "physical carrier" half of this ticket's stated outcome; it lives in `tiler-compiler`, not in `tiler-ir`.

**Inference.** The three enum widenings this ticket needs cannot be staged: `KernelConstant::Bf16Bits`, `BinaryOp::Bf16Add`/`Bf16Multiply`, and `ConvertOp::CanonicalizeBf16Nan` are all `#[non_exhaustive]` and would widen freely, but each one's `value_type`/`operand_type`/`result_type` must return a `KernelType`, so `KernelType::Bf16` is a hard prerequisite for all of them and cannot be deferred. There is therefore no subset of this ticket that both delivers part of the user-visible outcome and leaves the workspace compiling from `implementation/ir` alone.

**Consequence.** The compile-forced cross-crate minimum was extracted into `admit-the-bf16-type-and-carrier-into-every-total-map`, which this ticket now depends on. That ticket makes `msl_type` *refuse* BF16 rather than spell it, so no unmeasured Metal capability becomes reachable while `declare-the-bf16-rows-on-the-authoritative-metal-profile` is blocked. What remains here is the BF16 pointwise expression, the scalar-program variant, the schedule and kernel verification, the lowering, and the frontier carrier mapping — and the last of those is why this ticket's scopes now include `implementation/compiler`, and the `docs/dtype-support.md` cells in its "Closes when" are why they include `contracts/navigation`.

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

## Two questions the body did not state, found while measuring

**Fact.** `crates/tiler-ir/Cargo.toml` names no dependency on `tiler-reference`, and `crates/tiler-reference/Cargo.toml` depends on `tiler-ir`. The "Required evidence" bullet asking a *lowered kernel's* interpreted result to agree bit-for-bit with `tiler-reference` therefore cannot be discharged inside `crates/tiler-ir/**`: the only crate that already sees both is `tiler-compiler`. **Inference.** Either that bullet moves to a ticket holding `implementation/compiler` — which this ticket now holds — or it is discharged in-crate against an oracle that is not independent, which is the weaker claim and should not be recorded as the stronger one. Take the first route and say which crate the comparison ran in.

**Fact.** `NumericalRealization::canonical_arithmetic_nan_bits` is a `u32` (`crates/tiler-ir/src/schedule/numerics.rs`), and BF16's canonical arithmetic NaN is the 16-bit `0x7fc0` (`CANONICAL_BF16_ARITHMETIC_NAN_BITS`, `crates/tiler-ir/src/semantic/bf16.rs`). **Inference.** A BF16 region has to put a 16-bit pattern in a 32-bit field, and which reading applies is not stated anywhere today. Widening the field would move the schedule *and* kernel identity encodings and is an identity-domain step this ticket must not take on its own. The reachable answer is to state the invariant — the field carries the region's own arithmetic type's canonical pattern, zero-extended — document it at the field, and have the schedule verifier *require* it for a BF16 region, so the reading is checked rather than assumed. Note that `carry-bf16-through-the-artifact-encoding-and-identity` reaches the same question at the artifact's own `NumericalFacts` and defers it to `redesign-the-delivered-realization-record-from-typed-evidence`; keep the two answers consistent or say why they differ.

## Graph maintenance

- Depends on the operation signatures (nothing to schedule otherwise), on the reference oracle (nothing to compare against otherwise), on the boundary-alignment derivation (the physical carrier cannot state its alignment otherwise), and on `admit-the-bf16-type-and-carrier-into-every-total-map` (no green commit is reachable otherwise — see the measurement above).
- Gates `carry-bf16-through-the-artifact-encoding-and-identity` and `lower-bf16-to-metal` **for behaviour, not for compilation**. Both of those tickets say they depend on "the IR vocabulary existing"; what they actually needed to compile was extracted into `admit-the-bf16-type-and-carrier-into-every-total-map`, which lands ahead of all three. Neither of their real deliverables — the artifact round trip and identity, the `bfloat` emission and dispatch — is affected by that extraction.
- Optimizer legality is **not** in scope. A BF16 rewrite rule needs its own legality argument, because the reassociation and contraction permissions are per-arithmetic-type and BF16's are not F32's.
- This ticket moves identity tags in `tiler-ir`. Every downstream golden and every spike citing a kernel or program identity may drift; recompute on the merged tree rather than picking a side.

## Corrected 2026-08-05 by the coordinator's pre-resume sweep — two cited states moved

**`declare-the-bf16-rows-on-the-authoritative-metal-profile` is `done`, not blocked.** The Consequence paragraph above cites its blockage as the reason `msl_type` refuses; the profile rows landed (BF16 `Dispatchable` with complete exclusive subnormal tables on `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`). The refusal's standing reason, recorded by the carrier landing (`admit-the-bf16-type-and-carrier-into-every-total-map`, commit `129d783b`): `lower-bf16-to-metal` owns the `bfloat` spelling *together with* the NaN-canonicalization helper, constant reinterpretation, and dispatch on the measured row — the refusal holds until that ticket lands whole, not until a profile row exists.

**A caller-side BF16 contract now exists and is accepted.** `NumericalContract::STRICT_BF16` and `FLUSH_SUBNORMALS_TO_ZERO_BF16` landed under the sibling domain `tiler.contract.bf16.v1` and carry Tom's 2026-08-05 acceptance. This strengthens the ticket: the vocabulary work can verify its schedule/kernel admissions against a real stated contract rather than a placeholder, and the flush-accepting request's current terminal wall (the recognizer's `dtype-f32` rule) is precisely what this ticket's pointwise-expression and scalar-program work moves next.
