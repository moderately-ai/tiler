---
id: carry-a-bf16-subnormal-realization-the-reference-can-be-told
title: Carry a BF16 subnormal realization the reference can be told
status: deferred
priority: p3
dependencies: []
related: [apply-the-declared-numerical-conformance-on-every-reference-evaluation-path, derive-the-oracle-for-a-permitted-divergence-candidate]
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
tags: [numerics, reference, conformance, bf16]
---
## User-visible outcome

A BF16 candidate compiled for a target that flushes BF16 subnormals is qualifiable against a reference that was told so, instead of against one whose only subnormal vocabulary is binary32.

## Why this is deferred rather than open

**Fact — the conformance object's two dimensions are binary32 functions.** `ReferenceNumericalConformance::apply_to_operand` and `apply_to_result` (`crates/tiler-reference/src/conformance.rs`) take and return `f32`, and the BF16 family performs no binary32 arithmetic to apply them to: its operands are exact rationals decoded from BF16 encodings and its one rounding is over BF16's value set. So the object cannot reach this family, and threading it there would be applying a format's rule to values not in that format.

**Fact — the behaviour is declared rather than unstated, and it is declared as preservation.** `BF16_FACT_SUBNORMALS` resolves to `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed` (`crates/tiler-ir/src/semantic/bf16.rs:252`) and the value contract to `preserved-every-subnormal-encoding-denotes-a-distinct-constant` (`:207`). The reference realizes exactly that, so nothing is silently resolved today.

**Fact — no target Tiler compiles for has been measured to flush BF16 subnormals, and one measured row preserves the narrower format.** `crates/tiler-reference/src/conformance.rs`'s header records the measured Apple behaviour as flushing in `f32` "while preserving them in `f16`". BF16 is not `f16` and that row is not evidence about it, which is the gap this ticket would close and the reason it is not a claim either way.

**Inference — so the declared preservation is currently a fact about every reachable target, and a realization vocabulary for a case nobody can compile would be a type-system reservation dressed as support.**

## Trigger check log

- 2026-08-05 — **not fired.** No BF16 target row is measured, and no registered numerical contract carries a per-format subnormal resolution. Reproduce the second half with `grep -rn 'BF16\|bf16' crates/tiler-ir/src/schedule/numerics.rs` (empty: `NumericalRealization`'s subnormal fields are format-agnostic and are read as binary32 by every consumer).

## Trigger

Either of:

- a target profile declares BF16 arithmetic that flushes subnormals, or a measurement on a qualified row observes it; or
- a registered numerical contract resolves a subnormal dimension per format rather than once for the region.

## What this ticket must produce, once fired

- A declaration that names *which format* a subnormal resolution speaks about, so `NumericalRealization`'s two fields stop being implicitly binary32. This is a public boundary and is Tom's, not self-accepted.
- A reference that can be told it, applied at the BF16 rounding boundary where the family's own arithmetic commits a value.
- The counterexample population: a BF16 operand and a BF16 product in the subnormal range whose preserving and flushing readings differ, with exact encodings.
- The declared fact updated from an unconditional `preserved` to whatever the realization vocabulary makes it, and the reference's `bf16` module header — which currently names this ticket as where the gap lives — updated with it.

## Explicit non-goals

Widening the binary32 conformance object to stand in for a BF16 one; approximating BF16 flushing with the binary32 modes; any change to the exact-rational arithmetic or its single rounding.

## Closes when

The trigger has fired, a BF16 subnormal realization is declared and accepted, and the reference applies it with a case watched failing.

## Graph maintenance

Filed by [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md), which had to decide what the BF16 capabilities do with a conformance they cannot use. Filed `deferred` rather than `todo` because its triggers have not fired and the board must not offer non-work.
