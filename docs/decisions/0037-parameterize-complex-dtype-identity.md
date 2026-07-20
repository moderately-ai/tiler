---
schema: "tiler-doc/v1"
id: "ADR-0037"
kind: "decision"
title: "Parameterize complex dtype identity by component type"
topics: ["numerics","dtypes","complex"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0037: Parameterize complex dtype identity by component type

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

Tensor ecosystems commonly use names such as `complex64` and `complex128`, but
those names denote total storage width rather than component precision:
`complex64` usually contains two binary32 components. A flat list of complex
type keys would repeat the same structural contract and require a new unrelated
identity whenever another component format is admitted.

Complex values nevertheless need nominal identity. An unconstrained structural
pair is insufficient because a pair of real values is not automatically a
complex scalar, and complex operations have their own mathematical semantics.

## Decision

Complex is a built-in nominal, parameterized dtype family. Its canonical
identity is conceptually:

```text
tiler::complex@1<ComponentTypeKey>
```

The family defines one logical scalar as an ordered real and imaginary pair
whose components have the same admitted real floating-point dtype. The
component `TypeKey` is part of canonical identity, equality, serialization,
descriptor fingerprinting, diagnostics, and artifact identity.

The initial admitted instances are:

- `tiler::complex@1<tiler::f16@1>`;
- `tiler::complex@1<tiler::f32@1>`; and
- `tiler::complex@1<tiler::f64@1>`.

Frontend spellings such as `complex32`, `complex64`, `complex128`, `chalf`,
`cfloat`, and `cdouble` are aliases resolved to a complete canonical identity.
They are never canonical keys themselves.

Admission of the family does not make every `TypeKey` a valid component.
Extending the admitted component set is a catalog decision with a complete
semantic descriptor. Operation support remains signature-specific under ADR
0026; for example, representability of `complex<f16>` does not imply support
for division, transcendental functions, or reductions on it.

The logical identity does not prescribe interleaved versus planar storage,
alignment, padding, or ABI representation. Those remain explicit physical
storage and binding contracts. Complex-operation branch cuts, exceptional
values, rounding, and accuracy are operation-policy contracts rather than
facts inferred solely from the family constructor.

## Consequences

- Component precision is explicit and unambiguous.
- New admitted component formats can reuse the family without redesigning the
  identity system.
- Generic complex validation and rewrite rules can share the family contract
  while capabilities remain specialized by complete instance and operation.
- Frontends must resolve width-based aliases carefully; total width alone does
  not establish identity when storage encodings may include padding.
- Backends may choose different physical representations without changing
  logical graphs.

## Alternatives considered

Independent keys such as `complex32`, `complex64`, and `complex128` match some
framework APIs but encode the component format indirectly and scale poorly to
new formats. Treating complex as an arbitrary structural pair loses its nominal
mathematical meaning. Admitting `complex<T>` for every recognized dtype would
create nonsensical or underspecified instances such as `complex<bool>` before
their semantics were deliberately defined.
