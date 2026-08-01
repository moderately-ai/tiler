---
id: decide-the-prototype-baseline-workgroup-guarantees
title: Decide what the prototype baseline guarantees about workgroup resources and synchronization
status: done
priority: p2
dependencies: []
related: [implement-the-single-workgroup-synchronized-reduction-strategy, realize-parallel-reduction-strategies-on-metal, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/compiler, implementation/build, implementation/metal, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, feasibility, synchronization]
---
## User-visible outcome

`TargetProfile::governed()` — the bounded prototype target-neutral baseline — either declares a threadgroup-memory budget and a workgroup control-barrier realization, or it is recorded as deliberately silent about both and the compile path's cooperative alternatives stay unadmitted against it. Whichever holds is a stated decision with its authority named, not a fact discovered by reading a rejection.

## Why this exists

**Fact.** `TargetProfileBuilder::governed` (`crates/tiler-compiler/src/target.rs`) declares `local-memory-bytes` as `0` and declares no `SynchronizationRealizationFact` at all.

**Fact.** `implement-the-single-workgroup-synchronized-reduction-strategy` landed a single-workgroup tree the governed provider proposes for every reassociating reduction subject. Against the baseline it is rejected twice over: `local-memory-bytes` required 16, available 0, and — once storage is sufficient — the required control-barrier subject resolves to `NoPath`. Both rejections are correct, are typed, and are driven by that ticket's own tests.

**Inference.** The consequence is that the strategy is never *admitted* on the profile the prototype compile path actually assesses against. Its positive path is proven against `TargetProfile::workgroup_tree_target_for_test`, a test-only widening that says at length why it is test-only.

**Proposal, not a recommendation to adopt.** Raising the baseline's rows would be a capability claim, and the prototype authority (`governed_profile_source`) has no evidence for a threadgroup-memory budget or a barrier realization. The precedent that looks nearest — raising `buffer-bindings` from two to four — is not the same act: that bound was raised to match what the *request boundary* already admitted, and no device resource was being claimed. Against that, a baseline that can never admit any synchronized strategy makes the prototype path structurally unable to exercise the one it now has.

## The decision

One of:

1. **Declare.** The baseline states a threadgroup-memory budget and the workgroup control-barrier realization under the governed prototype authority, with the derivation of the number recorded beside it. Every artifact identity, cache subject, and pinned descriptor derived from the profile moves and must be recomputed on the merged tree.
2. **Stay silent.** The baseline keeps `local-memory-bytes = 0` and declares no realization; the cooperative alternatives remain enumerated-and-rejected there, and admission waits for `construct-and-bind-the-first-authoritative-metal-compile-profile` and `realize-parallel-reduction-strategies-on-metal` to declare a profile with real authority.

Option 2 is the current state, so choosing it costs nothing and needs only the record.

## Required evidence if option 1 is chosen

- The declared budget's derivation stated on the declaration, in the form the existing grid-axis and buffer-binding rows use: what primary document or measurement supports it, and what it explicitly does not claim.
- Every pinned value recomputed on the tree the change lands in — the governed descriptor bytes in `crates/tiler-compiler/src/physical.rs`, `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` in `crates/tiler-build/src/metal_plan.rs`, the explain request qualifier, and any artifact/cache pin in `crates/tiler/src/route/tests.rs` — never copied from another branch.
- The test-only `workgroup_tree_target_for_test` removed or reduced to whatever the baseline still cannot state, so two authorities do not describe one profile.

## Closes when

Tom records the choice. Option 2 closes on the record alone; option 1 closes when the declaration, its authority, and every recomputed identity land together and `make full` passes.

## Graph maintenance

Do not fold this into `realize-parallel-reduction-strategies-on-metal`: that ticket owns the *Metal* profile's real authority, and this one is about what the target-neutral prototype baseline claims. The two answers are independent — the Metal profile can gain a measured realization while the baseline stays silent.

## Decision (2026-08-01)

Tom chose **option 2 — stay silent**, at the review that approved this queue: the baseline keeps `local-memory-bytes = 0` and declares no synchronization realization, because the prototype authority has no evidence for either claim and a declaration without evidence is exactly the capability overstatement the profile discipline refuses. The cooperative alternatives stay enumerated-and-rejected against the baseline — both rejections typed and test-driven — and admission waits for a profile with real authority (`construct-and-bind-the-first-authoritative-metal-compile-profile`, `realize-parallel-reduction-strategies-on-metal`). The test-only `workgroup_tree_target_for_test` remains the positive path's harness, which its own documentation already justifies.
