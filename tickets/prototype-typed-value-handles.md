---
id: prototype-typed-value-handles
title: Implement exact typed semantic value handles
status: todo
priority: p0
dependencies: [prototype-resolved-value-type-registry]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions]
paths: [Cargo.lock]
tags: [implementation, prototype, semantics, rust-api, compile-tests]
---
Implement ADR 0059's exact typed authoring capability over the runtime-typed
heterogeneous graph.

- add privately constructed `Value<T>` with explicit erasure to `ValueId` and
  registry-checked fallible reification;
- keep `ValueId` as identity-and-inspection capability with `unknown`, never
  `any`, semantics;
- convert the bounded `F32` input, constant, multiply, add, strict serial Sum,
  and output paths to exact typed arguments and results;
- keep handles compact, `Copy`, and `Send + Sync` independently of marker
  layout while preserving runtime owner checks; and
- add downstream compile-pass/fail fixtures for mixed types, erased-ID misuse,
  forged construction/retyping, external marker authority, explicit
  conversions, and checked output resolution.

Every typed operation must delegate to the same transactional semantic
admission implementation used by registry-resolved paths. Do not add ambient
promotion, unchecked constructors, shape evidence, fluent duplicate operation
APIs, or physical specialization.
