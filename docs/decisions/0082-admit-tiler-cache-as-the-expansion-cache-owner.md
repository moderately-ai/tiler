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

- The workspace carries eight reusable libraries. `scripts/check_workspace.py` pins the member, the description, the directory, and the exact single edge, so the closure is a checked contract.
- A bundle carries the producer's compilation subject beside the artifact envelope, and a reader re-derives the key from it on every hit. The embedded key alone would only prove a bundle was published under some key; re-derivation is what refuses an internally consistent bundle filed under a key its own subject does not produce.
- What the crate cannot prove is stated rather than assumed. It cannot prove the producer's subject is *complete*; it cannot prove the subject describes the carried artifact, because that would require parsing the producer's encoding; and it cannot enforce the producer's `SameHost` reuse bound, because no code can decide that a configured root is host-local.
- **The complete subject does not yet exist as one canonical byte run, and the crate does not invent it.** A bundle carries a whole artifact envelope, so a conforming subject must determine the artifact and not only the compiled object — which is what `docs/backends/metal.md` already requires when it says "full artifact identity is the key". `tiler-metal-aot`'s landed subject determines the `metallib` and says nothing about the plan variants, ABI bindings, or routing wrapped around it, so it is half of a conforming subject. Composing the whole is `compose-the-complete-expansion-cache-subject`, and until it lands a caller passing the driver's subject alone is under-keying and this crate cannot detect it.
- Standard-library `File::lock` stays behind an internal adapter, as `docs/architecture.md` requires, because Rust documents that its mapping to a platform primitive may change and that the lock may be advisory.
- The cross-process crash and race properties remain **untested against this bundle**. The in-crate suite is threaded and says so; `spikes/cache/cache_harness.rs` exercises real killed processes against its own miniature frame. Porting the harness is a separate ticket and nothing here claims its result.

## Alternatives considered

**The cache lives in `tiler-artifact`, beside the envelope and the digest it validates.** The cheapest option: no new crate, no promotion, no edge. Rejected because it merges a storage protocol — with its own crash, race, and durability contract — into a crate whose stated responsibility is "encoding, compatibility, runtime fact binding", and because a consumer wanting the cache would then have no way to take it without the artifact model.

**The cache stays in `tiler-metal-aot` and its empty closure is spent on a `tiler-artifact` edge.** Rejected because the property ADR 0077 item 2 calls decided rather than incidental — "a reader auditing what Tiler asks the Metal compiler to do reads one crate with nothing underneath it" — is destroyed by the first dependency rather than degraded by it.

**A hash function local to the cache, needing no edge at all.** Rejected on the same ground that eliminated a local digest in the driver: `docs/artifact-abi.md` requires every digest use to name one governed algorithm, and a second implementation of the same subject is a second identity authority whether or not the two agree today.

## Traceability

Implements ADR 0050's accepted protocol and the [crash/race research note](../research/cache/crash-and-race-protocol.md). Corrects the `tiler-metal-aot` ownership row in favour of ADR 0077 item 1, and amends the packaging profile clause ADR 0056 introduced and ADR 0077 restated.
