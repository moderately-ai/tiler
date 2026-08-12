---
id: plan-concatenate-through-one-partitioned-copy-entry
title: Plan concatenate through one partitioned-copy entry
status: todo
priority: p1
dependencies: [lower-the-partitioned-copy-region-through-kernel-ir, derive-target-numerical-feasibility-from-reached-arithmetic-only, repair-the-scheduled-vocabulary-census-and-concatenate-law-standing]
related: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary]
scopes: [implementation/compiler, implementation/metal, implementation/build, implementation/artifact, implementation/conformance, contracts/optimizer, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, concatenate, compiler, metal, artifacts, conformance]
---
## Outcome

An admitted `tiler::concatenate-f32@1` occurrence is recognized, bound, planned, lowered, packaged, and executed as exactly one partitioned-copy stage, kernel entry, and dispatch. The old `operation-set` refusal is superseded with its reason preserved, while unsupported shapes retain named typed refusals.

## Required delivery

Consume the accepted schedule and KIR carriers without reconstructing their partitions. Preserve one occurrence/stage/entry, distinct-input bindings plus output, explicit non-arithmetic requirements and delivery, exact target/profile identity, and all existing preflight/commit ordering. No host materialization, pointwise-identity substitution, N-entry fallback, or backend retry is implicit.

Re-derive the physical-provider revision consequence: retain it only if every previously valid context produces identical offers; otherwise bump it and the reached provenance intentionally. Enumerate request, schedule, KIR, artifact, cache, proof, and conformance pins on the merged tree. Update the optimizer population and policy table only when the end-to-end path is real.

## Closes when

Arity 2 and 8, unequal and zero extents, first/middle/last axes, operand reorder, and `concat(x, x)` run through one entry with bit-identical reference output; one binding above the admitted limit refuses by name; malformed ownership/store subjects fail before packaging; and trace/artifact evidence reports arithmetic as explicitly not applicable rather than silently absent.
