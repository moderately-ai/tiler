---
id: scope-first-quantized-lm-profile
title: Scope the first workload-backed quantized language-model profile
status: in-progress
priority: p2
dependencies: [define-first-metal-lm-workload, spike-first-metal-contraction-vertical, prototype-quantized-value-vertical]
related: [implement-first-quantized-backend-profile, define-initial-affine-quantization-semantics, define-quantized-value-binding-contract, implement-workload-selected-quantized-parameter-maps, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics, research/scheduling, research/apple-targets, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, quantization, language-model, matmul, metal]
claimed_from: todo
assignee: worker-quant-profile
lease_expires_at: 1785546674
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
- Classify every selected logical type, compound scheme, component, storage carrier/encoding, kernel access/arithmetic type, and target-family dispatch fact in the dtype maturity ledger. A selected code width, packed layout, or native instruction cannot stand in for the other two.
- Name the exact physical-vocabulary widening required by the surviving profile. File it as a separate dependency of backend implementation with signature verification, KIR identity, ABI compatibility, target dispatchability, lowering/emission, and negative unsupported-combination tests; adding a carrier enum variant alone is not executable support.
- Separate a correctness-only execution proof from a device-optimal claim. Activate profile-specific analytical and measured cost work for packed/unpacked, explicit-dequantize, and fused candidates, keep unmeasured components `Unknown`, and make calibrated evidence a structural dependency before the selected route is described as optimal.

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

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Quantization must be selected against that exact model and its F32 baseline rather than against a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **The selected backend graph must be complete before activation.** Link the exact physical-vocabulary ticket, `admit-a-dtype-dispatchability-capability-axis`, `group-internal-compound-materializations-by-logical-value`, `implement-workload-selected-quantized-parameter-maps` when the profile is non-per-tensor, `implement-first-runtime-semantic-value-precondition-enforcement` when the valid domain has runtime value predicates, and profile-specific cost calibration before any device-optimal claim.
- **Update `own-the-dtype-support-maturity-matrix` from evidence.** Advance only the cells the selected profile actually implements or tests; leave neighbouring widths, schemes, layouts, operations, targets, and runtime paths absent or reserved.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

- **This consumes `prototype-quantized-value-vertical`'s answer** (is quantization a dtype or a compound contract) and `spike-first-metal-contraction-vertical`'s measurements — check both closed before starting, and cite their results rather than re-arguing them.
