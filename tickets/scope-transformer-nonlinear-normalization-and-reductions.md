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
