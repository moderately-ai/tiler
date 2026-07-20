---
schema: "tiler-doc/v1"
id: "tiler.research.reference.normative-reference-slice"
kind: "research"
title: "Normative reference evaluator slice"
topics: ["reference", "semantics", "correctness"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
informs: ["tiler.contract.optimizer"]
reproduced_by: ["tiler.spike.reference"]
ticket: "reference-evaluator-slice"
---

# Normative reference evaluator slice

## Role

The reference evaluator defines semantic results independently of fusion,
scheduling, device execution, and physical storage. It is deliberately slow
and bit-oriented. Optimized plans are correct only if their resolved numerical
contract permits every difference from this evaluator.

This is the tensor-compiler equivalent of evaluating a database logical plan
before choosing joins, partitioning, or operators. The analogy stops at numeric
representation: tensor materialization may introduce observable dtype rounding
that a relational value model usually does not expose.

StableHLO's specification gives operations mathematical algorithms and treats
program execution separately from implementations. ONNX's reference evaluator
similarly provides a simple correctness implementation rather than an optimized
backend. Tiler additionally needs explicit bit-level dtype and numerical-policy
contracts because fusion can remove rounding boundaries.

## Representative graph

Inputs:

```text
x:    tensor<2x3xf32>
bias: tensor<3xf32>
```

Canonical operations:

```text
r0 = CastF32ToF16(x, roundTiesToEven, canonicalNaN)
r1 = CastF16ToF32(r0)
b0 = BroadcastLastAxis(bias, [2, 3])
y  = AddF32(r1, b0, strict)
v  = ReshapeRowMajor(y, [3, 2])
return [y, v]
```

This one graph exercises:

- explicit broadcast rather than implicit shape magic;
- observable low-precision rounding before downstream arithmetic;
- widening as an exact conversion;
- a reshape/view that changes logical coordinates without reordering elements;
- a shared producer and two ordered graph outputs;
- canonical NaN behavior and stable shape/signature errors.

`y` and `v` have identical element-bit sequences but different shapes. They are
separate values and outputs; aliasing them physically is a later buffer-plan
choice, not reference semantics.

## Evaluation rules

The executable witness uses little-endian integers only as a convenient display
of IEEE bit patterns; semantic values are format bits, not host byte order.

- Every input tensor has a resolved dtype, static rank, extents, and exactly the
  product of its extents in logical row-major element order.
- `CastF32ToF16` rounds once to IEEE binary16 using round-to-nearest,
  ties-to-even. Overflow maps to signed infinity. NaNs canonicalize to binary16
  quiet NaN `0x7e00` for this portable bitwise profile.
- `CastF16ToF32` is exact for non-NaN values. Canonical binary16 NaN becomes the
  canonical binary32 quiet NaN when consumed by strict arithmetic.
- `BroadcastLastAxis` repeats the one-dimensional bias for each outer logical
  coordinate and requires an exact trailing-extent match.
- `AddF32` rounds once to binary32; a NaN input returns canonical
  `0x7fc00000`. No reassociation, contraction, reciprocal, or approximation is
  involved.
- `ReshapeRowMajor` preserves the linear logical element sequence and requires
  equal checked element counts.
- Results are returned in graph-declared order.

The first slice intentionally avoids subnormal-mode variation, other rounding
modes, payload-preserving NaNs, dynamic extents, strides, reductions, and
quantized compound values. Those extend the evaluator through named operation
contracts; they do not change its independence from physical planning.

## Mechanically checkable cases

[`reference_evaluator.py`](../../../spikes/reference/reference_evaluator.py)
tests:

1. an exactly representable f32 value halfway between two adjacent f16 values
   rounds to the even f16 value before the add;
2. deleting the `f32 -> f16 -> f32` boundary changes output bits and is
   therefore illegal under the strict contract;
3. the broadcasted add and reshaped second output retain the required shapes
   and ordered bits;
4. NaN output is canonical;
5. invalid broadcast and reshape shapes return stable error codes.

Run:

```sh
python3 spikes/reference/reference_evaluator.py
```

## Compiler obligations

- Graph validation occurs before reference callbacks and derives result types
  rather than trusting them.
- A rewrite test evaluates both graphs on conformance and adversarial vectors
  under the same semantic bindings.
- A fusion test compares against the explicitly rounded graph, not an
  accidentally higher-precision host expression.
- Backend differential tests compare ordered output shapes and bits—or the
  exact admitted tolerance/exception relation for a relaxed contract.
- Missing reference capability is conservative under ADR 0044: an exact
  verified decomposition may supply evaluation; otherwise phases requiring a
  reference reject the operation with a named capability reason.
- Reference results never serve as target-feasibility or cost evidence.

## Extension path

The evaluator dispatches by semantic operation key and resolved numerical
signature. New operations add independent conformance implementations or exact
decompositions. New dtypes add explicit bit/value conversion components.
Neither requires teaching the evaluator about Metal, CUDA, threadgroups,
buffers, or fusion regions.
