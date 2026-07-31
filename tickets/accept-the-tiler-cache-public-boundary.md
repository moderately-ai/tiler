---
id: accept-the-tiler-cache-public-boundary
title: Accept the tiler-cache public boundary
status: done
priority: p1
dependencies: [report-cache-publication-state-after-the-rename-boundary, decide-the-composed-subject-backend-compilations-shape, decide-where-an-unfillable-subject-facet-is-refused, decide-the-lookup-argument-type]
related: [implement-the-expansion-cache-protocol, prototype-candle-metal-adapter]
scopes: [implementation/cache, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [cache, api, decision]
---
## User-visible outcome

Tom receives one exact, consumer-backed `tiler_cache::expansion` public-surface diff after its remaining signature questions are resolved, and accepted documentation replaces reviewed-draft language only for the boundary actually exercised by `tiler-build`.

ADR 0082 admits the crate but not the interface, so `tiler_cache::expansion` documents itself as a **reviewed draft boundary** under ADR 0074 §7 and ADR 0075, on the same footing as `tiler_artifact::program` and `tiler_runtime::load` — `AGENTS.md` keeps admission and interface acceptance separate: "A tested implementation may serve as a concrete draft, but it is not implicit approval of its public interface."

The former recommendation to accept before resolving three split signatures was backwards: acceptance would freeze the interface whose shape the children exist to determine. Its “no consumer” premise is also stale. `tiler-build` now depends on `tiler-cache`; `metal_cache.rs` composes the ordered backend-compilation and artifact-program subject and calls `get_or_publish`, and `metal_plan.rs` carries the resulting subject and resolution through the checked plan path.

The subject cardinality and lookup argument decisions now retain the existing consumer-backed signatures: ordered `&[&[u8]]` backend compilation inputs at composition and lookup by `&ComposedSubject`, with one internal key derivation per operation. Composition-time refusal remains the tested invariant. Present those exact signatures in the atomic public-boundary review.

## Implementation keys

Review the exact existing surface as one consumer-backed unit. Retain ordered borrowed `&[&[u8]]` backend-compilation inputs because canonical composition performs the one necessary validation and allocation; retain lookup by `&ComposedSubject` because the real caller performs one operation and `CacheKey` already represents the checked derived token. Ratify the typed publication, refusal, reporting, digest, limits, and preflight surfaces listed below without adding a raw-key path, redundant checked wrapper, prepared token, or ambient filesystem probe.

## Items to ratify

Each item can be rejected on its own; accepting the ticket without naming an exception accepts all of them.

- [ ] `ExpansionCache` and its five methods: `open`, `lookup`, `get_or_publish`, `evict`, `sweep_temporaries`, plus the `with_durability` and `with_limits` configuration.
- [ ] The `get_or_publish` shape: a build closure returning `Result<Vec<u8>, E>`, a `Resolution` whose three variants all carry a validated artifact, and a `PublishFailure<E>` that is generic over the caller's error rather than erasing it.
- [ ] The composed-subject surface `compose-the-complete-expansion-cache-subject` added: `ComposedSubject` with `compose` and `as_bytes`, the `SubjectFacets<'_>` leaf record whose two fields are both required, the `SubjectFacet` vocabulary, and the `SubjectRefusal` that fires when a caller cannot fill a facet. `CacheKey::derive` narrowed to a `&ComposedSubject` at the same time, so a raw byte run can no longer name an entry.
- [ ] The rejection vocabulary — `MissReason`, `EntryRejection`, `BundleRejection`, `PublicationRefusal`, `QuarantineOutcome`, `CacheUnavailable`, `CacheOperation` — and whether the `CacheReport` accessor shape is the right way to surface an explanation.
- [ ] `Limits` as a plain struct with public fields and a `Default`, which makes adding a field a breaking change rather than a silent one.
- [ ] The digest promotion in `tiler-artifact`: `DigestAlgorithm`, `Digest`, and `DIGEST_BYTES` are now `pub` under `tiler_artifact::program`, while `digest_parts` and `envelope_digest` stay crate-private.
- [ ] `ExpansionCache::preflight(&self) -> PreflightReport` — scans, changes nothing that outlives the call, refuses nothing.
- [ ] `PreflightReport` with five verdict accessors — `same_device`, `create_new_excludes`, `lock_excludes_locally`, `rename_publishes`, `modification_time_reported` — plus `root`, `all_probed_properties_hold`, and the associated `cross_host_exclusion_caveat`.
- [ ] `PreflightVerdict` — `Holds`, `Refuted`, `NotRun`.

**Two shapes to review rather than skim.** `PreflightVerdict::NotRun` is deliberately not a refutation: a refuted property says the root is unsuitable, while an unrunnable probe says nothing was learned, and reporting the first for the second sends a caller to replace a filesystem when the answer is a permission. And `cross_host_exclusion_caveat` is an associated function returning *text*, not a `bool`, because it is a property of the probe rather than of any report — a caller rendering the report prints the caveat instead of having to know it.

Both were argued to a conclusion here and need ratification rather than deliberation, which is why they sit in this list and not in the split block below.

## The Metal AOT compilation facet is ratified (2026-07-28)

Tom selected the correctness-preserving prepared-compilation boundary for `promote-the-metal-aot-compilation-identity`. `tiler_metal_aot::identity` is public, as are `CompilationIdentity`, `ToolchainEvidence`, `IdentityReuseScope`, and `IdentityError`, but the derived identity has no public constructor. `Toolchain::prepare(&CompileRequest)` returns an opaque `PreparedCompilation` that exposes `identity()` for cache lookup and consumes itself through `compile()` using the same borrowed request and privately held resolved paths.

The staged `4f8ce90` constructor was rejected rather than merged: `CompilationIdentity::new(&CompileRequest, &ResolvedToolchain)` would have let a caller mint a derived identity from caller-constructed toolchain facts and would have allowed cache lookup and compilation to perform separate resolutions. The accepted token conforms to ADR 0074 convention 2, avoids a second resolution on the miss path, and makes request/toolchain agreement structural.

This makes both `SubjectFacets` fields producible. It does not create the orchestrator that holds both producers and the cache, so the cache remains unused end to end; the frontend orchestration decision still owns that integration.

## Split status (updated 2026-07-31)

All three shape questions remain explicit completed dependencies so their derivations stay visible to acceptance. `decide-where-an-unfillable-subject-facet-is-refused` keeps partial assembly in the composer. `decide-the-composed-subject-backend-compilations-shape` retains ordered borrowed `&[&[u8]]` input because a checked wrapper removes no validation or allocation. `decide-the-lookup-argument-type` retains `&ComposedSubject` because the real consumer performs one operation and `CacheKey` already is the checked derived token. No signature question remains; this ticket can proceed to its exact atomic public review.

## Excluded decision

Whole-cache accounting, collection, and purge are deliberately excluded. They operate over a namespace rather than one key and have different callers, cost, reporting, and lifecycle. `accept-the-expansion-cache-maintenance-boundary` owns that atomic public decision.

## The hold on the outcome vocabulary is discharged (2026-07-28)

This ticket previously said not to accept the outcome/report vocabulary until `report-cache-publication-state-after-the-rename-boundary` made a successful atomic rename distinguishable from a refusal to publish. That ticket is `done` (verified: `grep -n '^status:' tickets/report-cache-publication-state-after-the-rename-boundary.md` → `status: done`), and what it made distinguishable is exactly that pair — a publication that completed its atomic rename now reports a different outcome from one that refused to publish, so a caller reading the report can no longer read a refusal as a success. The vocabulary is therefore ratifiable along with the rest of the surface.

## Provenance of the preflight surface

`ExpansionCache::preflight` landed `pub(crate)` under ADR 0074 convention 7 and was staged here rather than promoted. It carries a module-level `#[allow(dead_code)]` whose reason states why no caller was wired to satisfy the lint: the only place a caller would naturally go is the expansion path, and putting a filesystem probe there is the one thing the design forbids.

**Fact — the author is `add-an-expansion-cache-root-preflight`, not `prototype-expansion-content-cache`.** Both authorship references on this ticket now name the same ticket, correcting one that named the wrong one. The exact check: `git log --oneline --diff-filter=A -- crates/tiler-cache/src/expansion/preflight.rs` → `0f98b1f Probe a cache root's filesystem properties on request`, and `git show --stat 0f98b1f` shows that commit editing `tickets/add-an-expansion-cache-root-preflight.md` and adding `crates/tiler-cache/src/expansion/preflight.rs`.

## Accepted (2026-07-31)

Tom ratified every item without exception, over the surface verified at `f5a1725`. The packet disclosed two facts beyond the item list and the acceptance covers both: `ExpansionCache::root()` is a public accessor the item list did not name, and the preflight surface was still `pub(crate)` at review time, so acceptance included its promotion. The promotion landed with this acceptance: `ExpansionCache::preflight`, `PreflightReport` (five verdict accessors, `root`, `all_probed_properties_hold`, associated `cross_host_exclusion_caveat` text), and `PreflightVerdict` are `pub` and re-exported from `tiler_cache::expansion`, the ADR 0074 §7 staging allow is removed, and two doc links that named a nonexistent `cross_host_exclusion_is_unchecked` accessor were corrected to `cross_host_exclusion_caveat`. Crate documentation now states the accepted status, and the stale claims in `expansion.rs`, `collect.rs`, `bounded-collection.md`, and `crash-and-race-protocol.md` that this ticket would promote the collection surface were repointed to `accept-the-expansion-cache-maintenance-boundary`, which the acceptance deliberately excludes.

## Graph maintenance

- Move this ticket through the exact public-boundary review now that all four dependencies are done; do not redispatch the two completed signature decisions.
- On acceptance, replace reviewed-draft wording only for the exercised `tiler_cache::expansion` surface and update every contract or catalog that names its status.
- Keep namespace-wide accounting, collection, and purge under `accept-the-expansion-cache-maintenance-boundary`; this ticket must not absorb that separate lifecycle and caller boundary.
