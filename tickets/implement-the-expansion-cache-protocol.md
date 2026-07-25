---
id: implement-the-expansion-cache-protocol
title: Implement the expansion cache protocol
status: done
priority: p1
dependencies: [decide-the-expansion-cache-owner-and-digest-authority]
related: [prototype-expansion-content-cache, compose-the-complete-expansion-cache-subject, bind-the-cache-subject-to-the-carried-payload-provenance, port-the-cache-harness-to-the-production-bundle, fuzz-the-expansion-cache-framing-paths, inject-deterministic-expansion-cache-io-failures, measure-expansion-cache-durability-policies, define-supported-expansion-cache-filesystems, design-bounded-expansion-cache-garbage-collection, exercise-the-expansion-cache-under-cargo-and-rust-analyzer, accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache, implementation/artifact, implementation/workspace, contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets, implementation/cargo-lock, contracts/navigation]
paths: []
tags: [implementation, cache, concurrency, durability]
claimed_from: todo
assignee: agent-cache2
lease_expires_at: 1785011688
---
Implement the cross-process expansion cache protocol ADR 0050 accepts and `docs/research/cache/crash-and-race-protocol.md` specifies, in whichever component `decide-the-expansion-cache-owner-and-digest-authority` names.

`prototype-expansion-content-cache` landed the complete-key *subject* — `crates/tiler-metal-aot/src/identity.rs`, canonical bytes over every input a compilation's output depends on, with the toolchain-evidence class that bounds reuse — and could go no further, because every candidate home for the protocol is blocked by an authority a worker cannot resolve. That ticket's Outcome records the block; this one is the remainder.

## In scope, all of it from the accepted protocol

- The cache namespace: `<root>/v1/entries/<K[0..2]>/<K>.bundle`, `locks/`, `tmp/`, with a path parser that accepts only fixed-width lowercase hexadecimal and **never truncates a key to fit**.
- Stable per-key advisory locking through an internal adapter — `docs/architecture.md` requires the adapter even though the pinned nightly carries the Rust 1.89 `File::lock` API — plus the post-lock recheck.
- `create_new` unique same-filesystem temporary publication, separate-descriptor validation of the completed bytes, one atomic rename, and `EXDEV` reported as a miss.
- One immutable self-validating bundle per key: cache magic, schema, algorithm/domain identifier, exact total length, embedded key equality against the requested key, the artifact manifest and envelope identity, and **every declared section length and digest**. This is the clause that needs the governed digest and is why the owner question blocks the work.
- Corruption recovery: a locked writer atomically replaces a corrupt final entry.
- Bounded limits and typed diagnostics: maximum bundle bytes, maximum entry count, quarantine bound, temporary-file grace period.
- Internal GC that takes the per-key lock before eviction and **retains lock files**.

## The rejection posture, stated so it is not rediscovered

ADR 0050 makes a corrupt, truncated, misplaced, or schema-invalid entry a **miss**, and the argument for it is in the record: treating cache failure as compilation failure "would make an optional accelerator a correctness dependency". That is control flow, not silence. Every such rejection must carry a typed reason the caller receives and can log, so a cache that is permanently rejecting every entry is observable rather than merely slow. A rejection that is not reported is a defect even though the miss is correct.

## Tests the ticket owes

Threaded lock exclusion and post-lock recheck are in-crate. The cross-process crash and race properties are not: they need real processes killed at each publication phase, which `spikes/cache/cache_harness.rs` already does for the spike's miniature frame. This ticket must either re-point that harness at the production bundle or state plainly that the crash/race property is exercised only by the spike, and claim nothing stronger. It must not report a threaded test as evidence for a process-crash property.

## Follow-up gates the research note already names

Items 2 through 7 of `docs/research/cache/crash-and-race-protocol.md`'s "Follow-up gates" — integrating the finalized artifact envelope and fuzzing every framing path, deterministic injected errors for disk-full and rename and directory-sync failure, measuring `process-crash` against `fsync` before an ADR fixes the default, defining the supported local filesystems, designing bounded GC separately, and running the harness under Cargo and rust-analyzer process patterns — are in scope for this ticket or must be split with their own owners. They are not closed by it silently.

## Outcome

**`tiler-cache` is admitted and the accepted protocol is implemented. `docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md` is the record; `crates/tiler-cache/` is the implementation; forty in-crate tests pass and the full repository gate is green.**

### What landed

The crate admission moved as one change, because a decision recorded is not a decision applied: ADR 0082, the `docs/architecture.md` ownership row and packaging profile, `scripts/check_workspace.py`'s pinned member set and closure, `ticketsplease.toml`'s `[scope_crates]` mapping, the root `Cargo.toml`, and `Cargo.lock`. The driver's row lost "cross-process content cache, atomic publication, byte embedding" and `tiler-metal-aot` keeps the empty closure ADR 0077 admitted it for. The profile's "generalized cache" omission is **amended** rather than reinterpreted — `tiler-runtime` was admitted by applying a stated test and no equivalent test admits this crate, so the clause is superseded on the point and still withholds every other cache.

`tiler-artifact` promotes a minimal digest surface — `DigestAlgorithm`, the opaque `Digest`, `DIGEST_BYTES` — while `digest_parts` and `envelope_digest` stay crate-private, so an outside caller can digest under one explicit domain and cannot express the ambiguous concatenation `digest_parts` puts on its caller.

The protocol is all of the in-scope list: the `<root>/v1/{entries,locks,tmp,quarantine}/<K[0..2]>/` namespace; a path parser accepting only fixed-width **lowercase** hexadecimal, refusing uppercase rather than folding it and refusing a wrong width rather than fitting it; an internal advisory-lock adapter over `File::lock` with the post-lock recheck; `create_new` unique temporaries, separate-descriptor validation of the completed bytes, one atomic rename, and `EXDEV` reported as a typed refusal; and internal eviction that takes the per-key lock and retains the lock file.

The bundle frames a magic, a schema, the governed algorithm tag, zeroed reserved bytes, the exact total length, the embedded key, and a descriptor table carrying every section's purpose, bounds, and digest. Validation on every hit checks all of it, requires the sections to be contiguous with no gap or trailing byte, and then hands the carried envelope to `decode_artifact`, which re-proves the manifest, every artifact section digest, and the canonical identity. **A bundle also carries the producer's compilation subject and the reader re-derives the key from it**, so a forgery that reseals every digest is still refused — that is the check a competent adversary does not get past by recomputing hashes.

Rejection is control flow, not silence: `MissReason`, `EntryRejection`, `BundleRejection`, `PublicationRefusal`, `QuarantineOutcome`, and `CacheUnavailable` reach the caller inside a `CacheReport`, and no path produces a miss without its reason. Replacing a rejected entry moves it to a bounded quarantine rather than letting the rename overwrite it, and reaching the bound reports `BoundReached` with the byte count it discarded instead of dropping evidence quietly.

### Two things `AGENTS.md`'s specification is not satisfied by, stated rather than buried

**Complete cache identity is not established, and this is the significant finding.** A bundle carries a whole artifact envelope, so a conforming subject must determine the envelope — which is what `docs/backends/metal.md` already says: "full artifact identity is the key". `crates/tiler-metal-aot/src/identity.rs` determines the `metallib` and says nothing about the plan variants, ABI bindings, or routing wrapped around it, so it is *half* of a conforming subject. No component emits the composed subject as one canonical byte run. The cache did not invent one — it cannot compose a subject without becoming an authority over encodings it does not own — and `key.rs` states the obligation and the gap in terms. `compose-the-complete-expansion-cache-subject` owns it.

**Crash and race behaviour is implemented and untested against this bundle.** The in-crate suite is threaded, and a thread that returns is not a process that was killed. `spikes/cache/cache_harness.rs` kills real processes at nine phases against its own miniature frame. Nothing here is offered as a substitute and no test is named as if it were. `port-the-cache-harness-to-the-production-bundle` owns it.

A third boundary: no in-crate test builds a real artifact envelope, because that needs a `SemanticProgram` and therefore a frozen registry of live inferencers, which this crate deliberately does not depend on — the same limitation `tiler-runtime` has. The public path's delegation to `decode_artifact` is proven negatively through the public API, and the protocol is exercised through a crate-private seam that takes any payload validator. The public API pins the validator, so no caller can weaken what a hit means.

### The follow-up gates

Gate 1 (MSRV) was already closed by the nightly pin and the adapter. Gate 2's envelope-integration half landed here; its fuzzing half is `fuzz-the-expansion-cache-framing-paths` and its process half is `port-the-cache-harness-to-the-production-bundle`. Gates 3 through 7 are `inject-deterministic-expansion-cache-io-failures`, `measure-expansion-cache-durability-policies`, `define-supported-expansion-cache-filesystems`, `design-bounded-expansion-cache-garbage-collection`, and `exercise-the-expansion-cache-under-cargo-and-rust-analyzer`. None is closed silently.

`accept-the-tiler-cache-public-boundary` carries the interface review ADR 0082 does not perform: the crate admission is Tom's decision, the exact public surface is a separate approval, and `expansion` documents itself as a reviewed draft boundary until he makes it.

### One limit deliberately absent

`Limits` carries no maximum entry count. Bounding the entry count means choosing which entry to evict, which is the garbage-collection policy the research note requires to be designed separately. A field recording a bound nothing enforced would read as a guarantee, which is worse than its absence.
