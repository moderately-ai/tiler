---
id: derive-transformer-operation-and-shape-surface
title: Derive the transformer operation and shape surface from the selected workload
status: done
priority: p1
dependencies: [define-first-metal-lm-workload]
related: [own-operation-family-support-matrix, scope-einsum-contraction-support, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [contracts/foundation, contracts/navigation, research/shapes, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, semantics, shapes, transformer, language-model, breadth]
---
## User-visible outcome

We know, per operation and per shape, exactly what the selected LM workload requires of Tiler — the workload-backed delivery graph that replaces "transformers need matmul and softmax" with an evidence-backed inventory of every semantic operation, dtype, extent class, and state surface between model inputs and logits.

Trace the selected workload from model inputs through logits and derive the
exact tensor operation, dtype, shape, layout, and state surface it requires.
This ticket turns the general operation-family matrix into a workload-backed
delivery graph.

## Required analysis

- Inventory every semantic operation and every observable conversion.
- Record static, sourced, bounded-dynamic, and unsupported extent requirements.
- Identify broadcast, reindex, reshape/view, transpose, slice, concatenate,
  gather, mask, and layout requirements without conflating semantic operations
  with physical views or materializations.
- Decide which behaviors require atomic semantic identities and which are
  explicit graph compositions.
- Map each requirement to the operation-family maturity ladder and inspect the
  construction sites behind any apparently matching existing type.
- State unsupported cases as typed refusals rather than approximations.

## Ticket-producing outcome

File vertical tickets for each coherent missing family. A vertical must cover
the necessary semantic identity and validation, normative reference,
compiler/lowering capability, target realization, runtime binding, and bounded
conformance evidence. Do not file one ticket per crate when those crate edits
exist only to deliver one user-visible operation.

## Closes when

The selected workload has a complete, reproducible capability inventory; every
requirement maps to implemented support, an existing ticket, a newly filed
vertical, or an explicit deferral; and the operation-family matrix and ticket
graph agree.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L2** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L1 names a workload with exact shapes and dtypes.

**Rests on:** L1.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **The workload profile L1 delivered is [`docs/research/program-planning/first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md)**, and it is this rung's input: the pinned per-file SHA-256 manifest, the configuration table read from `config.json`, the 310-tensor inventory read from the checkpoint's own safetensors header, the two bounded rows and their extent classes, and a per-layer operation and shape trace taken from the pinned `transformers` v4.51.0 reference. That trace is a starting point rather than this rung's deliverable: it names families and shapes without deciding which need atomic semantic identities, which are graph compositions, or what each owes in validation, normative reference, lowering capability, target realization, and conformance evidence.
- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Derive every operation and shape from that exact revision rather than from a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).
