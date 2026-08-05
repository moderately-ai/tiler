---
id: key-numerical-requirements-by-the-contract-s-own-resolved-type
title: Key numerical requirements by the contract's own resolved type
status: todo
priority: p2
dependencies: []
related: [redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, defect]
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
