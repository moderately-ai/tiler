---
id: accept-the-bf16-subnormal-resolution-carrier
title: Accept the carrier that tells the BF16 reference its subnormal resolution
status: done
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

Filed 2026-08-07 by the worker on [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md), which delivered everything both arms share and was forbidden to self-accept the fork. That ticket now reads `blocked` on this node rather than `in-progress` — corrected 2026-08-07 by the coordinator against the file, which carries `status: blocked` and `dependencies: [accept-the-bf16-subnormal-resolution-carrier]`. The edge is the accurate shape: its branch is integrable and its close gates on this decision, because the [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) exception paragraph it must retire stays true until a route supplies the value. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is the dependent that needs the answer.

## Decided — arm A, 2026-08-07

**Tom answered on 2026-08-07 in the coordination session, witnessed first-hand by the coordinator**, after asking for a full-source assessment rather than a decision from this node's summary. The answer is **arm A — derive the format at the point of use — with no mixed-width refusal**, and arm B closed against a named reopening trigger rather than parked as the eventual carrier.

### What the full reads changed, and why this node's own recommendation was wrong

This node recommended the staged shape: accept arm B's subject, land arm A's wiring behind a refusal. Reading the source in full refuted the reasoning behind it on three counts.

**1. Arm B's consumer does not exist.** `ReferenceNumericalConformance::from_realization` — the designed bridge from a region's declared realization to a reference conformance, and the only thing that would ever carry a subject — has **no caller anywhere** in `crates/` or `prototypes/`. Every construction site in the workspace is `strict()` or a test's `new()`. So no production path derives a reference conformance from a realization at all, and arm B would spend an identity-domain migration on a field that nothing reads. `NumericalRealization` is folded into artifact identity, so that migration is irreversible; this node's own text calls arm B's cost real, and the ground for not paying it now is that the path which would exercise it is unbuilt.

**2. Arm B would not inoculate against the hazard it was chosen for.** The hazard is two arithmetic types under one `NumericalRealization`. What prevents that today is `region_arithmetic_type` (`crates/tiler-ir/src/schedule/model.rs:1333`), a total function from `ScalarProgram` to exactly one `ArithmeticType` — reaching the hazard requires a new fused mixed-arithmetic variant. At that moment `canonical_arithmetic_nan_bits` breaks in the same instant, because it is one `u32` per region. Arm B would have subject-tagged two of the three format-bearing fields and left the third, so the same decision would still arrive, now with a migration already spent.

**3. The identical question was already answered in this struct, the other way.** `canonical_arithmetic_nan_bits` (`crates/tiler-ir/src/schedule/numerics.rs:238`) faced "does this field need to name its format?" on 2026-08-06 and answered no in two layers independently — schedule and artifact ABI — because "the arithmetic type that fixes how many of these bits are significant is already a total function of the region's scalar program", with agreement enforced by the schedule verifier at `crates/tiler-ir/src/schedule/builder.rs:664`. Arm B would make one record carry its subject two different ways.

**The refusal this node recommended is dropped.** A multi-format region cannot be constructed, so a "single-format region" refusal is unreachable and can never be watched failing. This repository treats an unfireable check as a defect in its own right — both realization-law acceptance nodes flag exactly that as the thing to object to — so adding one buys no safety and makes a maturity claim the evidence cannot support.

### Where the guard goes instead

`from_realization` **discards the subject deliberately**: its destructuring reads `canonical_arithmetic_nan_bits: _` (`crates/tiler-reference/src/conformance.rs:171`). That is the boundary at which format information is lost, and `crates/tiler-reference/src/registry.rs:181` then documents the resulting object as being for "a capability that performs host binary32 arithmetic" — structurally format-agnostic, documented as binary32. The mismatch is there, not inside the BF16 family.

So the obligation is recorded where it can actually fire: **when `from_realization` gains its first caller, that call must carry the arithmetic subject, and each capability must check the conformance it was handed matches its own format.** Handing a BF16 capability an `f32`-derived conformance is constructible in a test, so that check is watchable — which the refusal this node proposed is not. Recorded on [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md), which is the node that owns `from_realization`'s first caller.

> **Corrected 2026-08-07.** This sentence originally named `apply-the-declared-numerical-conformance-on-every-reference-evaluation-path` as the recording site. That was wrong and was caught by the worker on the wiring ticket: that node is `done` and carries no such obligation, so the reference pointed at a closed ticket that would never have discharged it. Adding it there would have been outcome expansion on a closed node; the obligation was always filed on the bridge ticket named above.

### The window this leaves open, stated rather than smoothed over

Between arm A's wiring landing and the subject check landing, the BF16 family could in principle be told a resolution stated for another format. That window is real. It is currently **unreachable**: every conformance in the tree is `strict()`, so there is no other format's rule to be told. It closes when the reference path lands with its subject check.

### Released work

- The wiring: [`wire-the-bf16-reference-to-the-realization-it-is-told`](wire-the-bf16-reference-to-the-realization-it-is-told.md), per this node's rule that accepted work lands under its own ticket rather than here.
- Arm B, closed against a trigger: [`subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types`](subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types.md).
