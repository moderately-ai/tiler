---
id: decide-where-an-unfillable-subject-facet-is-refused
title: Decide where an unfillable subject facet is refused
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

**refuse at composition time, or admit and reject at `lookup`?**

*Enables (refuse at composition):* a `ComposedSubject` that exists is complete, so no later stage has to re-ask. *Prevents (refuse at composition):* a caller assembling facets incrementally cannot hold a partial subject, and `SubjectRefusal` fires further from the site that knows why the facet is missing.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

Tom records a decision; the change (or the explicit keep-as-drafted) lands on the surface and the parent's checklist item is marked.
