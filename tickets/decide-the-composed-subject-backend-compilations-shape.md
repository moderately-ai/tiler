---
id: decide-the-composed-subject-backend-compilations-shape
title: Decide the composed subject's backend-compilations shape
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

**is `backend_compilations: &[&[u8]]` the right thing to ask a caller for?**

*Enables (keep it):* the cache never parses a producer encoding, and a caller that already holds compiled bytes passes them without a conversion. *Prevents (keep it):* a caller holding one compilation still constructs a slice of slices, and the type says nothing about ordering or cardinality that the composition then enforces at runtime rather than in the signature.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

Tom records a decision; the change (or the explicit keep-as-drafted) lands on the surface and the parent's checklist item is marked.
