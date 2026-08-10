---
id: decide-the-general-accumulation-width-contract-for-reductions-and-contractions
title: Reconcile accumulation-width records with the accepted operation-fact contract
status: review
priority: p2
dependencies: []
related: [implement-parallel-reduction-strategies, admit-the-rms-normalization-family, admit-the-softmax-family, scope-transformer-nonlinear-normalization-and-reductions, design-the-bf16-computation-and-accumulator-contract]
scopes: [research/numerics, contracts/numerics, research/scheduling, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, reduction, contraction, accumulator, widening]
claimed_from: todo
assignee: sol-accumulation-width-contract
lease_expires_at: 1786404386
---
## User-visible outcome

A source-first reconciliation showing that the purported open decision was already accepted: accumulator width is a registered operation fact in identity, never a schedule choice or a resolved numerical-contract dimension. The L3′ D-5 and L3/L4 D-6 records now preserve their workload evidence while pointing to that authority and the already-registered F32 facts. No operation key, public boundary, numerical permission, or implementation changes here.

## Source-first audit at `d5d5136eab64161533b61158a63d78a5a02cb5a5`

The ticket's stated Facts were audited before any edit, against the exact source anchors below.

- **Verified — the parallel-reduction rollup is `done`.** [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) has `status: done` and its outcome anchor `Criteria 1–7 stay met as closed` preserves that boundary.
- **Imprecise — accumulation is not an explicit field on every topology.** `ReductionTopology::MultiPass` and `ReductionTopology::CooperativeWorkgroup` carry `accumulation`; serial and contraction schedules derive their region arithmetic type. The accurate claim is that the two parallel topologies make accumulation explicit and every reducing region is checked against its operation-derived requirement.
- **Verified — mismatch is intrinsic schedule invalidity.** `verify_accumulation_width` derives the required region arithmetic type and rejects `declared != required`; both parallel topology gates call it. `ScheduledRegionDiagnostic::AccumulationWidth` identifies rule `accumulation-width` and explicitly covers both narrower and wider declarations.
- **Verified — equality-only verification admits no widening schedule.** A wider declaration is refused by the same intrinsic check as a narrower one. This is correct for the registered operations rather than evidence that policy is missing.
- **False — the general accumulator-width policy is undecided.** [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md), at `A wider accumulator is a property of the operation`, says it is carried in registered definition facts and identity, is never a schedule choice or contract dimension, and a different width requires a different registered operation key. [Numerical semantics](../docs/numerical-semantics.md), at `the accumulator's width is observable`, carries the same rule normatively.
- **Verified — D-5's two sums are already declared.** `RMS_NORM_F32_FACT_ACCUMULATOR_TYPE` and `SOFTMAX_F32_FACT_ACCUMULATOR_TYPE` both register `tiler::f32@1`. The softmax maximum correctly has no accumulator fact because it selects an input value rather than accumulating partial arithmetic.
- **False — D-5 retains a general half after those declarations.** The family facts consume the workload question; ADR 0091 supplies the general rule. Two declarations alone would not have established policy, but the accepted ADR does.
- **Verified, then imprecise — D-6's extents are evidence, not an identity selector.** L3 measures structure-1 folds of 1,024–3,072 contributors. L4 correctly extends the workload maximum to structure 3's symbolic `S`, 8,192 at B1-d prefill and 8,320 at the end of decode, with different conditioning. Calling either population evidence that D-6 remains open confuses workload sensitivity with operation meaning.
- **False — the contraction accumulator remains undecided.** `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE` registers `tiler::f32@1` for `tiler::strict-tensor-contraction-f32@1`; its normative reference decodes and enforces that fact. [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), at decision item 5, requires computation precision, accumulator precision, contributor order, seed, intermediate canonicalization, and result rounding to be explicit in the registered definition.
- **Imprecise — ADR 0091 is not only BF16/F32 ownership.** The conversion-family decisions remain with [`design-the-bf16-computation-and-accumulator-contract`](design-the-bf16-computation-and-accumulator-contract.md), but ADR 0091 item 1 states the general location and identity rule for every wider accumulator.

Repairing the false general-open Fact changes this ticket's purpose. The remainder is authority and navigation repair, not a second numerical decision.

## Authority chain and reconciled answers

1. **A wider accumulator is not a schedule alternative for an existing operation.** It denotes a different operation and therefore needs a different registered key and definition facts. This ticket neither proposes nor registers one.
2. **D-5 and D-6 are consumed for the selected workload operations.** RMS normalization, softmax's denominator, and strict F32 contraction each declare F32 accumulation. Reference reproduction, sensitivity, and contributor extents remain bounded evidence about those operations; they cannot select different semantics for them.
3. **The answer lives in registered operation facts and identity.** [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) is the accepted general authority; [Numerical semantics](../docs/numerical-semantics.md) is the normative contract restatement; [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) and the registered RMS-normalization, softmax, and contraction definitions instantiate it. A topology declares or derives the operation-required width so verification can reject a malformed realization; it never chooses the width.
4. **The existing typed schedule refusal remains exact.** A schedule whose region arithmetic disagrees with its registered operation continues to return `ScheduledRegionDiagnostic::AccumulationWidth` / `accumulation-width`, whether narrower or wider. If a future workload requests a genuinely different accumulating operation whose key is absent or unsupported, that is an operation registration/admission or target-support outcome, not a widening exception under the current key.

The authority order is accepted ADRs and normative contracts, then inspected registered definitions and their reference/verification consumers, then research measurements. The stale ticket and research ownership prose cannot override that chain.

## Scope repair

`research/scheduling` is added because L3 D-6 and the schedule verifier's topology semantics require correction. `research/program-planning` is added because L4 supplies the corrected longest-extent and conditioning evidence. These scopes authorize document reconciliation only; they do not widen this ticket into physical-planning or implementation work.

## Non-goals

- Reopening the seven close criteria of [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md).
- Accepting another ADR, registering an operation key, or changing a public boundary or identity.
- Implementing a widening strategy or changing the equality verifier.
- Forking BF16/F32 conversion ownership from [`design-the-bf16-computation-and-accumulator-contract`](design-the-bf16-computation-and-accumulator-contract.md).

## Result and remaining unknowns

No general accumulator-width question remains open. The only future unknown is population-specific: whether a named producer and consumer need a different registered operation with a different accumulator fact, and whether a target can honour that new operation. That question fires only with such a named workload request. It would require Tom's approval for the operation-key and public-boundary consequence under ADR 0075; it is not deferred work owned by this ticket.

This ticket ends as an authority-aligned repair: the L3′ D-5 and L3/L4 D-6 current conclusions point to ADR 0091 and registered operation facts; their contributor counts, conditioning, and sensitivity measurements remain intact; and `accumulation-width` remains the intrinsic schedule-mismatch refusal. No experiment is needed because an experiment cannot decide operation identity over accepted authority.

## Graph maintenance

- The closed rollup remains closed; its latest residual-routing correction is repaired to point here as the audit carrier, not as a new policy owner.
- BF16/F32 mixed-precision and conversion work remains under [`design-the-bf16-computation-and-accumulator-contract`](design-the-bf16-computation-and-accumulator-contract.md) and ADR 0091's unregistered public-boundary consequences.
- Enforcers restart restatement remains owned by [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md), not this ticket.
