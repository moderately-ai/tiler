---
id: bind-the-cache-subject-to-the-carried-payload-provenance
title: Bind the cache subject to the carried payload provenance
status: todo
priority: p2
dependencies: [compose-the-complete-expansion-cache-subject]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact, implementation/frontend]
shared_scopes: []
paths: []
tags: [cache, identity, correctness]
---
A `tiler-cache` bundle proves two things about its key on every hit: that the bundle was published under the key it is filed at, and that the key is the governed digest of the subject the bundle carries. It does **not** prove that subject describes the artifact beside it.

A writer that derived `K` from one subject and packaged an artifact compiled from another would produce a bundle every reader accepts. Nothing in the cache can catch it, because catching it means parsing the producer's subject encoding, which would make the cache a second authority over an encoding it does not own.

The carried envelope does record its own compilation subject: `PayloadMetadata` holds the exact source, target, flags, and toolchain provenance, and `decode_artifact` already proves the payload descriptor's digest equals `payload_identity` of those bytes. So the material for a cross-check exists on both sides; what is missing is a component that may read both.

## User-visible outcome

A cache result is never used as though it were built from a different
compilation subject. The cache validates framing, cardinality, ordering, and
artifact integrity. The orchestrator that legitimately understands both the
producer's compilation facts and artifact metadata validates their semantic
correspondence before publication and before accepting a hit.

A mismatch is a typed producer/protocol defect, not an ordinary cache miss:
rebuilding and republishing the same mismatch would hide and repeat the bug.
The cache must not parse the foreign inner subject encoding; that would make it
a second authority.

## What composing the subject changed, and what it did not

`compose-the-complete-expansion-cache-subject` landed and this ticket is **not** made unnecessary by it. Composition decides what a key covers; this ticket decides whether the covered subject describes the artifact beside it. A writer that composed a correct subject and packaged an envelope from a different compilation still produces a bundle every reader accepts.

The cache now owns the composed outer frame, so it can check cardinality and
ordering without parsing a producer encoding. The facts-level comparison
belongs to the orchestrator that owns both inputs.
