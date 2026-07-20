---
schema: "tiler-doc/v1"
id: "ADR-0035"
kind: "decision"
title: "Recognize IEEE decimal floating-point formats"
topics: ["numerics","dtypes","floating-point"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy","tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0035: Recognize IEEE decimal floating-point formats

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

IEEE 754 defines the decimal32, decimal64, and decimal128 interchange formats
alongside its binary formats. They have stable, unambiguous value and special-
value contracts, but are uncommon in current tensor accelerators. Excluding
them from Tiler's recognized vocabulary would leave independent frontends and
extensions to mint competing identities for established standard formats.

IEEE decimal interchange data can use densely packed decimal (DPD) or binary
integer decimal (BID) encoding. Those alternatives encode the same logical
format and values with different bits. Treating each as a different dtype would
confuse numerical identity with storage representation.

## Decision

Tiler's built-in recognized dtype catalog includes:

- `tiler::decimal32@1`, normatively defined by IEEE 754 decimal32;
- `tiler::decimal64@1`, normatively defined by IEEE 754 decimal64; and
- `tiler::decimal128@1`, normatively defined by IEEE 754 decimal128.

The canonical descriptors pin the applicable IEEE 754 revision/profile and
the complete logical datum contract, including precision, exponent range,
finite coefficient/exponent cohort members and quantum, signed zeros,
infinities, and NaNs. Numerically equal cohort members are never collapsed in
the logical datum; exponent- and quantum-sensitive IEEE decimal operations are
one reason this distinction matters. This follows the namespace and descriptor
governance of ADR 0034.

Recognition is not an execution promise. Arithmetic, literals, conversions,
reference evaluation, optimization, ABI support, and backend lowering remain
separate capabilities under ADR 0026. No initial GPU backend support is
required by this decision.

DPD and BID are distinct `StorageEncodingKey` identities for the corresponding
logical decimal format, not distinct `TypeKey`s. A materialized decimal value
must have a compatible explicit storage encoding. A conversion between DPD and
BID that preserves the complete logical decimal datum—including cohort
member/quantum and defined special-value information—is a storage transcode,
not a numerical dtype conversion. A transformation that preserves only
numerical equality is instead an explicit decimal conversion and cannot be
substituted for bit- or quantum-preserving behavior.

## Consequences

- Frontends can exchange standard decimal tensor values without inventing
  extension keys.
- CPU, software, or future accelerator providers can add capabilities without
  changing graph type identity.
- Physical layouts and artifacts cannot assume one decimal encoding from the
  logical dtype alone.
- Bit-preserving operations and ABI matching must distinguish DPD from BID
  even though ordinary value semantics use the same dtype.
- Tiler assumes the maintenance cost of canonical descriptors, serialization,
  literals or conversions when those capabilities are later admitted, and
  conformance vectors for any implemented capability.

## Alternatives considered

Leaving decimal formats extension-only reduces the initial built-in catalog,
but invites fragmented identities for formats already standardized by IEEE.
Deferring their identities until a GPU backend supports them incorrectly
couples representability to execution. Giving DPD and BID separate dtype keys
would preserve their bits but falsely turn a storage-encoding difference into
a numerical type difference.
