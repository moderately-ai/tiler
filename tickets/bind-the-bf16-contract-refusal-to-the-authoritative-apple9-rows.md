---
id: bind-the-bf16-contract-refusal-to-the-authoritative-apple9-rows
title: Bind the BF16 contract refusal to the authoritative Apple9 ledger rows
status: todo
priority: p2
dependencies: []
related: [state-and-check-a-bf16-numerical-contract, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, bf16, numerics, target-profiles]
---
## User-visible outcome

The BF16 numerical refusal is evidenced against `FIRST_MACOS_APPLE9` itself —
its own measured rows and its own `TargetCompileProfileMeasurementSource` —
rather than against a compiler-side profile that restates the same behaviour.

## Why this is a separate ticket

**Fact.** `FIRST_MACOS_APPLE9` lives in `crates/tiler-build/src/metal_declaration.rs`.
`tiler-build` depends on `tiler-compiler`, so no test in `tiler-compiler` can
reach it; the dependency direction, not a scope preference, is what splits the
evidence.

**Fact, at the landing commit of `state-and-check-a-bf16-numerical-contract`.**
`crates/tiler-compiler/tests/bf16_numerical_contract.rs` proves the compiler
boundary answers correctly for the measured behaviour — BF16 dispatchable,
subnormals flushed to the sign-preserving zero — under that file's own test
provenance. What it does not prove is that the authoritative ledger's rows
produce that answer, or that the refusal cites the ledger's measured source.

## Scope keys

- A `tiler-build` test states a pure-BF16 constant/multiply/add program under
  `NumericalContract::STRICT_BF16` against the profile
  `BoundMetalCompileDeclaration` builds, and asserts the refusal names
  `ArithmeticType::Bf16`, `Bf16::resolved_type()`, `SubnormalMode::Preserve`,
  `Unsupported`, the honoured sign-preserving flush, and the ledger's own
  measured source through `TargetDeclaredNumericalRefusal::evidence`.
- Deleting the ledger's BF16 subnormal row turns that refusal into `Unknown`,
  watched failing rather than asserted.
- No compiler change: the boundary already answers. This ticket only binds the
  answer to the authoritative rows.

## Required evidence

- The refusal's evidence names the ledger's measured producer identity and its
  exact compiler build and execution environment, not a test fixture's.
- The mutation above is run and reverted, and the report says so.

## Closes when

A `tiler-build` test binds the BF16 preservation refusal to
`FIRST_MACOS_APPLE9`'s own declared rows and measured source, and its
mutation was watched failing.
