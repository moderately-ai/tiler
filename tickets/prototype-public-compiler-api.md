---
id: prototype-public-compiler-api
title: Expose caller-composed compilation requests and provider installation
status: todo
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: [report-per-target-compilation-outcomes]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, dx]
---
The reviewed `tiler_compiler::session` facade and its opaque explain surface
have landed. `compile_governed` is the bounded convenience path, but an
external frontend still cannot construct the consumer-independent
`CompilationRequest` required by ADR 0069 or install a lowering-capability
authority through the public boundary.

## User-visible outcome

Let an external frontend state every semantically meaningful compilation input
that has more than one admitted value through one checked boundary:

- an ordered numerical-contract preference;
- target profiles;
- the shape environment and caller-known specialization inputs;
- resource or proof budgets and supported options; and
- installed lowering capabilities with their governed identities and
  revisions.

Preserve `compile_governed` as the simple bounded profile. Unsupported
combinations fail with typed diagnostics and a complete explain trace whenever
the trace boundary was reached.

## Boundaries

- Do not expose private strategy choices, temporary cardinality assumptions, or
  compiler-internal arenas merely because the current implementation uses them.
- Provider installation must preserve validation, deterministic resolution,
  versioned identity, and fail-closed ambiguity.
- Public request identity must cover every caller choice that can change
  semantics, feasibility, selected implementation, or produced bytes.
- Per-target outcomes are owned separately by
  `report-per-target-compilation-outcomes`.

## Public review

Tom already accepted the existing `session` facade. The exact request builder,
provider-installation call site, and the reshaped `CompileFailure` signature
remain consequential public boundaries and require review before this ticket
closes.

## Closes when

An out-of-crate frontend can construct and compile the admitted request profile,
install an external provider without an in-crate test hook, receive typed
failure plus complete explain evidence, and use the governed convenience path
without assembling the full request. The public contract and implementation
agree, and `make full` passes.
