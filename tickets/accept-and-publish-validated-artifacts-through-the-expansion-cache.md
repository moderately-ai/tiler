---
id: accept-and-publish-validated-artifacts-through-the-expansion-cache
title: Accept and publish validated artifacts through the expansion cache
status: done
priority: p2
dependencies: [assemble-prepared-metal-artifacts-in-tiler-build]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact, implementation/workspace, implementation/build, contracts/decisions, implementation/cargo-lock, implementation/metal-aot, project/tickets]
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

## Outcome — delivered

`a465bd32` closed the named Metal boundary. `tiler-build` composes the cache
subject from the prepared compilation and pending artifact identities, runs
external compilation only on a miss, validates the produced artifact before
publication, and revalidates a hit against the current prepared operation.
Correspondence and artifact-identity disagreement are typed protocol failures;
neither becomes a miss, replacement, or rebuild. The positive two-call fixture
publishes once and then accepts the cached envelope with the compilation
counter still at one, and ADR 0082 records that measured orchestrator path.

The later backend-composition work retained and generalized the same boundary.
At the current tree, `accept_or_publish_delivered_payload_artifact` is the
backend-neutral seam: it handles one declared payload per delivery position,
validates each produced object before publication, and repeats descriptor,
backend-correspondence, subject, and artifact-identity checks after every
resolution. Metal is one caller rather than the owner. The completed dependent
[`drive-the-build-orchestrator-from-a-checked-compiler-plan`](drive-the-build-orchestrator-from-a-checked-compiler-plan.md)
then connected checked compiler plans to this accepted path. No outstanding
graph release remains on this ticket.
