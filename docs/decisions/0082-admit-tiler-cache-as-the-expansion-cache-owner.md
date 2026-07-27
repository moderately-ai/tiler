---
schema: "tiler-doc/v1"
id: "ADR-0082"
kind: "decision"
title: "Admit tiler-cache as the expansion cache owner"
topics: ["rust", "workspace", "dependencies", "cache", "artifacts", "concurrency", "durability"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
evidence: ["tiler.research.cache.crash-race-protocol", "tiler.research.workspace.prototype-crate-layout-and-msrv"]
depends_on: ["ADR-0050"]
refines: ["ADR-0077", "ADR-0081"]
ticket: "implement-the-expansion-cache-protocol"
---

# 0082: Admit tiler-cache as the expansion cache owner

**Status:** accepted. Tom decided this on 2026-07-25 on the evidence below. It admits an eighth reusable library, `tiler-cache`, and amends the packaging profile clause that withheld a generalized cache.

## Context

**Fact — the accepted ownership table assigned the cache to `tiler-metal-aot` and, in the same row, forbade it every dependency.** [`docs/architecture.md`](../architecture.md)'s component ownership table gave that crate "Expansion-time Apple tool invocation, cross-process content cache, atomic publication, byte embedding, …" and stated its forbidden dependencies as "Every workspace and third-party dependency, Candle included: its empty closure is decided, not incidental". `scripts/check_workspace.py` pinned `"tiler-metal-aot": []` mechanically, so the closure was checked rather than merely described.

**Inference — that row was internally unsatisfiable, not merely awkward.** [ADR 0050](0050-use-immutable-self-validating-expansion-cache-entries.md) requires a reader to validate "bounded framing, embedded key, schemas, manifest, section lengths/digests, and required meanings on every hit". Section digests need a hash function. The governed one is `tiler.digest.sha-256.v1` in `crates/tiler-artifact/src/program/codec/digest.rs`, which was `pub(crate)`. The assigned owner therefore could not reach the governed algorithm even if its closure were opened, and a local digest inside the driver would make it a second identity authority over one subject — the thing `crates/tiler-metal-aot/src/family.rs` and the digest module's own documentation both refuse. No implementation satisfied the row as written.

**Fact — [ADR 0077](0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) says the opposite of the table, and both were accepted.** Its item 1 states that the driver "does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the proc-macro layer". Two accepted authorities assigned one responsibility to different components, so one of them was wrong.

**Fact — the packaging profile deliberately withheld a generalized cache.** `docs/architecture.md` says the profile "deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-*runtime* crates until the proof reaches those boundaries", and ADR 0077 item 5 restates the omission. [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) separately puts "a new publicly reachable namespace — a new crate" in the always-ask category.

**Fact — the key subject already landed, deliberately as bytes.** `crates/tiler-metal-aot/src/identity.rs` emits the driver's complete canonical compilation subject — domain-separated, length-prefixed, over every input determining the `metallib`, with the request and toolchain records destructured irrefutably so a new input fails to compile until it reaches identity. It emits bytes rather than a digest precisely because this crate owns no hash function, leaving digesting to whichever component owns the governed algorithm. It also excludes tool paths from the subject, because a path states where a host keeps a file rather than what the file contains, and bounds reuse to `SameHost` with a typed refusal.

## Decision

The expansion cache is a dedicated `tiler-cache` crate depending on `tiler-artifact`.

1. `tiler-metal-aot` does not implement the expansion cache, and its dependency closure stays empty. ADR 0077 item 1 stands; the ownership table is corrected to match it. The driver keeps the canonical compilation-key subject, which is a fact about its own inputs.

2. `tiler-cache`'s single edge is a decided property, not an accident of ordering. It reaches `tiler-artifact` for exactly two things ADR 0050 requires on every hit and a storage protocol cannot supply itself: the governed digest, which validates a stored bundle's section digests, and `decode_artifact`, which re-proves the carried envelope's manifest, section digests, and canonical identity. It acquires nothing that would let a cache decide something about a program.

3. `tiler-artifact` promotes a minimal digest surface: `DigestAlgorithm`, the opaque `Digest`, and `DIGEST_BYTES`. `digest_parts` and `envelope_digest` stay crate-private, so an outside caller can digest a subject under one explicit domain and cannot express the ambiguous concatenation that `digest_parts` documents as the caller's obligation, nor construct an envelope association.

4. The packaging profile's "generalized cache" omission is **amended**, not reinterpreted. `tiler-runtime` was admitted by applying a test ADR 0077 stated; no equivalent test admits this crate, because ADR 0050's expansion cache is the thing the clause named. The clause is superseded on this point and continues to withhold every other cache — a runtime pipeline-state cache, a compiler plan cache, and a general-purpose content-addressed store.

5. `AGENTS.md`'s five correctness properties are the crate's specification: complete cache identity, validation on every hit, immutable entries, atomic publication, and defined crash and race behaviour.

6. A rejection is control flow and never silence. ADR 0050 makes a corrupt, truncated, misplaced, or schema-invalid entry a miss, because treating cache failure as compilation failure "would make an optional accelerator a correctness dependency". Every such miss must carry a typed reason the caller receives, so a cache permanently rejecting every entry is observable rather than merely slow.

## Consequences

- The workspace carries eight reusable libraries. `scripts/check_workspace.py` pinned the member, the description, the directory, and the exact single edge, so the closure was a checked contract. `scripts/check_workspace.py` was deleted by `e197176`, which replaced the Python gate with the root `Makefile`; it is now a described one.
- A bundle carries the producer's compilation subject beside the artifact envelope, and a reader re-derives the key from it on every hit. The embedded key alone would only prove a bundle was published under some key; re-derivation is what refuses an internally consistent bundle filed under a key its own subject does not produce.
- What the crate cannot prove is stated rather than assumed. It cannot prove that a facet's bytes are the supplying authority's real subject, because telling a genuine one from an invented one means parsing an encoding it does not own; it cannot prove the subject describes the carried artifact, for the same reason; and it cannot enforce the producer's `SameHost` reuse bound, because no code can decide that a configured root is host-local.
- **The complete subject is composed by the cache and never interpreted by it.** A bundle carries a whole artifact envelope, so a conforming subject must determine the artifact and not only the compiled object — which is what `docs/backends/metal.md` already requires when it says "full artifact identity is the key". `tiler_cache::expansion::ComposedSubject` frames two facets, the backend compilations and the artifact program, each domain-tagged, counted, and length-prefixed; `SubjectFacets` is destructured irrefutably by the composer and its facet table is sized by the facet enumeration, so a facet added in either direction fails to compile until it reaches the bytes. `tiler-metal-aot`'s landed subject is **wrapped** as one run of the compilations facet, not restated — which is what keeps the driver from acquiring a dependency and what carries its `SameHost` evidence tag through composition byte for byte. `lookup` and `get_or_publish` take a composed subject and nothing else, so a caller can no longer key an entry on the driver's subject alone.
- **The composed subject is not yet fillable, and the refusal says so.** The artifact-program facet has a producer — `PayloadMetadata::identity` and `ArtifactProgramBuilder::push_pending_payload` derive it without an object. The backend-compilation facet does not: `tiler-metal-aot` keeps `CompilationIdentity::as_bytes` `pub(crate)`, so no other crate can obtain those bytes, and `promote-the-metal-aot-compilation-identity` owns closing it. `ComposedSubject::compose` refuses an empty facet rather than composing a shorter key, so the state of the crate is "composable, not yet usable" and it fails loudly instead of silently under-keying. This bullet previously named the artifact-program facet as the missing one, reasoning that `CanonicalArtifactProgramIdentity` needs the payload digest and therefore the compiled bytes; that was a false description of the code, since the digest is over the payload metadata — source, target, flags, toolchain provenance, and no object byte — and every fact it folds is a compilation input. The correction is to what this bullet described, not to what this ADR decided.
- Standard-library `File::lock` stays behind an internal adapter, as `docs/architecture.md` requires, because Rust documents that its mapping to a platform primitive may change and that the lock may be advisory.
- The cross-process crash and race properties are **measured against this bundle**, by `expansion::harness`. The in-crate threaded suite is still threaded and still says so; the harness is separate and uses real processes, re-executing the crate's test binary so an armed child aborts inside the real publication path at each of nine named phases. The seam is `cfg(test)` rather than a Cargo feature, because Cargo unifies features across a build graph and one unrelated crate enabling it would arm mid-publication aborts inside somebody's production cache. It is a bounded measurement on one host, recorded in `spikes/cache/results/`, not a portable guarantee.
- **The harness substitutes a stand-in payload validator, and that is a consequence of item 2 rather than an oversight.** Driving the public `get_or_publish` would need a real artifact envelope, which needs a `SemanticProgram`, which needs `tiler-ir` — an edge this record decides the crate does not have. The children therefore drive the crate-private protocol with a validator accepting any non-empty bytes. Every byte of the bundle frame and every filesystem operation is real, and the substituted validator sits strictly inside an envelope the frame has already delimited, so it changes how long the pre-rename window is and not what a killed writer leaves at a content path. A positive end-to-end hit carrying a real compiled artifact is unmeasured and belongs to the orchestrator holding both crates.

## Alternatives considered

**The cache lives in `tiler-artifact`, beside the envelope and the digest it validates.** The cheapest option: no new crate, no promotion, no edge. Rejected because it merges a storage protocol — with its own crash, race, and durability contract — into a crate whose stated responsibility is "encoding, compatibility, runtime fact binding", and because a consumer wanting the cache would then have no way to take it without the artifact model.

**The cache stays in `tiler-metal-aot` and its empty closure is spent on a `tiler-artifact` edge.** Rejected because the property ADR 0077 item 2 calls decided rather than incidental — "a reader auditing what Tiler asks the Metal compiler to do reads one crate with nothing underneath it" — is destroyed by the first dependency rather than degraded by it.

**A hash function local to the cache, needing no edge at all.** Rejected on the same ground that eliminated a local digest in the driver: `docs/artifact-abi.md` requires every digest use to name one governed algorithm, and a second implementation of the same subject is a second identity authority whether or not the two agree today.

## Traceability

Implements ADR 0050's accepted protocol and the [crash/race research note](../research/cache/crash-and-race-protocol.md). Corrects the `tiler-metal-aot` ownership row in favour of ADR 0077 item 1, and amends the packaging profile clause ADR 0056 introduced and ADR 0077 restated.
