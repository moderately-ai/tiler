---
id: decide-the-lookup-argument-type
title: Decide the cache lookup argument type
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

The public API must preserve two properties together: raw bytes cannot name an entry, and a caller that performs multiple operations for one subject need not pay repeated digest work.

Benchmark and draft a checked prepared-subject/key token derived only from `ComposedSubject` beside the current `&ComposedSubject` API. Do not expose a raw key constructor or make callers coordinate separate subject and key values. If no real call path performs repeated derivation, keep the current signature rather than adding a token without a consumer. `get_or_publish` must continue to derive once for its complete lookup/build/publish operation.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

The real `tiler-build` call path and a repeated-operation fixture establish whether digest reuse exists; the selected draft makes subject/key mismatch unrepresentable, raw bytes cannot reach lookup, no extra allocation appears on the hot path, and the exact signature is carried into `accept-the-tiler-cache-public-boundary` for Tom's review.
