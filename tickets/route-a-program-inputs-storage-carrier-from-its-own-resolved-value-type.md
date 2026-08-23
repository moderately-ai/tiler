---
id: route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type
title: Route a program input's storage carrier from its own resolved value type
status: todo
priority: p1
dependencies: []
related: [retire-the-gather-kernel-lowering-classification-after-the-body-landed, lower-the-indirect-gather-read-through-the-structured-kernel-body, emit-the-indirect-gather-on-metal, admit-a-storage-carrier-for-integer-program-inputs]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, compiler, program-identity, abi]
---
## User-visible outcome

A program input declared at `tiler::u32@1` is materialized at `StorageScalar::U32` and read through `KernelType::U32`, so a statically proved gather assembles into a kernel program instead of refusing because its index operand was declared `f32`.

## Why this exists

Filed 2026-08-23 by `worker-retireclass` from [`retire-the-gather-kernel-lowering-classification-after-the-body-landed`](retire-the-gather-kernel-lowering-classification-after-the-body-landed.md). That lane retired the `("kernel-lowering", "gather-kernel-body")` classification after [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md) emitted the indirect body, and the wall that surfaced behind it is this one. It is filed rather than absorbed because a per-input carrier is a separate piece of work with its own identity and ABI consequences, not a detail of retiring a classifier.

**Fact — the compiler's carrier is a total map from the program's arithmetic alone, with no input in it.** `crates/tiler-compiler/src/program.rs` declares `BoundedCarrier::of` under the anchor `The carrier one recognized arithmetic type materializes through`; it matches `ArithmeticType` and nothing else, answering `F32 -> (StorageScalar::F32, KernelType::F32)`, `Bf16 -> (StorageScalar::Bf16, KernelType::Bf16)`, and `ArithmeticType::F16 | ArithmeticType::F64 => None,`. One value is chosen once per program at the anchor `let Some(carrier) = BoundedCarrier::of(request.numerical_contract().arithmetic)` and then reaches every declared value: `fn program_input(` and `fn internal(carrier: BoundedCarrier, role: ValueRole, shape: Shape)` each stamp it into a `MaterializedValueSpec`'s `storage_scalar` and `element_type` together.

**Fact — the ABI byte formula shares that one carrier too, so this is wider than a value-spec field.** `declare_host_abi` pushes a single literal at the anchor `let element_bytes = builder.push_abi_root(AbiRoot::UnsignedLiteral(carrier.element_bytes()))?;` and multiplies *every* input's and internal's element count by it. A per-input carrier therefore needs a per-input width in the ABI expression arena, which is program identity, not a local field swap. The gather fixture happens to hide this — `StorageScalar::U32` and `StorageScalar::F32` are both four bytes wide — so a fix validated only against a `u32` index would leave a narrower integer input sized by the wrong width. That is the failure mode this ticket must discriminate.

**Fact — the refusal, measured rather than predicted.** At base `7d1219ec` with the classifier retired, `gather_program_over([4, 0], [2], 0)` compiles to `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`. The refusal is `crates/tiler-ir/src/program/builder.rs`'s, at the anchors `if buffer.element_type != value.element_type {` and `return Err(KernelProgramBuildError::StageElementType {`. Position 1 is the index operand; position 0 is the `f32` source, which agrees. `a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier` in `crates/tiler-compiler/src/request/tests.rs` pins it, and is the test that should change when this lands.

**Fact — the IR half already exists and this is not a duplicate of the ticket that landed it.** `crates/tiler-ir/src/program/model.rs` carries `StorageScalar::U32` at tag `0x04`, four bytes wide, whose own documentation says it is `physical storage and not an integer-arithmetic capability` and that its `natural access type is the exact-width` `KernelType::U32`. [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md) landed that pair and is `done`, and its own boundary paragraph states the limit at the anchor `This is a physical program-input carrier and exact access type only` — it admitted the vocabulary and the exact bind check, and deliberately did not wire any compiler-side selection into it. What is missing is the compiler asking the input what it is.

**Fact — the input already knows what it is.** `crates/tiler-ir/src/semantic/gather.rs`'s `gather_index_resolved_type` returns a nominal `ResolvedValueType` over `TypeKey::new("tiler", "u32", 1)` — anchor `the governed gather index key is valid` — and `gather_program_over` declares its index through `builder.input_resolved(...)` with exactly that type. So the resolved value type reaches the semantic program; the question is only how it reaches `build_plan_program`.

## Required work

- Re-audit every Fact above at your own base before editing. The anchors are quoted from source rather than from a rendered view; `natural access type is the exact-width` is deliberately shorter than the sentence it sits in, because that sentence wraps across two `///` lines and a full-sentence anchor returns zero.
- Route a declared input's carrier from its own resolved value type rather than from the program's arithmetic, so a `tiler::u32@1` input materializes at `StorageScalar::U32` / `KernelType::U32` while an unrecognized type refuses by name rather than defaulting to the arithmetic carrier. A silent default here sizes a caller's buffer by a width nobody stated, which is the hazard `BoundedCarrier::of`'s own `None` arm already names.
- Decide and state what happens to the ABI byte formula. A single shared `element_bytes` literal cannot express two widths; whether that becomes one literal per carrier, one per value, or something else is a program-identity choice and must be argued, not picked.
- State every identity domain that steps. **Inference, to be verified rather than trusted:** the *vocabulary* does not step, because `StorageScalar::U32` and `KernelType::U32` already hold tags `0x04` and `0x07`, and no program that this compiler can currently build carries a non-arithmetic input — every such program refuses at `StageElementType` first. So no existing pinned program's bytes should move. Verify that against the pins rather than reasoning to it.
- Test a width that actually differs, not only `u32`. `U32` and `F32` are both four bytes, so every byte-count and alignment path agrees by accident in the gather fixture and a wrong shared width stays invisible there.
- Perturb each behaviour on its own subject and quote the failure text.

## Non-goals

The kernel body, which landed. The Metal emission, which is [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) and refuses `KernelType::U32` at `msl_type` by an accepted named decision. Integer *arithmetic*, conversion, or reinterpretation — this is a carrier for a value the body only reads as an address. Widening `BoundedCarrier::of`'s arithmetic domain: `F16` and `F64` answering `None` is a separate question with its own evidence.
