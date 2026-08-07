---
id: key-numerical-requirements-by-the-contract-s-own-resolved-type
title: Key numerical requirements by the contract's own resolved type
status: closed
priority: p2
dependencies: []
related: [redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, defect]
closed_reason: obsolete
closed_note: Defect fixed by 6207fba4 on 2026-08-05; reproduce command no longer reproduces at 879dec67. Verified at source by the coordinator.
---
## The defect

**Fact.** `crate::policy::dimension_requirements` (`crates/tiler-compiler/src/policy.rs:636-654`) builds every `NumericalRequirement` with a hard-coded `F32::resolved_type()` while reading `contract.arithmetic` from the caller's contract:

```rust
NumericalRequirement::new(
    dimension,
    contract.arithmetic,
    F32::resolved_type(),
    contract.behaviour(dimension),
)
```

**Inference.** A contract stating `ArithmeticType::F16` therefore produces requirements whose resolved type is `tiler::f32@1` — a pair `ScalarArithmetic::new` would refuse. `CheckedTargetProfile::resolve_dimension` (`target/feasibility.rs:1146-1157`) matches facts on dimension, arithmetic, **resolved type**, and behaviour, so no declaration can ever match, and the outcome is `Unknown`.

**This fails closed and is not a live wrong answer.** `request.rs:4834`'s `a_contract_for_an_undeclared_arithmetic_type_is_unknown` pins that behaviour, and the direction is safe. The defect is that the refusal is structural rather than evidentiary: **no non-`f32` contract can be honoured whatever a profile declares**, and the rejection names an `Unknown` that no profile author can close by declaring anything.

Found while verifying the second cited defect of `redesign-the-delivered-realization-record-from-typed-evidence`, which is why it is filed rather than absorbed.

## What closes this

The requirement carries the resolved type the caller's contract is actually stated for, so the subject the requirement names is the subject a profile could declare. That likely means the contract carries a validated `ScalarArithmetic` rather than a bare `ArithmeticType`, which is a change to a crate-private type and its construction sites.

The existing `Unknown` test must be preserved in substance and re-derived: a contract for an arithmetic type the profile does not speak about must still be `Unknown` — by the profile's silence, not by an unmatchable resolved type. Perturb it to confirm it can still fail.

## Trigger check log

- 2026-08-05: not a deferral; filed dispatchable. Reproduce with `rg -n "F32::resolved_type\(\)" crates/tiler-compiler/src/policy.rs`.

## Closed 2026-08-07 — the defect was already fixed, verified at source

**The ticket's own reproduce command no longer reproduces.** `grep -n "F32::resolved_type()" crates/tiler-compiler/src/policy.rs` returns no match at `879dec67`. `dimension_requirements` now derives the subject from the caller's contract — `let Some(subject) = arithmetic_subject(contract.arithmetic) else { return Vec::new(); }` — where `arithmetic_subject` (`crates/tiler-compiler/src/policy.rs:695`) resolves the type through `registered_arithmetic_value_type` and builds it with the same `ScalarArithmetic::new` a profile declares against. That is exactly the "What closes this" requirement.

**Fixed by `6207fba4`** (2026-08-05, "Let a caller state a BF16 numerical contract so a measured BF16 row can refuse it"), whose message states the defect in this ticket's own terms: "every per-dimension requirement was stated for `tiler::f32@1` whatever arithmetic type the contract named". This ticket was filed the same day from a different reading and was never dispatched, so the fix landed under other work rather than under it.

**The `Unknown` obligation was met in substance, not merely left passing.** `a_contract_for_an_undeclared_arithmetic_type_is_unknown` (`crates/tiler-compiler/src/request.rs:8899`) survives and still reports `Unknown` for an `F16` contract against a silent profile. It is now `Unknown` *by the profile's silence* rather than by an unmatchable resolved type, which is the distinction this ticket required: the code comment beside `dimension_requirements` records that every arithmetic type resolves a subject "including the two this build registers no contract key for", deliberately, so that an unregistered width is reported `Unknown` rather than becoming vacuously feasible.

**Closed `obsolete` rather than `done`** because this ticket delivered nothing; the outcome exists but another change produced it. No dependent is orphaned: the only ticket referencing it, [`redesign-the-delivered-realization-record-from-typed-evidence`](redesign-the-delivered-realization-record-from-typed-evidence.md), carries it as a prose note recording that it "does not block acceptance", and that ticket is already `done`.

Verified by the coordinator by reading `policy.rs` and `request.rs` at `879dec67`, not relayed.
