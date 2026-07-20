---
schema: "tiler-doc/v1"
id: "ADR-0063"
kind: "decision"
title: "Check value graph ownership at semantic admission"
topics: ["rust", "semantics", "ownership", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle"]
ticket: "prototype-semantic-owner-and-commit"
---

# 0063: Check value graph ownership at semantic admission

**Status:** accepted

## Context

A public `Value<T>` is meaningful only in the graph that owns its underlying
value. Encoding that ownership in Rust appears attractive because a foreign
handle could become a compile-time error. An ordinary lifetime does not,
however, identify one unique graph: independent builders may share the same
lexical lifetime. True static separation requires a generative invariant brand
and normally a closure- or token-scoped authoring API.

That model makes helper functions and retained structures generic over an
unnameable brand, complicates values that cross the draft/completed-program
boundary, and conflicts with the required ordinary Rust construction path.
Naively borrowing a mutable builder for the handle lifetime also prevents
retaining earlier values while continuing to append operations. Dynamic
frontends, erased `ValueId`s, deserialization, extension boundaries, and
hostile inputs require runtime graph-ownership checks regardless.

## Decision

Canonical `Value<T>`, `ShapedValue<T, E>`, `ValueId`, and typed shape witnesses
carry or retain access to an opaque graph-owner identity checked at runtime.
Graph ownership is not a mandatory Rust lifetime or generative type parameter.

Every public operation, query, refinement, witness use, output declaration, and
checked reification validates that all supplied handles belong to the exact
owning draft or completed program before indexing graph storage or performing
any mutation. A mismatch returns a specific typed foreign-graph diagnostic that
identifies the argument role without exposing unstable owner-token values as
semantic data.

Fallible builder insertion remains transactional: graph ownership and all other
preconditions are checked before allocating IDs or mutating arenas, interfaces,
constraints, or caches. A foreign handle therefore leaves the draft
observationally unchanged.

Owner identities are process-local safety metadata. They are excluded from
canonical program, compilation, artifact, cache, and explanation identity.
Internal verified edges use private compact indices and do not retain redundant
owner tokens. Owner-token generation must avoid accidental live-graph aliasing;
wraparound or exhaustion is a typed construction failure rather than permission
to reuse a live identity.

A future optional branded authoring facade is not prohibited, but it must lower
to the same checked graph operations and cannot remove validation from dynamic
or trust-boundary paths. It requires separate ergonomic and compile-time-cost
evidence before becoming public.

## Consequences

- Values can pass through ordinary helper functions and structures without
  infecting their types with a graph lifetime or brand.
- Cross-graph misuse is rejected deterministically before mutation, although it
  is a Tiler diagnostic rather than a rustc type error.
- Runtime ownership checks remain present at public and erased boundaries even
  when a future typed facade proves ownership statically.
- Tests must cover every handle-consuming API, foreign values with coincident
  local indices, foreign witnesses, failed-insertion atomicity, and owner-token
  exhaustion behavior.

## Alternatives considered

`Value<'graph, T>` looks conventional but a lifetime alone does not prove unique
graph identity and can create mutable-borrow conflicts. A generative branded
closure can provide static separation but significantly constrains escape,
composition, and helper APIs while leaving dynamic validation necessary. A
globally unique durable graph UUID would conflate process safety metadata with
semantic identity and still require runtime checking.

## Traceability

The [IR contract](../ir.md) owns handle validity, transactional admission, and
identity exclusions. The [Rust construction lifecycle
research](../research/semantic-graph/rust-construction-lifecycle.md) owns the
builder/completed-program boundary.
