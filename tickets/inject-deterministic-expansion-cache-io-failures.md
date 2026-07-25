---
id: inject-deterministic-expansion-cache-io-failures
title: Inject deterministic expansion cache I/O failures
status: todo
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, testing, durability]
---
The research note's third follow-up gate: deterministic injected errors for disk full, rename failure, directory sync failure, compiler failure, and retry exhaustion.

`tiler-cache` has a typed refusal for each of these — `PublicationRefusal::Unavailable` carries the exact `CacheOperation` and the originating `io::Error`, and `PublicationRefusal::CrossesFilesystems` names the `EXDEV` case specifically. Compiler failure and retry exhaustion are covered by tests. **The disk-full, rename-failure, and directory-sync-failure paths are written and unexercised**, which means what is verified is that they compile.

## What this ticket owes

- Reach each failure deterministically. A read-only directory and a full filesystem image are both reachable without a fault-injection layer on the supported hosts; decide whether that is enough or whether a seam is warranted, and prefer the version that exercises the real `std` call.
- Assert the classification, not just the failure: the point of the typed report is that a caller can tell a full disk from a corrupt entry, and a test that only asserts "it fell open" would pass against a version that reported the wrong reason.
- Assert that every one leaves no entry at the content path and no temporary behind.
- `CrossesFilesystems` needs a second filesystem to reach at all. Say plainly if it cannot be exercised on the CI hosts, rather than leaving a passing suite that implies it was.
