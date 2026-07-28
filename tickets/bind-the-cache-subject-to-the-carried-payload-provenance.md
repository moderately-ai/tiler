---
id: bind-the-cache-subject-to-the-carried-payload-provenance
title: Bind the cache subject to the carried payload provenance
status: done
priority: p2
dependencies: [compose-the-complete-expansion-cache-subject]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact, implementation/frontend, implementation/metal-aot, implementation/workspace, contracts/decisions, contracts/foundation, contracts/navigation, implementation/build]
shared_scopes: [implementation/cargo-lock, project/tickets]
paths: []
tags: [cache, identity, correctness]
---
## Decision outcome (2026-07-28)

Tom authorized a new `tiler-build` crate and chose the complete build-time publication and acceptance path over a crate scoped to this check alone. ADR 0085 records the durable boundary: `tiler-build` is downstream of compiler, backend, artifact, AOT-driver, and cache authorities; it sequences them without reimplementing any identity or private subject encoding.

The crate initially depends only on `tiler-artifact` and `tiler-metal-aot`, because those are the authorities the implemented correspondence slice consumes. Later compiler, backend, and cache edges land only with the corresponding executable slices.

The alternatives were eliminated before the decision. `tiler-cache` would have to parse a producer vocabulary it does not own, becoming a second authority. `tiler-compiler` would acquire a downstream cache and packaging concern, inverting the dependency direction. A one-function crate would leave the actual sequencing ownerless and require another public boundary later.

## Implemented outcome

`Toolchain::prepare` now derives `ArtifactProvenance` from the same single resolved toolchain observation as `CompilationIdentity`. `PreparedCompilation` owns both records, lends the bound request and provenance for pre-compilation validation, and moves that exact provenance into `CompiledArtifact` when consumed. Cache-hit validation and miss execution therefore cannot silently select or derive two toolchains.

`tiler_build::validate_prepared_metal_payload` compares all eleven compilation facts the carried payload records: source representation, exact source, toolchain family, target, artifact family, language, deployment minimum, ordered tool components, SDK identity, ordered compiler flags, and ordered linker flags. Entry mappings and target obligations remain emission facts. The check allocates nothing, returns a typed producer/protocol mismatch in stable contract order, and was fault-injected by disabling one fact comparison; its exhaustive mismatch test failed before the comparison was restored.

Tom accepted the exact public facade on 2026-07-28: the borrowed `PreparedCompilation::{request, provenance}` accessors, `validate_prepared_metal_payload`, exhaustive `MetalPayloadFact`, and opaque `MetalPayloadMismatch`.

## Graph maintenance

The full path is the admitted responsibility, not an assertion that all of it landed in this slice. Dependency-ordered follow-ups now own artifact assembly around the checked correspondence (`assemble-prepared-metal-artifacts-in-tiler-build`), cache publication and hit acceptance (`accept-and-publish-validated-artifacts-through-the-expansion-cache`), and the upstream compiler/backend handoff (`drive-the-build-orchestrator-from-a-checked-compiler-plan`). Each names the exact first missing call site and preserves this ticket's rule that correspondence runs both before publication and before accepting a hit. This ticket closes once its gate is green; the split is what releases the next slice.

## Why the check is needed

A `tiler-cache` bundle proves two things about its key on every hit: that the bundle was published under the key it is filed at, and that the key is the governed digest of the subject the bundle carries. It does **not** prove that subject describes the artifact beside it.

A writer that derived `K` from one subject and packaged an artifact compiled from another would produce a bundle every reader accepts. Nothing in the cache can catch it, because catching it means parsing the producer's subject encoding, which would make the cache a second authority over an encoding it does not own.

The carried envelope does record its own compilation subject: `PayloadMetadata` holds the exact source, target, flags, and toolchain provenance, and `decode_artifact` already proves the payload descriptor's digest equals `payload_identity` of those bytes. So the material for a cross-check exists on both sides; what is missing is a component that may read both.

## User-visible outcome

A cache result is never used as though it were built from a different compilation subject. The cache validates framing, cardinality, ordering, and artifact integrity. The orchestrator that legitimately understands both the producer's compilation facts and artifact metadata validates their semantic correspondence before publication and before accepting a hit.

A mismatch is a typed producer/protocol defect, not an ordinary cache miss: rebuilding and republishing the same mismatch would hide and repeat the bug. The cache must not parse the foreign inner subject encoding; that would make it a second authority.

## What composing the subject changed, and what it did not

`compose-the-complete-expansion-cache-subject` landed and this ticket is **not** made unnecessary by it. Composition decides what a key covers; this ticket decides whether the covered subject describes the artifact beside it. A writer that composed a correct subject and packaged an envelope from a different compilation still produces a bundle every reader accepts.

The cache now owns the composed outer frame, so it can check cardinality and ordering without parsing a producer encoding. The facts-level comparison belongs to the orchestrator that owns both inputs.
