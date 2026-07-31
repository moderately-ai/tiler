---
id: decide-the-composed-subject-backend-compilations-shape
title: Decide the composed subject's backend-compilations shape
status: done
priority: p2
dependencies: []
related: [accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, public-boundary, decision]
---
## User-visible outcome

Cache subject composition retains one allocation-free borrowed input shape for ordered backend-compilation identity bytes, with canonical composition remaining the sole validation boundary.

## Implementation keys

The current ordered borrowed `&[&[u8]]` is semantically complete: payload order is artifact identity, one or more backend compilations are required, and the cache must not parse backend encodings. The remaining question is not product policy but whether the public signature should expose that raw nested slice or a checked borrowed wrapper that makes non-empty ordered cardinality structural.

Produce both exact call-site drafts against the real single-payload `tiler-build` consumer and a multi-payload fixture. Eliminate any wrapper that allocates, copies bytes, reorders payloads, or duplicates canonical validation. Prefer the smallest borrowed checked view if it removes repeated runtime cardinality checks without adding construction overhead; otherwise retain the raw ordered slice and document the invariant at `SubjectFacets`.

Split from `accept-the-tiler-cache-public-boundary` so one signature can be reshaped without rejecting the whole surface; the parent's ratification checklist, derivations, and history stay there and are not re-litigated here.

## Closes when

The selected implementation represents one and many payloads without allocation or order loss, empty collections and empty members fail closed, the multi-payload identity fixture proves order is significant, and the exact signature is carried into `accept-the-tiler-cache-public-boundary` for Tom's review.

## Outcome (2026-07-30)

Retain the existing ordered `&[&[u8]]` input at `ComposedSubject::compose`; do not add a wrapper. Source inspection found one canonical validation boundary, not repeated cardinality checks: composition rejects an empty outer collection and empty members exactly once, preserves payload order, and then necessarily allocates the composed subject bytes. `tiler-build` has one real prepared-compilation input today, while existing multi-payload fixtures already prove count, order significance, and empty-member refusal. A checked wrapper would remove no check or allocation and would add a second public nominal constructor with no consumer benefit.

The retained invariant is that backend compilations are an ordered non-empty sequence of non-empty opaque identity byte strings in artifact payload order. They are never sorted, deduplicated, parsed as backend data, or treated as a set. The consolidated `accept-the-tiler-cache-public-boundary` review should retain this signature and its existing typed refusals; no new public type is proposed.

## Graph maintenance

- Keep this completed decision as a dependency of `accept-the-tiler-cache-public-boundary` so the retained signature is visible in that atomic review.
- Reopen only when a named caller demonstrates repeated validation or a representation requirement the raw ordered borrowed slice cannot satisfy without ambiguity or extra work.
