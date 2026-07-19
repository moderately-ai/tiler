# Quantization representation in tensor IRs

**Status:** researched recommendation; representation decision remains open  
**Reviewed:** 2026-07-19

## Question

Should Tiler represent affine quantization as:

1. a parameterized dtype;
2. ordinary integer tensors plus explicit quantize/dequantize operations; or
3. a first-class quantized value interpretation plus explicit boundary
   operations?

This is separate from recognizing `i2`, `u2`, `i4`, and `u4`. Those are integer
value types under ADR 0028, not complete quantized tensor contracts.

## Facts from primary precedents

### MLIR Quant uses types and operations together

MLIR's `!quant.uniform` type relates a stored integer type to an expressed
floating-point type. It includes the permitted stored range, scale, zero point,
and per-layer, per-axis, or blockwise granularity. The verifier checks the
containing tensor rank, axis, block sizes, divisibility, and parameter-grid
shape.

MLIR also uses explicit operations:

- `quant.qcast` converts an expressed value to a quantized value;
- `quant.dcast` converts a quantized value to its expressed value;
- `quant.scast` exposes or restores the integer storage bits without a numeric
  conversion.

This makes a plain integer tensor and a quantized tensor observably different.
MLIR embeds constant parameters in the type, however, and deliberately leaves
some conversion-rounding choices to lowering. Tiler cannot inherit that latter
ambiguity because ADR 0010 requires a resolved conversion contract before
optimization.

Primary source: [MLIR Quant dialect](https://mlir.llvm.org/docs/Dialects/QuantDialect/).

### StableHLO also supports typed quantized values and QDQ alternatives

StableHLO quantized element types contain storage and expressed types, storage
ranges, positive finite scales, zero points, and optional quantization axes.
`uniform_quantize` covers quantization and requantization;
`uniform_dequantize` converts to the expressed type.

StableHLO has official transformations in both directions between a
dequantize–floating-operation–quantize pattern and a native quantized
operation. QDQ form and quantized-native operations are therefore complementary
logical alternatives rather than mutually exclusive product models.

StableHLO's parameters are currently constants in the type. Its model is good
precedent for strong value typing, but it is not sufficient for runtime-selected
scales and zero points.

Primary sources: [StableHLO quantized types and operations](https://openxla.org/stablehlo/spec#types)
and [StableHLO QDQ transformation passes](https://openxla.org/stablehlo/generated/stablehlo_passes).

### ONNX makes parameters graph operands

ONNX `QuantizeLinear` and `DequantizeLinear` keep the code tensor's ordinary
integer or low-precision dtype and receive scale and zero point as graph inputs.
Parameter shapes plus axis and block-size attributes distinguish per-tensor,
per-axis, and blocked quantization. The operation contract also defines
round-to-nearest-even and saturation behavior.

This naturally represents constants, runtime parameters, and data-dependent
quantization, but a bare integer edge does not retain a quantized numerical
interpretation on its own.

Primary sources: [ONNX `QuantizeLinear`](https://onnx.ai/onnx/operators/onnx__QuantizeLinear.html)
and [ONNX `DequantizeLinear`](https://onnx.ai/onnx/operators/onnx__DequantizeLinear.html).

### TOSA exposes the integer-domain realization

TOSA treats quantized computation as integer computation with explicit zero
points and `RESCALE` operations whenever scales change. This is useful evidence
that optimized quantized execution cannot always be modeled as merely
materializing float QDQ boundaries. Exact integer multiplication, shifts,
rounding, saturation, accumulator width, and rescaling remain semantic.

Primary source: [TOSA 1.0.1 specification](https://www.mlplatform.org/tosa/tosa_spec_1_0_1.html),
sections 1.12 and 2.13.2.

## Compatibility with accepted Tiler decisions

No accepted ADR currently chooses the quantization carrier. The following
decisions constrain it:

| Existing decision | Consequence for quantization |
|---|---|
| ADR 0009 | Quantization boundaries remain observable even when fusion removes materialization. |
| ADR 0010 | Quantization is a specialized typed conversion family with resolved rounding and overflow behavior. |
| ADR 0013 | Runtime parameter bindings are observable inputs to scoped determinism. |
| ADR 0021 | Optimizations using scale-domain or value assumptions need proof or runtime validation. |
| ADR 0026 | Integer recognition does not imply quantized-operation, evaluator, storage, or backend support. |
| ADR 0027 | Scale/zero-point combinations must not accidentally mint new nominal scalar `TypeKey` identities. |
| ADR 0028 | Low-bit integer types are code-value types; packing and quantized interpretation are separate. |

The current graph proposal also separates operands from attributes. Runtime
scale, zero-point, and codebook tensors must be ordered graph operands. They
cannot be stored as graph-local `ValueId`s inside canonical attributes.
Compile-time parameter tensors can use the same operand roles and be supplied
by `Constant` values. Axis mappings, block rules, rounding modes, and other
small semantic constants can remain canonical attributes.

## Inference

The two original options are a false binary.

- A parameterized dtype strongly associates codes with their numerical
  interpretation, but embedding every parameter value in scalar type identity
  is unsuitable for runtime parameters and creates a type explosion.
- QDQ operations with ordinary integer edges naturally support dynamic
  parameters, but an operation-only association is too weak for a value that
  crosses views or enters a native quantized operation. A bare `u4` value means
  integer codes, not encoded real values.
- Mature compiler precedent therefore supports a hybrid: a quantized tensor
  has a first-class semantic interpretation, and explicit conversion or rescale
  operations expose boundaries and changes in that interpretation.

## Proposed direction

Tiler should keep three concepts distinct:

```text
logical code element type       u4
quantized value interpretation  affine f32, per-block, parameter roles, conversion contract
physical storage encoding       packed nibbles, order, padding, alignment, memory space
```

The first-class interpretation should describe at least:

- scheme family, initially affine;
- code and expressed dtypes;
- valid code range;
- per-tensor, per-axis, or per-block mapping;
- scale and optional zero-point operand roles;
- rounding, saturation/overflow, and exceptional-value behavior;
- constraints over tensor and parameter shapes and values.

`Quantize`, `Dequantize`, and `Requantize` remain explicit semantic operations.
Native quantized contraction, convolution, rescale, or other operations may be
specialized logical nodes or derived alternatives when their exact accumulator,
rounding, and saturation contracts are known. Rewrites between QDQ and native
forms require those contracts to match; backend convenience is not proof of
equivalence.

This direction is compatible with current ADRs but is not yet accepted. In
particular, it does not decide the exact ownership model described below.

## Remaining architectural decision

The unresolved issue is how a value's first-class quantization interpretation
owns or references its parameter bindings.

A tensor type cannot simply contain graph-local `ValueId`s: those handles are
not durable identities, and doing so would blur the current type/operand
boundary. A detached specification is also insufficient because transformations
could lose the association. Candidate designs include a graph-owned
interpretation instance referenced by the value, a dependent value contract
whose bindings are operation result roles, or another host-owned relation with
canonical graph-topology identity.

The chosen model must prove:

- scale and zero-point bindings cannot become detached from their code value;
- constants and runtime tensors use one typed operand-role model;
- per-axis and block mappings remain valid through transpose, slice, reshape,
  broadcast, and arbitrary reindex operations;
- static facts hash as semantic constants while runtime bindings hash through
  canonical graph/interface identity;
- quantized constants, reference evaluation, serialization, ABI binding, and
  extension schemas remain deterministic;
- unsupported transformations reject or insert an explicit semantic
  conversion rather than silently changing the interpretation.

This carrier/ownership question should be resolved before accepting a
quantization IR ADR.
