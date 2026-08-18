---
id: demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture
title: Demote the M3 Pro subgroup declaration to an internal evidence fixture
status: todo
priority: p2
dependencies: [declare-metal-subgroup-realization-facts-in-the-target-profile]
related: []
scopes: [implementation/build, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`BoundMetalSubgroupDeclaration` and its `tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1` key are crate-private inside `tiler-build`: the pub re-export is removed, the type and factory are `pub(crate)`, and no public identity vocabulary carries a hardware model name. The declaration's validation, tests, and the retained on-device demonstration continue unchanged.

## Why this exists

Tom's 2026-08-18 revised acceptance on `declare-metal-subgroup-realization-facts-in-the-target-profile`: the stage surface is accepted; the host-named public profile is not — host model names belong in measurement provenance, not identity vocabulary. No further host-named profile key may be minted pending `decide-the-host-evidence-to-profile-composition-model`.

## Closes when

The re-export is gone, external unreachability has compile evidence, all existing tests and the spike still pass (the spike may need a crate-internal driver path or a recorded exception), and the module docs state the demotion with its provenance.
