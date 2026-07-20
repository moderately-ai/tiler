---
schema: "tiler-doc/v1"
id: "ADR-0062"
kind: "decision"
title: "Bind Rust markers to complete resolved value types"
topics: ["rust", "semantics", "dtypes", "quantization", "registry"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir", "tiler.contract.numerical-semantics", "tiler.contract.operation-extensions"]
evidence: ["tiler.research.numerics.dtype-identity-admission-policy", "tiler.research.numerics.quantized-value-and-transform-contract", "tiler.research.numerics.mature-dtype-taxonomy"]
ticket: "prototype-resolved-value-type-registry"
---

# 0062: Bind Rust markers to complete resolved value types

**Status:** accepted

## Context

ADRs 0059 and 0060 originally described `Value<T>` as binding `T` directly to
one nominal `TypeKey`. That is sufficient for primitive scalar types but not for
the complete accepted value taxonomy. Complex values use a nominal
parameterized family, while first-class quantized tensors carry a static
encoded-numeric scheme contract identified by a `QuantSchemeKey` that is
deliberately distinct from primitive `TypeKey` identity. Concrete scale,
zero-point, codebook, and other parameter tensors remain graph operands rather
than type identity.

Splitting these values into unrelated Rust handle families would weaken the
accepted invariant that they are all first-class semantic tensor values. Forcing
every scheme into a primitive `TypeKey` would instead collapse identity domains
that ADR 0030 intentionally separates.

## Decision

`T` in `Value<T>` identifies one complete, shape-independent resolved semantic
value-type contract. It does not necessarily identify one primitive dtype key.
Conceptually, the canonical domain is an explicitly tagged structure capable of
representing at least:

```text
ResolvedValueType =
    Nominal(TypeKey)
  | Parameterized {
        constructor: TypeKey,
        arguments: [ResolvedValueTypeArgument],
    }
  | EncodedNumeric {
        scheme: QuantSchemeKey,
        static_contract: CanonicalEncodedNumericContract,
    }
```

The exact Rust enum and field decomposition remain prototype work, but every
variant has a canonical versioned encoding, bounded recursion, deterministic
validation, and explicit identity-domain tags. Parameterized and encoded types
may reference admitted primitive or parameterized component identities without
turning their physical storage encoding or runtime parameter payloads into
semantic type identity.

Every canonical tensor value stores its complete resolved value type. The
frozen registry binds one local Rust marker to one complete
`ResolvedValueType`, rather than only to a `TypeKey`. `TypeId<T>`, marker names,
layouts, and monomorphized Rust structures remain process-local lookup details
and never enter durable identity.

The registry applies the same collision rule to the complete resolved identity:
one marker maps to at most one resolved value type and one resolved value type
has at most one canonical Rust marker within a frozen registry. Merely
implementing a marker trait grants no authority. Checked reification compares
the graph-stored resolved value type with the registry binding exactly.

Primitive, complex, quantized, and future admitted compound tensor types use
the same `Value<T>` capability, graph edge representation, interface machinery,
and generic shape-preserving operations. Operation support remains specific to
the complete resolved signature; representability of a value type does not
grant arithmetic, evaluator, optimizer, or backend support.

## Consequences

- Rust authoring remains uniform across primitive and compound semantic tensor
  values.
- Quantization scheme identity stays distinct from primitive dtype and physical
  storage identity while still participating in one resolved value-type domain.
- Canonical value and operation identity must encode the full resolved value
  type, not assume a single primitive key or a graph-wide implicit `f32` type.
- The registry and diagnostics must name both the outer value-type family and
  the exact failing component or scheme when validation fails.
- Adding a new resolved-value-type variant is a versioned semantic-format
  change, not an ungoverned extension-trait implementation.

## Alternatives considered

Separate `Value`, `ComplexValue`, and `QuantizedValue` handles permit narrowly
specialized APIs but duplicate graph, interface, and generic transformation
machinery. Treating every compound scheme as a flat `TypeKey` erases the
accepted distinction between primitive types and encoded-numeric schemes.
Putting runtime quantization parameter payloads into Rust or canonical type
identity causes type explosion and breaks ordinary SSA dataflow.

## Traceability

The [IR contract](../ir.md) owns canonical per-value type storage and typed
handle reification. [Numerical semantics](../numerical-semantics.md) owns the
resolved value-type taxonomy and operation signatures. The [operation extension
contract](../operation-extensions.md) owns registry authority and collision
handling.
