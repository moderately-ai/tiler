# Floating-point extrema precedents

**Status:** research synthesis with accepted initial decision  
**Ticket:** `numerical-policy-contract`

## Primary-source facts

- IEEE 754-2019 defines `minimum`/`maximum`, which propagate NaN, separately
  from `minimumNumber`/`maximumNumber`, which prefer the numeric operand when
  exactly one input is NaN. Both order `-0.0 < +0.0`.
  [IEEE 754-2019](https://standards.ieee.org/ieee/754/6210/).
- StableHLO floating-point `minimum` and `maximum` explicitly use the IEEE
  propagating operations.
  [StableHLO `minimum`](https://openxla.org/stablehlo/spec#minimum) and
  [`maximum`](https://openxla.org/stablehlo/spec#maximum).
- LLVM distinguishes propagating `minimum`/`maximum`, deterministic
  number-preferring `minimumnum`/`maximumnum`, and legacy `minnum`/`maxnum`
  with weaker signaling-NaN behavior. Strict forms order signed zero; `nsz`
  permits either zero.
  [LLVM language reference](https://llvm.org/docs/LangRef.html#floating-point-min-max-intrinsics-comparison).
- MLIR likewise exposes distinct propagating and number-preferring arithmetic
  operations rather than one target-selected `min`.
  [MLIR arithmetic operations](https://mlir.llvm.org/docs/Dialects/ArithOps/).
- Rust's `f32::min`/`max` ignore a single NaN and may choose either signed zero,
  while its IEEE-named `minimum`/`maximum` operations propagate NaN and order
  signed zero.
  [Rust `f32`](https://doc.rust-lang.org/core/primitive.f32.html#method.min).
- CUDA `fminf`/`fmaxf` treat NaN as missing data.
  [CUDA math API](https://docs.nvidia.com/cuda/cuda-math-api/cuda_math_api/group__CUDA__MATH__SINGLE.html).
- MSL `fmin`/`fmax` are number-preferring. Their specified comparison form
  selects an operand on equality, making opposite-signed-zero results sensitive
  to operand order.
  [Metal Shading Language specification](https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf).

## Inferences for Tiler

1. NaN handling and zero ties are semantic operation behavior, not target cost
   properties.
2. Direct StableHLO-style `maximum` to MSL `fmax` lowering is not exact. For
   example, strict `Maximum(-0.0, +0.0)` is `+0.0`, while native selection can
   preserve `-0.0` depending on operand order.
3. Reassociation, operand commutation, vector reductions, and clamp/ReLU
   recognition must preserve the selected extrema family and signed-zero
   policy.
4. Legacy or native number-preferring operations with weaker sNaN or zero-tie
   behavior require fixups or explicit relaxation; their spelling is not proof
   of compatibility.

## Accepted initial decision

Tiler uses distinct `Minimum`/`Maximum` and
`MinimumNumber`/`MaximumNumber` semantic families. Both have deterministic
signed-zero ordering in their strict contracts. Backend lowering proves exact
compatibility, emits a fixup, consumes an authorized relaxation, or rejects the
alternative.
