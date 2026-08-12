---
id: decide-how-vector-requirements-cross-the-artifact-boundary
title: Decide how vector requirements cross the artifact boundary
status: awaiting-decision
priority: p1
dependencies: [define-plural-operation-specific-vector-realization-requirements, package-selected-physical-implementation-provenance-in-artifact-identity]
related: [declare-cpu-vector-realization-facts-in-the-target-profile, establish-vector-execution-form-numerical-authority]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [vector, artifact, identity, schema, public-boundary]
---
## User-visible outcome

Vector feasibility evidence crosses only the layers that consume it, with a deliberate artifact schema and identity consequence rather than an assumed empty/default field.

## Decision boundary

Determine whether vector requirements remain compile-only evidence or are required in artifact entry resources for runtime/explain validation. Compare an unconditional fixed-record field and major schema migration against a separately framed conditional side table that preserves non-vector bytes. No optional/default interpretation or lossy projection is admissible.

## Source-first correction — 2026-08-12

The binary above is incomplete and this ticket is not yet decision-ready.

- `ResourceRequirements` is derived from a verified `ScheduledRegion` before physical implementation selection. It can truthfully carry provider-independent intrinsic operation/form requirements, but it cannot carry the selected provider and provider-owned execution-variant key that the accepted [`establish-vector-execution-form-numerical-authority`](establish-vector-execution-form-numerical-authority.md) boundary requires.
- Selected physical authority already has a separate accepted artifact owner. [`package-selected-physical-implementation-provenance-in-artifact-identity`](package-selected-physical-implementation-provenance-in-artifact-identity.md) will package occurrence-bound selected implementation evidence after selection. The provider-versioned arithmetic execution subject must bind through that reached-only projection and the delivered-realization record, not be forged early in schedule resources.
- The current artifact resource run is fixed and unframed relative to following entry fields. Adding an unconditional plural intrinsic-requirement field at the current base would step `tiler.kernel.v7`, `tiler.artifact-program.v16`, and manifest schema `16.0` to their next owning versions. A conditional side table may be injective but creates a second entry relation and content-dependent positional grammar; no external compatibility requirement justifies it in this pre-production tree.
- Compile-only retention is insufficient once a native CPU payload exists: an artifact/runtime validator must be able to compare the entry's complete intrinsic vector requirements, selected execution realization, and delivered numerical evidence without compiler-private reconstruction. That does not mean one record should own all three subjects.

The corrected frontier is therefore a coordinated two-carrier design: provider-independent plural intrinsic requirements in the fixed entry resource record, and provider/variant realization in occurrence-bound selected physical provenance plus delivered execution evidence. This ticket waits for both prerequisite carriers, then decides one coordinated schema migration and the exact cross-checks. It must not put a provider identity in `ResourceRequirements`, duplicate intrinsic requirements in a selected-provider row, or preserve old scalar bytes through an optional side table merely for compatibility.

## Required evidence

- Census every schedule, KIR, feasibility, explain, artifact, decoder, cache, reference, and runtime consumer.
- Prove the chosen encoding injective and versioned, including empty and plural populations.
- Recompute KIR/artifact domains, manifest schema, envelope/cache subjects, and all pins from the chosen ownership.
- Perturb empty/nonempty, ordering, member omission, unknown schema/feature, and legacy reader independently.

## Closes when

Tom accepts the exact carrier and migration after the consuming population is proved.
