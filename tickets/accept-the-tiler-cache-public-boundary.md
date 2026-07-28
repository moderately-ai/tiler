---
id: accept-the-tiler-cache-public-boundary
title: Accept the tiler-cache public boundary
status: awaiting-decision
priority: p1
dependencies: [report-cache-publication-state-after-the-rename-boundary]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact]
shared_scopes: []
paths: []
tags: [cache, api, decision]
---
## Decision needed (2026-07-28)

**Accept `tiler_cache::expansion`'s public surface as reviewed and drop its draft-boundary language, or name the items to change first?**

ADR 0082 admits the crate but not the interface, so `tiler_cache::expansion` documents itself as a **reviewed draft boundary** under ADR 0074 §7 and ADR 0075, on the same footing as `tiler_artifact::program` and `tiler_runtime::load` — `AGENTS.md` keeps admission and interface acceptance separate: "A tested implementation may serve as a concrete draft, but it is not implicit approval of its public interface."

| Option | Enables | Prevents |
| --- | --- | --- |
| **Accept as drafted.** The surface below becomes the accepted interface and the draft-boundary language comes out of the module documentation. | `implement-the-expansion-cache-protocol` and every downstream consumer stop coding against a draft. The `#[allow(dead_code)]` at `crates/tiler-cache/src/expansion/preflight.rs:2` and `crates/tiler-cache/src/expansion/collect.rs:102` come off with the promotion, because a `pub` item is not dead code. | The surface becomes a compatibility commitment while three shape questions are still open — the three named under **To be split** below. |
| **Hold pending the split questions.** Answer the three shape questions first, then accept whatever they leave. | The three shapes stay cheap to revise: an accepted surface is a commitment and a draft one is not, and each of the three changes a signature rather than an implementation. | Nothing can consume the cache in the meantime. `grep -rl tiler-cache crates/*/Cargo.toml prototypes/*/Cargo.toml` returns only `crates/tiler-cache/Cargo.toml` (re-run 2026-07-28), so the surface has no consumer to validate it either way — holding buys revision cheapness against a revision pressure nothing is currently generating. |

**Recommendation: accept as drafted, and decide the three split questions separately.** The "prevents" on the accept row is weakened by the same measurement that weakens the hold row: nothing depends on `tiler-cache`, so a compatibility commitment made today binds no caller, and the three shape questions are split into their own decisions rather than pre-answered by acceptance. **Counterpoint, and it is real:** accepting a surface with no consumer is accepting one nothing has exercised. The review is a reading rather than a use, and the first genuine consumer is the most likely source of a change request — which is exactly what acceptance makes expensive. If the answer is to wait for a consumer, say so, because that is a different reason to hold than the three shape questions and it has a different trigger.

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

## Split performed (2026-07-28)

The three shape questions were split into their own `awaiting-decision` tickets, each carrying its options and stated costs so one signature can be reshaped without rejecting this whole surface: `decide-the-composed-subject-backend-compilations-shape`, `decide-where-an-unfillable-subject-facet-is-refused`, `decide-the-lookup-argument-type`. This ticket keeps the ratification checklist and the derivations; a decision on any split applies here.

## Excluded decision

Whole-cache accounting, collection, and purge are deliberately excluded. They operate over a namespace rather than one key and have different callers, cost, reporting, and lifecycle. `accept-the-expansion-cache-maintenance-boundary` owns that atomic public decision.

## The hold on the outcome vocabulary is discharged (2026-07-28)

This ticket previously said not to accept the outcome/report vocabulary until `report-cache-publication-state-after-the-rename-boundary` made a successful atomic rename distinguishable from a refusal to publish. That ticket is `done` (verified: `grep -n '^status:' tickets/report-cache-publication-state-after-the-rename-boundary.md` → `status: done`), and what it made distinguishable is exactly that pair — a publication that completed its atomic rename now reports a different outcome from one that refused to publish, so a caller reading the report can no longer read a refusal as a success. The vocabulary is therefore ratifiable along with the rest of the surface.

## Provenance of the preflight surface

`ExpansionCache::preflight` landed `pub(crate)` under ADR 0074 convention 7 and was staged here rather than promoted. It carries a module-level `#[allow(dead_code)]` whose reason states why no caller was wired to satisfy the lint: the only place a caller would naturally go is the expansion path, and putting a filesystem probe there is the one thing the design forbids.

**Fact — the author is `add-an-expansion-cache-root-preflight`, not `prototype-expansion-content-cache`.** Both authorship references on this ticket now name the same ticket, correcting one that named the wrong one. The exact check: `git log --oneline --diff-filter=A -- crates/tiler-cache/src/expansion/preflight.rs` → `0f98b1f Probe a cache root's filesystem properties on request`, and `git show --stat 0f98b1f` shows that commit editing `tickets/add-an-expansion-cache-root-preflight.md` and adding `crates/tiler-cache/src/expansion/preflight.rs`.
