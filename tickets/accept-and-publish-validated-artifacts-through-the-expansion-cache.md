---
id: accept-and-publish-validated-artifacts-through-the-expansion-cache
title: Accept and publish validated artifacts through the expansion cache
status: todo
priority: p2
dependencies: [assemble-prepared-metal-artifacts-in-tiler-build]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact, implementation/workspace, implementation/build]
shared_scopes: []
paths: []
tags: [build, cache, correctness]
---
## User-visible outcome

The expansion cache is exercised through the real build-time orchestrator. Publication accepts only an artifact whose carried Metal payload matches the prepared compilation subject, and a hit is accepted only after the same correspondence is re-proved against the current prepared operation.

## Implementation keys

The first missing call site is the boundary between the assembled artifact from `assemble-prepared-metal-artifacts-in-tiler-build` and `tiler_cache::expansion::get_or_publish`. Add the `tiler-cache` edge only when this call exists.

Compose the backend-compilation facet from the prepared compilation identity and the artifact-program facet from the pending artifact identity. On a miss, validate correspondence before publication. On a hit, decode the returned artifact through the cache's existing governed validator, locate the payload deterministically, and validate correspondence before returning an accepted result. A correspondence mismatch is a typed producer/protocol failure and must not be converted into `Miss`, corruption replacement, or an automatic rebuild.

Cover the real positive end-to-end hit that ADR 0082 explicitly leaves unmeasured, including a two-call fixture proving the second call accepts the cached envelope without invoking compilation. Perturb the carried payload while preserving an internally valid outer bundle to prove the orchestrator check, rather than the cache frame or artifact decoder, is what refuses it.

## Graph maintenance

When both publication and hit acceptance run through the correspondence check and the positive real-artifact hit is measured, close this ticket and release `drive-the-build-orchestrator-from-a-checked-compiler-plan`. Update ADR 0082's consequence that currently says the end-to-end hit belongs to the orchestrator.
