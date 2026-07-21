---
id: add-checked-closure-convenience-for-shared-ir-builders
title: Add checked closure convenience for shared IR builders
status: todo
priority: p1
dependencies: [prototype-canonical-index-region-slice]
related: [prototype-shared-compiler-ir-ownership]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dx]
---
# Add checked closure convenience for shared IR builders

## Goal

Complete ADR 0071's accepted ergonomic layer with a closure-based convenience
that delegates to the same transactional builder and consuming `build()`
verifier. Keep the mutable draft scoped to the closure and return only an
immutable verified product.

## Work

- Decide one error composition that preserves both closure/admission failures
  and recoverable whole-object verification diagnostics without erasing either.
- Add the convenience first for `IndexRegionBuilder`; make the pattern reusable
  by later schedule, kernel, and program builders without a generic untyped IR
  abstraction.
- Document ordinary builder and closure call sites side by side.

## Acceptance

- The successful closure path produces the same canonical region as manual
  construction followed by `build()`.
- Admission and whole-region failures retain their typed distinctions and do
  not expose or forge verified storage.
- ADR 0071 and the public API docs no longer describe an unimplemented
  convenience as part of the implemented static slice.
