---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.dtype-resolution-precedents"
kind: "research"
title: "Dtype resolution and mixed-precision precedent"
topics: ["numerics","dtypes","conversion"]
catalog_group: "dtypes-quantization"
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0009","ADR-0010"]
ticket: "numerical-policy-contract"
---

# Dtype resolution and mixed-precision precedent

**Status:** research synthesis with an accepted initial decision  
**Ticket:** `numerical-policy-contract`  
**Scope:** semantic value typing, computation precision, accumulation, and
conversion boundaries

## Traceability

- **Current disposition:** adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Adoption:** [ADR 0009](../../decisions/0009-resolved-numerical-typing.md), [ADR 0010](../../decisions/0010-typed-conversion-contracts.md).
- **Work record:** [numerical-policy-contract](../../../tickets/numerical-policy-contract.md).


## Question

What numerical typing information must be resolved before a tensor program is
eligible for semantic optimization, and which policies may remain frontend or
backend concerns?

## Primary-source facts

### Compiler IRs use explicit typed operations

- StableHLO operations have typed input and output signatures. Ordinary
  non-quantized `add` requires matching operand and result element types.
  [StableHLO specification](https://openxla.org/stablehlo/spec#add).
- MLIR arithmetic uses separate conversion operations such as `extf`,
  `truncf`, and `convertf`. `truncf` can carry an explicit rounding mode.
  [MLIR arithmetic dialect](https://mlir.llvm.org/docs/Dialects/ArithOps/).
- StableHLO `convert` is explicit, but some inexact conversion behavior remains
  underspecified. This is precedent for representation, not a complete
  conversion contract for Tiler to copy.
  [StableHLO `convert`](https://openxla.org/stablehlo/spec#convert).

### Frontend promotion is not portable semantics

- PyTorch promotion distinguishes dimensioned tensors, rank-zero tensors, and
  Python scalars. A floating scalar uses the configurable default dtype, and
  promotion does not inspect scalar values.
  [PyTorch tensor attributes](https://docs.pytorch.org/docs/stable/tensor_attributes).
- PyTorch autocast is an operation-specific policy: matrix multiplication and
  convolution families may run in lower precision, while operations including
  sum, softmax, and normalization are assigned float32 behavior. Unlisted
  operations can still observe types changed by upstream autocast.
  [PyTorch AMP](https://docs.pytorch.org/docs/stable/amp.html).
- JAX uses a different promotion lattice and represents Python-like scalar
  behavior with a `weak_type` bit. Its strict promotion mode rejects many mixed
  strong types while retaining safe weak-scalar convenience.
  [JAX type promotion](https://docs.jax.dev/en/latest/type_promotion.html).

### Reduction type and reduction order are separate contracts

- StableHLO `reduce` converts input slices and initial values to the reduction
  body's destination types. It defines execution using an
  implementation-selected binary tree and explicitly notes that floating-point
  addition is not associative.
  [StableHLO `reduce`](https://openxla.org/stablehlo/spec#reduce).
- JAX `mean` returns a dtype selected by the operation contract and computes
  float16 and bfloat16 reductions at float32 precision.
  [JAX `mean`](https://docs.jax.dev/en/latest/_autosummary/jax.numpy.mean.html).
- Triton `sum` accepts an explicit dtype and casts its input before reduction;
  without one, it applies its own documented defaults. It separately requires
  an associative and commutative reduction.
  [Triton `sum`](https://triton-lang.org/main/python-api/generated/triton.language.sum.html).

### Contractions expose more than input and result dtype

- StableHLO `DotAlgorithm` separates input precision types, accumulation type,
  component decomposition, primitive-operation count, and permission for
  imprecise accumulation. Input precision is independent of storage type.
  [StableHLO `dot_general`](https://openxla.org/stablehlo/spec#dot_general).
- Triton `dot` separately exposes an accumulator input, result dtype, and input
  precision. Its available choices and defaults differ across NVIDIA and AMD;
  an NVIDIA tensor-core path may default float32 inputs to TF32.
  [Triton `dot`](https://triton-lang.org/main/python-api/generated/triton.language.dot.html).

### Conversion and materialization are not the same boundary

- Triton `cast` distinguishes numerical conversion from bitcast and exposes
  nearest-even and toward-zero floating downcast rounding.
  [Triton `cast`](https://triton-lang.org/main/python-api/generated/triton.language.cast.html).
- MLIR's explicit truncation and rounding attributes likewise attach numerical
  behavior to a value-producing operation, not to whether the value happens to
  be written to global memory.
  [MLIR `truncf`](https://mlir.llvm.org/docs/Dialects/ArithOps/#arithtruncf-arithtruncfop).

### Required FMA differs from optional contraction

- Rust's stable `f32::mul_add` guarantees the rounded infinite-precision
  multiply-add result and demonstrates an input where it differs from separate
  multiplication and addition.
  [Rust `f32::mul_add`](https://doc.rust-lang.org/stable/std/primitive.f32.html#method.mul_add).
- LLVM represents required fused multiply-add with `llvm.fma`, while its
  `contract` permission allows eligible multiply/add instructions to be fused
  without granting arbitrary reassociation.
  [LLVM language reference](https://llvm.org/docs/LangRef.html).

### Floating-point environment and subnormal behavior are explicit boundaries

- StableHLO specifies IEEE-754 default results while continuing execution
  without raising floating-point status flags. This is value semantics similar
  to `raiseNoFlag`, not an observable floating-point environment.
  [StableHLO floating-point exceptions](https://openxla.org/stablehlo/spec#floating-point-exceptions).
- LLVM's default environment assumes traps are disabled and status flags are
  unobservable. Its `denormal_fpenv` attribute independently describes result
  and input subnormal handling; older one-field syntax couples them only for
  compatibility.
  [LLVM floating-point environment](https://llvm.org/docs/LangRef.html#floating-point-environment).
- CUDA documents no trap handlers or status flags for GPU floating-point
  exceptions. Exceptional operations return their masked/default values, and
  compiler controls separately affect flush-to-zero and operation precision.
  [CUDA floating-point computation](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html).

## Inferences for Tiler

1. One `dtype` field cannot define a tensor operation. Value dtype,
   per-operand computation precision, accumulator dtype, result dtype,
   conversion behavior, reduction order, and algorithm permissions may all be
   independently observable.
2. PyTorch-like, JAX-like, strict, and custom promotion can all be valid
   frontend policies. None can remain ambient after admission into canonical
   Tiler IR.
3. Weak scalar typing is frontend resolution state. Canonical constants are
   strongly typed; weak-type provenance may remain diagnostic metadata.
4. Widening inside a reduction or contraction can be intrinsic to that
   operation's explicit numerical signature. It does not require a graph-level
   cast for every scalar iteration.
5. A semantic conversion remains observable when fusion removes the physical
   store and reload. Conversely, materializing a value must not invent a
   narrowing conversion absent from semantic IR.
6. Advisory backend precision preferences are not exact requirements. A
   backend must demonstrate that a physical implementation realizes the
   declared contract or reject it.
7. Required single-rounding FMA semantics and permission to contract two
   separately rounded operations are different contracts and require different
   semantic representations.

## Accepted initial decision

**Accepted by Tom on 2026-07-19:** every compilable semantic tensor value has a
resolved value dtype, and every operation has a resolved numerical signature.
Ordinary elementwise operations are homogeneous by default; frontends lower
mixed inputs through explicit semantic conversions.

Reductions, contractions, and other operations with internal mixed-precision
semantics use specialized signatures that explicitly identify applicable
computation/input precision, accumulator dtype, result value dtype, conversion
behavior, and order or algorithm contract. The exact fields are operation
capabilities, not one universal bag that every operation must populate.

```text
Cast<f16 -> f32, nearest_even>(lhs)
Add<f32>(lhs, rhs) -> f32

ReduceSum {
  input_value_dtype: f16,
  compute_dtype: f32,
  accumulator_dtype: f32,
  result_value_dtype: f16,
  final_conversion: nearest_even,
  order_contract: reassociation_allowed,
}
```

Frontend promotion tables, weak types, default dtypes, and autocast policies
must be resolved before canonical semantic admission. Their source/version may
remain explanation provenance, but no later compiler phase consults ambient
frontend state to reinterpret the graph.

Backend feasibility distinguishes exact native support, exact emulation,
support only under a declared relaxed policy, and unsupported contracts. A
backend cannot silently substitute TF32 input precision, narrower
accumulation, another rounding mode, contraction, or additional reduction-order
freedom.
