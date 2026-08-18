---
id: accept-the-tensor-contraction-successor-public-surface
title: Accept the tensor-contraction successor public surface
status: todo
priority: p1
dependencies: [decide-the-semantic-order-contract-for-relaxed-contractions]
related: [admit-reassociated-contraction-schedule-alternatives]
scopes: [implementation/ir, implementation/reference, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Tom accepts the exact included and excluded Rust surface of the accepted `tiler::tensor-contraction-f32@1` replacement — the thirteen-field typed definition descriptor (field 15 reduction record, field 14 stability record), `ContractionF32ReductionDescriptor` and its decode path, `EffectiveContractionF32Profile` and its resolver, `OrderedContractionF32Tree`/`ContractionF32PlanWitness`, and the bounded `ContractionF32TopologyEvaluator` with its caller-owned budget — exactly as drafted and independently reviewed in `decide-the-semantic-order-contract-for-relaxed-contractions` (packet audited at `368dcd25`, reviewed at `5a48c9ce`).

## Why this exists

The 2026-08-18 acceptance chose the complete key replacement and reassociation-only semantic contract; the packet's own downstream list makes the exact public surface a separate acceptance before implementation. This node carries that question to Tom; it authorizes no production edit itself.

## Closes when

Tom accepts the exact drafted surface, accepts with named exclusions, or revises it; the answer is recorded with provenance; and the implementation carrier's brief cites the accepted spelling.
