---
id: decide-where-an-unfillable-subject-facet-is-refused
title: Decide where an unfillable subject facet is refused
status: done
priority: p2
dependencies: []
related: [accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, public-boundary, decision]
---
## Outcome

Refuse an unfillable required facet at composition time. A `ComposedSubject` is the checked, identity-complete input to lookup and publication, so admitting a partial value would force every downstream consumer to repeat validation and would permit incomplete cache identity to travel farther than the typed construction boundary.

Incremental assembly remains possible in the composer state; it does not justify publishing an incomplete `ComposedSubject`. The existing subject construction and `tiler-build` consumers already enforce this invariant, so there is no remaining product trade-off.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Evidence

`subject.rs` refuses missing and empty required facets before producing a `ComposedSubject`; `store.rs` consumes the checked subject. The exact public surface remains reviewed by `accept-the-tiler-cache-public-boundary`.
