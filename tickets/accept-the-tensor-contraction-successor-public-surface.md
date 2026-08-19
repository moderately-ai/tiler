---
id: accept-the-tensor-contraction-successor-public-surface
title: Accept the tensor-contraction successor public surface
status: done
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


## Accepted decision — 2026-08-18

Tom accepted **the exact drafted surface as reviewed**, in the live coordination session with the orchestrator, relayed first-hand by the coordinator, by replying `agreed next decision` to the accept-with-exclusions-or-revise question presented in explain-then-recommend form.

The accepted included/excluded API population is exactly the reviewed packet's draft in `decide-the-semantic-order-contract-for-relaxed-contractions` (§"Recommended semantic and identity contract" through §"Exact topology witness and reference boundary", as repaired by the independent review at `5a48c9ce`): the thirteen-field typed definition descriptor with field 15's six-row reduction record and field 14's seven-field ADR-0013-bound stability record; decode-only `ContractionF32ReductionDescriptor` with its exhaustive error vocabulary and registration-time decoding; `EffectiveContractionF32Profile` with the single `CanonicalNanMismatch` error and no descriptor-bypassing path; `OrderedContractionF32Tree`/`ContractionF32PlanWitness` with the twelve-variant validation vocabulary and static-`K` uniform-template scope (`LiveContributorCount` refusal); and the bounded `ContractionF32TopologyEvaluator` with the caller-owned four-resource budget, `standard-reference@7→@8` and contraction capability revision 7→8 at implementation. Exact field ids and Rust spellings are re-derived at the implementation carrier's base per standing rule; the accepted population and semantics are fixed here.
