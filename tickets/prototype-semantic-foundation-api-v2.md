---
id: prototype-semantic-foundation-api-v2
title: Compile-check the corrected semantic foundation API
status: done
priority: p0
dependencies: [prototype-resolved-value-type-registry]
related: [prototype-semantic-reference-slice]
scopes: [research/extensions, research/semantic-graph, research/reference]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions, contracts/navigation]
paths: []
tags: [implementation, spike, semantics, rust-api]
---
Compile-check the corrected public semantic foundation before changing the
production crates. The spike must exercise one coherent call path containing:

- independent nominal, parameterized-constructor, and encoded-scheme semantic
  authority plus optional Rust marker bindings;
- a host-owned bounded canonical value/schema vocabulary with role-specific
  type-argument, encoded-contract, definition-fact, and operation-attribute
  wrappers;
- on-demand validation of `complex<f32>` and an encoded numeric type backed by
  registered `i8`, without registering every concrete instance or requiring a
  marker;
- one explicit frozen semantic registry containing type and operation
  definitions, with built-ins and an external provider using the same path;
- generic typed input admission, checked erased input/operation admission, and
  typed facades which cannot manufacture result types; and
- a downstream reference-capability registry which depends on semantic IR but
  is not imported by semantic IR.

Record exact public crate, module, trait, type, error, and call-site proposals.
Add a superseding crate-boundary ADR and a refining semantic-authority ADR.
Update the affected contracts and ticket graph, then stop for Tom's public API
review before production implementation. Do not edit `crates/` in this ticket.

Acceptance includes compile-pass/fail cases for absent markers, invalid
constructor/scheme instances, unsupported operations, forged result typing,
and an external provider. The spike is evidence, not a compatibility promise.

## Outcome

Compile-checked the corrected API across separate IR, reference, external
provider, and consumer crates. The spike established independent semantic type
authority, optional marker bindings, host-owned canonical values, registered
operation semantics, generic typed inputs, checked erased application, and a
downstream reference-capability boundary. ADRs 0065 and 0066 record the crate
and authority corrections used by the production tickets.
