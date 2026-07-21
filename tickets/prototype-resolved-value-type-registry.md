---
id: prototype-resolved-value-type-registry
title: Implement resolved value types and the frozen marker registry
status: done
priority: p0
dependencies: [prototype-semantic-owner-and-commit]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/extensions, research/numerics]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions]
paths: [Cargo.lock]
tags: [implementation, prototype, semantics, registry, dtypes]
---
Implement the bounded type-identity and registry authority required by ADRs
0060 and 0062.

- define a validated, recursively bounded canonical `ResolvedValueType` domain
  that can represent nominal, parameterized, and encoded-numeric contracts;
- store the complete resolved type on every semantic value;
- bind local Rust markers to exact resolved identities through an explicit
  per-session registry with collision checks and immutable freezing;
- provide the standard `F32` registration and one statically linked external
  marker/provider path through the same machinery;
- reject unregistered markers, duplicate marker/identity claims, invalid
  descriptors, post-freeze mutation, and unsupported complete signatures; and
- version canonical encoding so resolved types and frozen registry provenance
  participate without Rust `TypeId`, names, layouts, or addresses.

The external proof must demonstrate extension authority rather than merely use
a second hard-coded built-in. The initial executable operation profile may
remain `F32`; parameterized and encoded-numeric variants need canonical
validation and identity coverage, not arithmetic or backend support. Do not add
typed value handles, shape evidence, quantized kernels, or a public stable wire
format.

## Outcome

- Added a validated, versioned `ResolvedValueType` domain for nominal,
  parameterized, and encoded-numeric identities with fixed recursion, node,
  collection, and payload bounds.
- Added host-canonical type arguments and records, stable `TypeKey` and
  `QuantSchemeKey` identity domains, and collision-free v1 canonical encoding.
- Implemented transactional statically linked providers, a mutable standard
  registry baseline, consuming freeze, duplicate authority checks, referenced-
  type closure validation, and immutable canonical registry provenance.
- Bound open Rust marker types through process-local `TypeId` lookup without
  granting trait implementations authority or admitting Rust identity into
  durable bytes. Built-in `F32` and an external marker use the same provider
  path.
- Attached the cheap-clone frozen semantic snapshot to builders and completed
  programs, stored the complete resolved type on every value, and included
  retained value types in semantic identity.
- Covered invalid definitions, missing markers and required types, duplicate
  markers and resolved identities, transactional provider failure, dangling
  components, order-independent provenance, structural bounds, and compile-time
  prevention of post-freeze mutation.

The later `correct-semantic-identity-layering` ticket owns the corrective
transitive reached-authority closure across type-definition facts and operation
metadata. This completed slice established the registry and local validation
boundary; its outcome must not be read as claiming that the first downstream
projection implementation was transitively complete.
