---
id: replace-flat-selected-lowering-capability-keys-with-structured-subjects
title: Replace flat selected lowering capability keys with structured subjects
status: todo
priority: p1
dependencies: [reconcile-the-operation-identity-and-governed-key-grammars]
related: [reconcile-the-two-target-profile-key-grammars, package-selected-physical-implementation-provenance-in-artifact-identity, frame-provider-identities-before-using-them-as-explain-keys]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/runtime, contracts/foundation, contracts/artifacts, research/artifacts, research/cache, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [identity, validation, extensions, implementation, public-boundary, artifact, schema]
---
## User-visible outcome

Every legal selected lowering capability remains packageable and two distinct `(family, operation)` subjects can never collapse through a delimiter-composed string.

## Required delivery

- Reverify every Fact and identity/version consequence from the accepted decision at the exact implementation base before editing production source.
- Replace the compiler's flattened capability string with one typed subject containing the closed lowering-family value and exact `OpKey`. Mint it once at registration/resolution and retain it through `LoweringProviderIdentity` and public `SelectedCapability`; keep provider and capability revision separate.
- Replace artifact `CapabilityKey` usage for selected lowering provenance with a structured capability subject containing a governed family key and exact `OpKey`. Encode and decode the family, namespace, name, and version as separately framed fields. Preserve a typed invalid-operation-key decode cause.
- Keep the existing one-signature-per-provider/family/operation rule. Do not add signature to the selected subject or permit a second signature under a subject that cannot name it.
- Remove all seven downstream reconstructions of `CapabilityKey` from a flattened compiler string. Replace brittle assertions and the lossy target-profile error remapping with direct typed propagation from the structured carrier.
- Reconcile the exact artifact provider-row domain, artifact-program domain, manifest schema, identity ledger, ABI contract, fixtures, proof-sidecar subjects, envelope digests, cache subjects, and every derived pin. Do not step unrelated identity domains.
- Preserve a human-readable rendering only as presentation. Assert that no equality, ordering, deduplication, cache, receipt, or artifact identity consumer uses it.
- Add a public-boundary record under ADR 0075 for the exact included and excluded `SelectedCapability` and artifact subject surfaces.

## Required negative controls

- Register `("a.b", "c", 1)` and `("a", "b.c", 1)` under the same provider and capability revision; both must remain distinct through resolved-provider census, public selection evidence, artifact rows, codec round-trip, and identity.
- Perturb only the family, namespace/name boundary, operation version, provider, and capability revision independently and prove each identity-bearing assertion fails with its subject named.
- Exercise uppercase and maximum-length legal operation components and prove they remain packageable without folding, truncation, or a late text conversion.
- Corrupt each decoded structured component independently and observe a typed refusal; no legacy flattened-key interpretation is allowed.
- Prove a diagnostic display change alone cannot move canonical identity.

## Non-goals

Narrowing `OpKey`, changing the governed family-key grammar, permitting multiple signatures for one selected subject, hashing the subject in place of encoding it, retaining the ambiguous spelling as a compatibility fallback, or changing lowering/provider selection.

## Closes when

The structured subject is the only selected-capability authority across compiler and artifact layers, the collision pair remains distinct end to end, every legal current `OpKey` remains admitted, all schema and derived identities reconcile, exact-tip full gates pass, and an independent identity-sensitive review reports no findings.
