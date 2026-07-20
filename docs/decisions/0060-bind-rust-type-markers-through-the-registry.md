---
schema: "tiler-doc/v1"
id: "ADR-0060"
kind: "decision"
title: "Bind Rust type markers through the explicit registry"
topics: ["rust", "dtypes", "extensions", "registry"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir", "tiler.contract.operation-extensions", "tiler.contract.numerical-semantics"]
evidence: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "prototype-resolved-value-type-registry"
---

# 0060: Bind Rust type markers through the explicit registry

**Status:** accepted

## Context

ADR 0059 selects exact nominal `Value<T>` handles for Rust authoring while the
canonical graph stores runtime type keys. External dtypes need equally useful
typed handles, but an open trait whose implementation directly declares a
semantic key would let arbitrary Rust code impersonate built-ins or another
provider. Sealing the marker trait would instead contradict the accepted public
extension boundary.

Rust marker types are useful local evidence but cannot become semantic
authority or durable identity. The operation registry already owns semantic
registration, collisions, provider selection, freezing, and deterministic
session provenance.

## Decision

External crates may define intentional Rust tensor-type marker types, but the
marker trait carries no semantic key or operation authority by itself. Under
ADR 0062, an explicit registry registration binds one `'static` Rust marker to
one complete registered `ResolvedValueType` and its semantic definition. This
may contain a nominal `TypeKey`, a parameterized nominal identity, or an
encoded-numeric contract governed by a `QuantSchemeKey`. The registry validates
and freezes that binding before a builder may produce `Value<T>`.

The process-local association may use `TypeId<T>` internally for lookup. Rust
`TypeId`, type names, implementation addresses, and marker layout never enter
semantic graph, compilation, plan, artifact, or cache identity. Durable identity
continues to use the namespaced, versioned `TypeKey` and registered descriptor.

Within one frozen registry:

- one marker maps to at most one resolved value type;
- one resolved value type has at most one canonical Rust marker;
- duplicate semantic authority or marker/key registration is a deterministic
  hard error;
- the registered descriptor and provider revision must satisfy the existing
  registry contract; and
- implicit marker aliases are not admitted initially.

Only a builder or completed program using that exact frozen registration may
create or checked-reify `Value<T>`. Built-in markers are registered by Tiler's
standard registry profile. External integrations explicitly add their
definitions before freezing; merely implementing the marker trait grants no
capability.

Rust operation-support traits or extension facade functions are ergonomic
claims only. Every insertion still resolves and validates the registered
operation's ordered operands, canonical attributes, complete numerical
signature, and result type keys through host-owned transactional mutation. An
extension cannot manufacture typed values or grant itself semantic support by
implementing another Rust trait.

## Consequences

- External `Value<MyType>` authoring remains possible without trusting an open
  trait as semantic authority.
- A malicious or accidental marker/key collision fails during registry setup,
  before graph construction.
- Built-in call sites remain concise through a standard frozen registry profile;
  external sessions pay explicit setup consistent with ADR 0044.
- Generic Rust code may use marker types locally, but canonical programs remain
  portable across languages and processes.
- Alias ergonomics, namespace governance across independent packages, and exact
  typed operation-facade traits remain separate decisions.

## Alternatives considered

Trusting an associated key on an open trait is simple but permits impersonation
and falsely authoritative compile-time claims. Sealing every marker blocks
external typed DX. Using Rust `TypeId` as semantic identity is process-local and
not a serialization contract. Allowing several marker aliases for one key
creates distinct Rust types for one semantic type and complicates generic
interoperation before a demonstrated need exists.

## Traceability

The [operation extension contract](../operation-extensions.md) owns registry
authority and freezing. The [IR contract](../ir.md) owns typed handle creation
and checked reification. [Numerical semantics](../numerical-semantics.md) owns
nominal type identity and complete operation signatures.
