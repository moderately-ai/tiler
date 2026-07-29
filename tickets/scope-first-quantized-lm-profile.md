---
id: scope-first-quantized-lm-profile
title: Scope the first workload-backed quantized language-model profile
status: todo
priority: p2
dependencies: [define-first-metal-lm-workload, spike-first-metal-contraction-vertical, prototype-quantized-value-vertical]
related: [implement-first-quantized-backend-profile, define-initial-affine-quantization-semantics, define-quantized-value-binding-contract, implement-workload-selected-quantized-parameter-maps]
scopes: [research/numerics, research/scheduling, research/apple-targets, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, quantization, language-model, matmul, metal]
---
## User-visible outcome

The first quantized LM profile is *chosen from evidence* — workload memory/accuracy needs against measured contraction behaviour — instead of a format picked by fashion. The choice arrives with its elimination record, so it does not get re-litigated per-format later.

Use the selected workload and measured contraction evidence to choose the first
quantized language-model profile. This ticket must not select a format before
the model, target, numerical behavior, and performance evidence make that
choice meaningful.

## Required analysis

- Compare candidate weight and value representations against the workload's
  memory, accuracy, packing, and Metal execution requirements.
- Define code, scale, zero-point, grouping, axis, layout, and conversion
  identity for every surviving candidate.
- Determine whether contraction consumes packed values directly or through an
  explicit dequantization boundary.
- Define the normative reference, accumulation behavior, output dtype, error
  criteria, artifact identity, weight validation, and runtime binding.
- Measure memory and performance against the non-quantized baseline on the
  selected target where feasible.

Eliminate any profile that cannot be validated or whose numerical realization
is unknown. A smaller artifact is not by itself evidence of a correct or faster
model.

## Ticket-producing outcome

Activate and refine `implement-first-quantized-backend-profile` for the selected
profile, or supersede it with narrower delivery tickets. File any additional
work for weight ingestion, packed contraction, conversion, conformance, and
model-level comparison with exact dependencies and scopes.

## Closes when

One bounded profile is selected from reproducible evidence or every candidate
is rejected with explicit reasons; the generic quantized-value reservation is
connected to a model-visible execution path; and all surviving work has
dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L7** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L1 and L3 deliver **and** milestone 2Q supplies the quantized-value vertical proof.

**Rests on:** L1, L3, and milestone 2Q.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **These rungs consume Tom's workload selection** (`define-first-metal-lm-workload`, awaiting-decision). If the workload changes after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

- **This consumes `prototype-quantized-value-vertical`'s answer** (is quantization a dtype or a compound contract) and `spike-first-metal-contraction-vertical`'s measurements — check both closed before starting, and cite their results rather than re-arguing them.
