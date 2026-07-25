---
id: derive-the-pre-compilation-artifact-program-subject
title: Derive the pre-compilation artifact program subject
status: todo
priority: p1
dependencies: []
related: [compose-the-complete-expansion-cache-subject, bind-the-cache-subject-to-the-carried-payload-provenance]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [cache, identity, correctness, artifacts]
---
The expansion cache now composes its key from a declared facet set, and one facet has no producer. `tiler_cache::expansion::SubjectFacets::artifact_program` requires the canonical subject of the artifact program a bundle carries — its plan portfolio, ABI bindings, routing, declared target requirements, and selected capability providers — and `ComposedSubject::compose` refuses an empty one, so **no caller can key an entry today**. That is deliberate: a loud stop rather than the silent under-key that preceded it.

`docs/backends/metal.md` names this facet in the normative list: full identity comes "from canonical plans, MSL, target, SDK, compiler, linker, flags, and numerical realization", and "resolved accuracy contracts, selected helpers/intrinsics, and conformance-provider revisions invalidate entries". `tiler-metal-aot` covers everything from MSL rightward. The canonical plans and the provider revisions are what is missing.

## Why the existing identity does not answer it

`CanonicalArtifactProgramIdentity` folds exactly the right subjects, and it cannot be used. It is derived from a **verified** artifact, and building one requires a `BackendPayloadDescriptor` carrying a `PayloadDigest` — the digest of the compiled bytes. The cache key is needed on a **miss**, before compilation. An identity that exists only after the payload does cannot key the lookup that decides whether to produce the payload.

## What this ticket owes

- Derive a subject over the artifact program facts that are settled *before* backend compilation: the plan portfolio and each variant`s complete program identity, the routing policy and every guard, the ABI bindings and launch contracts, the declared target requirements and deferred predicates, and the selected capability providers.
- Decide what stands in for the payload descriptors, which are the only post-compilation part. Naming the descriptors` *non-digest* facets (backend, representation, payload schema, compatibility profile, execution policy) and their entry mappings may be enough, since the digest itself is determined by the compilation facet the cache already frames. State the argument rather than assuming it.
- Establish it by a mechanism, as `identity.rs` and the composer both do: a new identity-bearing field must fail to compile until it reaches the subject.
- Emit canonical bytes under a versioned domain, domain-separated and length-prefixed, exhaustively matched with no wildcard.
- Say explicitly whether the derived subject and `CanonicalArtifactProgramIdentity` are guaranteed to agree on the facts they share, or whether they are two subjects that may legitimately diverge. Two identity authorities over one subject is the failure ADR 0082 rejected for the digest.

Until this lands, `tiler-cache` is composable but not usable, and that is recorded as such in ADR 0050 and ADR 0082 rather than hidden.
