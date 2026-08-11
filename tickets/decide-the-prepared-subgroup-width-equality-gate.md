---
id: decide-the-prepared-subgroup-width-equality-gate
title: Decide the prepared subgroup-width equality gate
status: awaiting-decision
priority: p1
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [admit-an-atomic-subgroup-realization-subject-to-target-profiles, declare-metal-subgroup-realization-facts-in-the-target-profile, measure-metal-thread-execution-width-across-prepared-pipelines]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/runtime, implementation/metal, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [subgroup, metal, preflight, routing, feasibility, public-boundary, decision, needs-tom]
---
## User-visible outcome

A subgroup route commits only after the prepared pipeline's actual execution width exactly equals the width whose tree was verified. A missing observation or mismatch refuses that route before routing commit; it never retries with another width or backend.

## Fact

ADR 0094 public-boundary item 6 requires a `PreparedKernelPreflight` comparison of Metal `threadExecutionWidth`. The value belongs to `MTLComputePipelineState`, not the compile profile or live device. Existing prepared-entry requirements can express exact observed-versus-required comparison, while `RouteResourceDimension::SubgroupThreads` already means width equality and Metal correctly cannot answer it at device preflight.

## Decision questions

- Whether the verified program emits a dedicated typed prepared-entry subgroup requirement or reuses the generic route-resource dimension without losing phase/subject information.
- Which component reads `threadExecutionWidth`, constructs the observation, and proves it corresponds to the exact prepared payload/pipeline being routed.
- How the one-way routing commit orders preparation, comparison, publication/cache insertion, and any alternative selection.
- Which canonical subject binds required width, prepared payload identity, observation authority, and refusal without folding live observations into reusable artifact identity.

## Strict boundary

No compile-profile row alone can discharge this gate. No device-wide constant, cached observation from another pipeline, default width, “at least” comparison, pipeline rebuild, or post-commit fallback is admissible.

## Closes when

Tom accepts one exact preflight carrier and ordering, with complete identity/accounting and typed missing/mismatch failures demonstrated against the real prepared-entry path.
