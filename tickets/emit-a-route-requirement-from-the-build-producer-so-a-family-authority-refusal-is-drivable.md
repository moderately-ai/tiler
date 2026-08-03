---
id: emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable
title: Emit a route requirement from the build producer so a family-authority refusal is drivable
status: in-progress
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, declare-a-required-gpu-family-in-the-artifact, select-executable-variants-across-registered-backend-families, design-the-adapter-owned-route-requirement-answer-channel]
scopes: [implementation/build, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
tags: [implementation, artifacts, build, route-requirements, evidence-gap]
claimed_from: todo
assignee: agent-route-requirement
lease_expires_at: 1785786692
---
## User-visible outcome

A produced artifact carries the route requirement its payload actually needs of a device, so a consumer refusing on missing family or feature authority can be driven against a real produced artifact rather than only against a hand-built one.

## Why this exists

**Fact, reproducible in one line.** The build producer emits no route requirement at all:

```sh
git grep -n 'RouteRequirement' -- crates/tiler-build/src
```

returns nothing at `dc13abb`. A control pattern confirms the search works — `git grep -c 'DeviceFacts' -- prototypes/` returns hits — so this is an absence that was read, not a search that failed.

**Fact — the consumer side exists and the vocabulary is done.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md) and [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) are both `done`. The refusals exist and are exercised device-free — `each_undecidable_route_requirement_refuses_by_its_own_class` and `a_foreign_owner_is_refused_without_consulting_an_adapter` — and the production offer path refuses on `metal.host-applicability.unknown-translation-authority` before any routing commit.

**Inference — so the gap is producer-side, and it is what keeps one negative fixture off hardware.** [`realize-parallel-reduction-strategies-on-metal`](realize-parallel-reduction-strategies-on-metal.md) drove three of its four required negative fixtures on the qualified host and recorded the fourth-of-three — missing family/feature authority — as not re-driven on hardware for exactly this reason: no produced artifact carries a required family, so there is nothing for a real device to be asked about. That ticket recorded the reason rather than converting a device-free case into hardware evidence, which is the substitution its own body forbids.

## Required work

- Decide what the Metal producer actually knows about the payload it emits that constitutes a requirement of a device, and emit that — not a placeholder row added so a test can pass. A requirement the producer cannot derive from the payload is a requirement it must not state.
- State the requirement in the **backend-neutral** vocabulary. Do not add Apple vocabulary to the neutral artifact; that constraint is the reason the route-requirement layer exists.
- Keep the one-way preparation/commit boundary: a requirement is checked before routing commit, never after allocation, partial encoding, or submission.

## Required evidence

- A produced artifact carries a route requirement, read back from the encoded artifact rather than from the builder's own value.
- A consumer on a device lacking the required family refuses **before routing commit**, driven on hardware, with the refusal quoted.
- A consumer on a device that satisfies it proceeds — so the check is not refusing unconditionally.
- Watch each refusal fail: perturb the device facts and the declared requirement independently, and confirm the refusal names the axis that actually moved.

## Identity discipline

Adding a field to the artifact's encoded form moves artifact identity. If it does, that is an identity step: move the ledger in the same commit, recompute every pinned identity on the tree the change lands into, and enumerate each moved pin in the report. If the requirement rides in an already-encoded optional position and moves nothing, say so and show the pinned values unchanged rather than asserting it.

## Explicit non-goals

The adapter-owned answer channel ([`design-the-adapter-owned-route-requirement-answer-channel`](design-the-adapter-owned-route-requirement-answer-channel.md)) and its public boundary ([`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md)). No new resource dimension — [`correct-the-subgroup-threads-route-dimension-meaning`](correct-the-subgroup-threads-route-dimension-meaning.md) and [`rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`](rename-the-route-resource-floor-vocabulary-for-its-corrected-relation.md) own dimension changes.

## Closes when

A produced artifact carries a derived route requirement, a family-authority refusal is driven on hardware before routing commit with the refusal quoted, the positive case proceeds, each refusal has been watched failing under an independent perturbation, and any moved identity is enumerated.

## Graph maintenance

Filed 2026-08-02 at integration of `realize-parallel-reduction-strategies-on-metal`, which found the absence and recorded it rather than absorbing it.
