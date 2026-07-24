---
id: decide-whether-layered-subject-digests-exist-as-hashes
title: Decide whether layered subject digests exist as hashes
status: todo
priority: p3
dependencies: []
related: [record-the-implemented-artifact-envelope-in-the-contract, prototype-neutral-artifact-codec]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity]
---
**Fact — `docs/artifact-abi.md`'s identity block describes hashes where the tree carries canonical bytes.** It writes `semantic_digest = H("tiler-semantic-v1" || canonical semantic bytes)` and four siblings for the index, schedule, refinement, and plan layers. Every one of those subjects is implemented as an opaque newtype over its exact canonical byte encoding, compared byte for byte: `SemanticGraphIdentity`, `CanonicalIndexRegionIdentity`, `CanonicalScheduledRegionIdentity`, `CanonicalKernelProgramIdentity`, `CanonicalArtifactProgramIdentity`. None is a hash. ADR 0074 convention 2 states that shape as the accepted convention and makes short digests presentation-only.

**Fact — hashing occurs at exactly three sites, all of them envelope framing.** The manifest digest, each section digest, and the externally derived envelope digest, all SHA-256 under the governed tag `tiler.digest.sha-256.v1`.

**Fact — the placeholder spellings match nothing in the tree.** `"tiler-semantic-v1"` and its four siblings appear in no encoder. The real governed constants have the form `b"tiler.<subject>.v<N>\0"`: `tiler.semantic-graph.v2\0`, `tiler.index-region.v4\0`, `tiler.kernel-program.v1\0`, `tiler.artifact-program.v1\0`. One site, `crates/tiler-ir/src/schedule/model.rs`, writes `b"tiler.schedule.v1"` without the NUL terminator every other site uses.

**Why this is worth deciding rather than quietly deleting.** Canonical bytes are the stronger construction — identity comparison then rests on nothing, where a hash rests on collision resistance — but they are also unbounded, which is why the artifact identity budget is 64 MiB and why an envelope section carrying a kernel-program identity is budgeted at 64 MiB rather than at a digest width. A compact per-layer key is what an external cache index, a cross-reference value, or a diagnostic would actually want. The contract promises one and the tree provides none, so a reader cannot tell whether the compact keys are unbuilt or abandoned.

**What closes this.** Either specify each layer's compact key as an explicit derivation over its canonical bytes, with a governed algorithm tag and domain separator, and say where it is permitted to appear; or record that canonical bytes are the only layered identity Tiler has and remove the five derivations from the contract. Either way, resolve the NUL-terminator inconsistency at the schedule site, because a domain separator that is a prefix of another is exactly the hazard the terminator prevents.

**Scope note.** Deliberately left unscoped. The answer lands in `contracts/artifacts` and `contracts/foundation`, may want an ADR in `contracts/decisions`, and the NUL fix touches `implementation/ir`.
