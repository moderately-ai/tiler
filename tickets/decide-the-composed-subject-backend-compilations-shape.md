---
id: decide-the-composed-subject-backend-compilations-shape
title: Decide the composed subject's backend-compilations shape
status: todo
priority: p2
dependencies: []
related: [accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, public-boundary, decision]
---
## Draft the exact signature

The current ordered borrowed `&[&[u8]]` is semantically complete: payload order is artifact identity, one or more backend compilations are required, and the cache must not parse backend encodings. The remaining question is not product policy but whether the public signature should expose that raw nested slice or a checked borrowed wrapper that makes non-empty ordered cardinality structural.

Produce both exact call-site drafts against the real single-payload `tiler-build` consumer and a multi-payload fixture. Eliminate any wrapper that allocates, copies bytes, reorders payloads, or duplicates canonical validation. Prefer the smallest borrowed checked view if it removes repeated runtime cardinality checks without adding construction overhead; otherwise retain the raw ordered slice and document the invariant at `SubjectFacets`.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

The selected implementation represents one and many payloads without allocation or order loss, empty collections and empty members fail closed, the multi-payload identity fixture proves order is significant, and the exact signature is carried into `accept-the-tiler-cache-public-boundary` for Tom's review.
