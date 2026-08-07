---
id: wire-the-bf16-reference-to-the-realization-it-is-told
title: Wire the BF16 reference to the realization it is told
status: todo
priority: p2
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, carry-a-bf16-subnormal-realization-the-reference-can-be-told, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, bf16]
---
## User-visible outcome

A BF16 evaluation performed under a flushing contract returns the flushing answer, so a BF16 candidate compiled for the measured Apple9 row can be qualified against a reference that was told what that row does.

## The decision this implements

**Arm A, accepted by Tom on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, on [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md), which carries the full reasoning and the two arms it was chosen between. The format is **derived at the point of use** — the BF16 capability knows its own format by construction — rather than declared as a new subject on `NumericalRealization`. **No `implementation/ir` edit, no identity move.**

## The work, which is small because the machinery landed

Everything this needs already exists in `main`:

- `Bf16SubnormalRealization` and its two application sites, `Bf16Format::accept_operand` (before the decode) and `Bf16Format::commit` (after the single rounding), in `crates/tiler-reference/src/bf16.rs`.
- `Bf16BinaryReference::combine_under`, which takes a realization per evaluation.
- `ReferenceEvaluationRequest::conformance()` (`crates/tiler-reference/src/registry.rs:199`), already carried on every request.

What is missing is one link: `impl ReferenceOperation for Bf16BinaryReference::evaluate` calls `self.combine(left, right)`, and `combine` delegates with `Bf16SubnormalRealization::preserving()`. **Build the realization from `request.conformance()`'s two `SubnormalMode`s and call `combine_under` instead.**

Read [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md)'s decision section in full before starting — it records why a mixed-width refusal is deliberately *not* part of this, and adding one would reintroduce an unreachable check the decision rejected.

## What this must not do

- **No mixed-width refusal.** A multi-format region cannot be constructed — `region_arithmetic_type` (`crates/tiler-ir/src/schedule/model.rs:1333`) is a total function from `ScalarProgram` to one `ArithmeticType` — so such a refusal is unreachable and cannot be watched failing. The decision drops it explicitly.
- **No subject added to `NumericalRealization`.** That is arm B, closed against a trigger in [`subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types`](subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types.md).
- **No change to `BF16_FACT_SUBNORMALS`.** Its unconditional `preserved-…` states what the operation *means*; a flushing realization is a declared deviation a region's contract carries, not a second opinion about semantics. Weakening it is the authority substitution ADR 0076 forbids, and it would move the registry snapshot and every identity derived from it.
- **No widening of the binary32 conformance object to stand in for a BF16 one**, and no approximation of BF16 flushing with the binary32 modes. Read only the two format-agnostic `SubnormalMode` values; the `f32` appliers are not for this family.
- No change to the exact-rational arithmetic or its single rounding.

## Required evidence

- A BF16 evaluation under a flushing conformance returns the flushing answer, and the same evaluation under `strict()` returns the preserving one — driven through `ReferenceOperation::evaluate`, not by calling `combine_under` directly, since the link being added is precisely the one `evaluate` was missing.
- Watched failing: revert the link so `evaluate` reaches `preserving()` again, observe the flushing case fail, restore. Capture both outputs.
- The seven-case counterexample population in `crates/tiler-reference/src/bf16/tests.rs` still passes unchanged — those cases pin `combine_under` directly and must not move.
- No pinned identity moves. Confirm rather than assume: the declared facts are untouched, so nothing should.

## Also owed, and easy to miss

[Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) carries an **exception paragraph** recording that the declared-contract comparison rule has one family that cannot follow it, reproduced with `grep -n conformance crates/tiler-reference/src/bf16.rs`. That exception stops being true when this lands, and [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md) states in terms that closing the fork must retire it in the same change or it becomes a stale disclosure of a gap that no longer exists. `contracts/numerics` is required for that file — add the scope.

The `bf16` module header also names the fork as parked and must be updated to record what was decided.

## Closes when

`ReferenceOperation::evaluate` applies the realization it is told, the flushing case is watched failing under the preserving link, the correctness-and-testing exception paragraph is retired, the module header states the resolved decision, and the package's checks pass.
