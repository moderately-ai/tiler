---
schema: "tiler-doc/v1"
id: "ADR-0027"
kind: "decision"
title: "Use uniform nominal identities for built-in and extension dtypes"
topics: ["numerics","dtypes","identity"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0027: Use uniform nominal identities for built-in and extension dtypes

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

Tiler needs ergonomic built-in dtypes and a public path for adding exact tensor
element types. A closed enum plus a separate `Custom` case would create two
identity and capability mechanisms. It would also force an identity migration
if an external type later became officially supported.

Purely structural identity is insufficient. Formats with the same apparent bit,
exponent, and fraction widths can differ in exponent bias, infinity and NaN
encodings, signed-zero behavior, and other value semantics. Rust implementation
identity is also unsuitable for durable IR and artifact keys.

## Decision

Every canonical dtype uses one namespaced, versioned nominal identity model.
Conceptually, `TypeKey { namespace, name, semantic_version }` distinguishes
identities such as `tiler::f32@1` and `acme::fp8_special@1`. The exact Rust type
and serialized spelling remain API-design details.

Built-ins and extensions use this same mechanism for canonical hashing,
serialization, registry and capability lookup, and diagnostics. Public APIs may
still expose built-ins ergonomically, for example `DType::F32`; that spelling
resolves to the built-in durable key and does not create a second identity path.
This decision does not introduce wrapper scalar values such as `TilerF32` and
does not require dynamic string lookup at ordinary call sites.

A host-canonical descriptor associated with the key supplies structural and
value-semantic facts. The key is the nominal identity; the descriptor does not
make coincidentally similar formats identical. Provider identity and revision
separately identify output-affecting capability implementations.

Canonical keys are stable across support-level changes. If Tiler later bundles
support for an externally defined type, it supports the existing external key
rather than silently renaming it into the `tiler` namespace. Frontend aliases
resolve to exactly one canonical key before semantic admission and never imply
identity equivalence by themselves.

Durable IR must not use Rust enum discriminants, `TypeId`, vtable or function
addresses, registry insertion order, or unversioned display names as type
identity. Initial verified graphs continue to reject unknown or unregistered
type keys.

## Consequences

- Built-in and extension types follow one verification, hashing, serialization,
  and capability-diagnostic path.
- Common Rust call sites remain simple and statically discoverable.
- Official support can grow without rewriting an external type's semantic
  identity.
- The extension contract must eventually define namespace ownership,
  collisions, descriptor encoding, semantic-version compatibility, and provider
  selection.
- A semantic change requires a new type version or identity; a provider-only
  implementation change updates provider provenance instead.

## Alternatives considered

A closed built-in enum with a separate custom-type escape hatch is initially
simple but creates parallel identity systems and awkward promotion of external
types. Pure structural identity cannot distinguish formats whose encodings or
special-value semantics differ. Unversioned strings are readable but provide
insufficient ownership and compatibility boundaries. Wrapping ordinary runtime
scalars in Tiler-specific Rust types would conflate IR identity with host value
ergonomics and is not required.
