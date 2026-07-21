---
id: harden-semantic-registry-and-program-construction
title: Harden semantic registry and program construction
status: todo
priority: p0
dependencies: [correct-semantic-identity-layering]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, semantic-ir]
---

Correct the semantic registry and program builder defects found by the
fixed-point code audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`. This
ticket complements, rather than replaces, the transitive authority-closure
work in `correct-semantic-identity-layering`.

## Required outcome

- Make every provider registration batch transactional and sticky-failing.
  Duplicate types, operations, markers, ignored helper errors, or any partial
  `register_marked` failure must leave no committable replacement or partial
  authority.
- Make freeze and validator execution deterministic; randomized map iteration
  must not choose the first reported error or callback order.
- Reject zero-result operation schemas until the IR has an accepted
  effect/token model.
- Bound and validate all public keys, diagnostic codes/messages, canonical
  values, attribute collections, type arguments, schemas, provider-produced
  result vectors, nesting, and total retained bytes before unbounded
  collection or callback work. Infinite iterators and oversized inputs must
  fail without allocation blow-up or panic.
- Make failed semantic builds transactional with respect to graph identifiers
  and other observable allocator state. Oversized operand lists and malformed
  public inputs must return typed errors rather than reaching `expect` or an
  unchecked integer conversion.
- Require the committed attributes themselves to be canonical, including
  explicit defaults; transient normalization during inference is insufficient
  when graph identity commits the stored form.
- Add forward-compatible read-only inspection for semantic definition keys and
  operation schema/arity. Reserved variants must not force downstream exhaustive
  matching, while mutable authority remains private.
- Correct public documentation that describes the generic semantic program as
  intrinsically `f32`.

## Acceptance

Add adversarial tests for ignored registration errors, duplicate replacement,
nondeterministic collision order, zero-result schemas, infinite and oversized
iterators, oversized provider results, failed-build identifier reuse,
`u32::MAX` boundary handling, and noncanonical explicit defaults. The full
Rust gate must pass without weakening the public generic dtype contract.
