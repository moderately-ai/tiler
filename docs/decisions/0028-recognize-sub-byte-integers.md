---
schema: "tiler-doc/v1"
id: "ADR-0028"
kind: "decision"
title: "Recognize standardized sub-byte integer types"
topics: ["numerics","dtypes","integers"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy"]
ticket: "enumerate-the-mature-tensor-dtype-taxonomy"
---

# 0028: Recognize standardized sub-byte integer types

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [enumerate-the-mature-tensor-dtype-taxonomy](../../tickets/enumerate-the-mature-tensor-dtype-taxonomy.md).


## Context

Tensor interchange and compiler ecosystems use two- and four-bit integer types,
especially for compressed and quantized model data. If Tiler leaves these types
entirely to frontends, independent integrations can assign incompatible nominal
identities to the same standardized value formats.

Recognition must not be confused with execution support. Sub-byte values often
require packed physical storage, and backend support differs by operation and
target. An integer element type also does not by itself define a quantized
numeric interpretation.

## Decision

Tiler's built-in recognized element-type catalog includes:

- two-valued `bool`;
- signed `i2`, `i4`, `i8`, `i16`, `i32`, and `i64`;
- unsigned `u2`, `u4`, `u8`, `u16`, `u32`, and `u64`.

The two- and four-bit types have canonical built-in nominal identities. Their
recognized status does not imply any particular packing, ABI, literal,
arithmetic, reference-evaluation, optimization, storage, or lowering
capability. Each of those remains explicit under ADR 0026.

`i128`, `u128`, and arbitrary-width integers are not in the initial built-in
catalog. They remain expressible through the same nominal extension mechanism
accepted in ADR 0027.

Quantized formats are required scope, but they are not aliases for these
integer identities. A separate contract must represent their expressed type,
scale, zero point, granularity, conversion behavior, and operation semantics.

## Consequences

- Standardized low-bit model data can retain one canonical identity across
  frontends.
- Backends can support storage, views, selected operations, or native execution
  independently.
- Packed storage is modeled as a physical encoding rather than silently folded
  into logical integer identity.
- Supporting quantization still requires first-class semantic components beyond
  recognizing low-bit integer storage values.

## Alternatives considered

Leaving all sub-byte integers to extensions avoids an initial catalog
commitment but fragments interchange identities for already standardized
formats. Treating `i2`/`i4` as packed storage aliases for wider integers loses
their logical value domains. Treating them as synonymous with quantized values
omits the scale, zero point, expressed type, and granularity that define the
quantized interpretation.
