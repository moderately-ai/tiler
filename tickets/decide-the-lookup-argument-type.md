---
id: decide-the-lookup-argument-type
title: Decide the cache lookup argument type
status: awaiting-decision
priority: p2
dependencies: []
related: [accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, public-boundary, decision]
---
## Decision needed (2026-07-28)

**`&ComposedSubject` or `CacheKey`?**

They take a `&ComposedSubject` today so a caller never handles a key it did not derive from one. *Enables (subject):* a raw byte run cannot name an entry, which is the property `CacheKey::derive`'s narrowing established. *Prevents (subject):* the subject is re-digested on each call, so a caller doing a lookup and then a publish under the same key pays the digest twice and has no way to say it already has the key.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

Tom records a decision; the change (or the explicit keep-as-drafted) lands on the surface and the parent's checklist item is marked.
