# Mature tensor dtype taxonomy

**Status:** research inventory; no Tiler support set has been selected  
**Ticket:** `enumerate-the-mature-tensor-dtype-taxonomy`  
**Research date:** 2026-07-19

## Purpose and boundary

This document enumerates numerical and tensor element types that a mature
tensor compiler should recognize, even when it does not implement them. It is a
catalog, not a support promise.

“Dtype” is overloaded across tensor libraries and hardware documentation. The
inventory therefore separates:

```text
LogicalElementType     scalar value/operation semantics and nominal format
NumericInterpretation  quantized, fixed-point, normalized, block-scaled, ...
StorageEncoding        plain, padded, bit-packed, block-packed, interleaved
ComputeFormat          precision/algorithm used after loading
AccumulatorType        precision used for reduction or contraction state
ResultType             observable tensor element type
GraphValueKind         tensor, token, resource, key, opaque handle, ...
```

Backend support is a relation over the operation, all of these type/encoding
roles, memory space, instruction family, target profile, and toolchain version.
A flat `supported_dtypes` set is insufficient.

## Enumerated catalog at a glance

The detailed inventory below recognizes these identities or parameterized
families:

```text
Logical numeric scalars
  bool
  i2 i4 i8 i16 i32 i64 [i128, bounded iN extensions]
  u2 u4 u8 u16 u32 u64 [u128, bounded uN extensions]
  f16 f32 f64 f128
  bf16
  f8E3M4 f8E4M3 f8E4M3FN f8E4M3FNUZ
  f8E4M3B11FNUZ f8E5M2 f8E5M2FNUZ
  f6E2M3FN f6E3M2FN f4E2M1FN
  decimal32 decimal64 decimal128
  complex<f16> complex<f32> complex<f64> [other admitted components]
  positN [reserved extension]

Scale, execution, or target formats rather than ordinary tensor scalars
  f8E8M0FNU scale data
  ue4m3 and ue8m0 target scale-data encodings
  tf32 compute precision
  x86_fp80 ppc_fp128 target ABI formats

Numeric interpretations
  affine quantized: per-tensor, per-axis, per-block
  binary fixed-point, decimal fixed-point, UNORM, SNORM

Storage/encoded tensor families
  bit-packed bool/i2/u2/i4/u4
  MXFP8 MXFP6 MXFP4 MXINT8
  NVFP4
  versioned vendor/project block-quantized extensions

Nonnumeric tensor domains
  string/bytes, object/variant, temporal, structured/record,
  categorical/dictionary

Non-tensor graph values
  token, resource, pointer/handle, typed PRNG key, opaque extension,
  shape/index, tuple/future/control value
```

Bracketed entries are cataloged without implying portable product support.
This list is the reviewed inventory of standardized, multi-ecosystem, or
shipping accelerator formats identified in this research pass as of the
research date. It is not mathematically or historically exhaustive: a versioned
nominal extension mechanism remains necessary for newly identified vendor
formats, arbitrary-width compiler types, and research number systems.

## Maturity labels

- **established portable:** standardized and broadly exchanged by tensor
  systems;
- **established specialized:** mature but limited to particular domains or
  operations;
- **emerging standardized:** specified and shipping, but not general-purpose
  across tensor backends;
- **ecosystem/vendor-specific:** deployed identity that must not be aliased to a
  superficially similar format;
- **reserved/niche:** coherent type with real precedent but weak relevance to
  accelerator tensor arithmetic today.

Maturity does not decide whether Tiler will represent, evaluate, optimize, or
lower a type.

## Logical scalar value types

### Predicate and integers

| Canonical family | Exact identities to recognize | Maturity and notes |
|---|---|---|
| Predicate | `bool` | Established portable. Value semantics are two-valued; storage may be one bit, one byte, or another ABI representation. It is not synonymous with integer arithmetic on `i1`. |
| Signed integer | `i2`, `i4`, `i8`, `i16`, `i32`, `i64` | `i8`–`i64` are established portable. `i2`/`i4` are emerging standardized tensor types and normally require separate packing. |
| Unsigned integer | `u2`, `u4`, `u8`, `u16`, `u32`, `u64` | `u8` is broadly portable. Wider unsigned types are standardized but operation coverage varies; `u2`/`u4` are emerging and commonly packed. |
| Wide integer | `i128`, `u128` | Reserved/specialized. Mature compiler and CPU representations exist, but portable tensor ecosystems and GPU arithmetic generally stop at 64 bits. |
| Arbitrary-width integer | `iN`, `uN` under a bounded extension contract | Compiler precedent exists, but odd widths are not a portable tensor product surface. Useful for dialect extensions and legalization, not evidence of native arithmetic. |

StableHLO admits signed and unsigned widths 2, 4, 8, 16, 32, and 64. ONNX
likewise includes 2- and 4-bit integers. MLIR permits arbitrary widths, while
PyTorch source defines internal/barebones signed and unsigned widths 1 through
7 without documenting them as uniformly supported public arithmetic dtypes.
These precedents support a width-parameterized internal concept plus an
explicitly bounded admitted catalog, not a Rust enum whose variants imply
uniform support.

Primary sources: [StableHLO element types](https://openxla.org/stablehlo/spec#element-types),
[ONNX IR types](https://onnx.ai/onnx/repo-docs/IR.html),
[MLIR integer types](https://mlir.llvm.org/docs/Dialects/Builtin/#integer-type),
[PyTorch tensor attributes](https://docs.pytorch.org/docs/stable/tensor_attributes),
and [PyTorch `ScalarType`](https://github.com/pytorch/pytorch/blob/main/c10/core/ScalarType.h).

Signless compiler integers are intentionally absent from the canonical value
catalog. Signedness affects comparisons, division, overflow interpretation, and
conversion, so Tiler's semantic operation contracts need an unambiguous signed
or unsigned value type. Frontends may translate their own signless IR using
operation-specific interpretation.

Frontend aliases such as `half`, `float`, `double`, `byte`, `short`, `int`,
`long`, `cfloat`, and platform-sized `intp` are never canonical identities.
They resolve to explicit nominal formats and widths before semantic admission;
ABI-dependent `long double` additionally resolves to its actual format rather
than being assumed to mean IEEE binary128.

### Binary and reduced-precision floating point

The exact nominal format is the dtype identity. Bit width or an informal name
such as “FP8” is not sufficient.

| Canonical identity | Layout or defining property | Classification |
|---|---|---|
| `f16` / IEEE `binary16` | sign 1, exponent 5, fraction 10; precision 11 | Established portable logical/storage format and common accelerator operand |
| `f32` / IEEE `binary32` | 1/8/23; precision 24 | Established portable core logical/storage/compute format |
| `f64` / IEEE `binary64` | 1/11/52; precision 53 | Established portable logical/storage format; GPU throughput/support varies |
| `f128` / IEEE `binary128` | 1/15/112; precision 113 | Established standard, specialized software/compiler support, weak GPU tensor support |
| `bf16` / bfloat16 | 1/8/7; precision 8 | Established specialized logical/storage and matrix operand format; accumulation behavior is separate |
| `f8E4M3FN` / OCP E4M3 | 1/4/3, bias 7, finite-only, signed zero, NaN, no infinity | Emerging standardized/shipping FP8 logical element format |
| `f8E5M2` / OCP E5M2 | 1/5/2, bias 15, infinities, NaNs, signed zero | Emerging standardized/shipping FP8 logical element format |
| `f8E3M4` | nominal E3M4 IEEE-convention format | Recognized by StableHLO/MLIR; specialized ecosystem support |
| `f8E4M3` | nominal E4M3 IEEE-convention format distinct from finite-only E4M3FN | Recognized compiler type; must not alias E4M3FN |
| `f8E4M3FNUZ` | finite, NaN, unsigned-zero encoding; bias 8 | Ecosystem/vendor-specific deployed FP8 variant |
| `f8E5M2FNUZ` | finite, NaN, unsigned-zero encoding; bias 16 | Ecosystem/vendor-specific deployed FP8 variant |
| `f8E4M3B11FNUZ` | E4M3 finite/NaN/unsigned-zero with bias 11 | Ecosystem-specific HFP8 variant |
| `f6E2M3FN` | 1/2/3, bias 1, finite-only; no Inf/NaN | Emerging OCP MX element format |
| `f6E3M2FN` | 1/3/2, bias 3, finite-only; no Inf/NaN | Emerging OCP MX element format |
| `f4E2M1FN` | 1/2/1, bias 1, finite-only; no Inf/NaN | Emerging OCP/NVIDIA element format |
| `f8E8M0FNU` | unsigned exponent-only scale value | Emerging scale-data format; not an ordinary signed arithmetic dtype |

IEEE sources define binary16/32/64/128. OCP defines the interoperable E4M3 and
E5M2 FP8 formats and the MX FP4/FP6/scale elements. StableHLO and MLIR preserve
the additional nominal identities above because their infinity, NaN, negative
zero, exponent bias, and range differ.

Primary sources: [IEEE 754-2019](https://standards.ieee.org/ieee/754/6210/),
[OCP OFP8 specification](https://www.opencompute.org/documents/ocp-8-bit-floating-point-specification-ofp8-revision-1-0-2023-12-01-pdf-1),
[OCP MX specification](https://www.opencompute.org/documents/ocp-microscaling-formats-mx-v1-0-spec-final-pdf),
[StableHLO element types](https://openxla.org/stablehlo/spec#element-types),
and [MLIR built-in types](https://mlir.llvm.org/docs/Dialects/Builtin/).

Suffixes are naming conventions, not a universally compositional grammar. The
full nominal format definition is authoritative. In particular, `FN` formats
are finite-only, but whether they encode NaN is format-specific: OCP E4M3FN has
NaN encodings, while the listed OCP FP4/FP6 FN formats have neither NaN nor
infinity. For the listed FP8 variants, `FNUZ` conventionally indicates finite
values, NaN, and unsigned zero, and `B11` identifies exponent bias 11. Tiler
must store the complete nominal identity and its special-value contract rather
than reconstruct it from suffix text.

Primary definitions: [MLIR FP4/FP6 built-in types](https://mlir.llvm.org/docs/Dialects/Builtin/#float4e2m1fntype)
and [StableHLO element types](https://openxla.org/stablehlo/spec#element-types).

### Target ABI and execution-only floating formats

These formats must be recognized without automatically becoming portable tensor
element types.

| Identity | Correct category | Notes |
|---|---|---|
| `tf32` | Compute/operand precision contract | FP32-range input rounded to reduced significand precision for tensor operations, normally with FP32 storage/output and accumulation. StableHLO represents it with limited support, but Tiler should not infer ordinary tensor storage from the name. |
| PTX `.ue4m3` | Target scale-data format | Unsigned 7-bit S0E4M3 scale value stored in a byte with a padded zero MSB; no infinity and one NaN encoding. Distinct from signed `f8E4M3*` logical elements. |
| PTX `.ue8m0` / `f8E8M0FNU` family | Target/portable scale-data format | Unsigned exponent-only scale data used by block-scaled operations, not ordinary signed arithmetic. Backend spelling and exact encoding remain explicit. |
| `x86_fp80` | Target ABI/host extended format | x87 80-bit extended precision; memory padding is ABI-dependent. Not binary128. |
| `ppc_fp128` | Target ABI compound format | IBM/PPC double-double built from two binary64 values. Not IEEE binary128. |

LLVM explicitly distinguishes these target formats from IEEE `fp128`.
NVIDIA documents TF32 as an execution precision for Tensor Core paths.

Primary sources: [LLVM floating-point types](https://llvm.org/docs/LangRef.html#floating-point-types),
[PTX alternate floating-point formats](https://docs.nvidia.com/cuda/parallel-thread-execution/#alternate-floating-point-data-formats),
and [NVIDIA TensorRT accuracy considerations](https://docs.nvidia.com/deeplearning/tensorrt/latest/inference-library/accuracy-considerations.html).

### Decimal floating point

| Identity | Defining precision | Classification |
|---|---|---|
| IEEE `decimal32` | 7 decimal digits | Established standard; niche tensor/GPU use |
| IEEE `decimal64` | 16 decimal digits | Established standard; enterprise/CPU specialization |
| IEEE `decimal128` | 34 decimal digits | Established standard; enterprise/CPU specialization |

IEEE permits densely packed decimal and binary-integer-decimal encodings for
the same logical decimal formats, so storage encoding must remain explicit.
These types are reasonable extension/reservation candidates for a mature type
system, but current GPU tensor arithmetic does not justify silently treating
them as core binary-float variants.

Primary sources: [IEEE 754-2019](https://standards.ieee.org/ieee/754/6210/)
and [GCC decimal floating types](https://gcc.gnu.org/onlinedocs/gcc/Decimal-Float.html).

### Complex scalars

Complex is a compound logical scalar parameterized by a component floating
format, not a new exponent/significand encoding.

| Ecosystem spelling requiring frontend-specific resolution | Canonical structural identity | Maturity |
|---|---|---|
| PyTorch `complex32` / `chalf` | `complex<f16>`; 32 total bits | Emerging/specialized |
| Tensor `complex64` / `cfloat` conventions | `complex<f32>`; 64 total bits | Established portable |
| Tensor `complex128` / `cdouble` conventions | `complex<f64>`; 128 total bits | Established portable |
| `complex<bf16>` and other components | structurally coherent extension | Ecosystem-specific/reserved until operations are defined |
| Ecosystem-specific `complex256` spelling | `complex<f128>`; two IEEE binary128 components | Reserved/specialized software type |

StableHLO currently admits complex f32/f64; PyTorch additionally exposes
complex f16. DLPack can describe complex f16 and complex bf16, and MLIR's
structural complex type can contain integer or floating components. Those are
representation precedents, not evidence that Tiler should assign arithmetic
semantics to every structurally possible component. Canonical names must never
use ambiguous shorthand. Tensor ecosystems commonly use `complex64` for 64
total bits, while other language libraries use `Complex64` to mean components
of f64. Only `complex<component-format>` is canonical Tiler spelling; aliases
resolve in the owning frontend.

Primary sources: [StableHLO element types](https://openxla.org/stablehlo/spec#element-types)
[PyTorch complex numbers](https://docs.pytorch.org/docs/stable/complex_numbers.html),
[DLPack C API](https://dmlc.github.io/dlpack/latest/c_api.html), and
[MLIR complex type](https://mlir.llvm.org/docs/Dialects/Builtin/#complex-type).

### Posit and other tapered formats

The Posit Standard (2022) defines `positN` and corresponding exact-accumulation
`quireN` formats. It is a coherent numerical standard but remains experimental
in mainstream compiler, GPU, and tensor ecosystems. Older research notation
`posit<n, es>` is not automatically the same contract as the ratified standard.

Tiler should catalog posit as a reserved extension family rather than collapse
it into binary float parameters. Similar experimental logarithmic-number,
unum, rational, and arbitrary-precision families require named extension
semantics and are outside the finite mature catalog until an ecosystem or
target use case appears.

Primary source: [Posit Standard 2022](https://posithub.org/docs/posit_standard-2.pdf).

## Numeric interpretations over scalar storage

These are not adequately identified by a primitive storage dtype.

### Affine quantization

A mature quantized tensor contract may include:

```text
AffineQuantized {
    storage_integer,
    expressed_float,
    storage_min,
    storage_max,
    parameters: Static(scales, zero_points) | GraphValues(scale_ids, zero_point_ids),
    granularity: PerTensor | PerAxis(axis) | PerBlock(mapping),
    rounding,
    overflow_or_saturation,
}
```

Symmetric quantization is a constrained affine case, not a sufficient universal
model. Dynamic quantization describes when parameters are computed and applied;
it is an execution scheme, not another scalar dtype. Names such as `qint8` omit
too much information to be canonical semantic identity.

StableHLO quantized types carry storage and expressed types, ranges, scales,
zero points, and optional axis information. MLIR supports per-layer, per-axis,
and blockwise forms. ONNX represents scale/zero-point/granularity through
operators and tensors rather than a self-contained `qint8` scalar.

The structure above is a Tiler design requirement, stronger than any one
precedent. Some StableHLO storage-range/zero-point details remain under
discussion, and MLIR's quantize/dequantize casts intentionally leave rounding
realization to lowering. Tiler must resolve rounding and saturation before
optimization instead of inheriting those ambiguities.

Primary sources: [StableHLO quantized tensor types](https://openxla.org/stablehlo/spec#quantized-tensor-types),
[MLIR Quant dialect](https://mlir.llvm.org/docs/Dialects/QuantDialect/), and
[ONNX `QuantizeLinear`](https://onnx.ai/onnx/operators/onnx__QuantizeLinear.html).

### Fixed-point, normalized integer, and decimal fixed-point

These require separate parameterized interpretations:

- binary fixed-point: signedness, total/integer/fraction bits, radix point,
  rounding, overflow, wrap/saturation;
- decimal fixed-point: precision, scale, storage width, rounding, overflow;
- normalized integer: `UNORM`/`SNORM` mapping from integer storage into a
  bounded real interval, including endpoint behavior.

They are not equivalent to affine ML quantization merely because each can be
implemented with integer storage and a scale. Fixed-point remains
reserved/specialized for Tiler taxonomy; decimal32/64/128/256 fixed-point is
mature in Arrow but not general GPU arithmetic.

Primary source: [Arrow columnar types](https://arrow.apache.org/docs/format/Columnar.html).

## Packed and block-scaled encoded tensors

Packing can be a pure storage choice. Block scaling is broader: shared scales
change the represented numerical values, so MX/NVFP identities combine a
`NumericInterpretation` with a `StorageEncoding` rather than belonging to
storage alone.

### Bit-packed scalar storage

Logical `bool`, `i2/u2`, and `i4/u4` do not imply a physical packing. Examples
include:

```text
BitPacked {
    element,
    bits_per_element,
    bit_order,
    byte_order,
    row_or_block_alignment,
    padding,
}
```

ONNX specifies LSB-first packing for its int2/int4 tensors; DLPack describes
sub-byte packing and can separately flag padded storage. Other runtimes may use
byte-padded shell types. Shape, offset, and stride legality differs among these
encodings.

Primary sources: [ONNX int4](https://onnx.ai/onnx/technical/int4.html),
[ONNX int2](https://onnx.ai/onnx/technical/int2.html), and
[DLPack C API](https://dmlc.github.io/dlpack/latest/c_api.html).

### OCP microscaling formats

The following names describe blocks, not primitive scalar dtypes:

| Encoded family | Element choices | Scale/granularity | Classification |
|---|---|---|---|
| `MXFP8` | OCP FP8 elements | shared E8M0 scale per 32 elements | Emerging standardized |
| `MXFP6` | E2M3 or E3M2 elements | shared E8M0 scale per 32 | Emerging standardized |
| `MXFP4` | E2M1 elements | shared E8M0 scale per 32 | Emerging standardized |
| `MXINT8` | signed int8 elements | shared E8M0 scale per 32 | Emerging standardized |

The block axis, grouping, scale location, scaling/rounding/saturation rules,
and physical packing participate in identity. `MXFP4` is not an alias for a
tensor of independent `f4E2M1FN` values.

Primary source: [OCP MX specification](https://www.opencompute.org/documents/ocp-microscaling-formats-mx-v1-0-spec-final-pdf).

### NVIDIA NVFP4

NVFP4 is a distinct vendor block-scaled recipe: E2M1 elements use a conceptual
FP8 E4M3 local scale per 16 values plus an FP32 tensor-global scale; supported
weight layouts may add two-dimensional scaling. Concrete backends may encode
the local scale through a specialized unsigned format such as PTX `.ue4m3`, so
the storage/ABI contract cannot assume one universal signed E4M3 byte. NVFP4 is
not OCP MXFP4, whose group size and scale format differ.

Primary source: [NVIDIA Transformer Engine NVFP4](https://docs.nvidia.com/deeplearning/transformer-engine/user-guide/features/low_precision_training/nvfp4/nvfp4.html).

Other weight-only formats used by model runtimes—grouped int8/int4/int2,
binary/ternary weights, codebook/palette quantization, and project-specific
formats such as GGML-family blocks—belong in a versioned encoded-tensor
extension catalog. Their marketing names are not portable scalar dtypes.

## Nonnumeric tensor element domains

Mature array and interchange ecosystems also call these dtypes:

| Domain | Examples | Tiler classification |
|---|---|---|
| String/bytes | UTF-8 string tensors, fixed/variable byte strings | Genuine tensor element domain in systems such as ONNX, but requires offsets/buffers and a separate operation family; not numeric kernel scalar IR |
| Object/variant | host objects, tagged variants | Runtime-managed/opaque domain, not portable device arithmetic |
| Temporal | date, time, duration/timedelta | Parameterized semantic domain over integer storage; separate operation family |
| Structured/record | named fields, subarrays | Compound schema/layout, not one scalar arithmetic dtype |
| Decimal fixed-point | precision/scale decimals | Numeric domain described above; distinct from IEEE decimal floating point |
| Categorical/dictionary | integer codes plus dictionary | Encoded/relational domain, not primitive integer semantics |

ONNX supports string tensors; NumPy and Arrow expose the broader catalog.
Recognizing these types for future frontends does not require admitting them to
the initial tensor-kernel optimizer.

Primary sources: [ONNX IR](https://onnx.ai/onnx/repo-docs/IR.html),
[NumPy dtype classes](https://numpy.org/doc/stable/reference/routines.dtypes.html),
and [Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html).

## Values that are not tensor element dtypes

The broader graph type system may eventually contain:

- effect/order tokens with no runtime data payload;
- resources, device handles, and pointers with runtime identity and lifetime;
- typed PRNG keys;
- opaque extension handles;
- shape/index values;
- tuples, futures, and control values.

These must not enter `Tensor<LogicalElementType>` merely because another system
calls them a dtype. JAX typed PRNG keys deliberately reject ordinary arithmetic;
StableHLO tokens impose execution order; DLPack opaque handles require producer
and consumer agreement. Shape/index integers also remain distinct newtypes even
when their physical representation is i64.

Primary sources: [JAX typed keys](https://docs.jax.dev/en/latest/jep/9263-typed-keys.html),
[StableHLO token type](https://openxla.org/stablehlo/spec#token-type), and
[DLPack C API](https://dmlc.github.io/dlpack/latest/c_api.html).

## Cross-system inventory snapshot

| System | Particularly useful boundary |
|---|---|
| StableHLO | Broad nominal float/sub-byte catalog; quantized storage and expressed types; consumers need not support every type/algorithm |
| MLIR | Arbitrary representability is separate from dialect legality and target legalization |
| ONNX | Portable exchange set plus explicit sub-byte packing and operator-level quantization |
| PyTorch | “Shell dtype” proves representation/view support can precede arithmetic coverage |
| JAX | Extended key dtypes and `float0` show that a library dtype need not be a numeric tensor scalar |
| NumPy | Host array domains include strings, objects, temporal, structured, and ABI-dependent types |
| DLPack | ABI `(code,bits,lanes)` and packing describe exchange representation, not operation semantics |
| Arrow | Each data type prescribes a physical layout; extension types add semantics over a storage type rather than forming a universal tensor arithmetic type system |
| Metal | General scalar arithmetic differs from tensor-resource element formats and specialized-operation support |
| CUDA/PTX | FP8/FP6/FP4 and sub-byte integers may be conversion/MMA operands without general scalar arithmetic |
| SPIR-V | Storage capabilities, arithmetic capabilities, and packed-dot capabilities are explicitly separate |
| WGSL | Narrow portable shader arithmetic set (`i32/u32/f32`, optional f16) demonstrates backend independence |

Backend primary sources: [Metal capabilities](https://developer.apple.com/metal/capabilities/),
[Metal Shading Language specification](https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf),
[CUDA PTX ISA](https://docs.nvidia.com/cuda/parallel-thread-execution/index.html),
[SPIR-V specification](https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html),
and [WGSL](https://www.w3.org/TR/WGSL/).

## Capability levels to decide later

Every catalog entry will eventually receive support facts forming a partial
order of prerequisites rather than one Boolean or wholly independent flags:

```text
Recognized          // parser/diagnostics know the nominal identity
Representable       // canonical IR and serialization can carry it
LiteralSupported
OperationSemanticsDefined(op, numerical_policy)
ReferenceEvaluable(op, numerical_policy)
ConversionSupported(from, to, mode)
Optimizable(op, numerical_policy)
StorageSupported(target, encoding, memory_space)
LayoutViewSupported(encoding, transform)
AbiOrInterchangeSupported(boundary)
ScalarNative(target, op)
VectorNative(target, op)
MatrixOperandNative(target, op, algorithm)
WidenedOrEmulated(target, op, realization)
Lowerable(target, op, realization, numerical_policy)
ProductProfileEnabled(profile, op, use_role)
```

Unknown remains distinct from unsupported. Later capabilities require relevant
earlier semantics/representation facts but do not follow from them:
representable FP8 may be matrix-operand native, storage-only for an elementwise
operation, and widened/emulated for another.

**Accepted by Tom on 2026-07-19:** representable element types may be broader
than executable operation support. A recognized exact type can participate in
an operation only when that operation explicitly admits its full typed
signature. Representability never implies reference evaluation, optimizer, or
backend support.

## Inventory conclusions, not support decisions

1. Tiler needs nominal float format identities; `{bits, exponent, mantissa}` is
   insufficient because special encodings and biases differ.
2. Integer width should be an explicit bounded newtype, with signedness
   separate and canonical; odd-width extension does not imply admission.
3. Predicate is a logical scalar independent of physical bit/byte packing.
4. Complex is structural over an explicitly admitted component dtype.
5. Quantization and fixed-point are numeric interpretations, not aliases for
   their integer storage.
6. Packing and block scaling belong to storage/encoding contracts with layout
   and metadata.
7. TF32 and similar reduced product precision belong to operation/physical
   compute contracts unless an explicit tensor interchange use case proves
   otherwise.
8. Nonnumeric array domains and non-data graph values need explicit extension
   boundaries outside numeric tensor scalar IR.
9. The target-independent catalog must be broader than every backend's native
   subset, while each product profile remains vertically constrained.

The next phase is a review of omissions and naming, followed by explicit
selection of Tiler's recognized, representable, reference, optimizer, backend,
and first-product-profile sets. This document intentionally makes none of those
selections.
