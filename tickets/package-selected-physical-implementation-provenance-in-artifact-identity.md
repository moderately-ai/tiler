---
id: package-selected-physical-implementation-provenance-in-artifact-identity
title: Package selected physical implementation provenance in artifact identity
status: in-progress
priority: p1
dependencies: [disclose-the-physical-provider-environment-a-compilation-was-offered, publish-occurrence-bound-selected-physical-implementation-evidence, replace-flat-selected-lowering-capability-keys-with-structured-subjects]
related: [disclose-offered-and-selected-physical-provider-sets-separately, reconcile-the-operation-identity-and-governed-key-grammars]
scopes: [implementation/artifact, implementation/build, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, provenance, artifact, identity, schema, public-boundary]
claimed_from: todo
assignee: sol-physical-artifact
lease_expires_at: 1786933579
---
## User-visible outcome

An artifact records exactly which physical authority produced each selected region while remaining invariant to every offered provider the selected plan did not use.

## Required delivery

- Replace the lowering-only construction authority with an explicit role-separated `CompilationEnvironment`: required canonical lowering and physical offered sets, no union, no default, and no inference from payload/backend/profile.
- Validate existing selected lowering rows only against the lowering set and new selected physical rows only against the physical set. A missing member is a typed artifact-build refusal; never substitute the governed provider or omit the row.
- Add a separately tagged occurrence-bound physical-selection run inside each artifact variant, carrying the compiler projection: occurrence binding, exact implementation-proposal identity, provider identity, and proposal kind. Preserve multiplicity and association; do not reduce it to an artifact-global provider set, an iterator position, a backend entry, or a payload association.
- Encode the complete selected physical row population in artifact canonical identity and the manifest/envelope bytes. Step the owning artifact-program/schema domains coherently at the implementation base and recompute all derived pins and cache/envelope subjects. Do not step semantic, schedule, structured-kernel, payload-content, or unrelated wire domains.
- Keep both offered sets construction-only and discard them after validation. Perturb an unused lowering provider and an unused physical provider independently and prove artifact identity, bytes, envelope digest, and cache subject are unchanged.
- Perturb selected provider identity, implementation-proposal identity, occurrence association, and proposal kind independently and prove the artifact identity/bytes checks fail with assertions unchanged.
- Update `docs/artifact-abi.md`, crate rustdoc, identity ledgers, build translation, codec/decode, equality, limits, and all exhaustive consumers as one coherent identity step.

## Non-goals

Serializing the full offered environments, changing provider selection or cost policy, defining provider precedence, retrying another provider, inferring a missing selected provider, or changing executable kernel semantics.

## Closes when

The production assemble path forwards every selected physical row, construction rejects cross-role or absent authority, unused offered providers remain byte-invariant, selected authority is identity-bearing, all schema/domain pins reconcile, and full artifact/build gates plus independent exact-commit review pass.
