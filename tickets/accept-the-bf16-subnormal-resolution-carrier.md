---
id: accept-the-bf16-subnormal-resolution-carrier
title: Accept the carrier that tells the BF16 reference its subnormal resolution
status: awaiting-decision
priority: p2
dependencies: []
related: [carry-a-bf16-subnormal-realization-the-reference-can-be-told, conform-the-bf16-vertical-end-to-end, declare-the-bf16-rows-on-the-authoritative-metal-profile, state-and-check-a-bf16-numerical-contract, land-the-bf16-conversion-and-accumulator-adr]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, numerics, reference, bf16]
---
## User-visible outcome

Tom picks which of two carriers supplies the BF16 reference's subnormal resolution, and the wiring lands as a small follow-up against a machinery that is already built and tested.

## Why this node exists

**Fact — the machinery is landed and neither arm is built into it.** [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md) delivered `Bf16SubnormalRealization` and the two application sites in `crates/tiler-reference/src/bf16.rs` on 2026-08-07, with a seven-case counterexample population watched failing in both directions. Every registered capability is constructed under `Bf16SubnormalRealization::preserving()`, so **nothing supplies a flushing realization**. The one line that would is `Bf16BinaryReference::evaluate`, and what it should read there is this node's question.

**Fact — the question is which subject a `SubnormalMode` speaks about, and it is a public boundary.** `NumericalRealization`'s `input_subnormals` and `result_subnormals` (`crates/tiler-ir/src/schedule/numerics.rs`) name no format. Every consumer today reads them as binary32 — `ReferenceNumericalConformance::apply_to_operand` and `apply_to_result` are `f32` functions. A BF16 evaluation that read those same two fields would be reading a binary32 statement as a BF16 one, or would be relying on a caller never to mean two formats at once.

## The two arms

### Arm A — derived at the point of use

The BF16 capability knows its own format by construction, so `Bf16BinaryReference::evaluate` reads `request.conformance()`'s two format-agnostic `SubnormalMode`s and builds a `Bf16SubnormalRealization` from them. No new field, no new public type, no `implementation/ir` edit.

- **Enables:** the whole vertical closes in one file. Scope set is `implementation/reference` alone; the change is roughly ten lines in `Bf16BinaryReference::evaluate` plus a mapping test.
- **Prevents:** nothing today. It is correct exactly while no program mixes widths — while a region's declared realization is a statement about *one* arithmetic type.
- **Strongest counterpoint:** [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) already decides the shape of a BF16/binary32 conversion family, and nothing registers one yet. The first registered conversion puts a binary32 arithmetic and a BF16 arithmetic in one region under one `NumericalRealization`, and the measured Apple9 row resolves the two formats **differently** (`f32` flushes, `f16` preserves, `bf16` flushes — findings 21 and 24). At that moment arm A silently applies one format's resolution to the other's values, which is the exact defect this ticket's non-goals forbid. The failure is silent, not typed.

### Arm B — declared as a subject on the realization

`NumericalRealization`'s two subnormal fields acquire a subject, the way the target profile's rows already do: `declare_metal_bf16_subnormal_behaviour` (`crates/tiler-build/src/metal_declaration.rs:848`) declares its input and result rows against `ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())`, and `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` (`crates/tiler-compiler/src/session.rs:1522`) is a registered contract that already resolves the dimensions per format on the caller side. The realization is the one place in the chain where the subject is still absent.

- **Enables:** survives ADR 0091's first conversion by construction, and closes the asymmetry that a *profile* row and a *contract* both name their arithmetic type while the realization between them does not. A BF16 evaluation asks for the BF16 subject and gets a typed absence rather than a binary32 answer.
- **Prevents:** it is an identity-domain change. `NumericalRealization` is folded into artifact identity (ADR 0076 item 4, landed 2026-08-05 by `wire-the-delivered-realization-record-into-the-artifact`), so widening it moves the delivered-realization record, its cross-check against the packaged entries, and every identity pin derived from them.
- **Strongest counterpoint:** it spends an identity-domain migration on a case no program can express — nothing registers a mixed-width program, and until one does the two arms return identical bits on every reachable input. It also widens a public type before any consumer needs it, which is the "premature crates and APIs harden unsupported assumptions" caution `AGENTS.md` states in terms.

## Recommendation

**Arm B, but not now — accept the shape and gate the landing on the first registered conversion.** The ground is asymmetric failure modes rather than a cost comparison: arm A's defect at the first mixed-width program is a silently wrong answer, and arm B's cost is a scheduled identity migration. A cheaper path that can silently return wrong results is a defect and not a trade-off. But arm B's cost is real and its trigger is not here yet, so the recommended shape is:

1. accept **arm B's subject** as the eventual carrier, so nothing is built that assumes format-agnostic realization fields are permanent;
2. land **arm A's wiring now** *with a refusal*: the BF16 capability reads the conformance and evaluates under it **only** when the region's arithmetic is single-format, and returns a typed refusal otherwise. That keeps the vertical closable today and makes the mixed-width case fail loudly rather than silently;
3. file arm B's identity migration against the first registered BF16/binary32 conversion.

If Tom prefers a single answer, arm A alone is defensible **only** with that refusal; arm A without it is the one shape this node recommends against.

## What the follow-up wiring is, under each arm

| | arm A | arm B |
| --- | --- | --- |
| files | `crates/tiler-reference/src/bf16.rs` (`Bf16BinaryReference::evaluate`), its tests | the above, plus `crates/tiler-ir/src/schedule/numerics.rs`, every `NumericalRealization` construction site, `crates/tiler-reference/src/conformance.rs`, the artifact's delivered-realization record and its identity pins |
| scopes | `implementation/reference` | `implementation/reference`, `implementation/ir`, `implementation/artifact`, `implementation/compiler`, `implementation/build` |
| identity | untouched | moves; owning version, ledgers, and pins must stay coherent |
| public boundary | none new if `Bf16SubnormalRealization` stays crate-internal | `NumericalRealization`'s field shape |

## Explicit non-goals

Re-deciding the reference machinery, which is landed. Widening the binary32 conformance object to stand in for a BF16 one. Any change to the exact-rational arithmetic or its single rounding. Registering a BF16/binary32 conversion, which is ADR 0091's and has its own work.

## Closes when

Tom names an arm, or names the staged shape above, and the wiring is released to its own implementation ticket rather than landed under this node.

## Graph maintenance

Filed 2026-08-07 by the worker on [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md), which delivered everything both arms share and was forbidden to self-accept the fork. That ticket stays `in-progress`: its branch is integrable and its close gates on this decision, because the [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) exception paragraph it must retire stays true until a route supplies the value. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is the dependent that needs the answer.
