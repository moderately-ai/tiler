---
id: carry-structured-provenance-through-numerical-rejections
title: Carry structured provenance through numerical rejections
status: todo
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: []
paths: []
tags: [implementation, numerics, provenance]
---
## User-visible outcome

A numerical rejection retains and exposes the exact checked honourability fact that refused the required behaviour, including its authority, validity scope, compiler builds, and execution environments. Diagnostics can therefore explain why a target rejected a contract without collapsing measured evidence into scalar means and a profile key.

## Implementation keys

`CheckedTargetProfile::resolve_dimension` currently carries the exact checked fact only through `HonouredDimension`. Its refusing branch reconstructs `UnhonouredDimension` from dimension, arithmetic type, required behaviour, means, one honoured alternative, and profile identity, losing `FactSourceProvenance` before `Rejection`, pipeline trace, and explain output.

Retain the exact refusing `NumericalHonourabilityFact` by shared immutable ownership or an equivalent checked reference that survives the rejection pipeline. Keep the caller-required behaviour separate from the declaration: the required value and the refusing fact answer different questions. Synthetic rejection fixtures must construct honest checked evidence rather than using a provenance-free escape hatch.

Prove the path end to end from feasibility through `Rejection`, `OpaqueCallRejectionCause`, pipeline trace, and the proposed borrowed diagnostic view. Mutate only compiler build or execution environment and show the rejection evidence and its identity/rendered explanation change. Prove each new check can fail.

## Graph maintenance

This follows the selected-evidence foundation rather than expanding it. It must land before any artifact or public diagnostic boundary claims that all ADR 0076 rejection provenance is readable. Relate any public borrowed view to the same facade review as `redesign-the-delivered-realization-record-from-typed-evidence`; do not publish internal `Arc` storage or compiler-private verified structs.
