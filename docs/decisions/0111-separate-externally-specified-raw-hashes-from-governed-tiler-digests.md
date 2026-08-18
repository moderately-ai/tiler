---
schema: "tiler-doc/v1"
id: "ADR-0111"
kind: "decision"
title: "Separate externally specified raw hashes from governed Tiler digests"
topics: ["digest", "identity", "conformance", "testing", "public-boundary"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.scheduling.first-metal-contraction-realizations", "tiler.research.artifacts.manifest-fixed-content-growth"]
depends_on: ["ADR-0074", "ADR-0104", "ADR-0106"]
ticket: "route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not"
---

# 0111: Separate externally specified raw hashes from governed Tiler digests

**Status:** accepted by Tom on 2026-08-12 in the live coordination session; implemented on 2026-08-18 by [`route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not`](../../tickets/route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not.md), which records the per-site evidence and the demonstrated perturbations.

**Implementation note — 2026-08-18.** Everything decided below is in force. `ExternalDigest` and `DigestAlgorithm::digest_external_record` are the public surface; both result paths run one private `compress` dispatch, so the workspace holds one SHA implementation and no consumer depends on `sha2`. All four copies are deleted and every caller spells `Sha256`. Retained external digest strings are byte-identical. Two mechanisms hold the boundary that prose cannot: `ExternalDigest` carries `compile_fail` doctests for the absent wire constructor, both `From` directions, the cross-type comparison, and the return-type substitution; and `crates/tiler-digest/tests/one_sha_implementation.rs` censuses the workspace member sources for any second SHA implementation or `sha2` reach, and the four migrated callers for the governed alias and the empty-domain spelling. The direct edges this decision requires are declared and appear in `docs/architecture.md`'s live block.

## Context

**Fact — four Cargo-workspace members carry copied SHA-256 implementations for one externally defined evidence subject.** The copies are in `tiler-compiler`'s contraction conformance test, `tiler-reference`'s contraction profile test, `tiler-conformance`'s retained-result path, and `tiler-prototype-run`. Each hashes the exact little-endian row-major binary32 result bytes that an external probe passed to `CC_SHA256`, then compares the lowercase result with a retained `result_sha256`. The compiler and reference copies run only in tests; the latter two also support device-reaching evidence. Repository spikes carry further isolated hash implementations, but they are not Cargo-workspace members and are not part of the gated authority population this decision changes.

**Fact — the bytes cannot acquire a Tiler domain.** The retained record names ordinary FIPS 180-4 SHA-256 over the result bytes. Prefixing a `tiler.*` domain would ask a different question and invalidate every retained comparison. Conversely, treating the empty byte string as a normal domain on [`DigestAlgorithm::digest`](../../crates/tiler-digest/src/lib.rs) would make a raw externally specified record look like another governed Tiler subject, weakening the domain-separation discipline that API exists to hold.

**Fact — `DigestAlgorithm::GOVERNED` is the wrong selector for an external record even while it aliases SHA-256.** `GOVERNED` means the algorithm this Tiler build writes. The retained probe record means SHA-256 regardless of a future Tiler writer policy. A call through `GOVERNED` would therefore let an unrelated policy change silently change the algorithm used to reproduce old evidence; an explicit `Sha256` spelling instead becomes a compile-time stop if that exact algorithm ceases to be available.

**Fact — copied implementations are not independent evidence.** They are byte-identical transcriptions checked against the same small FIPS vector set. `tiler-digest` already owns the measured `sha2` implementation and checks it against published vectors, every padding residue, and an independently generated digest. Keeping four copies adds algorithm authorities and maintenance surface without adding an independent implementation or an independently sourced expected value.

## Decision

`tiler-digest` exposes a separately typed raw-external result path in addition to its governed domain-separated path. The two subject classes remain mutually exclusive in the public type system:

- [`Digest`](../../crates/tiler-digest/src/lib.rs) remains the result of a Tiler-governed, explicitly domain-separated pre-image.
- A new opaque `ExternalDigest` names only the result of reproducing an externally specified raw digest record. It has no conversion to or from `Digest`, no wire constructor, and exposes only the observation needed by evidence consumers, such as exact bytes and a lowercase hexadecimal label.
- `DigestAlgorithm::digest_external_record(bytes)` returns `ExternalDigest`. Every consumer names the exact algorithm variant from the external record — `DigestAlgorithm::Sha256` for the retained `CC_SHA256` corpus — and does not call the `GOVERNED` alias.

Both paths use one private implementation dispatch inside `tiler-digest`. The external path adds no domain, no algorithm tag, no identity value, and no serialization. It is a reproduction mechanism for evidence another authority defined, not an alternate spelling for a Tiler identity.

All four Cargo-workspace copies route through this surface and are deleted. The compiler and reference add direct development dependencies on `tiler-digest`; conformance and the prototype runner add direct normal dependencies because their non-test/device paths use the result helper. A source file that names a crate's public item owns a direct Cargo edge. This deliberately corrects ADR 0106's statement that conformance reaches `tiler-digest` only transitively: that statement describes the current source and ceases to be true when conformance acquires a direct use.

The standalone `spikes/` trees remain outside this implementation. Their local hashes are bounded experimental producers, not workspace-gated identity or conformance authorities; their exclusion is recorded explicitly rather than hidden behind a crates-only grep. A spike promoted into a Cargo workspace member must route through this decision's surface or bring a new accepted reason not to.

## Consequences

- Every Cargo-workspace use of raw SHA-256 has one implementation authority, while governed Tiler subjects still require a real domain.
- The four retained digest populations and their little-endian row-major pre-images remain byte-identical. No artifact schema, digest tag, identity domain, cache key, or retained measurement moves.
- A future change to Tiler's governed writer algorithm cannot silently reinterpret a SHA-256 evidence record. The explicit variant either remains available or the consumer stops compiling.
- `tiler-digest` gains one public result type and one public operation. That is intentional public-language surface under ADR 0075, but it does not make the external result a Tiler identity or grant an external caller access to any governed subject domain.
- The live architecture dependency table moves only when implementation lands. At that point it names the new direct edges and retains `tiler-digest` as the bottom implementation authority.
- Subject perturbations must change `to_le_bytes()` to `to_be_bytes()` and separately replace the raw path with a non-empty-domain governed path; each must make a retained comparison fail with the subject restored afterwards.

## Alternatives considered

**Call `DigestAlgorithm::GOVERNED.digest(b"", bytes)`.** This produces the right bytes today and needs no new type. Rejected because it represents an externally fixed SHA-256 record through mutable Tiler writer policy, publishes an empty-domain convention on an API whose governed subjects require real domains, and leaves governed and raw results indistinguishable.

**Return ordinary `Digest` from an explicitly named raw method.** This fixes algorithm selection if callers name `Sha256`, and runtime cost is identical to the decision. Rejected because it erases the subject distinction at the return boundary and lets raw evidence flow through APIs documented to carry governed content digests.

**Depend directly on `sha2` from every evidence consumer.** This removes the handwritten compression code. Rejected because each consumer again becomes an algorithm-selection and rendering authority, defeating the structural purpose of the bottom crate.

**Keep the copied implementations as independent checks.** Rejected because copied code with shared vectors is not independent evidence. It adds defect and maintenance surface while the retained external digest strings already provide the independent expected values.
