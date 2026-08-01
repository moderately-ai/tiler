---
id: promote-the-build-time-cache-and-correspondence-seam
title: Promote the build-time cache and correspondence seam
status: done
priority: p2
dependencies: []
related: [produce-a-custom-backend-payload-through-the-build-orchestrator]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, build, cache]
---
## User-visible outcome

A backend that is not Metal reaches cache-subject composition, miss-only external compilation, and payload correspondence validation through `tiler-build` rather than reimplementing them, without the Metal cache protocol becoming the generic model.

## Why this is separate

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 11 promotes the *assembly* seam and names nothing else; `produce-a-custom-backend-payload-through-the-build-orchestrator` landed exactly that. What stayed Metal-shaped is `crates/tiler-build/src/metal_cache.rs`: `accept_or_publish_single_payload_metal_artifact` takes a `PreparedMetalPayload`, validates the descriptor against the hardcoded `tiler.metal`/`metallib`/`NativeImage` constants, and runs `validate_metal_payload_metadata` — a fact-level Apple correspondence check — inside the miss closure and again after resolution.

The reason it was not promoted with the assembly seam is a finding rather than an omission: the cache orchestration's structural obligations (subject composition, identity agreement before publication, re-validation of every result) are **interleaved** with payload-specific validation at three points, so a single closure parameter does not factor them the way `assemble_artifact`'s did. Promoting it therefore needs a design decision, not a move.

## Implementation keys

- Decide whether the neutral shape is a structural facade with a post-decode hook, a declaration record plus one compile closure, or a split into two functions; state the elimination.
- Keep the artifact layer's derivation intact: the payload digest is derived from canonical metadata bytes, the composed subject from `tiler_cache::expansion::ComposedSubject::compose`.
- Preserve the Metal path's exact refusal kinds and their order. `MetalPayloadFact`-level diagnostics are a refinement over the descriptor-digest comparison that already subsumes them; losing them is a real regression in explainability even though it changes no accept/reject decision.
- Keep the standard Metal artifact identity and composed cache subject byte-identical; `the_standard_metal_path_publishes_its_recorded_identities` in `crates/tiler-build/src/metal_plan.rs` is the pinned evidence.
- Extend the non-Metal producer in `crates/tiler-build/tests/custom_backend` to drive the promoted path instead of `backend::accept_or_publish`, and delete that helper when it is superseded.

## Closes when

The non-Metal producer publishes and re-accepts through the promoted seam, the Metal goldens have not moved, every refusal has been watched failing, and the public boundary is in a packet for Tom.

## Graph maintenance

- Do not introduce a new crate; if one seems required, stop and report.
- Keep dynamic plugin loading and provider discovery out of scope.

## Outcome

**Landed.** `crates/tiler-build/src/payload_cache.rs` is the promoted seam: `accept_or_publish_single_payload_artifact`, `DeclaredPayload`, `AcceptedArtifact`, `SinglePayloadCacheError<M, C, A>`, and `SinglePayloadProtocolError<M>`. No new crate. `metal_cache.rs` is now Metal's specialization of it and keeps `MetalArtifactProtocolError` and `MetalCacheError<E>` unchanged; `AcceptedMetalArtifact` was deleted as superseded by the neutral `AcceptedArtifact`.

**The three-shape elimination, run against the three interleaving points.** The points are: (1) before the subject exists, the pending artifact's sole payload must be the declared one and its digest must be the compilation the key will name; (2) inside the miss closure after encoding and before publication, the produced artifact must agree on identity, carry the declared payload, correspond to the compilation just performed, and hold the exact object the compile step returned; (3) after any resolution, the same payload rules again, minus the object comparison, plus identity.

- *A structural facade with one post-decode hook* fails at point 1 — the pending artifact is never decoded, so a post-decode hook cannot express the check at all — and fails at points 2 and 3, because the correspondence refusal has to fire *between* the facade's metadata-presence and descriptor-equality steps and a single hook before or after the structural block reorders the refusals.
- *A declaration record plus one compile closure* fails at points 2 and 3. A neutral record supports only equality of declared and carried metadata, which collapses eleven named Apple facts into one undifferentiated refusal. It changes no accept/reject decision — the digest is derived from the canonical metadata bytes, so any metadata disagreement is caught one step later — which is exactly why losing the naming is the regression the ticket forbids.
- *A split into two functions* fails outright: both halves need the same operands (the subject is composed from the pending artifact's identity, which every identity check then compares against), so the first function's only output is redundant with an input the second still requires, and the resulting signature would admit composing a subject from one artifact and publishing another under it.

**What survives is the first shape refined by the second, and the split is sharper than the ticket assumed.** Reading `validate_descriptor` showed the backend's *descriptor* statement is four governed values compared for equality — data, not behaviour, and a neutral comparison names the disagreement exactly as well as Metal's did. So `DeclaredPayload` carries backend key, representation key, schema, execution policy, payload digest, and compilation-facet bytes, and the seam performs that comparison itself. Only the *correspondence* statement is genuinely behaviour, because only a backend knows which facts its metadata asserts and what to call each one. One data record, one closure — not two closures, and not a trait, per ADR 0090 item 11.

**The promoted signature, verbatim:**

```rust
pub fn accept_or_publish_single_payload_artifact<M, C, A>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    declared: &DeclaredPayload<'_>,
    correspondence: impl Fn(&PayloadMetadata) -> Result<(), M>,
    compile: impl FnOnce() -> Result<PayloadContent, C>,
    assemble: impl FnOnce(PayloadContent) -> Result<VerifiedArtifactProgram, A>,
) -> Result<AcceptedArtifact, SinglePayloadCacheError<M, C, A>>
```

Three type parameters because three different authorities carry three different remedies: `M` names a payload fact the backend compared, `C` an external compilation that failed and may legitimately be retried, `A` an assembly the caller could not perform. Collapsing `M` into `C` would make a protocol defect that must never become a rebuild indistinguishable from an environment failure that can.

**Goldens unchanged.** `the_standard_metal_path_publishes_its_recorded_identities` passes with both constants untouched (`7a11d035…`, `58a71c54…`); the diff contains no edit to `crates/tiler-build/src/metal_plan.rs` goldens. The object-comparison domain separator was renamed from `tiler.build.metal-object-validation.v1\0` to `tiler.build.object-validation.v1\0`, and the unmoved goldens are the evidence it is a local comparison rather than an identity input.

**Refusals preserved and watched failing.** Metal's protocol vocabulary maps one-to-one onto the neutral one by an exhaustive `From` arm, so a new neutral refusal is a build error rather than a lost Metal case. Each of the seam's six structural comparisons was separately disabled and re-run: the declared-descriptor comparison is caught by `a_declared_payload_the_pending_artifact_does_not_carry_is_refused`, the pending-digest comparison by `a_declared_digest_other_than_the_pending_payloads_is_refused`, the correspondence call by both `a_compilation_other_than_the_one_expected_is_named_fact_by_fact` and `a_cache_entry_whose_payload_moved_is_refused_after_resolution`, the expected-descriptor comparison by the latter, the object-digest comparison by `an_artifact_carrying_other_object_bytes_is_refused_before_publication`, the post-resolution identity check by `a_cache_entry_naming_another_artifact_is_refused_rather_than_accepted`, and the pre-publication identity check by `a_produced_artifact_whose_identity_moved_is_refused_before_publication`. Disabling the correspondence call degraded the point-3 refusal from `Correspondence(Target)` to `PayloadSubject`, which is the subsumption this ticket describes, measured rather than argued.

**Two refusals cannot fire today, and the reason is exact rather than assumed.** `PayloadPortfolio` is unreachable through `assemble_plan_artifact`: an entry's `BackendEntryRef` requires a `PayloadId` that only a push mints, so no verified artifact carries zero payloads, and a second payload is refused as `ArtifactDiagnostic::UnusedPayload` (pinned by `a_second_artifact_family_cannot_yet_share_one_envelope`). `MissingPayloadObject` is unreachable because `decode_artifact` populates `payload_metadata` and `payload_content` from one presence flag (`crates/tiler-artifact/src/program/codec/view.rs:147-156`), so `MissingPayloadMetadata` always fires first. Both are retained, and the variant documentation says why. `PayloadDescriptor` and `PayloadSubject` are reachable at points 1 and 3 but not at point 2, because the payload descriptor is folded into canonical artifact identity and `ArtifactIdentity` is compared first. `CacheArtifact` is unreachable through this seam, which decodes the envelope with the same `decode_artifact` the cache's own validator uses.

**The superseded helper is deleted.** `backend::accept_or_publish` in `crates/tiler-build/tests/custom_backend/backend.rs` is gone, along with `ScalarHostRefusal::CacheIdentity` which only it produced. The non-Metal producer now states a `PayloadDeclaration` and a `correspondence` function over its own five-variant `ScalarHostFact`, assembles a pending artifact through `assemble_pending`, and drives the promoted seam. Its miss closure is counted, so miss-only compilation is a measurement: two resolutions, one compile.

**Public boundary for Tom.** `accept_or_publish_single_payload_artifact`, `DeclaredPayload`, `AcceptedArtifact` (with `resolution`, `cache_subject`, `decoded`, `into_resolution`), `SinglePayloadCacheError<M, C, A>`, `SinglePayloadProtocolError<M>`, and the delegating `AcceptedMetalPlanArtifact::decoded`. `accept_or_publish_single_payload_metal_artifact` keeps its name and its error types but now returns `AcceptedArtifact`, and `AcceptedMetalPlanArtifact::into_parts` returns it too.

**Named and not done.** `validate_metal_payload_metadata` stays `pub(crate)`, so an out-of-crate *partial Metal* provider can drive the promoted seam but cannot supply Metal's own correspondence check without restating it; the in-crate wrapper is how Metal reaches it. Publishing it is a separate public-boundary decision and was not taken here. Separately, [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s status paragraph still reads "the item-11 orchestration promotion remain unimplemented", which was already stale after `produce-a-custom-backend-payload-through-the-build-orchestrator` and is doubly stale now; correcting it is a `contracts/decisions` edit outside this ticket's scopes.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the promoted seam — `accept_or_publish_single_payload_artifact` with `DeclaredPayload`/`AcceptedArtifact`/the two error types — under the ADR 0090 item-11 umbrella, with the three-error-parameter remedy separation and the Metal one-to-one `From` mapping noted as the design's load-bearing halves. `validate_metal_payload_metadata`'s visibility stays a separate future boundary. Recorded for Tom's morning review.
