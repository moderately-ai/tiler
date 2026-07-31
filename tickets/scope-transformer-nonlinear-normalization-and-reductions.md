---
id: scope-transformer-nonlinear-normalization-and-reductions
title: Scope the workload's transformer nonlinear, normalization, and reduction families
status: in-progress
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface]
related: [implement-parallel-reduction-strategies, research-region-accuracy-contracts-and-analyzable-error-budgets, own-operation-family-support-matrix]
scopes: [research/numerics, contracts/numerics, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, transformer, normalization, softmax, language-model]
claimed_from: todo
assignee: loop-scope-l3
lease_expires_at: 1785523879
---
## User-visible outcome

Every nonlinear/normalization/reduction family the workload needs has an exact formula, dtype signature, and accuracy-or-order contract — with lookalikes separated (exact vs tanh-GELU, LayerNorm vs RMSNorm are different semantic operations), so a kernel author implements the operation the model actually uses.

Define the exact activation, normalization, softmax, masking, and reduction
families required by the selected workload. Similar names are not sufficient:
for example, exact and approximate GELU are different semantic operations, as
are LayerNorm and RMSNorm.

## Required analysis

- Give each required family an exact formula, dtype signature, conversion
  behavior, exceptional-value behavior, and accuracy or order contract.
- Derive softmax and normalization requirements from small tensor examples,
  including extrema reduction, exponentiation, accumulation, division or
  reciprocal, empty domains, masks, and materialization boundaries.
- Evaluate the Metal feasibility of required transcendental realizations using
  bounded source inspection or measurement.
- Separate a composite graph spelling from a justified atomic semantic
  operation and from a fused physical implementation.
- Identify which requirements are already covered by generic reduction,
  numerical-policy, and accuracy-contract work.

## Ticket-producing outcome

File coherent operation-family verticals—such as activation, normalization, and
softmax—rather than tickets organized around private modules. Each vertical
must include reference behavior, compiler legality, Metal realization,
explainable refusal, and bounded conformance evidence.

## Closes when

Every nonlinear, normalization, mask, and reduction requirement of the selected
workload has a precise contract or a named unresolved decision; Metal
feasibility boundaries are recorded; and all justified delivery work has
dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L3′** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L2 lists the non-linearities, normalization, and reductions the workload needs.

**Rests on:** L2.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Its initial normalization and nonlinear surface is RMSNorm, per-head Q/K RMSNorm, SwiGLU, masking, and softmax—not GPT-2 LayerNorm/GELU. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

- **Softmax and normalization are reductions** — their order/accuracy contracts feed `implement-parallel-reduction-strategies` (accumulation dtype, deterministic vs relaxed order). Cross-link findings there rather than duplicating the contract in two places.

## Delivered outcome (2026-07-31)

The derivation is [Transformer non-linear, normalization, and reduction contracts](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md). It gives softmax, RMS normalization (both extent classes), SiLU, causal-mask application, and the attention scale an exact formula, dtype signature, conversion behaviour, exceptional-value behaviour, and an order or accuracy contract, and names five unresolved decisions (D-1 to D-5) with what would close each. Two retained probes support it: [Metal transcendental emission](../spikes/numerics/metal_transcendental_emission/README.md) and [transformer reference semantics](../spikes/numerics/transformer_reference_semantics/README.md).

Three findings correct what a competent implementer would otherwise have assumed, and each was established by measurement rather than by reading a formula:

- The causal mask's fill value is the most negative **finite** F32 (`0xff7fffff`), not `-inf`, and an attended entry is **negative zero**. The two conventions disagree observably on a fully masked row — uniform against NaN.
- The reference softmax multiplies by the denominator's **reciprocal**; it does not divide. At row widths 2 and 3, where the denominator has no accumulation-order freedom left, every discriminating element matches the reciprocal form and none matches division.
- `x * sigmoid(x)` and `x / (1 + exp(-x))` are one ULP apart at `-88.0`; the reference matches the second. An earlier corpus without an input near the exponential's overflow threshold reported them identical.

Capability tickets filed, in dependency order: [`admit-the-silu-activation-family`](admit-the-silu-activation-family.md) → [`admit-the-rms-normalization-family`](admit-the-rms-normalization-family.md) → [`admit-the-softmax-family`](admit-the-softmax-family.md). Accumulation-dtype and order findings are cross-linked into [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) rather than duplicated. Masking files nothing: its two composed families are already delivered by [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) and `tiler::add-f32@1`, and what this rung adds to them is the mask's value rather than a new requirement.

**Two edits this rung owes and did not make**, because both files are held exclusively by other live tickets:

- `docs/roadmap.md` (`contracts/navigation`) — the L3′ ladder row's maturity cell still reads "none" and should read: `non-linear, normalization, and reduction contracts derived; three capability verticals filed; nothing executes`.
- `docs/research/README.md` (`contracts/navigation`) — the *Numerical operations* catalog group needs the line below, in alphabetical position after "Sound region-accuracy analyzer integration spike". It is fenced rather than inline because its link targets are relative to `docs/research/README.md` and would not resolve from this file:

```text
- [Transformer non-linear, normalization, and reduction contracts](numerics/transformer-nonlinear-normalization-and-reductions.md) — pending; primary-source-synthesis, bounded-measurement; informs: [Numerical semantics](../numerical-semantics.md), [Correctness and testing](../correctness-and-testing.md); experiments: [Metal transcendental emission probe](../../spikes/numerics/metal_transcendental_emission/README.md), [Transformer reference semantics](../../spikes/numerics/transformer_reference_semantics/README.md)
```
