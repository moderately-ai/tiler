---
id: route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not
title: Decide and route the three raw conformance hashes through the digest authority
status: awaiting-decision
priority: p3
dependencies: []
related: [site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [implementation/compiler, implementation/reference, implementation/conformance, implementation/workspace, implementation/cargo-lock, implementation/digest, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Every workspace-authored raw SHA-256 conformance hash routes through the one digest implementation without changing the retained external digest bytes, or the artifact contract records why raw external hashes are an explicit exception to the governed-domain rule.

## Per-Fact audit — 2026-08-09

- **False population.** There are **three** hand-written SHA-256 copies outside `tiler-digest`, not two: `crates/tiler-compiler/src/governed/contraction_conformance.rs`, `crates/tiler-reference/tests/contraction_profile_cells.rs`, and the later `crates/tiler-conformance/src/envelope.rs` copy anchored at `pub(crate) fn sha256_hex`. Together with `tiler-digest`'s `sha2::Sha256`, the workspace has four implementations.
- **Verified common subject.** All three local copies hash exact little-endian row-major `f32` result bytes and compare them with retained external `CC_SHA256` records. Their FIPS-vector checks establish the required bytes are ordinary, undomained FIPS SHA-256.
- **False reachability premise.** Compiler and reference cannot name a transitive `tiler-digest` dependency; they need direct dev-dependencies, which changes both manifests and the workspace lockfile. Conformance already reaches the algorithm through its artifact dependency, but its own comment refuses that API because it requires a domain. The required implementation population is therefore compiler, reference, conformance, workspace manifests, and Cargo.lock.
- **Verified authority conflict.** `tiler-digest` says every governed subject carries a real domain and exposes only `digest(domain, bytes)` / `digest_qualified`. Its tests use `b""` solely to reproduce published FIPS vectors. The three callers are not Tiler identities: they reproduce external device/probe records whose pre-image is the raw result bytes. Adding a `tiler.*` domain would change every retained digest and stop comparing the same evidence.
- **Verified one-authority rule, but it does not answer the exception.** ADR 0104 and the digest crate require one mapping from governed algorithm to implementation. They do not decide whether an external raw conformance record may call that implementation with an empty domain or needs a separately named raw-external API/contract.

The original two-site scope was incomplete. `implementation/conformance`, `implementation/workspace`, `implementation/cargo-lock`, `implementation/digest`, and `contracts/artifacts` are now declared before a decision or implementation is attempted.

## Decision boundary

Tom decides how external raw digest evidence reaches the governed implementation while preserving the external bytes:

1. **Admit an explicit empty-domain exception** for published FIPS vectors and externally defined raw device/probe records. Route all three sites through `DigestAlgorithm::GOVERNED.digest(b"", bytes)` and document that this is not a governed Tiler subject.
2. **Add an explicit raw-external SHA entry point** whose name and documentation make the exception unambiguous, while still mapping to the same sole implementation.
3. **Keep local copies.** This preserves the current domain API but abandons the digest crate's structural one-implementation purpose for three byte-identical implementations.

**Recommendation: option 1.** The raw bytes are fixed by external evidence, the existing implementation already exposes exactly those bytes for its FIPS vectors, and a documented external-record exception avoids inventing a second algorithm surface. **Strongest counterpoint:** passing `b""` through a public API whose crate documentation says every governed subject has a real domain makes a test-only convention look like a supported subject class; an explicit raw-external method is clearer if this exception is expected to grow.

No synthetic domain is an option: it would no longer reproduce the retained records.

## Required work after the decision

- Route all three callers, not only the two the original ticket named.
- Preserve every retained digest string and the exact `to_le_bytes()` pre-image.
- Add an independent raw-byte-order check at the conformance site; do not prove a helper by comparing it only with itself.
- Remove the three hand-written compression implementations and their stale Cargo/reachability explanations if either shared route is accepted.
- Record the external-raw exception in the digest/artifact authority at the narrowest accepted surface.
- Perturb the subject twice: change `to_le_bytes()` to `to_be_bytes()`, then change the accepted raw-domain input to a non-empty domain. Each must fail a retained-digest comparison; restore both before gates.

## Explicit non-goals

No new `tiler.*` domain, retained digest rebaseline, digest algorithm/tag change, identity-domain step, artifact wire change, or direct `sha2` dependency in consumer crates.

## Closes when

Tom resolves the raw-external boundary; all three workspace-authored copies are routed or explicitly justified against that decision; retained digests and byte order are independently checked; and the complete `fn sha256|Sha256::` census contains only the governed implementation plus sites whose current reason is recorded.
