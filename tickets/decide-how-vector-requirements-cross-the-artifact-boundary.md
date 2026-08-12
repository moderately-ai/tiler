---
id: decide-how-vector-requirements-cross-the-artifact-boundary
title: Decide how vector requirements cross the artifact boundary
status: awaiting-decision
priority: p1
dependencies: [define-plural-operation-specific-vector-realization-requirements]
related: [declare-cpu-vector-realization-facts-in-the-target-profile]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [vector, artifact, identity, schema, public-boundary]
---
## User-visible outcome

Vector feasibility evidence crosses only the layers that consume it, with a deliberate artifact schema and identity consequence rather than an assumed empty/default field.

## Decision boundary

Determine whether vector requirements remain compile-only evidence or are required in artifact entry resources for runtime/explain validation. Compare an unconditional fixed-record field and major schema migration against a separately framed conditional side table that preserves non-vector bytes. No optional/default interpretation or lossy projection is admissible.

## Required evidence

- Census every schedule, KIR, feasibility, explain, artifact, decoder, cache, reference, and runtime consumer.
- Prove the chosen encoding injective and versioned, including empty and plural populations.
- Recompute KIR/artifact domains, manifest schema, envelope/cache subjects, and all pins from the chosen ownership.
- Perturb empty/nonempty, ordering, member omission, unknown schema/feature, and legacy reader independently.

## Closes when

Tom accepts the exact carrier and migration after the consuming population is proved.
