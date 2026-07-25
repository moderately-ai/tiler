---
id: accept-the-tiler-cache-public-boundary
title: Accept the tiler-cache public boundary
status: todo
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact]
shared_scopes: []
paths: []
tags: [cache, api, decision]
---
ADR 0082 admits the crate. It does not accept the exact interface, and `AGENTS.md` keeps those separate: "A tested implementation may serve as a concrete draft, but it is not implicit approval of its public interface."

`tiler_cache::expansion` therefore says in its own documentation that it is a **reviewed draft boundary** (ADR 0074 §7, ADR 0075), on the same footing as `tiler_artifact::program` and `tiler_runtime::load`, so a reader cannot mistake it for a settled interface.

## What Tom is being asked to review

- `ExpansionCache` and its five methods: `open`, `lookup`, `get_or_publish`, `evict`, `sweep_temporaries`, plus the `with_durability` and `with_limits` configuration.
- The `get_or_publish` shape: a build closure returning `Result<Vec<u8>, E>`, a `Resolution` whose three variants all carry a validated artifact, and a `PublishFailure<E>` that is generic over the caller's error rather than erasing it.
- Whether `lookup` and `get_or_publish` should take a subject or a `CacheKey`. They take a `&ComposedSubject` so a caller never handles a key it did not derive from one, at the cost of re-digesting the subject on each call.
- The composed-subject surface `compose-the-complete-expansion-cache-subject` added: `ComposedSubject` with `compose` and `as_bytes`, the `SubjectFacets<'_>` leaf record whose two fields are both required, the `SubjectFacet` vocabulary, and the `SubjectRefusal` that fires when a caller cannot fill a facet. Two shape questions are open in it — whether `backend_compilations: &[&[u8]]` is the right thing to ask a caller for, and whether refusing an unfillable facet at composition time is preferable to admitting it and rejecting at `lookup`. `CacheKey::derive` narrowed to a `&ComposedSubject` at the same time, so a raw byte run can no longer name an entry.
- The rejection vocabulary — `MissReason`, `EntryRejection`, `BundleRejection`, `PublicationRefusal`, `QuarantineOutcome`, `CacheUnavailable`, `CacheOperation` — and whether the `CacheReport` accessor shape is the right way to surface an explanation.
- `Limits` as a plain struct with public fields and a `Default`, which makes adding a field a breaking change rather than a silent one.
- The digest promotion in `tiler-artifact`: `DigestAlgorithm`, `Digest`, and `DIGEST_BYTES` are now `pub` under `tiler_artifact::program`, while `digest_parts` and `envelope_digest` stay crate-private.

## The collection surface, staged crate-private

`design-bounded-expansion-cache-garbage-collection` added whole-cache accounting, a bounded collection, and an out-of-service purge in `expansion::collect`. **Every type is `pub(crate)` under ADR 0074 convention 7 and none is re-exported**, so nothing here is public yet and no consumer can collect anything. Promoting it is part of this review, and these are the shape questions it raises:

- `ExpansionCache::account`, `collect`, and `purge` — three more methods on the type whose five methods are already listed above, taking the count to eight. Whether accounting and collection belong on `ExpansionCache` at all, or on a separate maintenance handle, is a real question: they are the only operations that walk the whole namespace rather than one key, and the only ones a build-time caller would never invoke.
- `CollectionBound` as a leaf record of two `Option<u64>` ceilings whose `Default` removes nothing. The absence of a default bound is a decision, not an omission — the research note says exact defaults require workload measurement, and a guessed default would delete entries invisibly.
- `CollectionReport`, which lists every removed entry individually rather than counting them, plus `CacheAccounting`, `EntryFact`, `RemovedEntry`, `CollectionOutcome`, `CollectionOrder`, `Disposition`, and `PurgeReport`. That is eight more names on the boundary, and whether the report should be that granular is worth deciding rather than inheriting.
- `CollectionOrder` has one variant, `OldestPublicationFirst`. It exists so the report can name the rule that selected an entry, and as the seam for a use-recency order that `define-supported-expansion-cache-filesystems` gates. A single-variant public enum is a deliberate reservation and should be accepted as one or removed.
- `CacheOperation` gained two variants, `RetireNamespace` and `RemoveRetired`. That enum is deliberately **not** `#[non_exhaustive]`, so a variant is a breaking change to a public enum — acceptable while the boundary is a reviewed draft, and worth confirming.
- `Limits` still carries no whole-cache ceiling, and its documentation now says why: every field there bounds one operation the type is passed to, whereas a cache-wide ceiling bounds the collection that enforces it.

`decide-the-expansion-cache-collection-schedule` depends on this ticket, because nothing outside the crate can call a collection until the facade is accepted.
