---
id: expose-the-built-artifact-canonical-identity
title: Expose the canonical identity a built artifact already derived
status: todo
priority: p2
dependencies: []
related: [carry-the-metal-payload-in-an-artifact-envelope, route-the-runtime-proof-through-the-artifact-envelope]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, public-surface, needs-tom]
---
A producer can encode an artifact and a reader can observe a decoded one's identity, but nothing can ask a *built* artifact what its identity is — so the two cannot be compared across the boundary they exist to bind.

**Fact — the value exists and is unreachable.** `ArtifactProgramBuilder::build` (`crates/tiler-artifact/src/program/builder.rs:500-516`) derives the identity from the canonical envelope and stores it: `Ok(VerifiedArtifactProgram { data, identity })`. `identity` is a private field, and the type's public surface is `selected_providers`, `payloads`, `inputs`, `outputs`, `variants`, `expressions`, and — since the codec promotion — `encode`. There is no accessor: `cargo check` on `artifact.identity()` reports `private field, not a method`.

**Fact — the read side has one.** `DecodedArtifact::identity()` (`codec/view.rs:130`) returns a `CanonicalArtifactProgramIdentity` re-derived from decoded content.

**Inference — the missing half is the one a producer needs.** `decode_artifact` already proves the decoded content's identity equals the one the manifest carries, and a byte-identical `re_encode` pins the manifest's stored bytes to that derivation, so the round trip is provable without this. What is *not* provable from outside is the direct statement "the artifact I built is the artifact these bytes name" — the comparison a cache lookup, an expansion-cache key, and a runtime binding-by-identity all make. `codec/view.rs` states the last one explicitly: "a consumer that must evaluate one holds the program it compiled and uses the decoded identity to prove the bytes it loaded name that same artifact." It cannot, today.

**Measured, not inferred.** `prototypes/serial-sum-compile/src/bundle.rs` is the first out-of-crate assembler and wanted exactly this comparison. Its round-trip case asserts decode success and a byte-identical re-encode instead, with the reasoning written into the test rather than left implicit.

## Scope

Add an accessor for the identity `build` already derived. Adding a `pub` item to this module is an ADR 0075 always-ask decision, which is why this is a ticket rather than a line of code.

Decide what it returns. `CanonicalArtifactProgramIdentity` is already public and `DecodedArtifact::identity` returns one by value, so the obvious shape is `VerifiedArtifactProgram::identity(&self) -> &CanonicalArtifactProgramIdentity`. State whether the two are deliberately the same type — they should be, since comparing them is the point.

## Closes when

An out-of-crate producer can assert that the identity of the artifact it built equals the identity re-derived from the bytes it encoded; the `bundle` round-trip case makes that assertion directly rather than through byte equality; and `uv run --locked python scripts/check_repository.py` passes.
