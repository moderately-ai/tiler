---
id: publish-occurrence-bound-selected-physical-implementation-evidence
title: Publish occurrence-bound selected physical implementation evidence
status: todo
priority: p1
dependencies: [disclose-the-physical-provider-environment-a-compilation-was-offered, accept-the-installed-physical-provider-public-surface]
related: [accept-the-installed-physical-provider-public-surface, disclose-offered-and-selected-physical-provider-sets-separately, carry-complete-access-alignment-requirements-on-physical-proposals]
scopes: [implementation/compiler, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, provenance, identity, public-boundary]
---
## User-visible outcome

A neutral artifact assembler can forward the compiler's exact selected physical authority for every covered region without reconstructing private selection state or collapsing a mixed plan to a provider set.

## Required delivery

- Add one compiler-minted, borrowed or owned projection for every selected cover-region implementation, in canonical region-occurrence order.
- Carry the canonical occurrence binding, the exact `ImplementationProposalIdentity`, the readable `ProviderIdentity`, and the closed proposal-kind code. Do not expose body internals, cost, rejected alternatives, the offered provider environment, or provider installation order.
- Derive every field from the retained `RegionSelection` / `AdmittedImplementation`; callers and physical providers must not construct or replace the authority.
- Keep the projection occurrence-bound. A deduplicated provider set, provider-plus-kind set, or order-only list is insufficient for a plan that mixes providers or selects one provider more than once.
- Add a subject perturbation showing that changing only provider authority or only the occurrence association changes the projected evidence while the structural body fixture remains unchanged.
- Record the exact public included/excluded surface under ADR 0075 and update the artifact contract language that consumes it.

## Non-goals

Packaging the artifact, exposing private implementation bodies, serializing offered providers, changing provider installation precedence, or changing selection policy.

## Closes when

The build layer can consume complete compiler-owned selected physical evidence without re-derivation, its population and ordering are pinned, and independent review confirms no private selection authority was widened beyond the four required subjects.
