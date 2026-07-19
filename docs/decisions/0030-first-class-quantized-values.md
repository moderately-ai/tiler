# 0030: Represent quantized tensors as first-class assembled values

**Status:** accepted

## Context

Static parameterized quantized types strongly associate codes with their
numerical interpretation but cannot naturally carry runtime scale, zero-point,
codebook, or hierarchical-scale tensors. Ordinary integer tensors plus QDQ
operands support runtime dataflow but do not intrinsically remain quantized
values through views, graph boundaries, or specialized consumers. Detached
metadata tables bypass ordinary use-def dependencies and are fragile under
rewriting.

Tiler must also support scheme families broader than affine scalar codes. Some
formats use multiple scale levels, codebooks, compound blocks, vector codes, or
multiple payload components. These semantics are distinct from primitive dtype
identity and physical packing.

## Decision

A quantized tensor is one first-class semantic tensor value with a static,
versioned encoded-numeric type contract. The static contract contains:

- a namespaced semantic scheme key and version;
- primitive code and expressed dtypes or value shapes;
- valid code domain and ordered typed component roles;
- one bounded parameter-coordinate map per applicable role;
- normative decode and optional encode/conversion semantics;
- resolved rounding, saturation, exceptional-value, and evaluation rules;
- transformation and operation capability requirements.

It contains no concrete parameter payload and no graph-local value handle.

A dedicated pure `AssembleQuantized`-family semantic operation consumes code
and parameter tensors as ordered graph operands and produces the first-class
quantized tensor. `Quantize` similarly consumes expressed data and parameters
but performs a numeric conversion. `Dequantize`, `Requantize`, native quantized
operations, and explicit component extraction consume the first-class value
under their typed contracts. Constants and runtime parameters use the same
operand mechanism.

Canonical type identity covers the static scheme contract. Canonical program
identity additionally covers the producing operation and canonical identities
of its ordered component producers. Arena IDs, textual names, and registry
order never participate.

Shape and index transformations preserve quantization only when composing the
data transform with every parameter-selection map preserves the exact selected
parameters and yields an admitted, verified mapping. Otherwise the compiler
uses an explicit conversion, a supported richer scheme, or rejection.

The logical value may lower to several physical buffers or ABI bindings. Each
component is identified by a stable role. Physical packing, byte/bit order,
interleaving, alignment, padding, and memory placement remain independent
storage-encoding decisions and cannot redefine scheme semantics.

`QuantSchemeKey`, primitive `TypeKey`, provider revision, and physical
`StorageEncodingKey` are distinct identities. Built-in affine schemes remain
strongly typed and ergonomic over the same versioned extension shape used by
external schemes. Scheme composition is bounded and acyclic; parameter data or
calibration state cannot be hidden inside providers.

## Consequences

- Static and runtime quantization parameters use one graph-dataflow model.
- Cloning, DCE, inlining, serialization, and explanation see component
  dependencies through ordinary operands.
- A quantized tensor survives graph and ABI boundaries as one logical value
  even when its realization uses several bindings.
- Transformations receive an explicit preservation proof obligation instead of
  carrying stale metadata.
- Codebook, hierarchical-scale, block-scaled, and multi-component schemes can
  extend the model without redefining primitive dtypes or affine fields.
- The compiler needs aggregate logical-value lowering and component-role ABI
  validation.

## Alternatives considered

Embedding concrete parameters in nominal dtype identity cannot support runtime
parameters and causes type explosion. Bare integer tensors plus QDQ nodes do
not preserve an intrinsic quantized value contract. Detached annotation tables
are outside normal use-def structure. Generic tuples transport components but
do not encode quantization-specific invariants or diagnostics. Folding packing
into the semantic scheme prevents independent storage planning and can silently
change numerical meaning.
