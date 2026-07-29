---
schema: "tiler-doc/v1"
id: "tiler.roadmap.dtype-support"
kind: "roadmap"
title: "Dtype support maturity"
topics: ["roadmap", "dtypes", "implementation"]
roadmap_status: "proposed"
related: ["tiler.roadmap","tiler.contract.numerical-semantics"]
---

# Dtype support maturity

**Status:** proposed visibility ledger

This ledger answers one question: for each recognized or deliberately excluded dtype family, what has Tiler actually built at each layer? The [mature dtype taxonomy](research/numerics/mature-dtype-taxonomy.md) owns the semantic universe, [Numerical semantics](numerical-semantics.md) owns meaning and legality, and this document owns delivered maturity. Listing a family authorizes no implementation.

The matrices are deliberately non-monotone. A target measurement can exist before a semantic identity is registered; a physical carrier can exist without tensor arithmetic; and a tested reference evaluator does not imply runtime dispatch. Read a cell only for its named family and layer.

## Cell vocabulary

Every cell uses exactly one maturity claim:

- **absent/unsupported** — no family-specific authority exists at this layer, or the family is deliberately outside this axis.
- **type-system reservation** — a generic type can represent the shape of the concept, but does not fix this family's meaning.
- **architectural seam** — an accepted decision or normative contract fixes family-specific obligations, but no mechanism at this layer implements them.
- **implemented mechanism** — family-specific production code exists, but this document cites no checked guarantee for the complete claim in this cell.
- **tested guarantee** — checked evidence exercises the linked claim within the boundary stated in the family notes.

An implemented generic provider, opaque canonical byte carrier, enum variant, or target measurement is evidence about that mechanism only. It never promotes an unregistered family.

## Semantic maturity

| Family | Recognized identity | Semantic operation signatures | Reference evaluation | Numerical contract and honourability |
| --- | --- | --- | --- | --- |
| Logical `bool` | [architectural seam](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) |
| Signed exact-width integers `i2/i4/i8/i16/i32/i64` | [architectural seam](#signed-and-unsigned-integers) | [architectural seam](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [architectural seam](#signed-and-unsigned-integers) |
| Unsigned exact-width integers `u2/u16/u32/u64` and unregistered uses of `u4/u8` | [architectural seam](#signed-and-unsigned-integers) | [architectural seam](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [architectural seam](#signed-and-unsigned-integers) |
| Governed nominal U4/U8 code-domain types | [tested guarantee](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [tested guarantee](#signed-and-unsigned-integers) | [tested guarantee](#signed-and-unsigned-integers) |
| IEEE `f32` | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) |
| IEEE `f16/f64/f128` | [architectural seam](#other-ieee-binary-floats-and-bf16) | [architectural seam](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [architectural seam](#other-ieee-binary-floats-and-bf16) |
| BF16 | [architectural seam](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [architectural seam](#other-ieee-binary-floats-and-bf16) |
| OCP E4M3FN, E5M2, E2M3FN, E3M2FN, E2M1FN, and E8M0FNU scale data | [architectural seam](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [architectural seam](#ocp-reduced-precision-formats) |
| IEEE decimal32/64/128 | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) |
| `tiler::complex@1<ComponentTypeKey>` for f16/f32/f64 components | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) |
| Per-tensor strict-affine U4/F32 | [tested guarantee](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) |
| Per-tensor strict-affine U8/F32 | [tested guarantee](#strict-affine-u8f32) | [tested guarantee](#strict-affine-u8f32) | [tested guarantee](#strict-affine-u8f32) | [tested guarantee](#strict-affine-u8f32) |
| Other affine parameter maps, code widths, expressed types, and `Requantize` | [architectural seam](#other-affine-and-ocp-mx-schemes) | [architectural seam](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [architectural seam](#other-affine-and-ocp-mx-schemes) |
| OCP MX compound schemes | [architectural seam](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [architectural seam](#other-affine-and-ocp-mx-schemes) |
| External or vendor formats | [architectural seam](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) |
| Wide or bounded integer extensions, fixed-point, UNORM/SNORM, posits, and other reserved numeric families | [type-system reservation](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) |
| Execution-only formats such as TF32, PTX scale encodings, `x86_fp80`, and `ppc_fp128` | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [architectural seam](#execution-only-formats) |
| Nonnumeric tensor element domains | [type-system reservation](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) |
| Sparse or ragged tensor representations | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) |
| Non-tensor graph values | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) |

## Physical and execution maturity

| Family | Physical carrier and encoding | ABI and materialization | Optimizer legality | Kernel vocabulary | Backend lowering | Backend execution | Runtime semantic validation | Target-family dispatchability | Conformance evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Logical `bool` | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) | [absent/unsupported](#logical-bool) |
| Signed exact-width integers | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) |
| Unsigned exact-width integers outside governed U4/U8 code-domain recognition | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) |
| Governed nominal U4/U8 code-domain types | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [absent/unsupported](#signed-and-unsigned-integers) | [tested guarantee](#signed-and-unsigned-integers) |
| IEEE `f32` | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [tested guarantee](#ieee-f32) | [absent/unsupported](#ieee-f32) | [absent/unsupported](#ieee-f32) | [tested guarantee](#ieee-f32) |
| IEEE `f16/f64/f128` | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) |
| BF16 | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) | [absent/unsupported](#other-ieee-binary-floats-and-bf16) |
| OCP reduced-precision formats and E8M0FNU scale data | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) | [absent/unsupported](#ocp-reduced-precision-formats) |
| IEEE decimal32/64/128 | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) |
| Parameterized complex | [architectural seam](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) |
| Per-tensor strict-affine U4/F32 | [tested guarantee](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) | [absent/unsupported](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) | [absent/unsupported](#strict-affine-u4f32) | [absent/unsupported](#strict-affine-u4f32) | [absent/unsupported](#strict-affine-u4f32) | [tested guarantee](#strict-affine-u4f32) |
| Per-tensor strict-affine U8/F32 | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [absent/unsupported](#strict-affine-u8f32) | [tested guarantee](#strict-affine-u8f32) |
| Other affine quantization | [type-system reservation](#other-affine-and-ocp-mx-schemes) | [type-system reservation](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) |
| OCP MX compound schemes | [architectural seam](#other-affine-and-ocp-mx-schemes) | [type-system reservation](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) | [absent/unsupported](#other-affine-and-ocp-mx-schemes) |
| External or vendor formats | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [type-system reservation](#external-and-vendor-formats) | [absent/unsupported](#external-and-vendor-formats) | [absent/unsupported](#external-and-vendor-formats) | [absent/unsupported](#external-and-vendor-formats) | [absent/unsupported](#external-and-vendor-formats) |
| Other reserved numeric families | [type-system reservation](#decimal-complex-and-other-reserved-numeric-families) | [type-system reservation](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) | [absent/unsupported](#decimal-complex-and-other-reserved-numeric-families) |
| Execution-only formats | [type-system reservation](#execution-only-formats) | [type-system reservation](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [type-system reservation](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) | [absent/unsupported](#execution-only-formats) |
| Nonnumeric tensor element domains | [type-system reservation](#nonnumeric-and-non-tensor-domains) | [type-system reservation](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) | [absent/unsupported](#nonnumeric-and-non-tensor-domains) |
| Sparse or ragged tensor representations | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) |
| Non-tensor graph values | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) | [absent/unsupported](#sparse-ragged-and-non-tensor-values) |

## Evidence and triggers

### Logical bool

**Fact.** [ADR 0028](decisions/0028-recognize-sub-byte-integers.md) recognizes logical `bool` as two-valued and distinct from integer `i1`, while leaving bit-, byte-, and ABI-sized storage independent. The generic nominal type mechanism can carry it, but the standard registry does not register it. `KernelType::Bool` in [structured kernel IR](../crates/tiler-ir/src/kernel/model.rs) and `AbiType::Boolean` in the host ABI are control predicates, not logical bool tensor values.

**Trigger.** Admit the first exact vertical only for a named `Select`, comparison, logical reduction, or frontend workload, with semantic operations, speculation rules, storage and ABI, runtime validation, target facts, and conformance named together.

### Signed and unsigned integers

**Fact.** [ADR 0028](decisions/0028-recognize-sub-byte-integers.md) recognizes signed and unsigned widths 2, 4, 8, 16, 32, and 64. [ADR 0039](decisions/0039-explicit-integer-overflow-operations.md) and [ADR 0040](decisions/0040-specialize-integer-division-families.md) make overflow-specialized arithmetic and division/remainder families explicit; no general integer evaluator, optimizer, or backend vertical implements them. [Quantization semantics](../crates/tiler-ir/src/semantic/quantization.rs) does register nominal U4 and U8 definitions and the reference provider tests their exact code domains, but the only governed operation signatures use them as strict-affine code and zero-point roles. That tested recognition does not establish integer arithmetic, physical ABI, or dispatch. `KernelType::Index`, `KernelType::U8`, and `KernelType::I32` remain address, carrier, and widened-subtract machinery rather than tensor arithmetic support.

**Trigger.** A named tensor workload must select an exact width, operation family, overflow/division/conversion behavior, storage, target, and corpus. Quantized codes alone do not trigger an integer arithmetic vertical.

### IEEE f32

**Fact.** [Standard semantics](../crates/tiler-ir/src/semantic/registry.rs) registers F32 plus constant, multiply, add, and strict serial sum; [standard reference evaluation](../crates/tiler-reference/src/standard.rs) implements them; [compiler request normalization](../crates/tiler-compiler/src/request.rs) and governed fusion roles implement the narrow F32 profile; [structured kernel lowering](../crates/tiler-ir/src/kernel/lower.rs), program construction, artifact construction, and [Metal emission](../crates/tiler-metal/src/emit.rs) preserve its storage and numerical requirements. Checked tests cover exact bits, separate rounding, contributor order, NaN canonicalization, subnormals, signed zero, overflow, ABI identity, deterministic source, typed numerical refusal, and the target-neutral pipeline.

**Measurement.** The retained [runtime proof](../prototypes/serial-sum-run/src/proof.rs) executed the bounded strict F32 program on one Apple M4 Max host and compared thirty cases bit-for-bit. That is a tested guarantee for that exact host, toolchain, program shape, and corpus, not production runtime support or portable Apple-family dispatchability. Strict subnormal-preserving arithmetic remains rejected by the governed Metal profile where the measured target cannot honour it.

**Trigger.** Widen only from a named workload. Any new operation, numerical realization, artifact family, or device row gets its own support statement and cannot inherit this profile's guarantee.

### Other IEEE binary floats and BF16

**Fact.** [ADR 0036](decisions/0036-recognize-standard-binary-and-microscaling-formats.md) recognizes IEEE f16/f32/f64/f128 and BF16's logical identity. `ArithmeticType::{F16,Bf16,F64}` and the [Apple numerical measurements](research/apple-targets/numerical-behaviour.md) are contract vocabulary and bounded target evidence, not semantic registrations. No F16, BF16, or F64 tensor type is registered by standard semantics, and no reference, physical ABI, optimizer, kernel, lowering, runtime-validation, dispatchability, or conformance vertical exists for one. BF16's value contract does not imply an arithmetic or accumulator policy.

**Trigger.** A selected workload must name the exact dtype, operation signature, conversions, compute and accumulator types, numerical contract, target, ABI, and conformance vectors. A second target measurement alone does not trigger registration.

### OCP reduced-precision formats

**Fact.** [ADR 0036](decisions/0036-recognize-standard-binary-and-microscaling-formats.md) recognizes OCP E4M3FN, E5M2, E2M3FN, E3M2FN, E2M1FN, and E8M0FNU with complete nominal identity. None is registered. E8M0FNU is unsigned exponent-only scale data in compound formats, not an ordinary signed arithmetic dtype. FNUZ, alternate-bias, and vendor variants are not aliases without exact equivalence evidence.

**Trigger.** A selected model or kernel must name the exact format, operations, conversion and accumulation policy, physical representation, runtime refusal rules, target dispatchability, and conformance corpus.

### Decimal, complex, and other reserved numeric families

**Fact.** [ADR 0035](decisions/0035-recognize-ieee-decimal-floating-formats.md) recognizes decimal32/64/128 and keeps DPD and BID as physical encodings of the same logical dtype. [ADR 0037](decisions/0037-parameterize-complex-dtype-identity.md) recognizes `tiler::complex@1<ComponentTypeKey>` initially over f16/f32/f64 and keeps planar versus interleaved storage physical. Neither family is registered or evaluated. Wide `i128/u128`, bounded `iN/uN`, fixed-point, UNORM/SNORM, posit, and other numeric candidates remain extensions or reservations under the [dtype taxonomy](research/numerics/mature-dtype-taxonomy.md), not accepted built-ins.

**Trigger.** Decimal, fixed-point, and normalized formats require a named frontend or accelerator consumer. Complex requires a named operation and component type plus branch-cut, exceptional-value, accuracy, storage, ABI, target, and conformance choices. Other reservations require an exact producer and consumer before architectural work.

### Strict-affine U4/F32

**Fact.** [Quantization semantics](../crates/tiler-ir/src/semantic/quantization.rs) registers U4, the strict-affine U4/F32 scheme, and `AssembleQuantized`, `Quantize`, and `Dequantize`; [quantization reference evaluation](../crates/tiler-reference/src/quantization.rs) tests compound roles and shapes, positive finite scale, zero-point range, nearest-even rounding, infinity saturation, NaN refusal, widened subtraction, F32 multiply, and positive zero. [Schedule and kernel IR](../crates/tiler-ir/src/kernel/model.rs) implement the exact whole-component packed-U4 LSB-first, zero-tail dequantization path, and program/artifact tests preserve component roles and structural ABI.

**Fact.** The compiler has no quantized candidate and explicitly refuses this scalar program during region-subject verification. Metal mechanically emits the structured U4 dequantization vocabulary, but a checked honourability test rejects the strict profile because its required F32 subnormal preservation is unavailable. Runtime does not enforce semantic value preconditions or encoded payload validity, and no dtype dispatchability axis exists.

**Trigger.** The selected backend must structurally depend on [runtime semantic enforcement](../tickets/implement-first-runtime-semantic-value-precondition-enforcement.md), [dtype dispatchability](../tickets/admit-a-dtype-dispatchability-capability-axis.md), the exact selected grouping/map and physical-widening tickets, and calibrated costs before making a device-optimal claim.

### Strict-affine U8/F32

**Fact.** U8/F32 has the same registered operations and tested reference semantics as U4/F32 in [quantization semantics](../crates/tiler-ir/src/semantic/quantization.rs) and [quantization reference evaluation](../crates/tiler-reference/src/quantization.rs), including the full byte code domain. It has no schedule, kernel, program, artifact, Metal, runtime, or dispatchability vertical. The U8 carrier spellings used internally by the U4 path do not implement this compound value.

**Trigger.** A selected workload must name the U8 physical representation, component ABI, candidate, cost, target, runtime predicates, and conformance corpus independently of U4.

### Other affine and OCP MX schemes

**Fact.** [ADRs 0029 through 0033](decisions/0029-affine-quantization-parameter-maps.md) establish parameter maps, first-class encoded values, strict affine semantics, and the semantic-validation/physical-enforcement split. Generic encoded-numeric identity, compound reference tensors, and bit-packed structural records are reservations, not support for per-axis, per-block, alternate widths, alternate expressed types, `Requantize`, or integer `Rescale`. [ADR 0038](decisions/0038-recognize-ocp-mx-schemes.md) recognizes six OCP MX 1.0 32-element compound scheme identities; none is registered or implemented.

**Trigger.** Generalized affine work starts only when a selected profile names its exact code/expressed/scale/compute types, parameter map, grouping, storage, ABI, runtime predicates, target, cost, and corpus. MX requires a selected model format and exact constituent scheme; it is not a scalar-dtype widening ticket.

### External and vendor formats

**Fact.** The [semantic registry](../crates/tiler-ir/src/semantic/registry.rs), [reference registry](../crates/tiler-reference/src/registry.rs), compiler capability registries, and opaque artifact type bytes implement provider seams. No bundled external or vendor format is supported. Test-only external F8 and U8 fixtures prove provider identity or transport, not arithmetic, physical ABI, runtime validation, or dispatch.

**Trigger.** A real consumer must publish a stable owner-namespaced identity, immutable descriptor, normative reference, encode/decode vectors, operation set, storage and ABI, runtime refusal rules, target evidence, and versioned conformance. Similar spelling or bit width is never equivalence.

### Execution-only formats

**Fact.** The [dtype taxonomy](research/numerics/mature-dtype-taxonomy.md) keeps TF32, PTX scale encodings, `x86_fp80`, and `ppc_fp128` out of logical built-in identity when they describe an execution mode, target register/ABI format, or interchange spelling rather than an ordinary tensor element contract. Typed target profiles and kernel vocabularies can reserve such facts without exposing them as semantic dtypes.

**Trigger.** Add a physical fact only when a selected backend operation needs it and can state conversion boundaries, delivered numerical behavior, target detection, artifact identity, and refusal. Promote nothing into logical identity without a separate semantic decision.

### Nonnumeric and non-tensor domains

**Fact.** String/bytes, temporal, categorical/dictionary, structured/record, object, and variant values are potential tensor element domains, not numeric kernel scalars. Generic nominal identities reserve representation without defining their operations, storage, lifetime, ABI, verifier, or runtime behavior.

**Trigger.** A named frontend or product workload must require the exact domain and define its operation and lifetime contracts. Numeric dtype breadth does not trigger it.

### Sparse, ragged, and non-tensor values

**Fact.** Sparse and ragged describe container, shape, layout, or value representation, not scalar dtype identity. Tokens, resources, handles, PRNG keys, shapes, indices, tuples, futures, and control values are graph value kinds rather than tensor dtypes. They therefore remain explicitly outside this ledger's dtype axis instead of occupying misleading reservation cells.

**Trigger.** A workload whose representation cannot be expressed by current shape/layout/value contracts requires a separate container or graph-value design; it does not widen the dtype catalog.

## Reproducible negative checks

Run these from the repository root and read the named construction site before changing a cell:

```sh
# Standard semantic construction: F32 plus governed strict-affine U4/U8 are the only shipped dtype registrations.
rg -n 'register_marked_value_type::<|register_value_type\(|bind_marker::<|register_operation\(' crates/tiler-ir/src/semantic/{registry.rs,quantization.rs}

# Shipped reference construction: F32 and strict-affine U4/U8 are the only family-specific providers.
rg -n 'registrar\.register(_value_type)?\(' crates/tiler-reference/src/{standard.rs,quantization.rs}

# No reduced float, decimal, complex, MX, sparse, or ragged construct appears in the physical/reference construction areas.
rg -n 'F16|Bf16|BF16|F64|FP8|FP6|FP4|E8M0|Decimal|Complex|MX|Sparse|Ragged' crates/tiler-ir/src/kernel crates/tiler-ir/src/program crates/tiler-ir/src/schedule/model.rs crates/tiler-reference/src/{standard.rs,quantization.rs} crates/tiler-metal/src/emit.rs

# No strict-affine U8 physical vertical exists in compiler, schedule, kernel, program, artifact, or Metal construction.
rg -n 'StrictAffineU8|U8::resolved_type' crates/tiler-ir/src/{schedule,kernel,program} crates/tiler-artifact/src/program crates/tiler-metal/src crates/tiler-compiler/src

# No dtype-family dispatchability axis exists.
rg -n 'DType|dtype.*dispatch|dispatch.*dtype' crates/tiler-compiler/src/{feasibility.rs,request.rs} crates/tiler-runtime/src

# Runtime and artifact program construction consume no semantic value precondition.
rg -n 'semantic_precondition|SemanticPrecondition' crates/tiler-runtime/src crates/tiler-artifact/src/program
```

Each expected-empty check was run over the named population. Its failure path is demonstrated during this ticket by substituting a known-present spelling before relying on the empty result.

## Graph policy

A cell becomes an implementation ticket only when a named producer and consumer select the exact dtype or scheme, operation set, workload, target, physical layout, numerical contract, runtime predicates, cost boundary, and conformance corpus. Do not file a generic “support all dtypes” ticket.

The accepted built-in catalog, dtype dispatchability, runtime semantic enforcement, selected quantized profile, compound grouping/map, exact physical vocabulary, backend candidate, and cost calibration remain separate authorities. A selected backend profile must make its actual prerequisites structural in the ticket graph; a prose list is not a dependency.
