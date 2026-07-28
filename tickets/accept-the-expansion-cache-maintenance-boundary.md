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

This is separate from key-oriented lookup and publication because maintenance has different callers, cost, reporting, and lifecycle. Half the siting question is already answered and the ticket should not re-ask it: `evict` (`crates/tiler-cache/src/expansion/store.rs:345`) and `sweep_temporaries` (`:376`) are already methods on `ExpansionCache` (`:228`), and both are already in front of Tom on `accept-the-tiler-cache-public-boundary` as two of the five methods under review there — find that item with `grep -n "five methods" tickets/accept-the-tiler-cache-public-boundary.md`, since that ticket is being restructured and its line numbers move. So the open part is narrower: whether namespace-scoped accounting, explicit bounding, and deliberate purge join them on `ExpansionCache` or move to a separate maintenance handle — and, if a handle, whether the two existing methods move onto it too rather than leaving maintenance split across two types. Also decide which entry facts and removal outcomes callers need, and which operations remain deliberately explicit rather than running on an expansion path.

## User-visible outcome

A maintenance caller can explain what occupies the cache, enforce an explicit
bound, and report what was removed without making collection an implicit
correctness dependency for ordinary compilation.

## Closes when

Tom accepts the public maintenance types and call-site boundary, reports do not
overstate durability or removal, and the collection-scheduling ticket can name
the accepted caller surface.
