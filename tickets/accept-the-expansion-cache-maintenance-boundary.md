---
id: accept-the-expansion-cache-maintenance-boundary
title: Accept the expansion-cache maintenance boundary
status: todo
priority: p2
dependencies: [accept-the-tiler-cache-public-boundary]
related: [design-bounded-expansion-cache-garbage-collection, decide-the-expansion-cache-collection-schedule]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, api, decision, needs-tom]
---
Decide how a caller inspects, bounds, collects, and deliberately purges a whole
expansion-cache namespace.

This is separate from key-oriented lookup and publication because maintenance
has different callers, cost, reporting, and lifecycle. Review whether it
belongs on `ExpansionCache` or a separate maintenance handle, which entry facts
and removal outcomes callers need, and which operations remain deliberately
explicit rather than running on an expansion path.

## User-visible outcome

A maintenance caller can explain what occupies the cache, enforce an explicit
bound, and report what was removed without making collection an implicit
correctness dependency for ordinary compilation.

## Closes when

Tom accepts the public maintenance types and call-site boundary, reports do not
overstate durability or removal, and the collection-scheduling ticket can name
the accepted caller surface.
