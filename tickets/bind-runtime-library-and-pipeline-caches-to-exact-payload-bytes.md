---
id: bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes
title: Bind runtime library and pipeline caches to exact payload bytes
status: done
priority: p1
dependencies: []
related: [correct-stale-artifact-identity-and-delivery-authorities, prototype-candle-metal-adapter]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, runtime, cache, correctness]
---
## User-visible outcome

The adopted runtime execution contract keys loaded libraries and prepared pipelines by an identity that binds the exact emitted backend object, so two non-reproducible links of one compilation subject cannot alias in a device-scoped runtime cache.

## Why this is a correctness ticket

- **Fact:** `docs/research/runtime/runtime-execution-contract.md` defines `RuntimeArtifactKey`, `LibraryCacheKey`, and `PipelineCacheKey` using `BackendPayloadDigest`.
- **Fact:** the implemented artifact model has no current identity subject by that name. Its backend payload descriptor digest covers the canonical compilation-subject metadata and deliberately excludes emitted code bytes; the governed `BackendPayloadCode` section digest and the complete `EnvelopeDigest` bind the exact transported object.
- **Fact:** Tiler explicitly admits two non-reproducible links of one compilation subject having equal artifact identity and different envelope bytes.
- **Inference:** interpreting the runtime contract's `BackendPayloadDigest` as the descriptor digest can reuse a library or pipeline prepared from different object bytes. That is not a cache-efficiency trade-off; it can execute the wrong code.

## Implementation keys

- Read the runtime execution research record and the artifact identity authorities in full. Preserve the distinction among compilation-subject identity, artifact identity, section integrity/content identity, and complete-envelope identity.
- Replace each ambiguous `BackendPayloadDigest` use in the runtime artifact, library, and pipeline key formulas with a current exact-object-bearing subject: the complete `EnvelopeDigest` or the governed `BackendPayloadCode` section digest, as appropriate to the cache's reuse granularity.
- State why the compilation-subject digest is insufficient and retain the existing live-device/context, entry, specialization, pipeline-descriptor, and runtime-mode dimensions.
- Search runtime, cache, artifact, and integration authorities for the same obsolete name or semantic conflation. Correct only `research/runtime` here; split findings in other scopes.
- Prove the stale-key check can fail against the current base, run `tkt lint`, `git diff --check`, the documentation link/authority checks available in this repository, and one final `make full`.

## Outcome

The three key layers now name the subject each actually consumes. `RuntimeArtifactKey` is the complete `EnvelopeDigest` plus the selected payload's canonical descriptor position, so it identifies both the exact validated bytes and the payload selected within them. `LibraryCacheKey` uses the selected `BackendPayloadCode` section's governed `SectionDigest`, permitting safe reuse of identical object bytes across envelopes. `PipelineCacheKey` uses that exact code digest plus the resolved backend entry symbol. The neutral `BackendEntryKey` alone cannot distinguish metadata that maps one neutral key to different functions, while the whole metadata digest would unnecessarily prevent reuse when only transport slots or unrelated entries differ.

Using `EnvelopeDigest` for every cache would be correct but unnecessarily prevent library and pipeline reuse across envelopes carrying identical payload sections. Using only the compilation-subject digest would preserve reuse by silently allowing two non-reproducible objects of one subject to alias. The split keys retain correctness and the useful reuse granularity without inventing a new identity authority.

The existing live-device/context, entry, specialization, canonical pipeline descriptor, and runtime-mode dimensions remain present. The graph now relates `prototype-candle-metal-adapter`, the ticket that owns the first reusable live-device cache. A corpus search found no other current authority using the retired `BackendPayloadDigest` name.

The stale-name check was proven able to fail: `rg -n 'BackendPayloadDigest' docs/research/runtime/runtime-execution-contract.md` finds all three obsolete formulas at base `0e74053` and finds none in the corrected record.

## Closes when

Every adopted runtime cache formula binds exact emitted object bytes; no current runtime authority uses the retired `BackendPayloadDigest` name; the corrected contract agrees with `docs/artifact-abi.md` and `docs/research/artifacts/target-neutral-artifact-envelope.md`; and the targeted and full gates pass.

## Graph maintenance

- Relate any implementation ticket that introduces a reusable runtime library or pipeline cache.
- Split stale cache-key authorities outside `research/runtime` instead of silently expanding this ticket.
- Close this ticket once the runtime contract is coherent; implementing a production cache remains separate work unless an existing ticket already owns it.
