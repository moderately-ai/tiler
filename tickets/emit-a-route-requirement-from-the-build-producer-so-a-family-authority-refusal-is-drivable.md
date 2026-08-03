---
id: emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable
title: Emit a route requirement from the build producer so a family-authority refusal is drivable
status: deferred
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, declare-a-required-gpu-family-in-the-artifact, select-executable-variants-across-registered-backend-families, design-the-adapter-owned-route-requirement-answer-channel, accept-the-public-route-requirement-answer-boundary]
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

## Dispatch outcome — 2026-08-03

**The producer-side absence is real, but no truthful requirement value is derivable from a payload the producer can emit today. This ticket is deferred rather than implemented with a placeholder.**

**Fact — the activation condition recorded by the vocabulary's owning ticket has not fired.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md) defers producer minting until "the first emission whose payload uses a capability the measured profile does not universally provide". It separately records that no particular Apple-family threshold is qualified: both reachable measured devices report Apple9, so the cross-generation experiment needs a device of another generation or an observed feature-specific pipeline refusal. No later ticket supplied either fact.

**Fact — the complete production Metal type-emission map admits no floating-point kernel type except `KernelType::F32`.** Reading the exhaustive `msl_type` match at `crates/tiler-metal/src/emit.rs:811-818` gives the complete population at `2fdb71e`: `Bool`, `U8`, `I32`, `Index`, and `F32`; `rg -n 'KernelType::Bf16|KernelType::F16' crates/tiler-metal/src/emit.rs` returns zero matches over that file. [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md) remains `todo` and explicitly requires Metal to *refuse* BF16 at that step, while `lower-bf16-to-metal` owns the later `bfloat` spelling and hardware dispatch. The currently emitted F32 arithmetic, workgroup barrier, threadgroup storage, and launch-index vocabulary is already checked against the compile profile; nothing in the emitted unit names a family-specific feature or supplies a qualified minimum Apple family.

**Fact — the construction path has no hidden second insertion site.** `accept_or_publish_metal_plan` calls the same neutral `assemble_plan_artifact` three times — pending, cache-miss carried, and read-back carried — at `crates/tiler-build/src/metal_plan.rs:283-346`. That function's complete builder sequence is `crates/tiler-build/src/plan_artifact.rs:152-227`: providers, payloads, compiler-minted prepared-entry predicates, entries, `push_variant`, and `build`; it never calls `require_route`. Reproduce the population with `rg -n 'assemble_plan_artifact|require_route' crates/tiler-build/src/{metal_plan,plan_artifact}.rs`: the three production assembly calls are present and `require_route` is absent.

**Fact — the existing consumer would preserve the required pre-commit refusal if a truthful row existed.** `crates/tiler-runtime/src/load.rs:759-782` binds each decoded row and refuses a foreign owner; `crates/tiler-runtime/src/load/route.rs:439-474` resolves every row and returns `Unowned`, `Misanswered`, or `Unsatisfied` before producing `RoutePreparation`; routing commitment is only available later at `route.rs:740`. The missing fact is therefore producer derivability, not consumer ordering.

**Inference — `Apple9` in the profile key is not a payload requirement.** The declaration records the exact environment on which the profile was measured. Treating that measured host label as `minimum-gpu-family = Apple9` would convert bounded evidence into a portable lower-bound guarantee and would reject devices without evidence that the emitted payload needs the missing family. An `Apple8` value copied from ADR 0092's worked proposal is equally unavailable: that proposal labels the minting path unimplemented and the cross-generation qualification above never established the threshold. Either value would violate this ticket's instruction to emit what the payload actually needs rather than a row added so a test can pass.

**Fact — the contract-preserving minting boundary is also deliberately unavailable.** ADR 0092 says the governed row is minted through `tiler-metal`, keeping its key, version, payload codec, and Apple ordering in the owning backend. `rg -n 'BackendFeatureRequirement::new|minimum-gpu-family' crates/tiler-metal/src` returns no matches at `2fdb71e`. Adding that constructor requires the undeclared `implementation/metal` scope and activates [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md), whose boundary list reserves the minting constructor and governed key/version for Tom. Constructing the opaque bytes directly in `tiler-build` would duplicate the backend authority ADR 0092 eliminated.

**Why no hardware fixture was run.** A negative hardware drive needs a produced artifact with a truthful row. With no derived row, mutating the artifact into one would be the hand-built evidence this ticket exists to replace. The existing device-free loader refusals remain valid; no new hardware claim is made.

**Reconsideration triggers, both required.** Resume when (1) an emitted Metal operation has a primary-source or cross-generation-measured minimum-family/feature requirement not already derivable from the verified route, and (2) Tom has accepted or revised the reserved `tiler-metal` minting-constructor boundary and it is available to the producer. The BF16 lowering is the nearest named candidate, but its mere existence is insufficient: its owning work must establish the exact live-device requirement. Then emit the row and drive the positive and pre-commit negative cases on qualified hardware.
