# Affine quantization numerical semantics

**Status:** strict baseline accepted

**Reviewed:** 2026-07-19

## Why the formula is insufficient

The familiar relationship

```text
expressed = (code - zero_point) * scale
```

does not completely define a portable operation. Exact semantics also require
the code range, parameter map, intermediate dtype and evaluation order,
rounding, saturation, NaN and infinity behavior, signed-zero and subnormal
rules, and the distinction between logical requantization and an integer
rescaling algorithm.

## Primary precedents

### StableHLO

StableHLO provides the strongest nearly complete baseline. It requires positive
finite scales and in-range storage bounds and zero points. Quantization divides
in the expressed type, adds the converted zero point, clamps in floating point,
rounds to nearest with ties to even, and converts to storage. Requantization is
normatively source dequantization followed by destination quantization.

Important gaps remain. Subtraction occurs in the storage integer type and can
overflow for some unsigned domains. NaN eventually reaches an incompletely
specified float-to-integer conversion. Subnormal handling is not explicit.

Primary source: [StableHLO specification](https://openxla.org/stablehlo/spec#uniform_quantize).

### MLIR Quant

MLIR supplies strong structural types and operations but explicitly delegates
rounding and several arithmetic details to lowering. Its dequantization converts
code and zero point separately before subtraction, while its quantization
converts to integer before clamping. It is useful IR precedent but not a
portable-bitwise numerical contract.

Primary source: [MLIR Quant dialect](https://mlir.llvm.org/docs/Dialects/QuantDialect/).

### ONNX

ONNX `QuantizeLinear` specifies nearest-even rounding, saturation, low-bit code
ranges, and a selectable calculation precision. `DequantizeLinear` uses the
output/scale dtype for multiplication. It does not fully specify scale validity,
NaN, subnormal behavior, signed zero, or the exact subtraction domain.

Primary sources: [ONNX `QuantizeLinear`](https://onnx.ai/onnx/operators/onnx__QuantizeLinear.html)
and [ONNX `DequantizeLinear`](https://onnx.ai/onnx/operators/onnx__DequantizeLinear.html).

### TOSA integer rescaling

TOSA treats quantized integer operators as exact integer computations followed
by explicit `RESCALE`. `RESCALE` subtracts the input zero point in a wide
intermediate, applies a specified multiplier/shift and single- or double-round
integer algorithm, adds the output zero point, clips, and casts. Multiplier
width, shift range, intermediate range, signedness, and rounding algorithm are
observable semantics.

This is not generally bit-equivalent to decode-then-encode requantization. The
rational multiplier may approximate the scale ratio, and its integer rounding
and intermediate widths can produce different results.

Primary source: [TOSA 1.0.1 specification](https://www.mlplatform.org/tosa/tosa_spec_1_0_1.html),
sections 1.12 and 2.13.2.

## Derived initial affine baseline

The following choices follow current Tiler invariants and mature precedent:

- scale is positive and finite;
- zero point and logical codes are within the declared logical code range;
- code and zero point are widened to a signed difference type that cannot
  overflow before subtraction;
- the integer difference is converted once to an explicit computation dtype;
- multiplication, division, and any result conversion have fixed evaluation
  order and named dtypes;
- encoding rounds to nearest with ties to even;
- finite out-of-range results and positive/negative infinity saturate to the
  declared code endpoints;
- integer encoding cannot preserve an input signed zero; decoding `code ==
  zero_point` produces the contract's canonical positive zero;
- subnormal input, scale, intermediate, and output behavior is explicit and
  never inherited from a backend default; the strict family preserves
  subnormals at every named boundary;
- every contract field participates in semantic, explanation, plan, and
  artifact identity.

The accepted strict evaluation form is conceptually:

```text
decode(code, scale, zero_point):
    difference = widen_signed(code) - widen_signed(zero_point)
    return convert<compute>(difference) * convert<compute>(scale)

encode(value, scale, zero_point):
    scaled = convert<compute>(value) / convert<compute>(scale)
    shifted = scaled + convert<compute>(zero_point)
    require_not_nan(shifted)
    clamped = clamp(shifted, convert<compute>(qmin), convert<compute>(qmax))
    rounded = round_ties_even(clamped)
    return exact_convert<code>(rounded)
```

The computation dtype is a resolved field rather than inferred from the
backend. Conversion into it and any conversion from it are separate typed
boundaries. Strict decoding of `code == zero_point` produces canonical positive
zero; integer encoding collapses both input zero signs to the zero-point code.
Positive and negative infinity clamp to the upper and lower endpoints
respectively.

## Requantize and integer Rescale are distinct

Logical affine `Requantize` means decode under the source scheme into an
explicit intermediate dtype, then encode under the destination scheme. The two
rounding boundaries are preserved.

An integer `Rescale` is a separate specialized semantic family naming its
multiplier width, multiplier and shift operands, integer rounding algorithm,
signedness interpretation, zero-point rules, clip range, and intermediate
widths. An optimizer or backend may replace `Requantize` with `Rescale` only
when it proves equivalence for the reachable domain under the resolved
contract. Imported TOSA `RESCALE` retains its source semantics rather than being
reinterpreted as ideal affine requantization.

## Validation layers

| Layer | Responsibility | Failure classification |
|---|---|---|
| Static type | Scheme/version, dtypes, code range, role schema, maps, complete numerical contract | Invalid semantic type |
| Static operation | Component arity/dtypes/shapes, constant parameter domains, map totality | Invalid graph |
| Constraint solver | Symbolic grid sizes, block counts, map bounds | Contradiction rejects; unknown becomes runtime semantic obligation |
| Runtime semantic binding | Dynamic grids, scale domain, code/zero-point range, required components | Invalid program input; validate before dependent work |
| Physical plan | Backend support, layout, packing, alignment, fast-path facts | Plan inapplicable; choose an equivalent plan or fallback |
| Artifact/ABI | Stable logical-value/component roles, widths, encodings, accessible ranges | Corrupt or incompatible artifact/binding |
| Reference evaluator | Normative evaluation order, parameter selection, exceptional cases | Authoritative semantic oracle |

Semantic invalidity never becomes a plan guard. Conversely, lack of backend
support for a valid per-axis or per-block scheme does not make the graph
invalid.

## Accepted NaN policy

The mature precedents do not define one portable behavior for a NaN input to
integer affine quantization.

**Accepted by Tom on 2026-07-19:** the initial strict affine `Quantize`
conversion rejects NaN as invalid semantic input. It does not silently map NaN
to the zero point or an endpoint. A frontend or extension may select a separate,
explicitly named conversion family such as `NaNToZeroPoint` or a scheme-defined
reserved-code mapping; that choice participates in semantic and artifact
identity.

For dynamic values, NaN absence must be proven or validated before the strict
conversion can successfully commit an observable result. Failure is a semantic
input error, not a physical-plan miss and not permission to retry with a
different mapping. The exact validation execution strategy remains a runtime
design question constrained by the no-hidden-semantic-change and no-unsafe-
partial-fallback rules.

These strict evaluation choices are recorded durably in ADR 0032. Other
computation dtypes, subnormal policies, rounding modes, or exceptional mappings
are separate typed conversion families rather than backend discretion.
