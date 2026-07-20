---
id: prototype-typed-value-handles
title: Implement exact typed semantic value handles
status: done
priority: p0
dependencies: [prototype-resolved-value-type-registry, prototype-semantic-operation-registry]
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
- remove the public `input_f32`, `scalar_f32`, `scalar_f32_bits`,
  `multiply_f32`, `add_f32`, and `strict_serial_sum_f32` builder methods;
- add generic registry-checked `input::<T>` plus a distinctly named checked
  erased input path for parsed frontends;
- expose constant, multiply, add, strict serial Sum, and output typed facades
  over the canonical operation-registry `apply` path, generalizing only where
  the registered operation family genuinely preserves its resolved signature;
- keep handles compact, `Copy`, and `Send + Sync` independently of marker
  layout while preserving runtime owner checks; and
- add downstream compile-pass/fail fixtures for mixed types, erased-ID misuse,
  forged construction/retyping, external marker authority, explicit
  conversions, and checked output resolution.

Every typed operation must delegate to the same transactional semantic
admission implementation used by registry-resolved paths. Do not add ambient
promotion, unchecked constructors, shape evidence, fluent duplicate operation
APIs, or physical specialization.

## Outcome

Implemented compact exact `Value<T>` handles over the runtime-typed graph,
with explicit erasure and registry-checked reification on both drafts and
completed programs. Generic `input::<T>` and checked `input_resolved` now serve
typed Rust callers and parsed frontends respectively; the misleading public
F32-suffixed builder methods were removed.

Governed constant, multiply, add, and strict serial Sum facade types delegate
to the sole registry-backed `apply` path. Typed `Output<T>` selectors cross
commitment safely and are rechecked against the completed program's frozen
registry. Downstream trybuild fixtures prove mixed-type rejection, erased-ID
misuse rejection, constructor privacy, retyping rejection, external marker
authoring, explicit conversion structure, and checked output resolution.
Runtime tests cover exact mismatch diagnostics, owner safety, compact layout,
and marker-independent `Send + Sync`; strict Clippy, tests, and doctests pass.
