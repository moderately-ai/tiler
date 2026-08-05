---
id: bind-the-bf16-contract-refusal-to-the-authoritative-apple9-rows
title: Bind the BF16 contract refusal to the authoritative Apple9 ledger rows
status: review
priority: p2
dependencies: []
related: [state-and-check-a-bf16-numerical-contract, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, bf16, numerics, target-profiles]
claimed_from: todo
assignee: agent-bf16-bind
lease_expires_at: 1785949102
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

## Outcome

Two tests landed in `crates/tiler-build/src/metal_declaration.rs`'s own test module rather than a new integration target, because only a crate-private test can compare the refusal's evidence against `FIRST_MACOS_APPLE9`'s own fields. Every expected value is read from the ledger constant, so what is pinned is that each row *reaches* the refusal; the rows' values stay pinned by `the_declaration_states_exactly_the_ledger_rows` and the descriptor perturbation sweeps.

**Fact.** `the_ledger_rows_refuse_a_strict_bf16_contract_with_their_own_measured_evidence`: `STRICT_BF16` over a pure-BF16 constant/multiply/add program on `BoundMetalCompileDeclaration::first_macos_apple9().profile()` is refused `DeclaredUnhonourable`, subject `(ArithmeticType::Bf16, tiler::bf16@1)`, required `SubnormalMode::Preserve`, means `Unsupported`, honoured `InputSubnormals(FlushToZero { PreservesSign })`. Its evidence reports `CompileProfile` / `MeasuredProfile` / `MeasuredEnvironment`, authority identity `tiler.metal.first-macos-apple9-msl4.measured.v1@1`, and one measurement context carrying the ledger's four offline components -- `apple.metal-offline-compiler` (code generator), `apple.air-lld` (linker), `apple.xcode` and `apple.macos-sdk` (producer-defined roles) -- with the ledger's own execution environment.

**Fact.** `the_ledger_bf16_rows_leave_the_remaining_dimensions_unknown`: `FLUSH_SUBNORMALS_TO_ZERO_BF16` clears both subnormal dimensions and then meets `Contraction { subject: bf16, required: Forbidden }` with disposition `Unknown`. The ledger declares BF16 dispatchability and the two subnormal tables and nothing else, so that is the ledger's current boundary, now asserted rather than described.

**Correction to this ticket's scope keys.** "Deleting the ledger's BF16 subnormal row turns that refusal into `Unknown`" is not what that row does. Removing the BF16 entry from `LedgerRows::facts` refuses the whole declaration with `BoundMetalDeclarationError::UnstatedBf16SubnormalBehaviour` before any profile is built -- watched, with five BF16 tests failing at `declared()`. The `Unknown` degradation was watched instead by flipping the same row to `MetalSubnormalArithmetic::PreservesSubnormals`, which moves the strict refusal's first unhonourable dimension to `Contraction`/`Unknown` (test one fails) and simultaneously makes the flush-accepting contract refuse `DeclaredUnhonourable` on input subnormals (test two fails) -- so both tests are bound to that one row, in both directions. Two further perturbations exercised the evidence block itself: dropping the Xcode build from `measured_source` failed the four-component assertion, and stopping `measured_source` from reading `rows.execution.hardware` failed the environment assertion. All four perturbations were reverted, the file restored byte-identically from a saved copy and re-verified by checksum.

Checks on the landed tree: `cargo fmt --check`, `cargo check -p tiler-build --all-targets`, `cargo clippy -p tiler-build --all-targets -- -D warnings`, `cargo nextest run -p tiler-build` (83 passed, up from 81), `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-build --no-deps`, and `cargo test -p tiler-build --doc` (3 compile-fail doc-tests).
