---
id: bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes
title: Bind runtime library and pipeline caches to exact payload bytes
status: ready
priority: p1
dependencies: []
related: [correct-stale-artifact-identity-and-delivery-authorities]
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

## Closes when

Every adopted runtime cache formula binds exact emitted object bytes; no current runtime authority uses the retired `BackendPayloadDigest` name; the corrected contract agrees with `docs/artifact-abi.md` and `docs/research/artifacts/target-neutral-artifact-envelope.md`; and the targeted and full gates pass.

## Graph maintenance

- Relate any implementation ticket that introduces a reusable runtime library or pipeline cache.
- Split stale cache-key authorities outside `research/runtime` instead of silently expanding this ticket.
- Close this ticket once the runtime contract is coherent; implementing a production cache remains separate work unless an existing ticket already owns it.
