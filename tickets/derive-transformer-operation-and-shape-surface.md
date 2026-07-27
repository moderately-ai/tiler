---
id: derive-transformer-operation-and-shape-surface
title: Derive the transformer operation and shape surface from the selected workload
status: todo
priority: p1
dependencies: [define-first-metal-lm-workload]
related: [own-operation-family-support-matrix, scope-einsum-contraction-support, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [contracts/foundation, contracts/navigation, research/shapes, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, semantics, shapes, transformer, language-model, breadth]
---
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
