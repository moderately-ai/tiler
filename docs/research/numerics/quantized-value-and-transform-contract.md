# Quantized value and transformation contract

**Status:** researched contract supporting ADR 0030  
**Reviewed:** 2026-07-19

## Resolved carrier

Static quantization schema belongs to a tensor's semantic type contract.
Concrete parameter values belong to graph dataflow. A dedicated semantic
operation assembles those components into one first-class quantized tensor
value:

```text
codes, scale, optional_zero_point
    -> AssembleQuantized<affine u4 -> f32, per_axis(1)>
    -> QuantizedTensor
```

The static contract contains no graph-local `ValueId` and no concrete scale or
zero-point payload. It names the scheme, primitive code and expressed dtypes,
ordered parameter roles, code range, parameter index maps, and resolved
numerical behavior. The operation operands supply the actual tensors.

`AssembleQuantized` associates already existing codes and parameters without
performing numeric conversion. It is distinct from:

- `Quantize`, which computes codes from expressed values under a conversion
  contract;
- `Dequantize`, which computes expressed values;
- `Requantize`, which changes a quantized interpretation;
- a component-extraction operation, which exposes codes or parameters without
  pretending they are the same semantic tensor value;
- physical bit packing, which belongs to storage lowering.

Constants and runtime parameters use the same model. A static scale is an
ordinary `Constant` operand; a runtime scale is an input or computed tensor.
Both participate in normal use-def structure, canonical graph identity,
verification, dead-code analysis, and explanation.

## Why this carrier

MLIR Quant and StableHLO parameterized types strongly associate static
parameters with quantized values but cannot naturally carry runtime SSA
parameters. ONNX QDQ operations support dynamic parameter operands but leave
the produced integer tensor without an intrinsic reusable quantized value
contract. ONNX's detached annotation table is serializable but is outside
ordinary operand use-def dependencies.

MLIR Sparse Tensor provides the closest structural precedent. A static encoding
describes the logical sparse tensor type, while `sparse_tensor.assemble`
consumes runtime position, coordinate, and value tensors and produces one
first-class sparse tensor. Tiler applies that pattern to encoded numeric tensor
values, with stricter host verification of component integrity.

Primary sources: [MLIR Quant dialect](https://mlir.llvm.org/docs/Dialects/QuantDialect/),
[StableHLO quantized types](https://openxla.org/stablehlo/spec#types),
[ONNX `QuantizeLinear`](https://onnx.ai/onnx/operators/onnx__QuantizeLinear.html),
[ONNX `TensorAnnotation`](https://github.com/onnx/onnx/blob/main/onnx/onnx.proto),
and [MLIR Sparse Tensor `assemble`](https://mlir.llvm.org/docs/Dialects/SparseTensorOps/#sparse_tensorassemble-sparse_tensorassembleop).

## Identity and verification

Canonical type identity includes only the static schema. Canonical program
identity additionally includes the assembly or conversion operation and the
canonical identities of its ordered component producers. Arena IDs, textual
SSA names, interface display names, and registry insertion order do not enter
semantic identity.

Verification is layered:

1. The type verifier validates the scheme key/version, dtype roles, code range,
   parameter-role schema, and bounded coordinate maps.
2. The assembly/conversion verifier validates component arity, kinds, dtypes,
   ranks, and shape relationships.
3. The constraint solver proves symbolic parameter-grid relationships where it
   can.
4. Residual runtime guards validate unresolved shape and value-domain facts,
   such as positive finite scales.
5. Each consumer proves support for the complete quantized typed signature.

Semantic validity is not a plan guard. An invalid runtime scale or mismatched
component shape is an invalid program binding, not permission to execute a
different meaning.

## Transformation invariant

A shape or index operation preserves a quantized value exactly when every
result element continues to select the same parameter or codebook entry as the
corresponding source element.

Let `D` map result coordinates to source data coordinates and let `Q` map source
data coordinates to parameter coordinates. The transformed interpretation is
valid only if `Q ∘ D` can be represented and verified by an admitted target
parameter map. The operation must transform the logical tensor and parameter
components together.

Consequences include:

- transpose remaps axes and transposes full-rank parameter grids;
- slicing a quantized axis slices its parameter tensor; an unaligned block
  slice may destroy regular block membership;
- reshape is legal only when it preserves quantization-group membership;
- broadcasting a singleton quantized dimension repeats its parameter;
- arbitrary gather or reindex may produce an irregular map and require a more
  general scheme, explicit requantization/dequantization, or rejection;
- concatenation either concatenates compatible parameter arrays, proves equal
  parameters on unaffected regions, promotes to a representable richer map, or
  requires conversion.

StableHLO provides useful conservative precedent: it remaps per-axis
quantization through transpose and broadcast, constrains reshape by group
membership, and initially restricts several harder operations to per-tensor
quantization. Tiler can begin conservatively and admit more cases by proving
the composed map; the semantic rule does not change.

Primary source: [StableHLO operation specification](https://openxla.org/stablehlo/spec).

Semantic legality does not imply a zero-copy physical view. A transpose may
preserve quantized meaning while requiring unpacking or repacking for a chosen
storage encoding.

## Scheme extension boundary

The core abstraction must not assume affine `scale + optional zero point`, one
parameter level, or one scalar code decoding to one scalar value. Mature
families include:

- uniform affine integer codes;
- scaled FP8/FP4 values;
- OCP MX block-scaled formats;
- hierarchical local and global scaling;
- fixed or learned scalar codebooks such as NF4;
- binary and ternary alphabets;
- nested codecs whose scales are themselves encoded;
- vector/product codebooks;
- hybrid outlier decompositions with multiple payload components.

The semantic extension contract therefore reserves:

- a namespaced, versioned `QuantSchemeKey`, separate from primitive `TypeKey`
  and physical storage-encoding identity;
- code domain or alphabet, expressed value shape/type, invalid codes, and
  normative decode semantics;
- ordered typed component roles, including multiple scale levels, codebooks,
  offsets, masks, or outlier payloads;
- an independent coordinate map for every parameter role;
- optional encode semantics distinct from decode-only representability;
- exact evaluation precision, rounding, exceptional-value, and saturation
  behavior;
- bounded acyclic composition for encoded metadata;
- transformation capabilities and per-operation typed support;
- host-owned canonical serialization, validation, hashing, reference vectors,
  and explanation.

Physical encoding separately defines component buffers, packing, bit order,
interleaving, alignment, padding, scale layout, memory space, and ABI access.
It links to a compatible semantic scheme but cannot redefine decode meaning.

The OCP MX specification illustrates why this split matters: an MX value is a
block of low-precision elements sharing an E8M0 scale, and a NaN scale gives
block-wide semantics. It is a compound numerical value, not an alias for an FP4
or FP8 scalar and not merely a nibble-packing convention.

Primary source: [OCP Microscaling Formats specification](https://www.opencompute.org/documents/ocp-microscaling-formats-mx-v1-0-spec-final-pdf).
