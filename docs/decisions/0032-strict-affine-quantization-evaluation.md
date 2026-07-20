---
schema: "tiler-doc/v1"
id: "ADR-0032"
kind: "decision"
title: "Fix strict affine quantization evaluation"
topics: ["numerics","quantization","semantics"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.affine-quantization-semantics"]
ticket: "define-initial-affine-quantization-semantics"
---

# 0032: Fix strict affine quantization evaluation

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [affine quantization semantics](../research/numerics/affine-quantization-semantics.md).
- **Work record:** [define-initial-affine-quantization-semantics](../../tickets/define-initial-affine-quantization-semantics.md).


## Context

An affine formula does not determine integer subtraction width, computation
dtype, operation order, intermediate rounding, subnormal handling, saturation
order, signed-zero behavior, or requantization boundaries. Existing interchange
specifications resolve different subsets and leave some behavior to lowering.
Backend-selected defaults would make fusion, reference evaluation, and fallback
disagree.

## Decision

The strict affine conversion family has a resolved computation dtype and fixed
evaluation order.

Strict dequantization:

1. widens code and zero point to a signed integer difference type that cannot
   overflow for the declared code domain;
2. subtracts exactly in that type;
3. converts the difference and positive finite scale to the resolved
   computation dtype;
4. multiplies in that dtype;
5. performs any output conversion as an explicit typed boundary.

Strict quantization:

1. converts the expressed input and positive finite scale to the resolved
   computation dtype;
2. divides input by scale in that dtype;
3. adds the converted zero point in that dtype;
4. rejects NaN under ADR 0031;
5. clamps to the converted logical code endpoints;
6. rounds to integral with round-to-nearest, ties-to-even;
7. exactly converts the now-in-range integral result to the code dtype.

Positive and negative infinity saturate to the upper and lower code endpoints.
Integer encoding collapses both input zero signs to the zero-point code. Strict
decoding of `code == zero_point` produces canonical positive zero.

Strict input, scale, intermediate, and result boundaries preserve subnormals.
Any flush behavior is a separate resolved conversion family under ADR 0019.
The computation dtype and every conversion boundary participate in canonical
semantic and artifact identity.

Logical `Requantize` is strict source dequantization followed by destination
quantization through an explicit intermediate dtype. Both boundaries remain
observable. An integer multiplier/shift `Rescale` is a different typed semantic
family. It may realize `Requantize` only after an equivalence proof covering its
ratio approximation, rounding algorithm, intermediate widths, zero points, and
reachable value domain.

## Consequences

- Reference evaluation has one backend-independent operation order.
- Unsigned and low-bit code domains cannot overflow during zero-point
  subtraction.
- Backend native instructions or compiler flags are usable only when they match
  the complete contract or exactly emulate it.
- Faster computation dtypes, flush behavior, or direct integer rescaling remain
  possible as separately named contracts or proven physical alternatives.
- Fusion cannot remove either boundary of logical requantization.

## Alternatives considered

Subtracting in the code dtype follows some existing specifications but can
overflow. Converting code and zero point independently before subtraction can
round them differently for wider domains. Leaving computation precision or
subnormal behavior to lowering defeats portability. Treating integer rescale as
synonymous with decode-then-encode loses observable approximation and rounding
differences.
