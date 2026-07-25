---
id: bind-the-cache-subject-to-the-carried-payload-provenance
title: Bind the cache subject to the carried payload provenance
status: todo
priority: p2
dependencies: [compose-the-complete-expansion-cache-subject]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, identity, correctness]
---
A `tiler-cache` bundle proves two things about its key on every hit: that the bundle was published under the key it is filed at, and that the key is the governed digest of the subject the bundle carries. It does **not** prove that subject describes the artifact beside it.

A writer that derived `K` from one subject and packaged an artifact compiled from another would produce a bundle every reader accepts. Nothing in the cache can catch it, because catching it means parsing the producer's subject encoding, which would make the cache a second authority over an encoding it does not own.

The carried envelope does record its own compilation subject: `PayloadMetadata` holds the exact source, target, flags, and toolchain provenance, and `decode_artifact` already proves the payload descriptor's digest equals `payload_identity` of those bytes. So the material for a cross-check exists on both sides; what is missing is a component that may read both.

## What this ticket owes

- Decide whether the check belongs in the cache (needing a narrow, versioned way to read a subject's compilation facts) or in the orchestrator that holds both crates.
- Decide what a mismatch is. It is a rejection either way, but whether it is a miss under ADR 0050's fall-open rule or a hard error deserves an explicit argument: unlike a corrupt entry, a mismatch means a *writer* is wrong, and falling open would hide a defect that reproduces on every publication.
- Whatever is decided, the rejection carries a typed reason like every other one in this crate.

## What composing the subject changed, and what it did not

`compose-the-complete-expansion-cache-subject` landed and this ticket is **not** made unnecessary by it. Composition decides what a key covers; this ticket decides whether the covered subject describes the artifact beside it. A writer that composed a correct subject and packaged an envelope from a different compilation still produces a bundle every reader accepts.

What changed is the first decision above. The cache now owns the composed frame — versioned domain, facet tags, counted and length-prefixed runs — so it can count the backend-compilation facet's runs and reach each one's bounds **without parsing any producer's encoding**. The "narrow, versioned way to read a subject's compilation facts" this ticket asked for exists for the outer frame; what is still foreign is the inside of a facet, which is where `PayloadMetadata`'s source, target, flags, and toolchain would have to be compared. So the choice is now narrower than it was: a cardinality and ordering cross-check is available to the cache today, and a facts-level comparison still needs an authority that may read both encodings.
