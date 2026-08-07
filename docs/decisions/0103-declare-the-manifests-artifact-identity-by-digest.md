---
schema: "tiler-doc/v1"
id: "ADR-0103"
kind: "decision"
title: "Declare the manifest's artifact identity by digest rather than by preimage"
topics: ["artifacts", "codec", "identity", "limits", "public-boundary"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.artifact-abi"]
evidence: ["tiler.research.artifacts.manifest-fixed-content-growth"]
depends_on: ["ADR-0050", "ADR-0072", "ADR-0074", "ADR-0075", "ADR-0082"]
ticket: "decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest"
---

# 0103: Declare the manifest's artifact identity by digest rather than by preimage

**Status:** accepted. **The choice this record carries was made by Tom on 2026-08-06, at the live session's decision round** — presented by the orchestrator under explain-then-recommend, relayed rather than witnessed by the author of this file, and the provenance packet is the `## Decision — the digest` section of [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](../../tickets/decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md). What Tom decided is the *substance*: the manifest carries a digest, the producer-disagreement refusal is retained rather than deleted, and the ADR 0074 convention-2 argument below is the ground accepted. This record states that decision, its identity-domain step, and its measured consequence so that acceptance is a separate act on a written record. **Accepted by Tom on 2026-08-06, at the live session's decision round, as recommended and with the implemented step's measured numbers (114,059 → 57,978 bytes, zero pins) in the packet** — presented by the orchestrator, relay source the same ticket's Outcome.

## Context

**Measurement, 2026-08-06, at `8bd720b8`** ([Where the artifact envelope's fixed content came from](../research/artifacts/manifest-fixed-content-growth.md)). The largest single component of every artifact envelope was the canonical-identity run the manifest carried at its end — **49.2% of a 114,043-byte fixed-content envelope**. The run grew ×4.2 over the attributed interval against ×3.4 for the rest of the manifest, because everything added to the manifest is added again to the identity that folds it.

**Fact — the run is a complete function of the manifest content above it.** `encode_manifest` ended with `push_slice(&mut bytes, identity.as_bytes())`. `encode_identity` reads the *envelope* — schema, routing policy, the three semantic subjects, the interface, providers, payloads, the arena, the variants, the delivered-realization record — and reads nothing from the manifest's own encoding.

**Fact — every decode re-derives it and compares, and no consumer reads the carried copy.** `decode` calls `envelope.canonical_identity()` after `validate` and rejects a disagreement as `ArtifactIdentityMismatch`. `DecodedArtifact::identity` returns the derivation and documents it: "Re-derived, never read from the bytes."

**Fact — what the carried run therefore buys is a declaration check.** It fires when a producer's two derivations of one artifact disagree — the same class of hazard the codec was found carrying for canonical arena order, where the crate "had been carrying two definitions of canonical arena order that only happened to agree". That refusal does not need the preimage.

**The consumer that binds first is not a future one.** The same record measures that the envelope stores per-occurrence coverage evidence four times, putting the 1 MiB per-invocation embedding ceiling between 32 and 33 semantic operations, against a governed `semantic_operations` budget already raised to 62. A 49% constant saving does not repair that — [ADR 0104](0104-fold-the-per-record-graph-identity-as-a-digest.md) is the lever that changes the curve's shape — but the constant in front of it is large, cheap, and independently justified.

## Decision

1. **The manifest declares its artifact identity as a digest of that identity, not as the identity.** The trailing run is `H("tiler.artifact-envelope.identity-digest.v1\0" || exact canonical artifact identity bytes)`, thirty-two bytes, written **unframed**. Unframed is the codec's existing convention for digests rather than a new one: the header's manifest digest and every section descriptor's content digest are already fixed-width and unprefixed, because the governed algorithm the header names fixes the width. It also saves the eight-byte length prefix the framed spelling would keep.

2. **The producer-disagreement refusal is retained and keeps its exact meaning.** A decoder derives the identity from the decoded content, digests it under the same domain, and compares. `ArtifactIdentityMismatch` is unchanged and gains no sibling: the refused set is identical, because this check has always been between two things the decoder holds rather than between the wire and the world. A "digest disagreed" case and an "identity disagreed" case are one case.

3. **A fourth governed envelope digest domain is admitted.** `tiler.artifact-envelope.identity-digest.v1` joins `manifest-digest`, `section-digest`, and `envelope-digest`. It is separate from `manifest-digest` because that domain covers the manifest bytes this digest is written into. The no-prefix obligation is now over the crate's **eight** domains and is checked over the union of both containers, as [the ABI contract](../artifact-abi.md) already requires normatively.

4. **`MANIFEST_SCHEMA` takes a major step, `14.0` → `15.0`.** The run changes width *and* meaning, and this is the first step to need both of the reasons the earlier steps give separately. A `15.0` reader admitted at `minor <= implemented` would otherwise go on accepting a `14.0` manifest and read that manifest's eight-byte length prefix as the head of a digest — refusing, as `TrailingManifestBytes`, an artifact that is well formed at its own schema. And a manifest schema names one canonical byte spelling of an artifact, which this changes for every artifact.

5. **No identity domain moves, and no pin moves.** `encode_identity` reads the envelope and never the manifest, so `tiler.artifact-program.v15`, the expansion-cache subject derived from it, and the standard Metal path's two pinned values are unchanged.

## Consequences

**Measurement, on the zero-object hot-path fixture at this branch's base `eee734cf`, through the retained attribution harness.** Fixed content falls from **114,059 bytes to 57,978 — 56,081 bytes, 49.17%**. The manifest falls from 88,069 to 31,988. The `KernelProgramSubject` section stays 22,911 bytes and `BackendPayloadMetadata` stays 2,974, so the whole reduction is the identity run and its length prefix, less the thirty-two bytes that replace them. The derived identity is still 56,105 bytes, which is the step's own evidence that the subject did not move.

**Measurement — zero pins moved, verified rather than argued.** The complete workspace suite passes at this step with no identity constant, golden, or ledger value recomputed. That is the assertion decision 5 makes, checked.

**The three envelope consumers all improve by the same factor, immediately.** A validated cache hit runs its fail-closed integrity over 49% fewer bytes; an embedded artifact is 49% smaller against a fixed per-invocation ceiling; and the expansion cache's steady state halves at the same entry count.

**The envelope's coverage-evidence multiplicity falls from four to two.** The four copies were the framed `KernelProgramSubject` section, the manifest's per-entry stage subjects, the identity run's verbatim fold of the section, and the identity run's restatement of those stage subjects. The last two are the identity run. **The per-invocation embedding-ceiling crossing therefore moves from between 32 and 33 semantic operations to between 50 and 51** — `2 × (134·50² + 3650·50 + 719) = 1,036,438` and `2 × (134·51² + 3650·51 + 719) = 1,070,806`. That is still at the roadmap's ≥ 51-operation decoder layer and still below the governed budget of 62, which is the point ADR 0104 exists to make: this is a large constant in front of a quadratic, not a repair of it.

**What is given up.** A reader holding only the wire can no longer lift the artifact's identity without running the derivation. No such reader exists in this workspace, and the property is revisitable if one appears — reinstating the preimage is another schema step and no identity move.

**A diagnosis narrows.** A mismatch used to be inspectable byte by byte against the carried preimage. It is now "these disagree". Nothing consumed that difference: `ArtifactIdentityMismatch` is a unit variant and carries no payload today.

## The ADR 0074 convention-2 objection, and why it is answered rather than waived

Convention 2 holds that a canonical identity is opaque bytes a receiving crate treats as opaque and never re-derives locally, so a digest standing where canonical bytes stood needs an argument that the site is a **fold input** rather than an identity a consumer compares.

**The argument available here is a fact rather than a position.** This run is compared by the crate that is the *authority* for it, against bytes that same crate derives in the same call, and every public reader already reads the derivation instead. It is a producer's declaration to its own decoder, not an identity crossing a boundary.

**It does not reopen the 2026-07-27 no-layered-digests decision**, and the distinction is exact. That decision refused a *second identity value* for a layer that already has canonical-byte identity — a `semantic_digest` or `plan_digest` a consumer could compare instead of the real bytes, whose agreement with the real identity "could only ever be argued and never checked". This mints no such value. The digest has no type, no accessor, no public surface; nothing holds it, nothing compares it, and it cannot be lifted back out. All five layered identities remain canonical bytes compared byte for byte, `CanonicalArtifactProgramIdentity` included. What became a digest is the manifest's *declaration* of an identity, which is envelope framing — the same category as the other three hashing sites, all of which the decision names and admits.

## Alternatives considered

**Keep the preimage.** The wire stays self-describing: anything holding only the bytes could lift the identity without implementing the derivation. Rejected on cost against benefit — 56,081 bytes per envelope and half of every future manifest addition, for a reader that does not exist in this workspace and whose absence is checkable rather than assumed.

**Carry nothing.** Cheapest in bytes, and it needs the strongest justification of the three because it *deletes* the producer-disagreement refusal rather than making it cheaper — and that refusal is the reason the run exists. Rejected, explicitly, by the decision this record carries.

**Frame the digest behind a length prefix.** Eight bytes dearer and inconsistent with the three existing digest sites, none of which frames. Rejected: the governed algorithm tag in the header already fixes the width, so a prefix would state what is already stated.

**Reuse `MANIFEST_DIGEST_DOMAIN`.** Rejected. That domain covers the manifest bytes this digest is written into, so sharing it would let two subjects with different meanings collide under one separator — the exact hazard the domain-separation obligation exists for.

**Take a byte-overhead pin instead.** The originating research recommends one and this does not displace it: a pin on one fixture at one fixed operation count measures a coefficient, not a curve, and the research demonstrates from its own ladder that the budget widening which admits a program past the ceiling moved that fixture by exactly zero. The two answer different questions; this record does not close the pin's.
