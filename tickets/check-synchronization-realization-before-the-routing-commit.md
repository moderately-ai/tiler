---
id: check-synchronization-realization-before-the-routing-commit
title: Check synchronization realization before the routing commit
status: todo
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal]
scopes: [implementation/compiler, implementation/runtime, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A host refuses a route whose entries require a synchronization realization the executing backend does not declare, before the routing commit and with a typed reason naming the exact subject.

## Why this is not already covered

**Fact — the requirement reaches the artifact and nothing reads it.** `ResourceRequirements::synchronization` is an `Option<SynchronizationSubject>` carrying the complete five-dimension subject a region's schedule requires (`crates/tiler-ir/src/schedule/model.rs:871`); it is encoded into the envelope by `push_resources`, decoded by `crates/tiler-artifact/src/program/codec/decode.rs`, and published through `DecodedEntry::resources()` (`crates/tiler-artifact/src/program/codec/view.rs:673`). No loader, adapter, or prototype compares it against anything. Reproduce in one line:

```sh
rg -n 'resources\(\)\.synchronization' crates prototypes spikes
```

**Fact — the comparison authority exists and is crate-private.** `CheckedTargetProfile::resolve_synchronization` compares a whole subject against the profile's `DeclaredSynchronizationRealization` rows, and both it and `SynchronizationRealization` are `pub(crate)` in `tiler-compiler` (`crates/tiler-compiler/src/target/feasibility.rs`). A runtime-side check therefore needs a decided public boundary, which is why `realize-parallel-reduction-strategies-on-metal` filed this rather than absorbing it: that ticket held no compiler scope.

**Fact — the refusals that do exist are at emission, not preflight.** `MetalEmitError::UnsupportedBarrier` with `BarrierRejection::{MemoryVisibility, FencedSpace}` refuses an unrealizable barrier while emitting (`crates/tiler-metal/src/emit.rs:1601`), and those two arms are covered by `no_metal_barrier_establishes_device_wide_visibility`, `a_simd_group_barrier_cannot_claim_workgroup_visibility`, and `a_space_without_a_fence_flag_is_rejected`. That is a *producer-side* guarantee about bytes this workspace emitted. It says nothing about an envelope reaching a host whose backend realizes a different subject, which is the case a delivered artifact actually has.

**Fact — two of `BarrierRejection`'s arms are unreachable rather than untested.** `BarrierRejection::ExecutionScope` and `BarrierRejection::Ordering` have no test because no constructible input reaches them: `ExecutionScope` declares exactly `Subgroup` and `Workgroup` and both are accepted by the preceding arm, and `BarrierOrdering` declares exactly `AcquireRelease` and it is accepted likewise (`crates/tiler-ir/src/kernel/model.rs:595` and `:633`). Both enums are `#[non_exhaustive]`, so the arms are the required defensive positions for a widened vocabulary. Do not "fix" them with a test; a test becomes possible only when a variant is added, and adding one is that vocabulary's decision.

## Implementation keys

Decide the public boundary first — Tom's under ADR 0075, because it exposes a compiler comparison to a consumer. The subject is matched as one whole value and never dimension by dimension: each of its five dimensions is separately true of some realization, so a conjunction inferred from independent facts is not a statement about any realization. Refuse with a typed reason naming the required subject and the declared one, classified as a route miss — another variant may declare a subject this backend does realize.

The check belongs before the routing commit, beside the live-device requirement resolution that already refuses a missing GPU family, not after it. A synchronization subject is a *backend* fact, so the neutral loader must not interpret it: the same split `tiler.metal.route-requirement.minimum-gpu-family` already takes, where `tiler-runtime` owns the decision and the adapter owns the observation.

## Required evidence

A negative fixture refuses an entry requiring a subject the backend does not declare, before the commit, and the refusal names the subject. A positive neighbour routes, so the refusal is evidence about the perturbation. Every new check is mutation-proved. Targeted tests plus `make full` pass.

## Graph maintenance

- Keep the emission-time barrier refusals where they are; this adds a delivery-time check rather than moving one.
- If the boundary decision widens `ExecutionScope` or `BarrierOrdering`, the two unreachable `BarrierRejection` arms above become testable and gain cases in the same change.
