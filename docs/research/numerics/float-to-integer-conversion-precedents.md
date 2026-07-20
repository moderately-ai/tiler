---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.float-to-integer-conversion-precedents"
kind: "research"
title: "Floating-point to integer conversion precedents"
topics: ["numerics","conversion","integers"]
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0010","ADR-0041"]
reproduced_by: []
ticket: "numerical-policy-contract"
---

# Floating-point to integer conversion precedents

**Status:** adopted decision research supporting ADRs 0010 and 0041

## Traceability

- **Current disposition:** adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Adoption:** [ADR 0010](../../decisions/0010-typed-conversion-contracts.md), [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md).
- **Work record:** [numerical-policy-contract](../../../tickets/numerical-policy-contract.md).


## Finding

A floating-point source and integer destination do not determine a conversion.
Rounding, ordered overflow, infinities, NaN, exactness, and subnormal input
handling are independently observable. In particular, saturation does not
mathematically determine a NaN result because NaN is unordered.

## Existing contracts

- LLVM `fptosi`/`fptoui` round toward zero and produce poison when the rounded
  value is not representable. Its separate saturating intrinsics clamp ordered
  values and explicitly map NaN to zero.
- WebAssembly separates trapping truncation from total saturating truncation;
  its total form also maps NaN to zero.
- Rust `as` uses truncation, endpoint saturation, and NaN-to-zero as a fully
  defined language-specific totalization.
- C++ makes an unrepresentable result undefined. StableHLO leaves it TBD, and
  PyTorch documents platform variation. These contracts cannot be imported as
  portable results outside a proven valid domain.
- PTX exposes multiple rounding directions and clamps many out-of-range
  results, but its NaN result varies with source/destination widths. A native
  conversion instruction therefore cannot supply Tiler semantics implicitly.

Primary sources:

- [LLVM floating-point to integer instructions](https://llvm.org/docs/LangRef.html#fptosi-to-instruction)
- [LLVM saturating conversions](https://llvm.org/docs/LangRef.html#saturating-floating-point-to-integer-conversions)
- [WebAssembly numeric execution](https://webassembly.github.io/spec/core/exec/numerics.html#op-trunc)
- [Rust numeric casts](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast)
- [C++ floating-integral conversions](https://eel.is/c++draft/conv.fpint)
- [StableHLO convert](https://openxla.org/stablehlo/spec#convert)
- [PTX conversion instructions](https://docs.nvidia.com/cuda/parallel-thread-execution/#data-movement-and-conversion-instructions-cvt)

## Boundary details

Validation is defined against the rounded mathematical integer, not a naive
floating comparison with an integer endpoint converted into the source dtype.
An endpoint may not be exactly representable as a float, and values such as an
unsigned input in `(-1, 0)` validly truncate to zero.

Signed zero converts numerically to integer zero. NaN absence must be checked
independently of ordered range comparisons. Exact conversion additionally
requires an integral finite source value. A backend poison-producing cast is
usable only after all required preconditions are proven or enforced.

## Tiler implication

The strict portable family rejects NaN and every unrepresentable rounded
result. Ordered saturation clamps finite overflow and infinities but does not
invent a NaN mapping. NaN-to-zero remains useful as a separately named total
compatibility family for Rust, LLVM saturation, and WebAssembly imports. Other
mappings, validity results, and future seeded rounding families remain additive
versioned contracts.
