---
schema: "tiler-doc/v1"
id: "ADR-0038"
kind: "decision"
title: "Recognize OCP microscaling schemes as compound values"
topics: ["numerics","quantization","microscaling"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy","tiler.research.numerics.quantized-value-and-transform-contract"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0038: Recognize OCP microscaling schemes as compound values

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

OCP MX defines block-scaled formats in which 32 element codes share an E8M0
scale. The numerical meaning therefore cannot be recovered from the element
dtype alone. The scheme also defines conversion and special-value behavior,
including the effect of a NaN scale on the block.

ADR 0030 provides a first-class encoded-numeric value with a versioned
`QuantSchemeKey`, ordered component operands, parameter-coordinate maps, and
separate storage encoding. ADR 0036 recognizes the OCP element and E8M0 scale
formats as scalar `TypeKey`s but deliberately does not equate those constituents
with an MX tensor.

## Decision

Tiler recognizes these built-in compound scheme identities, pinned to OCP
Microscaling Formats version 1.0:

- `tiler::mxfp8_e4m3@1`;
- `tiler::mxfp8_e5m2@1`;
- `tiler::mxfp6_e2m3@1`;
- `tiler::mxfp6_e3m2@1`;
- `tiler::mxfp4_e2m1@1`; and
- `tiler::mxint8@1`.

Each canonical scheme descriptor pins the exact constituent `TypeKey`s, block
size, element-to-scale selection map, decode meaning, encode/conversion rules,
rounding and saturation requirements, and block-wide special-value behavior.
These are `QuantSchemeKey` identities under ADR 0030, not scalar `TypeKey`s or
physical packing formats.

An assembled MX tensor has ordinary graph operands for its element-code and
scale components. Its logical block structure is explicit in the static scheme
and parameter-selection map. Physical packing, interleaving, byte/bit order,
padding, alignment, and whether components occupy one or several buffers remain
separate `StorageEncodingKey` and ABI decisions.

Recognition does not imply encode, decode, reference-evaluation, native
operation, optimization, storage, or backend support. Each is an explicit
scheme/operation/target capability under ADR 0026. Transformations preserve an
MX value only when they preserve or validly remap its block membership and
scale-selection map under ADRs 0029 and 0030.

## Consequences

- An FP4, FP6, FP8, or i8 tensor is never mistaken for an MX tensor merely
  because its codes are compatible.
- MX values can cross graph and artifact boundaries without detached scale
  metadata.
- Backends can choose target-specific packing without changing numerical
  identity.
- Views, slicing, concatenation, and reshaping acquire explicit preservation
  proof obligations for block membership and scale selection.
- Future OCP revisions or incompatible profiles require compatibility review
  and, where semantics change, a new Tiler scheme version.

## Alternatives considered

Treating MX as a scalar dtype erases its shared-scale dependency. Treating it
as an element tensor plus informal metadata removes the scale from ordinary
use-def and validation. Folding a particular nibble or byte layout into the
scheme would prevent independent storage planning. Leaving published OCP
schemes entirely to extensions would invite competing identities for an
interoperability standard already covered by Tiler's compound-value model.
