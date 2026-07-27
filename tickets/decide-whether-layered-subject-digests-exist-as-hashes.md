---
id: decide-whether-layered-subject-digests-exist-as-hashes
title: Decide whether layered subject digests exist as hashes
status: todo
priority: p3
dependencies: []
related: [record-the-implemented-artifact-envelope-in-the-contract, prototype-neutral-artifact-codec]
scopes: [contracts/artifacts, contracts/foundation, contracts/decisions, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity]
---
**Fact — `docs/artifact-abi.md`'s identity block describes hashes where the tree carries canonical bytes.** It writes `semantic_digest = H("tiler-semantic-v1" || canonical semantic bytes)` and four siblings for the index, schedule, refinement, and plan layers. Every one of those subjects is implemented as an opaque newtype over its exact canonical byte encoding, compared byte for byte: `SemanticGraphIdentity`, `CanonicalIndexRegionIdentity`, `CanonicalScheduledRegionIdentity`, `CanonicalKernelProgramIdentity`, `CanonicalArtifactProgramIdentity`. None is a hash. ADR 0074 convention 2 states that shape as the accepted convention and makes short digests presentation-only.

**Fact — the placeholder spellings match no layered identity encoder.**
Current semantic, index, schedule, kernel-program, and artifact-program
identities use versioned domain-separated canonical bytes. Their version
numbers evolve independently and are intentionally not pinned in this ticket.
The schedule domain now has the same NUL terminator discipline as the other
encoders.

**Fact — governed hashing has broader uses than these layered identities.**
Artifact envelope framing, proof payloads, and cache framing or keys use the
governed digest. That does not turn the canonical layered identity newtypes into
hashes.

**Why this is worth deciding rather than quietly deleting.** Canonical bytes are the stronger construction — identity comparison then rests on nothing, where a hash rests on collision resistance — but they are also unbounded, which is why the artifact identity budget is 64 MiB and why an envelope section carrying a kernel-program identity is budgeted at 64 MiB rather than at a digest width. A compact per-layer key is what an external cache index, a cross-reference value, or a diagnostic would actually want. The contract promises one and the tree provides none, so a reader cannot tell whether the compact keys are unbuilt or abandoned.

**What closes this.** Either specify each layer's compact key as an explicit
derivation over its canonical bytes, with a governed algorithm tag and domain
separator, and say where it is permitted to appear; or record that canonical
bytes are the only layered identity Tiler has and remove the five nonexistent
hash derivations from the contract.

**Scope note.** The ticket declares every contract and implementation area its
stated outcome may change. If research eliminates the need for an ADR or code
change, leave the unused areas untouched rather than narrowing scope after work
has begun.
