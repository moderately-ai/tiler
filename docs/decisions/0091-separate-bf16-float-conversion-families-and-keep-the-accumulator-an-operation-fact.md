---
schema: "tiler-doc/v1"
id: "ADR-0091"
kind: "decision"
title: "Separate BF16 float conversion families and keep the accumulator an operation fact"
topics: ["numerics", "dtypes", "bf16", "conversion", "accumulator", "contraction"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.bf16-computation-accumulator-and-conversion"]
depends_on: ["ADR-0009", "ADR-0010", "ADR-0015", "ADR-0024", "ADR-0026", "ADR-0041", "ADR-0043", "ADR-0075", "ADR-0076", "ADR-0087"]
ticket: "land-the-bf16-conversion-and-accumulator-adr"
---

# 0091: Separate BF16 float conversion families and keep the accumulator an operation fact

**Status:** accepted. Tom accepted the derivation and this record at the morning review of 2026-08-01, unamended — the five decisions below are transferred verbatim from the drafted body the research record carried, and nothing was reworded at acceptance. The acceptance and the exact channel it arrived through are recorded under "Correction — 2026-08-01, before the work started" in [`land-the-bf16-conversion-and-accumulator-adr`](../../tickets/land-the-bf16-conversion-and-accumulator-adr.md), which is also where a reader can check it in one line; that ticket was dispatched to land this record `proposed` and the acceptance arrived first, which is the shape [ADR 0089](0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) took for an in-session ratification.

**What the acceptance covers, and what it deliberately does not.** Acceptance is of the *model* — where a wider accumulator lives, that BF16/F32 conversion is two directional families, that a pointwise mixed-width program spells its conversions explicitly, that no fused BF16 operation is admitted, and that a rejection is one of five typed outcomes owned by two authorities. It registers nothing. The six concrete public boundaries the research record lists under "Public-boundary consequences" — the two conversion operation keys and their names and versioning, the typed conversion-contract vocabulary, a future `(value type, accumulator type)` reducing key, the mixed-precision fact-field numbering, whether `ScalarArithmetic` gains a second subject, and the fifth refusal class — each still come to Tom at implementation time under [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md). No key is registered, no evaluator exists, no `docs/dtype-support.md` cell moved, and the `Cast and convert` row of the [roadmap matrix](../roadmap.md#operation-family-support-matrix) stays at R2.

This record states the decision and cites [BF16 computation, accumulator, and conversion](../research/numerics/bf16-computation-accumulator-and-conversion.md) for the derivation; it does not re-derive it. Every witness, population count, elimination, worked example, portable-versus-host-bound split, and measurement boundary lives there, and the evidence under them is [the BF16 second-dtype spike](../../spikes/numerics/bf16-second-dtype/README.md) at its own recorded base commit `59a2fe2` together with transcribed findings 24, 25, 26, 28, 29, and 30 of [the Apple numerical behaviour record](../research/apple-targets/numerical-behaviour.md), each quoted there with its own boundary.

## Context

The landed pure-BF16 family states computation, accumulator, intermediate-materialization, and result types as four separate fields, all `tiler::bf16@1`, so that any widening is an explicit edit. The first workload wanting BF16 storage with binary32 accumulation, or any BF16/F32 conversion, has no accepted answer, and the difference between a decided accumulator and an inherited one is invisible in the code and decisive in the results. Separately, no `bfloat` FMA exists in MSL on the measured toolchain row, so a fused BF16 operation has no primitive to lower to.

## Decision

1. **A wider accumulator is a property of the operation**, carried in its registered definition facts and therefore in its identity, as `tiler::strict-tensor-contraction-f32@1` already carries `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE`. It is never a schedule choice and never a dimension of the resolved numerical contract. A different accumulator type is a different registered operation key, not an attribute value under one key.
2. **BF16/F32 conversion is two separate typed families, one per direction.** The widening family carries no rounding, overflow, or NaN-mapping field, because BF16-to-binary32 widening is exact and total. The narrowing family carries round-to-nearest-ties-to-even, overflow to a signed infinity at the inclusive midpoint above the largest finite BF16 value, canonicalization of every NaN to `0x7fc0`, gradual underflow, and preserved signed zero. A contract carrying a field its direction does not have is refused at construction.
3. **A pointwise mixed-width BF16 program uses explicit conversion operations**, and an internal accumulator is admitted only inside a reduction or contraction, where a graph-level per-contributor conversion would encode the operation's internal scalar iteration in the public graph. This is an instance of the no-implicit-promotion rule and not an exception to it.
4. **No fused BF16 operation is admitted.** If a workload requires the promoted route, it is admitted as a mixed-precision operation whose facts state binary32 computation over exactly widened operands, one binary32 rounding at the fused step, and one narrowing to BF16 — under a name that does not imply single-rounding BF16 semantics.
5. **Every rejected combination returns one of five typed outcomes** — not defined, malformed request, defined but unimplemented, unhonourable on this target, or unknown on this target — with the first owned by the semantic registry and the last two by the declaring target profile, so a registry answer and a target answer can never be confused. A rejected conversion names its direction.

## Consequences

- Two new operation keys and a typed conversion-contract vocabulary enter the public boundary; the `Cast and convert` support-matrix row moves off R2 only when they are registered.
- A BF16 program can state binary32 accumulation without a new accumulating key, because widening is exact and BF16 products are exact in binary32, so the explicit spelling is bit-identical to an internal accumulator at a pointwise boundary.
- A BF16 contract declaring ADR 0015 contraction forbidden is currently unhonourable on the measured Apple runtime path, and that is an explicit typed refusal rather than a silent one.
- Truncating narrowing, payload-preserving NaN narrowing, and directed roundings remain separately admittable named families and are not reachable by relaxing this one.

## Alternatives considered

**The accumulator as a schedule property.** Rejected on correctness: two folds differing only in accumulator width return `0x3f80` and `0x3f81` on the same contributors in the same order, so a planner choosing between them would price meaning.

**The accumulator as a numerical-contract dimension.** Rejected structurally: the resolved contract speaks for exactly one arithmetic type and honourability is keyed by that subject, so an operation using two arithmetic types cannot be spoken for by one contract; and every contract dimension is a permission, which by construction does not change what an operation means.

**One float-to-float conversion family with a direction field.** Rejected under ADR 0010 and 0041: it makes "widening with ties-away" constructible, and the exactness result is precisely that no such thing exists.

**Admitting `tiler::fma-bf16@1` and letting the backend promote.** Rejected because the promotion is observably not the contract: it differs by one ulp on a derived witness and on 21,546 of 262,144 swept triples.

## Implementation boundary

**This record decided; nothing follows automatically.** `implementation_status` is `not-started` and every clause above is downstream of a seam that is still closed: `ScalarArithmetic` in `crates/tiler-compiler/src/target.rs` exposes one public constructor, `f32()`, so no BF16 numerical row can be stated on a compiler target profile at all, and [`admit-a-bf16-scalar-arithmetic-subject`](../../tickets/admit-a-bf16-scalar-arithmetic-subject.md) owns that. Item 2's two families have no registered key, no reference evaluator, no physical carrier, and no target row; item 1's reducing BF16 key does not exist and this record does not propose it for registration. Propagating any clause here into a contract as normative text beyond what [Numerical semantics](../numerical-semantics.md) already states is deliberate per-landing follow-up, not a default, and every public spelling stays Tom's under ADR 0075.

## Open questions

Four deferrals are carried by the research record with the evidence that would close each and its reconsideration trigger, and this record adopts them unchanged rather than restating them: whether an unfused-BF16 contract is honourable on any Apple runtime path, owned by [`probe-the-bf16-contraction-pragma-on-the-metal-runtime-path`](../../tickets/probe-the-bf16-contraction-pragma-on-the-metal-runtime-path.md) and triggered before any BF16 contract declaring contraction forbidden is offered to a Metal profile; whether a chain can separate native `bfloat` arithmetic from binary32-precision evaluation on a device, unowned because no contract depends on the answer; whether a truncating `f32 → bf16` narrowing is ever wanted, closing on a named producer or consumer; and whether the conversion family generalizes beyond BF16/F32, closing on the second float pair a workload selects, with the F16 vertical as its trigger.

One thing is deliberately *not* an open question here. Item 4's premise — that MSL has no `bfloat` overload of `fma` — is a compile failure on one recorded toolchain row and not a property of Metal or of `bfloat`. A future MSL revision adding the overload retires the premise, and that is item 4's own reconsideration trigger; until then the promoted route is the only expressible realization and it is measurably not the fused contract.

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md), which already carries the accumulator-observability rule and the widening/narrowing derivation this record decides, under the "Floating-point widening and narrowing, derived at the BF16/binary32 pair" section.
- **Evidence:** [BF16 computation, accumulator, and conversion](../research/numerics/bf16-computation-accumulator-and-conversion.md), whose reproductions are [the BF16 second-dtype spike](../../spikes/numerics/bf16-second-dtype/README.md) and the transcribed findings of [the Apple GPU numerical behaviour record](../research/apple-targets/numerical-behaviour.md).
- **Work record:** [`design-the-bf16-computation-and-accumulator-contract`](../../tickets/design-the-bf16-computation-and-accumulator-contract.md) for the derivation and [`land-the-bf16-conversion-and-accumulator-adr`](../../tickets/land-the-bf16-conversion-and-accumulator-adr.md) for this record and its acceptance.
- **Accepted authorities this record preserves rather than amends:** [ADR 0009](0009-resolved-numerical-typing.md) (internal mixed precision belongs to an operation's signature, and backend defaults never silently widen a program's numerical permissions), [ADR 0010](0010-typed-conversion-contracts.md) (a conversion contract defines only the fields relevant to its semantics), [ADR 0015](0015-fma-vs-contraction.md) (required FMA and optional contraction are different contracts), [ADR 0024](0024-initial-arithmetic-rounding.md) (round-to-nearest ties-to-even for initial arithmetic, which item 2's narrowing matches), [ADR 0026](0026-dtype-representability-vs-operation-support.md) (representability is not operation support), [ADR 0041](0041-separate-float-to-integer-conversion-families.md) (separate conversion families rather than one optional-field bag, and an exact conversion carries no rounding rule), [ADR 0043](0043-use-typed-phased-target-feasibility.md) (`Unknown` fails closed, which item 5's last row depends on), [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) (every public spelling named here is Tom's), [ADR 0076](0076-declare-target-honourable-numerical-realizations.md) (honourability is keyed by `(subject, dimension)`, which is why item 1 eliminates the contract), and [ADR 0087](0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) item 5 (a contraction's numerical signature is stated once, generically — the asymmetry that separates an index structure from an accumulator).
