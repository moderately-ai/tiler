---
id: implement-the-expansion-cache-protocol
title: Implement the expansion cache protocol
status: todo
priority: p1
dependencies: [decide-the-expansion-cache-owner-and-digest-authority]
related: [prototype-expansion-content-cache]
scopes: [implementation/cache]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, cache, concurrency, durability]
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
