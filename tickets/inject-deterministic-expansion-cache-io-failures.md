---
id: inject-deterministic-expansion-cache-io-failures
title: Inject deterministic expansion cache I/O failures
status: done
priority: p2
dependencies: [report-cache-publication-state-after-the-rename-boundary]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, testing, durability]
---
The research note's third follow-up gate: deterministic injected errors at the
cache's filesystem boundaries.

`tiler-cache` has typed I/O classifications, but current post-rename reporting
does not preserve the fact that publication already occurred. Compiler build
failure is not cache I/O, and no retry-exhaustion path exists here; neither is
part of this ticket.

## What this ticket owes

- Reach each failure deterministically. A read-only directory and a full filesystem image are both reachable without a fault-injection layer on the supported hosts; decide whether that is enough or whether a seam is warranted, and prefer the version that exercises the real `std` call.
- Assert the classification, not just the failure: the point of the typed report is that a caller can tell a full disk from a corrupt entry, and a test that only asserts "it fell open" would pass against a version that reported the wrong reason.
- Assert the resulting filesystem state. Pre-rename failures leave no content
  entry and no temporary. A post-rename sync or lock-release failure reports
  the valid published entry plus weakened durability or cleanup status.
- `CrossesFilesystems` needs a second filesystem to reach at all. Say plainly if it cannot be exercised on the CI hosts, rather than leaving a passing suite that implies it was.

## Closes when

Every cache I/O boundary is induced deterministically and produces both its
typed diagnostic and the correct filesystem state. A following call either
observes the published entry, recovers normally, or refuses for the documented
reason.

## Outcome (2026-07-27)

**Decision on the seam question the ticket posed: real filesystem conditions, not an injection layer, wherever the boundary is reachable that way.** A read-only shard makes `File::create_new` and `fs::rename` fail for the same reason a full or read-only filesystem would, so what runs is the real `std` call on the real path rather than an injected return value. The ticket stated that preference and the two pre-rename boundaries meet it, so no new seam was added for them.

The existing `fault::Injection` seam stays for the two **post-rename** faults it already covers — `EntryDirectorySync` and `LockRelease` — because those cannot be reached by permissions: the rename has already succeeded and the directory is the one the writer just wrote into.

### The two pre-rename boundaries

| boundary | induced by | asserted |
| --- | --- | --- |
| `CacheOperation::CreateTemporary` | read-only `tmp/<shard>/` | `PublicationRefusal::Unavailable` with that operation; no content entry; no temporary left; the next call publishes normally |
| `CacheOperation::Publish` | read-only `entries/<shard>/` | same refusal shape with that operation; no content entry; the complete validated temporary is cleaned up |

The two are the opposite sides of the publication instant — one fails before any bytes are written, the other after a complete validated temporary exists — and both must leave the same observable state. That is what makes the rename the single publication point rather than a step among several.

**Classification is asserted, not merely the refusal**, as the ticket required. A version reporting either of these as an oversize bundle or a rejected temporary would still "fail closed" and would send a caller to rebuild differently instead of to fix the filesystem.

### The mistake worth recording

The first version derived the cache key with `CacheKey::derive(&composed_subject)` while `resolve` derives it with `CacheKey::derive_bytes(subject)`. The read-only shard was therefore a *different* shard, publication succeeded normally, and both tests reported a hit. They failed loudly because they assert `ProtocolOutcome::Uncached` — a weaker test that only checked "the call returned `Ok`" would have passed while exercising nothing at all. The key derivation now matches `resolve`'s, with a comment saying why taking the wrong one is not a detail.

### `CrossesFilesystems` is not exercised, and this is not a gap the suite hides

It needs a second filesystem to reach at all: the layout keeps every temporary under the same cache root, so ordinary operation cannot produce it, and only a bind mount or a symbolic link inside the namespace can. Constructing one requires either mount privileges or a disk image, neither of which the development host offers a test. **Stated here rather than left as a passing suite that implies coverage**, exactly as the ticket asked. Its handling is a two-line refusal path that is read rather than run.
