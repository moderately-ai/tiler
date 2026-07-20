---
schema: "tiler-doc/v1"
id: "ADR-0034"
kind: "decision"
title: "Govern admitted built-in dtype keys in Tiler"
topics: ["numerics","dtypes","governance"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0034: Govern admitted built-in dtype keys in Tiler

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).


## Context

Many tensor scalar formats are defined by standards or external ecosystems.
Canonical keys could place those authorities directly in the namespace, such
as `ieee::binary32@2019`, or Tiler could own the IR key while normatively
referencing the external definition, such as `tiler::f32@1`.

Standards organizations generally do not publish or govern Tiler-compatible IR
key registries. Document revisions also do not necessarily correspond to
semantic compatibility versions, and formats such as bfloat16 have no single
unambiguous namespace owner. Conversely, Tiler must not appropriate or rename a
project/vendor identity that is already published and deployed.

## Decision

Formats deliberately admitted into Tiler's built-in vocabulary use
Tiler-governed canonical keys. Each immutable canonical descriptor contains a
mandatory normative-definition reference including authority, document,
revision/profile, and exact format where applicable.

The Tiler key owns IR identity and compatibility; the external reference owns
the cited numerical definition. Public aliases such as `f32`, frontend enum
values, and source-format spellings resolve to the canonical key before
semantic admission and do not create additional identities.

Published key meanings are immutable:

- an incompatible semantic change requires a new key semantic version;
- a later standards revision proven semantically identical may be recorded as
  additional non-semantic provenance/equivalence evidence;
- canonical serialization records the key and validates its registered
  descriptor fingerprint;
- key identity never uses Rust discriminants, `TypeId`, provider addresses, or
  insertion order.

This policy applies only at initial built-in admission. An already-published
external project/vendor canonical identity remains external when Tiler later
recognizes or bundles support for it. Tiler does not mint an equivalent built-in
key or migrate graphs. External equivalence is explicit, versioned, and backed
by bit/value and conversion conformance; spelling or structural similarity is
not sufficient.

Before minting a built-in key, admission checks the registry and catalog for an
existing canonical owner of the same exact format. Exact Rust structures,
display syntax, and the external namespace registration API remain evolvable
implementation details.

## Consequences

- Tiler controls the stability of its portable built-in vocabulary.
- Normative provenance remains machine-readable without pretending standards
  bodies govern Tiler's namespace.
- Standards revisions can be evaluated for semantic compatibility rather than
  mechanically changing every graph key.
- External identities never change merely because Tiler's support level grows.
- Importers need explicit alias/equivalence and source-provenance handling for
  faithful round trips.
- Admission and external providers require collision, ownership, descriptor-
  fingerprint, and equivalence governance.

## Alternatives considered

Authority-qualified keys make provenance visible but place compatibility in
namespaces Tiler does not control and overfit document revision to semantic
versioning. URI-style authorities have the same governance problem with more
serialization complexity. Renaming external identities when they become
officially supported breaks equality, artifacts, caches, and round trips and is
already rejected by ADR 0027.
