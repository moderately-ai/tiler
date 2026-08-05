---
id: retain-canonical-msl-under-a-debug-expansion-cache-entry
title: Retain canonical MSL and tool diagnostics under a debug expansion-cache entry
status: done
priority: p3
dependencies: []
related: [retain-and-attribute-a-real-msl-failure-through-an-expansion, retain-succeeding-metal-stage-tool-output, state-a-debug-retention-from-the-inline-frontend]
scopes: [implementation/cache, implementation/build, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, cache, diagnostics]
---
## User-visible outcome

Under a debug configuration, the canonical MSL an expansion emitted and the diagnostics the offline tools produced are readable from the expansion-cache entry the compilation resolved to, without recompiling and without weakening what a validated hit proves.

## Why this exists

**Fact — split out of `retain-and-attribute-a-real-msl-failure-through-an-expansion` on 2026-08-04, not absorbed.** That ticket's own implementation keys say the permission "is separable and may split into its own ticket; it changes what an entry stores, which is cache-identity-adjacent and needs its own reasoning about whether retained diagnostics participate in validation". The split is also a scope fact: the retention ticket holds `implementation/frontend`, and every file this work touches is under `crates/tiler-cache/**` and `crates/tiler-build/**`.

**Fact — the permission exists and nothing delivers it.** `docs/integration/frontends.md` states "Debug configuration may retain canonical MSL and tool diagnostics under the cache entry". `crates/tiler-cache/src/expansion.rs` mentions neither MSL nor tool output; a bundle carries the composed subject and the artifact envelope and nothing else.

**Inference — the hard part is validation, not storage.** `ExpansionCache::lookup` has no fast path: every read decodes the whole frame, re-derives the key from the carried subject, and re-proves the envelope. A new section has to answer three questions before any bytes are written — whether retained text participates in the key (it must not: the same compilation must resolve to one entry whether or not a debug configuration asked for text), whether it participates in the frame's digest set (it must, or an entry could be edited in place and stay valid), and what a hit does when the section is absent because the entry was published by a build that did not retain (a hit, with nothing to show, rather than a miss).

**Inference — a failing compilation has no entry to attach to.** The diagnostics most worth retaining are the ones from a compilation that produced no artifact, and nothing is published for a miss that failed. So "tool diagnostics under the cache entry" covers the *succeeding* compilation's warnings, and a failed compilation's diagnostics reach a consumer only through the family-scoped `compile_error!` the frontend already emits. Saying which of the two this ticket delivers is part of its first deliverable.

## Implementation keys

- Decide participation before storage: key, digest set, and absent-section behaviour, each with the reason, and none of them decided by what is convenient to encode.
- A debug configuration is an input a caller states, never a host or environment sniff — `tiler_cache` must not read the process environment to decide what an entry carries.
- Retained text is bounded the way `ToolOutput` already is, and truncation is recorded rather than hidden.
- Say explicitly whether a failed compilation is in scope; if it is not, record why and leave the frontend's retained `compile_error!` as the only route.

## Closes when

An expansion cache entry carries the canonical MSL and the retained tool diagnostics under a stated debug configuration; a hit validates the same properties it validates today with the section present, absent, and damaged, each exercised; the identity decision is recorded where cache identity is specified; and `docs/integration/frontends.md`'s remaining-checks list names what is delivered.

## Outcome — 2026-08-05

**Scope added: `contracts/integrations`, and why it belongs here.** The close condition names `docs/integration/frontends.md`'s remaining-checks list, that file maps to `contracts/integrations` in `ticketsplease.toml`, and no live ticket held that scope when this branch edited it (checked against `tkt list --status in-progress,review,blocked`). The edit is confined to the two named list entries. This is declaration and scheduling metadata for already-authorized work, not a product-scope expansion.

**The three identity answers, as implemented.** All three are recorded in `crates/tiler-cache/src/expansion/retention.rs`'s module documentation — which is where cache identity is specified, beside `bundle.rs`'s frame and `key.rs`'s derivation — and summarized in `crates/tiler-cache/src/expansion.rs` under the immutability property.

1. **Retained text does not participate in the key.** `CacheKey::derive_bytes` is called on the subject bytes alone, before the retention is even encoded; a retention is not a `SubjectFacet`, so there is no path by which one could reach a key. A compilation therefore resolves to one entry, at one entry path, with or without retention. `a_retention_does_not_reach_the_key` asserts the equality *and* asserts that the two framed bundles differ and that a key taken over the framed bundle would separate them — so the equality is a statement about the derivation rather than about two identical inputs.
2. **It does participate in the frame's digest set.** The section carries its own content digest in the descriptor table, sits inside the declared total length, and must begin exactly where the previous section ended. An entry edited to add, alter, or remove retained text is refused rather than served.
3. **An absent section is a hit with nothing to show.** The retention is the one optional section; `decode_sections` deliberately does not `ok_or` it. An entry published before retention existed still validates, and a caller that states a retention still hits it — with `DebugRetention::is_empty()` true, because the publishing build is what a hit reports. Absence is also the *canonical* spelling of "nothing retained": an empty retention encodes to no section, and a framed section declaring zero runs is refused, so one fact has one encoding.

**What the section carries, which is not the canonical MSL — and the evidence that changed the plan.** The ticket assumed the MSL needed new storage. It does not: `tiler_build::metal_compile_request` puts the emitted translation unit's exact source into `PayloadMetadata::source`, that record is the preimage the payload digest is taken over, and the artifact envelope carrying it is already inside the cache bundle. So the canonical MSL is readable from every validated hit today, under the digest that names it — `one_prepared_compilation_becomes_pending_and_carried_payloads` in `crates/tiler-build/src/metal_assembly.rs` pins the producer half (`payload.metadata().source == unit.source().as_bytes()`), and `a_retained_diagnostic_survives_publication_and_returns_from_the_hit` in `crates/tiler-build/tests/custom_backend` pins that it survives publication and comes back from the hit. Copying it into a non-keyed section was rejected on the same ground ADR 0082 rejects a second digest authority: an unkeyed copy can disagree with the keyed original, and nothing could refuse the disagreement. What has no other home is the tool run's output — a warning is not a compilation input, and folding one into payload metadata would give two hosts two identities for one compilation — so that is what `BundleSection::DebugRetention` carries.

**Failed compilations are out of scope, and the ticket's own inference is why.** Nothing is published for a miss whose build step failed: `resolve_retaining` propagates `PublishFailure::Build` before any bundle is encoded, so there is no entry for a failed compilation's diagnostics to attach to. Attaching them would require publishing a *failure* entry, which is a different cache with different identity, validation, and invalidation rules — and would put a fail-open artifactless entry on the hit path, which ADR 0050's fall-open rule does not admit. A failed compilation's text reaches a consumer through the family-scoped `compile_error!` `retain-and-attribute-a-real-msl-failure-through-an-expansion` delivered, and that route is left exactly as it is.

**Bounded like `ToolOutput`, with truncation recorded.** `MAX_RETAINED_RUN_BYTES` is 16 KiB, matching `tiler_metal_aot::diagnostic::MAX_RETAINED_OUTPUT_BYTES` and restating its reasoning rather than importing it (ADR 0082 item 2 fixes this crate's closure to `tiler-artifact`). `RetainedText` keeps bytes rather than a `String` for `ToolOutput`'s own reason, and reports `total_bytes`, `is_truncated`, and `is_valid_utf8`. `MAX_RETAINED_RUNS` (16) and `MAX_RETENTION_LABEL_BYTES` (64) bound the run table and the labels, checked when a run is added and again when a stored section is decoded.

**Debug configuration stays a caller-stated input.** `tiler_cache` reads no environment and neither does `tiler_build`: retention arrives as a value out of the build closure — which is when it exists, since a compiler's diagnostics are produced by the run and an entry is framed once and published by one rename. `ExpansionCache::get_or_publish` is `get_or_publish_retaining` with `DebugRetention::none()`, so there is one publication route rather than two.

**The Metal producer retains nothing today, and that is a fact about the driver.** `Toolchain::run_stage` keeps a stage's captured output only in the `!status.success()` arm and drops both streams on success, so a succeeding Metal compilation has no diagnostics to state. `accept_or_publish_delivered_metal_artifact` states `DebugRetention::none()` and says so at the call site rather than framing an empty section that would read as a delivered capability. Filed [`retain-succeeding-metal-stage-tool-output`](retain-succeeding-metal-stage-tool-output.md), with [`state-a-debug-retention-from-the-inline-frontend`](state-a-debug-retention-from-the-inline-frontend.md) deferred behind it. The mechanism is not unexercised in the meantime: `crates/tiler-build/tests/custom_backend` is an out-of-crate-shaped producer that shares no code with the Metal path, and it states a retention, publishes it, and reads it back from a validated hit.

**Public surface changed — Tom's to accept.** `tiler-cache`: `DebugRetention`, `RetainedText`, `RetentionRefusal`, `RetentionRejection`, `MAX_RETAINED_RUN_BYTES`, `MAX_RETAINED_RUNS`, `MAX_RETENTION_LABEL_BYTES`, `BundleSection::DebugRetention`, `BundleRejection::RetainedDebug`, `ExpansionCache::get_or_publish_retaining`, `CachedEntry::retained_debug`, and a `retained` field on `Resolution::Uncached`. `tiler-build`: `CompiledPayloads`, and `accept_or_publish_delivered_payload_artifact`'s compile closure now returns one (`From<Vec<PayloadContent>>` covers every non-retaining caller). `accept_or_publish_delivered_metal_artifact`, `accept_or_publish_metal_plan`, and every `tiler-macros` call site are unchanged.

**Every new check was watched failing.** Four perturbations, each reverted: making the retention participate in the key (`a_retention_does_not_reach_the_key` failed on the key equality); making an absent section a `MissingSection` (the absent bundle test and the publication path both failed); dropping a damaged retention instead of refusing it (the resealed-forgery assertion failed); and making the build seam discard the backend's retention (`a_retained_diagnostic_survives_publication_and_returns_from_the_hit` failed).
