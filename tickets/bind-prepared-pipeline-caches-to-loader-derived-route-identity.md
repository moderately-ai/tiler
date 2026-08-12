---
id: bind-prepared-pipeline-caches-to-loader-derived-route-identity
title: Decide whether to retain the non-reusable Candle pipeline cache
status: awaiting-decision
priority: p1
dependencies: []
related: [prototype-candle-metal-adapter, bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes, decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality]
scopes: [implementation/candle, contracts/artifacts, contracts/integrations, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, cache, identity, correctness, decision, needs-tom]
---
## User-visible outcome

Tiler does not retain a cache-shaped prototype abstraction that cannot produce a hit and whose keys name the wrong identity subject. A reusable runtime cache is introduced only with a real owner and is then keyed by the exact validated object and pipeline inputs it consumes.

## Source-first Fact audit — 2026-08-12, `c539c558`

- **Verified:** `TilerPlan::load` derives the canonical artifact identity from decoded bytes and the normal wrapper passes those bytes to `CandleMetalAdapter::new`.
- **False:** the constructor is a public downstream path through which another caller can vouch for arbitrary identity bytes. `prototypes/candle-metal-adapter` is a binary-only package, `main.rs` declares private modules, and every constructor call is inside that package. `pub` on the item changes only same-package reachability.
- **False as a reachable cache-hit defect:** every `TilerFusedOp::route` constructs a fresh adapter, and every adapter constructs a fresh `PipelineCache`. Within that one route, each execution-order entry position is validated and prepared once. `LibraryKey` includes that unique position and `PipelineKey` adds its symbol, so neither map can hit an entry inserted by another route or another position. The maps are populated and counted, not reused.
- **False as a complete key:** canonical artifact identity deliberately excludes emitted backend object bytes. Two non-reproducible links may have equal artifact identity and different code sections. The adopted runtime contract therefore keys a library by the governed digest of the selected `BackendPayloadCode` section and keys a pipeline by that digest plus the resolved symbol and every applicable specialization, descriptor, translation, archive, and runtime-mode input. Authenticating the current artifact identity would still permit the wrong object to alias once a cache became reusable.
- **Verified:** the decoder computes and validates every section digest, then drops that derived vector after the canonicity comparison. `RoutedEntry` publishes exact object bytes, the compilation-subject payload descriptor, and the resolved symbol, but not the already-verified code-section digest. A real reusable cache therefore needs a typed decoder-to-route projection; it must not rehash under a second domain or substitute `BackendPayloadDescriptor::digest`.
- **Verified:** current supported integrations construct one adapter per invocation. `DispatchAdapter::dispatcher` documents and enforces that factory shape, the facade calls it once per region invocation, the Candle wrapper constructs one adapter inside each custom-op route, and the runtime conformance consumers construct a new adapter per call. The accepted scalar CPU runtime is a future real sibling consumer, but its initial decoded scalar-image path declares no library or prepared-pipeline cache.
- **Imprecise:** an entry ordinal is sufficient navigation inside one already-bound attempt and insufficient reusable identity outside it. A durable observation key is the exact prepared-pipeline subject plus the property query; a satisfaction verdict remains per attempt and is never cached.

## Decision

Choose whether to preserve and widen the prototype cache now or delete it until a real reusable owner exists.

### Recommended — delete the current cache rather than repair it speculatively

Remove `cache.rs`, `PipelineCache`, `DeviceScope`, `LibraryKey`, `PipelineKey`, the caller-supplied `artifact_identity` field and constructor argument, cache occupancy reporting, `ForeignDeviceScope`, and their cache-only tests/proof prose. Validate and retain each library and prepared pipeline directly in the adapter's per-invocation state. This is physical deletion, not deprecation, an empty compatibility wrapper, or a cache-shaped side table under another name.

This does not remove a working optimization: the current maps have no reachable hit. It removes hashing, key allocation, map insertion, cloning, diagnostics, and a false claim of runtime reuse while preserving the exact libraries and pipelines the route already retains through dispatch. The prepared-property comparison still reads the pipeline prepared for that execution-order entry in the same attempt, and `make-prepared-entry-observations-typed-and-key-dispatched` separately makes unknown property ownership fail closed.

Remove this ticket from `carry-subgroup-width-through-exact-prepared-entry-equality`'s dependencies. A route-local pipeline does not need a reusable-cache proof before its own property can be compared; retaining the edge would make an absent optimization a correctness prerequisite.

Create one deferred cache ticket whose trigger is the first accepted cache object that outlives a route attempt. Its required first delivery is a decoder-minted opaque `BackendPayloadCodeDigest` (or equivalently typed received identity) carried through `DecodedArtifact` and `RoutedEntry` from the section digest already computed during decode. The cache owner then keys:

- a library by exact live device/context scope plus exact code-section digest;
- a pipeline by that library subject plus resolved symbol and every present specialization, canonical descriptor, translation/archive, and runtime-mode input; and
- a prepared observation by that exact pipeline subject plus the exact property query.

Artifact identity and entry ordinal are deliberately absent from the reusable library/pipeline keys: they are not inputs those objects consume and would prevent safe reuse of identical code across envelopes or entries. The route keeps the ordinal only as local navigation. Cache values may retain prepared objects and observations, never satisfaction verdicts, fallback authority, or routing commitment. The real owner must also decide concurrency, eviction, in-flight retention, device/context loss, and negative-result stability rather than inheriting unstated prototype behaviour.

No new enum is justified in this deletion slice. The future exact code identity is one non-optional state and should be an opaque newtype; hit, miss, transient failure, and stable negative retention belong to the future cache owner's result vocabulary only when that owner exists.

## Nondominated alternatives

1. **Delete now and defer the reusable cache to its first real owner — recommended.** Correctness is highest because no wrong-key reuse is representable; maintenance and current host cost are best because dead maps and identities disappear; future performance is unchanged because no current hit exists.
2. **Implement the complete reusable cache now.** It can equal option 1 on correctness and eventually improve warm runtime, but only by adding the decoder's typed code digest, a persistent owner, concurrency and eviction policy, device-loss invalidation, exact prepared-entry association, and real hit/miss evidence. No accepted production Metal runtime or other current consumer owns that lifetime, so the extra public and lifecycle surface is speculative today. Choose this only if a named persistent consumer is accepted at the same time.
3. **Replace artifact identity with the code-section digest but keep the per-attempt maps.** Correct keys, but still zero hits and more state than direct retention. It is dominated by deletion on maintenance and current runtime and by option 2 on future usefulness.

Rejected: authenticating the existing artifact-identity key; adding route identity beside it; hashing raw object bytes again under a consumer-owned digest; keying by payload compilation-subject digest; retaining entry ordinal as durable identity; caching a property-satisfaction verdict; or preserving an empty cache compatibility surface. Each either keeps the exact-object collision, creates a second identity authority, overkeys reusable objects, or preserves machinery with no consumer.

## Strongest counterpoint and reversal evidence

Deleting the cache means a future persistent Metal runtime must reintroduce a cache rather than gradually widening this one. That is intentional because the present abstraction has the wrong identity, ownership lifetime, and untested concurrency model; retaining it saves names, not sound implementation. Reverse to immediate implementation only when a named accepted consumer owns one cache across at least two route attempts and a test demonstrates a real second-attempt hit for identical code plus misses for independently perturbed code digest, symbol, pipeline configuration, device, and context.

## Identity and compatibility

Deleting transient cache state changes no semantic, schedule, kernel, artifact, manifest, envelope, or expansion-cache bytes. The future `BackendPayloadCodeDigest` projection would expose an identity the envelope already carries and validates; it would not change the wire grammar or any existing identity value. A persistent runtime cache key is runtime-local and never artifact identity.

## Closes when

Tom chooses deletion or immediate complete implementation. Under the recommendation, the non-reusable cache and every claim/report/refusal that exists only for it are physically removed, the subgroup implementation no longer depends on an absent optimization, the prototype retains exact per-attempt libraries and pipelines without caller-stated artifact identity, and a deferred real-cache ticket carries the explicit trigger and complete consumed-subject key contract.
