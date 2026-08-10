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

- **False population.** There are **three** hand-written SHA-256 copies outside `tiler-digest` under `crates/`, not two: `crates/tiler-compiler/src/governed/contraction_conformance.rs`, `crates/tiler-reference/tests/contraction_profile_cells.rs`, and the later `crates/tiler-conformance/src/envelope.rs` copy anchored at `pub(crate) fn sha256_hex`. Together with `tiler-digest`'s `sha2::Sha256`, the **crates** population has four implementations. Absolute "the workspace has four" is false: `prototypes/serial-sum-run/src/proof.rs` is a fourth hand-rolled raw-external twin (same L3 retained-digest subject class and domain-refusal rationale), and spikes carry further local digests plus one direct `sha2::Sha256` use. Those residual sites are outside this ticket's declared scopes (see Residual sites and Closes when).
- **Verified common subject.** All three local copies hash exact little-endian row-major `f32` result bytes and compare them with retained external `CC_SHA256` records. Their FIPS-vector checks establish the required bytes are ordinary, undomained FIPS SHA-256.
- **False reachability premise.** Compiler and reference cannot name a transitive `tiler-digest` dependency; they need direct dev-dependencies, which changes both manifests and the workspace lockfile. Conformance already reaches the algorithm through its artifact dependency, but its own comment refuses that API because it requires a domain. The required implementation population is therefore compiler, reference, conformance, workspace manifests, and Cargo.lock.
- **Verified authority conflict.** `tiler-digest` says every governed subject carries a real domain and exposes only `digest(domain, bytes)` / `digest_qualified`. Its tests use `b""` solely to reproduce published FIPS vectors. The three callers are not Tiler identities: they reproduce external device/probe records whose pre-image is the raw result bytes. Adding a `tiler.*` domain would change every retained digest and stop comparing the same evidence.
- **Verified one-authority rule, but it does not answer the exception.** ADR 0104 and the digest crate require one mapping from governed algorithm to implementation. They do not decide whether an external raw conformance record may call that implementation with an empty domain or needs a separately named raw-external API/contract.

The original two-site scope was incomplete. `implementation/conformance`, `implementation/workspace`, `implementation/cargo-lock`, `implementation/digest`, and `contracts/artifacts` are now declared before a decision or implementation is attempted.

**Id note.** The stable ticket id still says "two"; the title and body say three. Do not rename the id.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The independent little-endian byte-order check already exists at the conformance site for the current local helper: `crates/tiler-conformance/src/envelope/tests.rs` `the_digest_helper_reproduces_the_published_vectors` asserts `result_digest(&[0x3f80_0000])` equals `sha256_hex` of LE `00 00 80 3f`, inequality for BE, and row-major order inequality. Post-decision work re-binds that check to the routed implementation rather than introducing it from zero. Compiler and reference still only run the two FIPS vectors. After any shared route, still require the two subject perturbations (`to_be_bytes`, non-empty domain) each reddening a retained-digest comparison, then restore both.

## Residual sites (out of declared scopes)

These hits remain after routing only the three named crate callers. They are not in this ticket's required work or scopes; close conditions record reasons rather than demanding they vanish:

- `prototypes/serial-sum-run/src/proof.rs` — fourth hand-rolled raw-external SHA-256 twin for the same retained L3 digest subject class; exploratory prototype, not gate-owned under current scopes. If Tom chooses option 1 or 2 and this ticket stays crates-only, file a remainder (likely needing `implementation/runtime`) so the full-tree census can close with a recorded route or reason rather than an unmentioned hit.
- `spikes/cache/cache_harness.rs`, `spikes/artifacts/artifact_envelope.rs` — hand-rolled digests in exploratory spikes (out of gate).
- `spikes/artifacts/decoder-allocation/harness` — direct `sha2::Sha256` use in a spike (out of gate).

## Decision boundary

Tom decides how external raw digest evidence reaches the governed implementation while preserving the external bytes:

1. **Admit an explicit empty-domain exception** for published FIPS vectors and externally defined raw device/probe records. Route all three sites through `DigestAlgorithm::GOVERNED.digest(b"", bytes)` and document that this is not a governed Tiler subject.
2. **Add an explicit raw-external SHA entry point** whose name and documentation make the exception unambiguous, while still mapping to the same sole implementation.
3. **Keep local copies.** This preserves the current domain API but abandons the digest crate's structural one-implementation purpose for three byte-identical implementations.

**Recommendation: option 1.** The raw bytes are fixed by external evidence, the existing implementation already exposes exactly those bytes for its FIPS vectors, and a documented external-record exception avoids inventing a second algorithm surface. **Strongest counterpoint:** passing `b""` through a public API whose crate documentation says every governed subject has a real domain makes a test-only convention look like a supported subject class; an explicit raw-external method is clearer if this exception is expected to grow.

No synthetic domain is an option: it would no longer reproduce the retained records.

## Required work after the decision

- Route all three crate callers under the declared scopes, not only the two the original ticket named.
- Preserve every retained digest string and the exact `to_le_bytes()` pre-image.
- Re-bind the existing independent raw-byte-order check at the conformance site to the routed helper; do not prove a helper by comparing it only with itself. Still require LE + non-empty-domain subject perturbations after any shared route.
- Remove the three hand-written compression implementations and their stale Cargo/reachability explanations if either shared route is accepted.
- Record the external-raw exception in the digest/artifact authority at the narrowest accepted surface.
- Perturb the subject twice: change `to_le_bytes()` to `to_be_bytes()`, then change the accepted raw-domain input to a non-empty domain. Each must fail a retained-digest comparison; restore both before gates.

## Explicit non-goals

No new `tiler.*` domain, retained digest rebaseline, digest algorithm/tag change, identity-domain step, artifact wire change, or direct `sha2` dependency in consumer crates. No required edit of prototypes or spikes under this ticket's scopes.

## Closes when

Tom resolves the raw-external boundary; all three crate-scoped workspace-authored copies are routed or explicitly justified against that decision; retained digests and byte order are independently checked on the routed path (conformance LE check re-bound, not invented); and the `fn sha256|Sha256::` census **within declared scopes** contains only the governed implementation plus sites whose current reason is recorded. Residual prototype and spike sites listed above count as reason-recorded for this ticket; they do not block close unless Tom expands scope or a post-decision remainder owns the prototype twin.
