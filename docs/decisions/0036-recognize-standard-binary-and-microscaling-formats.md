---
schema: "tiler-doc/v1"
id: "ADR-0036"
kind: "decision"
title: "Recognize standard binary and microscaling scalar formats"
topics: ["numerics","dtypes","floating-point"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy","tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0036: Recognize standard binary and microscaling scalar formats

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

Tiler needs stable identities for established binary floating-point formats and
the reduced-precision scalar formats used by modern tensor ecosystems. Width
and exponent/fraction counts are insufficient identity: nominally similar
formats differ in exponent bias, infinities, NaNs, signed zero, and overflow
behavior.

ADR 0026 separates recognizing a value format from supporting operations on it.
ADR 0034 requires every admitted built-in identity to have an immutable
descriptor and a pinned normative definition.

## Decision

Tiler recognizes these built-in logical scalar formats:

- `tiler::f16@1`, `tiler::f32@1`, `tiler::f64@1`, and `tiler::f128@1`,
  normatively defined as IEEE 754-2019 binary16, binary32, binary64, and
  binary128;
- `tiler::bf16@1`, pinned to the ratified RISC-V BF16 operand-format value
  contract;
- `tiler::f8e4m3fn@1` and `tiler::f8e5m2@1`, pinned to OCP OFP8 revision 1.0;
- `tiler::f6e2m3fn@1`, `tiler::f6e3m2fn@1`, `tiler::f4e2m1fn@1`, and
  `tiler::f8e8m0fnu@1`, pinned to OCP MX version 1.0.

The exact canonical spellings are part of these Tiler identities; source
spellings such as OCP `E4M3`, framework enum names, and aliases resolve to them
without becoming identities. The immutable descriptor, not suffix parsing,
defines each value set and classifies its special-value encodings. Arithmetic
propagation, exception, canonicalization, and conversion behavior remain
operation-policy contracts; Tiler does not import them from a source ISA merely
because that ISA supplies the normative value-format definition.

Recognition establishes only logical value identity and canonical descriptor
availability. Literal parsing, encoding, conversion, arithmetic, comparison,
reference evaluation, optimization, ABI, storage, and backend support remain
independent operation- and target-specific capabilities.

OCP MX compound block formats are not scalar `TypeKey`s. They are separately
versioned scheme identities that compose element and scale types, block shape,
scale selection, and conversion rules. In particular, recognizing
`f8e8m0fnu` as a scalar scale-data format does not make an MX tensor an ordinary
tensor of that dtype. Its descriptor records the OCP value set exactly: positive
powers of two plus NaN, with no zero, sign, or infinity.

TF32 is not admitted as a tensor value dtype by this decision. It remains an
execution/input-precision contract because its common use stores binary32
values while reducing operand precision for selected operations.

## Consequences

- Frontends share canonical identities for established binary, BF16, OFP8,
  FP6, FP4, and exponent-only scale values.
- Binary128 recognition does not imply native GPU arithmetic.
- Similar third-party FP8 formats, including FNUZ and alternate-bias variants,
  cannot alias an admitted OCP format without exact conformance evidence.
- Target spellings such as PTX `.ue8m0` do not alias the OCP E8M0 identity
  without exact value-and-encoding equivalence evidence.
- MX tensor semantics remain explicit rather than being inferred from a scalar
  dtype or storage width.
- Providers can add capabilities progressively without changing graph type
  identity.

## Alternatives considered

Admitting only formats supported natively by the first backend would couple the
portable graph vocabulary to Metal. Treating every reduced-precision format as
an extension would fragment identities for published interoperable standards.
A structural floating-point constructor would make superficially similar
formats collide and would fail to capture their complete special-value and
conversion contracts. Treating MX as a scalar dtype would erase its block
scale and parameter-selection semantics.
