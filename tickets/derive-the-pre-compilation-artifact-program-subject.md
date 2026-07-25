---
id: derive-the-pre-compilation-artifact-program-subject
title: Derive the pre-compilation artifact program subject
status: done
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

## Outcome

**The premise of this ticket is false, and correcting it is the result.** Delivered in `de9e2e8`.

**Fact — the payload digest is not a digest of the compiled object.** This ticket's opening paragraph, ADR 0050's traceability section, `crates/tiler-cache/src/expansion/subject.rs:56-64`, `key.rs:45-49`, and every dispatch that repeated them all assert that `CanonicalArtifactProgramIdentity` folds the compiled bytes. It does not. `push_carried_payload` derives `BackendPayloadDescriptor::digest` as `payload_identity(encode_metadata(&metadata))`, and `crates/tiler-artifact/src/program/codec/payload.rs:17-19` states in terms that those bytes "contain the source, the target, the flags, and the toolchain provenance and **no object byte at all**". `check_payload_identity` (`codec/validate.rs:246`) re-proves it on every decode, and the pre-existing test `payload_identity_follows_the_compilation_subject_and_not_the_object` already asserted that relinking the same source yields *equal* artifact identity.

**Inference — every fact the identity folds is a compilation input, so the identity was already the pre-compilation subject.** There was nothing to derive. What was genuinely missing was a way to *reach* it without an object: both payload constructors demanded either compiled `PayloadContent` or a `PayloadDigest` whose derivation was not exported.

**Delivered.** `PayloadMetadata::identity` is that derivation, and `PayloadContent::identity` delegates to it, so there is one derivation rather than two. `ArtifactProgramBuilder::push_pending_payload` declares the payload a compilation that has not yet run will produce; `push_carried_payload` delegates to it and adds the object, so **one construction site builds both descriptors**. That single site is the correctness bar rather than a tidiness preference: had the pre- and post-compilation callers built descriptors independently and one field drifted, the cache would file a compiled artifact under a key derived from a *different* artifact's subject — a wrong-artifact hit, not a slow build.

**The mechanism this ticket demanded was genuinely absent and now exists.** `encode_metadata` and both `canonical_key` sites destructure irrefutably, so a field added to the payload's compilation subject fails to compile until it reaches the encoder. Previously `encode_metadata` read fields by name and a new field would have silently left identity.

**This ticket's own suggested shortcut is rejected as an under-keying hole.** Naming the descriptors' non-digest facets instead of the digest would give two payloads compiled from *different MSL* — sharing backend, representation, schema, compatibility profile and execution policy — the same key. The ticket hedged that the `BackendCompilations` facet covers the difference; it does not for a **descriptor-only** payload, which contributes no compilation run at all, leaving the digest as the only discriminator.

**The last bullet is answered in the strongest available form: they do not diverge, because they are the same object.** One authority, one encoder, one domain. A second pre-compilation subject type was rejected rather than deferred — it would be the second identity authority over one subject that ADR 0082 names, and its agreement with the real identity could only ever be argued, never checked.

**Still not usable end to end, with the exact blocker.** The artifact-program facet now has a producer; the `BackendCompilations` facet does not. `crates/tiler-metal-aot/src/lib.rs:74` declares `mod identity;` and `CompilationIdentity::as_bytes` is `pub(crate)`, so nothing outside that crate can obtain those bytes. Promoting it is ADR 0075's always-ask category in `implementation/metal-aot`. Split to `promote-the-metal-aot-compilation-identity`. A second gap is structural rather than a defect: no crate holds both a compiler plan and a Metal compilation, and ADR 0050 already records the end-to-end hit as the orchestrator's.

**Left to follow-ups.** The three false statements in ADR 0050, `subject.rs` and `key.rs` are corrected by `correct-the-artifact-identity-post-compilation-claim`; the new surface is listed on `accept-the-tiler-cache-public-boundary`.
