---
id: inject-deterministic-expansion-cache-io-failures
title: Inject deterministic expansion cache I/O failures
status: todo
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
