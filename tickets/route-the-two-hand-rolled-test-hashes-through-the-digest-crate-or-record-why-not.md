---
id: route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not
title: Route the four workspace raw conformance hashes through the digest authority
status: in-progress
priority: p3
dependencies: []
related: [site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [implementation/compiler, implementation/reference, implementation/conformance, implementation/runtime, implementation/workspace, implementation/cargo-lock, implementation/digest, contracts/artifacts, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, identity, conformance]
claimed_from: todo
assignee: worker-external-digest
lease_expires_at: 1787097598
---
## User-visible outcome

Every Cargo-workspace member that reproduces an externally specified raw SHA-256 result uses the one digest implementation, without changing retained external digest bytes or weakening the governed-domain rule.

## Decision — accepted 2026-08-12

Tom accepted [ADR 0111](../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md) in the live coordination session. Implement its separately typed raw-external boundary:

- `Digest` remains exclusively the result of a Tiler-governed, domain-separated pre-image.
- Add opaque `ExternalDigest`, with no conversion to or from `Digest` and no wire constructor.
- Add `DigestAlgorithm::digest_external_record(bytes) -> ExternalDigest` and route retained `CC_SHA256` evidence through the explicit `DigestAlgorithm::Sha256` variant, never `GOVERNED`.
- Share one private algorithm dispatch inside `tiler-digest`; do not add a second SHA implementation, a synthetic domain, or a consumer dependency on `sha2`.

## Per-Fact audit — 2026-08-12 at `02ab5153`

- **False population and close condition.** The Cargo workspace has **four** handwritten raw SHA-256 copies outside `tiler-digest`, not three: `crates/tiler-compiler/src/governed/contraction_conformance.rs`, `crates/tiler-reference/tests/contraction_profile_cells.rs`, `crates/tiler-conformance/src/envelope.rs`, and `prototypes/serial-sum-run/src/proof.rs`. `Cargo.toml` lists the prototype as a workspace member. Treating it as a reason-recorded residual would contradict this ticket's outcome, so `implementation/runtime` is now in scope and all four copies must be deleted.
- **Verified common subject.** All four copies hash exact little-endian row-major `f32` result bytes and compare them with retained external `CC_SHA256` records. Their FIPS-vector checks establish that the required bytes are ordinary, undomained FIPS SHA-256.
- **Verified producer distinction.** The three crate sites and the prototype reconstruct the bytes independently of the retained string; conformance and the prototype additionally hash device readback. Centralizing only the digest algorithm does not merge the result producer with the external expected value.
- **False future-compatibility premise in the old recommendation.** `DigestAlgorithm::GOVERNED` means the algorithm this Tiler build writes, while the external record means SHA-256 permanently. Calling `GOVERNED.digest(b"", bytes)` would silently follow a future Tiler algorithm change. The exact `Sha256` variant must be stated at every external-record call.
- **Verified authority conflict.** `tiler-digest` currently documents only two governed, domain-bearing pre-image shapes. Its tests use `b""` only to reproduce published vectors. A public empty-domain convention would make a raw external record look like a governed Tiler subject; a distinct result type preserves the subject split.
- **Verified reachability with a contract correction.** Compiler's copy is behind `#[cfg(test)]` and reference's is an integration test, so both need direct development dependencies on `tiler-digest`. Conformance and the prototype use their helpers from device-reaching paths and need direct normal dependencies. ADR 0106 and `docs/architecture.md` currently say conformance reaches `tiler-digest` transitively and deliberately does not name it; once source names the new API, that description must be corrected rather than silently contradicted.
- **Verified identity consequence.** These strings are evidence observations, not artifact or layered identities. Routing them through the same SHA-256 implementation changes no retained bytes, artifact schema, digest tag, identity domain, cache key, or canonical encoding.
- **Verified independent byte-order check.** `crates/tiler-conformance/src/envelope/tests.rs`, anchor `the_digest_helper_reproduces_the_published_vectors`, checks little-endian `1.0f32`, rejects its big-endian spelling, and rejects element reordering. The compiler, reference, and prototype also pin FIPS vectors; the shared implementation's own suite adds complete padding-residue evidence.
- **Imprecise residual description.** `spikes/cache/cache_harness.rs` and `spikes/artifacts/artifact_envelope.rs` contain local `sha256` implementations, and the decoder-allocation harness uses `sha2::Sha256` directly. They are repository-authored but are not Cargo-workspace members or gate authorities. Keep them as explicit standalone-experiment exceptions; do not claim a repository-wide grep is empty.

**Id note.** The stable ticket id still says “two”. Keep it for graph identity; the title and body carry the corrected population.

## Required delivery

### Digest authority

- Factor one private SHA-256 dispatch shared by governed and external calls.
- Add `ExternalDigest` as an opaque, fixed-width result with exact-byte and lowercase-label observation only. No public constructor, `from_wire`, `From`, `Into`, comparison bridge, or serialization with `Digest`.
- Add the explicit external-record method on `DigestAlgorithm`. Document that callers select the algorithm variant named by the external record and that `GOVERNED` is not that authority.
- Move the published FIPS-vector reproduction onto the external path. Retain governed-domain, qualified-preimage, tag, padding, and performance tests on their actual subjects.
- Add a compile-time or source-population check strong enough that the four migrated callers cannot drift back to `GOVERNED.digest(b"", ...)` or a local SHA implementation unnoticed.

### Consumers and dependencies

- Route all four helpers through `DigestAlgorithm::Sha256.digest_external_record` while preserving each `to_le_bytes()` pre-image and retained string.
- Delete all four compression implementations and their obsolete “cannot reach the digest” explanations.
- Add direct development dependencies for compiler/reference and direct normal dependencies for conformance/prototype. Update `Cargo.lock` and the live architecture dependency block atomically.
- Correct ADR 0106's retained transitive-only statement with a dated note rather than rewriting its historical accepted text.

### Evidence

- Re-bind the existing conformance little-endian and row-order test to the shared path.
- Preserve FIPS vector checks at the owning digest surface; consumer tests should pin their subject encoding and retained comparisons, not duplicate the algorithm's complete vector suite without a distinct purpose.
- Perturb `to_le_bytes()` to `to_be_bytes()` and show a retained comparison failing. Separately substitute a governed non-empty-domain result and show a retained comparison failing. Restore both before gates and quote both failure messages in the outcome.
- Census Cargo-workspace sources and assert the only `sha2::Sha256` implementation remains in `tiler-digest`; record the three standalone spike exceptions separately.

## Explicit non-goals

No new `tiler.*` domain, retained digest rebaseline, governed algorithm/tag change, artifact wire or identity-domain step, direct `sha2` dependency in a consumer, conversion between the two digest result types, or required edit of standalone `spikes/` trees.

No attempt to make the external result authentic. It reproduces an externally recorded digest; the provenance and trustworthiness of that record remain the evidence owner's responsibility.

## Closes when

The separately typed raw-external surface is implemented and documented; all four Cargo-workspace copies are deleted and routed through explicit SHA-256 selection; direct dependency records and architecture prose match source; retained strings are unchanged; byte-order, element-order, algorithm-selection, and domain-substitution checks reach their subjects; `tkt guard`, package tests, Clippy, rustdoc, workspace nextest/doc-tests, and the repository gate pass; and the outcome records the exact census and both demonstrated perturbation failures.
