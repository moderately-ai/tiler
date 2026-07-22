---
id: harden-semantic-registry-and-program-construction
title: Harden semantic registry and program construction
status: in-progress
priority: p0
dependencies: [correct-semantic-identity-layering]
related: []
scopes: [implementation/ir, contracts/foundation, implementation/compiler, implementation/reference, research/extensions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, semantic-ir]
assignee: codex
lease_expires_at: 1784720644
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
  result vectors, nesting, and aggregate canonical bytes before unbounded
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

## Outcome

- Provider registration is an isolated, sticky-failing transaction. Canonical
  count and byte reservations occur before staged insertion, and deterministic
  tests cover ignored errors, replacement attempts, marker collisions, and
  canonical freeze order.
- Public keys, shapes, canonical values and collections, operation schemas,
  diagnostics, registry batches, operand lists, and inferred result streams
  have typed finite bounds. Arbitrary iterators stop at the first over-limit
  item; aggregate canonical budgets are checked incrementally rather than
  after large temporary collection.
- Bounded public text and byte constructors validate borrowed data before
  copying it and provide explicit checked owned paths. Program construction
  charges one private aggregate canonical-work budget before every arena
  mutation, and final graph identity allocation is tied to an exact encoded
  length proven beneath that budget.
- Operation inference uses an immutable request and host-owned sticky result
  writer. It commits only callback-successful, arity-valid, registry-admitted
  facts. Host schema preflight precedes type or attribute validators and the
  inferencer itself. Provider panics propagate without graph mutation.
- Stable diagnostic codes use the validated cheaply cloned
  `ProviderDiagnosticCode` newtype. Dynamic-message contract failures remain
  typed causal sources; independent later inference failures remain explicit
  secondary evidence.
- Failed graph edits and failed consuming validation do not spend observable
  local or completed graph identifiers. Stored attributes canonicalize explicit
  defaults before graph identity.
- Frozen registries expose borrowed canonical-order definition inspection;
  schemas expose read-only attributes and bounded arity inspection, including
  exactness. The generic semantic-program documentation no longer implies an
  intrinsically `f32` graph.
- The provider threat model and the distinction between canonical-byte work
  budgets and exact heap accounting are recorded in `docs/ir.md` and
  `docs/operation-extensions.md`.
