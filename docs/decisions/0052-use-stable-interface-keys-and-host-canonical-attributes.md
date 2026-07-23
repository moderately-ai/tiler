---
schema: "tiler-doc/v1"
id: "ADR-0052"
kind: "decision"
title: "Use stable interface keys and host-canonical attributes"
topics: ["identity", "attributes", "semantics"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir", "tiler.contract.operation-extensions"]
evidence: ["tiler.research.extensions.operation-extension-surface"]
ticket: "research-readiness-gate"
---

# 0052: Use stable interface keys and host-canonical attributes

**Status:** accepted

## Context

Semantic graph identity feeds rewrite caches, plan identity, artifact identity,
and the expansion cache. Diagnostic input/output names and provider-owned Rust
attribute serialization are not stable semantic authorities. Leaving them
ambiguous would bake source spelling, serializer behavior, or host integer
width into durable hashes.

## Decision

Every semantic input and output has a stable newtyped `ProgramInputKey` or
`ProgramOutputKey` that participates in semantic identity. Optional display
names and source spans do not. Frontends without authored keys derive canonical
ordinal keys.

Operation attributes use Tiler's bounded discriminated
`CanonicalValue`: booleans, explicit-width signed/unsigned bits, governed
float-format bits, bytes, exact UTF-8, type keys, ordered sequences, and records
with stable `AttributeFieldId(u32)` keys. Records are sorted and duplicate-free;
schema defaults have one canonical representation; unknown fields are rejected
in the initial lockstep profile.

The v1 identity encoder uses one-byte kind/width tags, big-endian integer
payloads, `u64` byte/item lengths, `u32` field IDs, and exact payload bytes.
Provider Rust structs,
`Serialize` output, map iteration order, `usize`, and diagnostic labels never
define identity. This internal identity encoding does not choose or expose the
future public artifact serialization codec.

## Consequences

- Display renames do not invalidate a semantic program.
- Named multi-output interfaces retain stable machine-readable keys.
- Built-ins and extensions use the same bounded attribute representation.
- Signed zero and NaN payload distinctions survive where the attribute schema
  makes them meaningful.
- Schema evolution changes an operation's semantic version or follows a future
  explicitly governed compatibility rule; unknown fields are not guessed.

## Implementation boundary

The semantic and scalar-operation registries now share the governed canonical
value representation, including `AttributeFieldId`, explicit integer widths,
format-typed exact float bits, schema-owned defaults, and normalization of an
explicit default to omission before structural identity is computed. Registry
definition identities commit to the field requirement/default policy, and
registered defaults are checked against the semantic type authority.

This is still partial implementation of the ADR. `CanonicalValue` is the single
spelling used by this decision, the IR and extension contracts, and the Rust
type; the earlier `CanonicalAttrValue` design-document spelling is retired. The
broader artifact codec and schema-evolution compatibility policy remain
intentionally unselected. The canonical identity encoding is internal and
versioned; it is not yet a public interchange format.

## Alternatives considered

Using names directly conflates diagnostics with interface identity. Relying
only on output ordinal loses stable named-output references. Provider-owned
serialization is noncanonical across libraries and versions. A universal
untyped byte blob would prevent host validation, explanation, and bounded
parsing.
