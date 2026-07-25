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
- Whether `lookup` and `get_or_publish` should take a subject or a `CacheKey`. They take a subject so a caller never handles a key it did not derive from one, at the cost of re-digesting the subject on each call.
- The rejection vocabulary — `MissReason`, `EntryRejection`, `BundleRejection`, `PublicationRefusal`, `QuarantineOutcome`, `CacheUnavailable`, `CacheOperation` — and whether the `CacheReport` accessor shape is the right way to surface an explanation.
- `Limits` as a plain struct with public fields and a `Default`, which makes adding a field a breaking change rather than a silent one.
- The digest promotion in `tiler-artifact`: `DigestAlgorithm`, `Digest`, and `DIGEST_BYTES` are now `pub` under `tiler_artifact::program`, while `digest_parts` and `envelope_digest` stay crate-private.
