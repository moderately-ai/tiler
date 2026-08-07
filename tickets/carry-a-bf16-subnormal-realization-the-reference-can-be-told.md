---
id: carry-a-bf16-subnormal-realization-the-reference-can-be-told
title: Carry a BF16 subnormal realization the reference can be told
status: in-progress
priority: p2
dependencies: []
related: [apply-the-declared-numerical-conformance-on-every-reference-evaluation-path, derive-the-oracle-for-a-permitted-divergence-candidate, conform-the-bf16-vertical-end-to-end, declare-the-bf16-rows-on-the-authoritative-metal-profile, state-and-check-a-bf16-numerical-contract]
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
tags: [numerics, reference, conformance, bf16]
claimed_from: todo
assignee: agent-bf16-subnormal
lease_expires_at: 1786079690
---
## User-visible outcome

A BF16 candidate compiled for a target that flushes BF16 subnormals is qualifiable against a reference that was told so, instead of against one whose only subnormal vocabulary is binary32.

## Both triggers have fired (2026-08-06); the section below is the state that deferred it

The two Facts marked **superseded** below were true when written and are false now. They are struck rather than deleted because the reason this ticket was deferred is the reason its deliverables are shaped as they are, and a reader who cannot see the original ground cannot judge whether the shape still fits.

## Why this was deferred rather than open

**Fact — the conformance object's two dimensions are binary32 functions.** `ReferenceNumericalConformance::apply_to_operand` and `apply_to_result` (`crates/tiler-reference/src/conformance.rs`) take and return `f32`, and the BF16 family performs no binary32 arithmetic to apply them to: its operands are exact rationals decoded from BF16 encodings and its one rounding is over BF16's value set. So the object cannot reach this family, and threading it there would be applying a format's rule to values not in that format.

**Fact — the behaviour is declared rather than unstated, and it is declared as preservation.** `BF16_FACT_SUBNORMALS` resolves to `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed` (`crates/tiler-ir/src/semantic/bf16.rs:252`) and the value contract to `preserved-every-subnormal-encoding-denotes-a-distinct-constant` (`:207`). The reference realizes exactly that, so nothing is silently resolved today.

~~**Fact — no target Tiler compiles for has been measured to flush BF16 subnormals, and one measured row preserves the narrower format.** `crates/tiler-reference/src/conformance.rs`'s header records the measured Apple behaviour as flushing in `f32` "while preserving them in `f16`". BF16 is not `f16` and that row is not evidence about it, which is the gap this ticket would close and the reason it is not a claim either way.~~ **Superseded 2026-08-06.** Finding 24 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures BF16 arithmetic flushing on the macOS row across all seven flush dimensions, and `declare_metal_bf16_subnormal_behaviour` (`crates/tiler-build/src/metal_declaration.rs:768`) projects that measurement into declared input and result subnormal rows against `ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())` on the authoritative profile. The target this reference is compared against is now measured *and* declared to flush.

~~**Inference — so the declared preservation is currently a fact about every reachable target, and a realization vocabulary for a case nobody can compile would be a type-system reservation dressed as support.**~~ **Superseded 2026-08-06 by the same two facts.** The declared preservation is now a fact about *no* reachable target's BF16 arithmetic, which inverts the inference: it is the reference, not the vocabulary, that is now the reservation.

## Trigger check log

- 2026-08-05 — **not fired.** No BF16 target row is measured, and no registered numerical contract carries a per-format subnormal resolution. Reproduce the second half with `grep -rn 'BF16\|bf16' crates/tiler-ir/src/schedule/numerics.rs` (empty: `NumericalRealization`'s subnormal fields are format-agnostic and are read as binary32 by every consumer).
- 2026-08-06 — **fired, both conditions independently.** The command the line above records as empty now returns 49 matching lines, and what it returns is the second condition met rather than incidental mentions: `ArithmeticType` (`crates/tiler-ir/src/schedule/numerics.rs:354`, `Bf16` at `:358`) names `Bf16` as a subject a behaviour is declared *for*, `BF16_NUMERICAL_CONTRACT_KEY_DOMAIN` renders `bf16` contracts under their own closed grammar, and `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` (`crates/tiler-compiler/src/session.rs:1522`) is a registered contract resolving the subnormal dimensions per format. The first condition is met separately by the authoritative profile declaration cited above. Reproduce the whole verdict in one line: `grep -c 'BF16\|bf16' crates/tiler-ir/src/schedule/numerics.rs && grep -n 'declare_metal_bf16_subnormal_behaviour' crates/tiler-build/src/metal_declaration.rs` — a nonzero count beside a declaring function is both conditions. Moved to `todo` and to p2 by the worker on `conform-the-bf16-vertical-end-to-end`, which is blocked on this ticket's deliverable and has been given the dependency edge.

## Trigger

Either of:

- a target profile declares BF16 arithmetic that flushes subnormals, or a measurement on a qualified row observes it; or
- a registered numerical contract resolves a subnormal dimension per format rather than once for the region.

## What this ticket must produce

- A declaration that names *which format* a subnormal resolution speaks about, so `NumericalRealization`'s two fields stop being implicitly binary32. This is a public boundary and is Tom's, not self-accepted. **The fork a dispatcher should brief explicitly**, since it is what makes this a decision rather than a mechanical widening: the format can be *derived* at the point of use — a BF16 capability knows its own format by construction, so it could read the format-agnostic `SubnormalMode` off the conformance and apply it at its own rounding boundary with no new field — or it can be *declared*, adding a subject to `NumericalRealization`'s two fields. The derived route needs no public boundary and no `implementation/ir` edit, and it is correct exactly while no program mixes widths; the declared route survives the first admitted BF16/binary32 conversion, which [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) already decides the shape of but nothing registers. Neither is dominant on correctness today, which is why it reaches Tom rather than being settled by a worker.
- A reference that can be told it, applied at the BF16 rounding boundary where the family's own arithmetic commits a value.
- The counterexample population: a BF16 operand and a BF16 product in the subnormal range whose preserving and flushing readings differ, with exact encodings.
- The declared fact updated from an unconditional `preserved` to whatever the realization vocabulary makes it, and the reference's `bf16` module header — which currently names this ticket as where the gap lives — updated with it.

## Explicit non-goals

Widening the binary32 conformance object to stand in for a BF16 one; approximating BF16 flushing with the binary32 modes; any change to the exact-rational arithmetic or its single rounding.

## Closes when

The trigger has fired, a BF16 subnormal realization is declared and accepted, and the reference applies it with a case watched failing.

## Graph maintenance

Filed by [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md), which had to decide what the BF16 capabilities do with a conformance they cannot use. Filed `deferred` rather than `todo` because its triggers had not fired and the board must not offer non-work; moved to `todo` on 2026-08-06 when both fired, per the log above.

- `implementation/ir` is required only by the *declared* route in the fork above. A dispatcher choosing the derived route can run this on `implementation/reference` alone; the scope stays declared either way, because the fork is not resolved until Tom resolves it and a brief must not pre-commit the scope set to one arm.
- The exception this gap creates in [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) — that the declared-contract comparison rule has one family that cannot follow it — was recorded on 2026-08-06 by `conform-the-bf16-vertical-end-to-end`. Closing this ticket must retire that paragraph in the same change, or it becomes a stale disclosure of a gap that no longer exists.
