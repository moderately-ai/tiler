---
id: accept-the-expansion-cache-maintenance-boundary
title: Accept the expansion-cache maintenance boundary
status: done
priority: p2
dependencies: [accept-the-tiler-cache-public-boundary]
related: [design-bounded-expansion-cache-garbage-collection, decide-the-expansion-cache-collection-schedule]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
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

## Accepted (2026-07-31)

Tom decided both halves. Siting: maintenance stays on `ExpansionCache` — `account()`, `collect(&CollectionBound)`, and `purge()` join the accepted `evict` and `sweep_temporaries`, because a separate handle would duplicate the root state, split one root's operations across two types, and supersede an accepted surface while the never-on-the-expansion-path invariant is behavioural (explicit calls only) rather than structural. Vocabulary: accepted in full as implemented — `CollectionBound` (public optional ceilings, `UNBOUNDED` the only supplied bound, no default ceiling ever), single-variant `CollectionOrder::OldestPublicationFirst` with its recorded eliminations and honest insertion-recency cost, `EntryFact`, `CacheAccounting` (unrecognized reported never removed, quarantine counted never collected), `RemovedEntry`, `CollectionOutcome` with `BoundNotReached` carrying the remainder, `CollectionReport` with individual removals and the `accounts_for_every_entry` partition check, and `PurgeReport`.

One narrowing against the packet as asked: the public `collect` takes only a bound — the order is reported on the result, not chosen by the caller, because exactly one order exists and a parameter would imply a choice the vocabulary cannot yet offer. The private removal step (`remove_if_unchanged`) and its `Disposition` remain crate-private as implementation. The promotion landed with the acceptance: the ADR 0074 §7 staging allow is removed, the vocabulary is re-exported from `tiler_cache::expansion`, crate and research docs state the accepted status, and `decide-the-expansion-cache-collection-schedule` can now name the accepted caller surface.

## Closes when

Tom accepts the public maintenance types and call-site boundary, reports do not
overstate durability or removal, and the collection-scheduling ticket can name
the accepted caller surface.

## Current-surface correction — 2026-08-09

The accepted 2026-07-31 surface above is historical authority, not the complete
current maintenance vocabulary. [`admit-an-age-bounded-automatic-eviction-into-the-expansion-cache`](admit-an-age-bounded-automatic-eviction-into-the-expansion-cache.md)
later added and accepted `CollectionBound::max_entry_age`, the
`MaxEntryAge` bound, and removal-reason reporting through
`RemovedEntry::reason` and `RemovalReason`. That extension is additive: it does
not reopen the decision that maintenance remains on `ExpansionCache`, that
collection is explicit, or that one unbounded/default-free bound vocabulary
governs the call.
