---
id: report-cache-publication-state-after-the-rename-boundary
title: Report the true cache publication state after atomic rename
status: todo
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol, accept-the-tiler-cache-public-boundary, inject-deterministic-expansion-cache-io-failures]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, correctness, diagnostics]
---
A caller must be able to distinguish “no entry was published” from “a valid
entry was published but a later durability or cleanup operation failed.”

## Fact

The cache's atomic rename is the publication point. Once it succeeds, another
process may observe the valid immutable entry. A later parent-directory sync or
lock-release failure cannot undo that fact. Current paths can nevertheless
return `Uncached` with a `PublicationRefusal`, which describes the entry as
unpublished.

## Outcome

Model publication, durability, and cleanup as separate facts. Pre-rename
failures report that no content entry was published. Post-rename failures
report a published valid entry together with weakened durability or cleanup
status. No outcome or explanation may claim that a successful rename did not
occur.

## Closes when

The outcome vocabulary matches the filesystem state at every failure boundary,
callers cannot mistake a published entry for a rebuildable miss, and positive
and fault-injected tests cover both sides of the publication point.
