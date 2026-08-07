---
id: emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable
title: Emit a route requirement from the build producer so a family-authority refusal is drivable
status: deferred
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, declare-a-required-gpu-family-in-the-artifact, select-executable-variants-across-registered-backend-families, design-the-adapter-owned-route-requirement-answer-channel, accept-the-public-route-requirement-answer-boundary, admit-the-bf16-type-and-carrier-into-every-total-map, lower-bf16-to-metal, carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit]
scopes: [implementation/build, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
tags: [implementation, artifacts, build, route-requirements, evidence-gap]
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

## Dispatch outcome and two review corrections — 2026-08-03

**This ticket is deferred on its original trigger: the first emitted capability requirement that is additional to, and not derivable from, the verified program. No such requirement is reachable today.** The first review correctly found a current family-scoped live-device obligation that the initial audit missed; the second review correctly showed why it is not this ticket's row.

**Fact — the missed obligation is real.** Every region proposal requires complete unsigned-64 index arithmetic (`crates/tiler-compiler/src/physical.rs:2060-2082`, exact type-to-axis map at `:2119`), every Metal translation unit declares the structured index type as `uint64_t` (`crates/tiler-metal/src/emit.rs:261-266,811-818`), and all six current Metal goldens carry the widening. The authority ledger at `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:90-97` sources that operation family at `Metal3 | Apple3 | —`: Apple3-or-newer, with no Mac-family guarantee.

**Fact — copying that obligation into `RouteRequirement::BackendFeature` is forbidden by the accepted derivability rule.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md):23-25 says structural requirements already derivable from the verified route remain direct checks and the artifact records only *additional* requirements not already stated by the verified program. [`docs/artifact-abi.md`](../docs/artifact-abi.md):280 gives the reason: copying a derived requirement into a route row mints a second producer authority that can contradict the first. `IndexArithmeticU64` is derived from `KernelType::Index` before target feasibility and is therefore an eliminated non-example, however useful its Apple3 authority is.

**Elimination, corrected.** Relying on compile-profile feasibility alone still fails because a producer declaration is not a live-device observation. Inferring Apple9 from the target-profile key still overstates the Apple3 requirement. But a backend feature row also fails: it duplicates a verified-program fact under an independently editable key/version/payload. The surviving architecture is a direct live-device check over the verified-program-derived requirement, alongside the existing direct accessible-window and local-memory checks.

**The direct-check defect has a separate owner.** [`carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`](carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit.md) records the missing neutral carrier and the backend comparison. `ResourceRequirements` is what an encoded artifact preserves for direct checks, but its complete field population omits index arithmetic; a decoded consumer does not receive KIR operations from which to re-derive it. That ticket owns carrying the nominal requirement through the artifact schema and checking it against live Metal device authority before commitment. [`check-synchronization-realization-before-the-routing-commit`](check-synchronization-realization-before-the-routing-commit.md) is related rather than reused: synchronization already has a carried field and needs a different whole-subject comparison.

**Reconsideration trigger for this producer-row ticket.** Resume only when Metal emission first consumes a device capability that is not already derivable from the verified program or its direct dispatch/resource record. BF16 carrier/lowering remain the nearest named candidate, but BF16 support alone does not fire the trigger; its own authority must establish an additional payload feature. At that point the reserved ADR 0092 mint/decode boundary becomes concrete and goes to Tom before implementation. Until then, zero route rows is the accepted canonical statement rather than a missing producer feature.

## Trigger check log

- 2026-08-04 — **not fired.** The trigger is the first Metal emission consuming a device capability not derivable from the verified program; the nearest named candidate is BF16, and both [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md) and [`lower-bf16-to-metal`](lower-bf16-to-metal.md) are `todo`, so no BF16 payload feature exists to establish its own authority. Zero route rows remains the accepted canonical statement. Recheck: `git grep -n 'RouteRequirement' -- crates/tiler-build/src` still returns nothing.
