---
id: carry-the-tree-participant-cap-as-a-target-profile-row
title: Carry the tree participant cap as a target profile row
status: todo
priority: p2
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, public-boundary]
---
## The gap between what the code does and what its own doc says it should

**Fact — the constant's doc says a second profile must not inherit it.** `MEASURED_TREE_PARTICIPANT_CAP` (`crates/tiler-compiler/src/physical.rs`) documents itself as a property of one host row — one Apple family, one OS row, one contract, one program family, `f32` — and states that "a second target profile should carry its own row rather than inherit this one".

**Fact — the code makes inheritance mandatory and silent.** `capped_tree_partition(contributors: u64)` takes no target. Its sole caller `single_workgroup_tree_region` receives a `VerifiedTargetRequest` that carries the profiles and does not consult it for the width. There is no typed row, no `Unknown` fallback, and nothing fails when a second profile is added: the Apple9 number governs it silently.

**Inference — this is the failure direction AGENTS.md names.** Hardware is to be modelled "through typed target profiles, properties, schedule alternatives, feasibility predicates, and cost models", and the compiler core is to stay independent of any one consumer's facts. Today the doc states the right architecture and the code states the opposite, and only the code runs. The defect is latent while one profile exists and becomes a wrong width the moment a second arrives.

**A smaller instance of the same leakage, worth fixing with it.** `capped_tree_partition`'s doc, in compiler core, asserts facts about a downstream crate's declaration and about the repository's whole profile inventory — that the measured row declares 32,768 bytes and that "no profile in this repository sits in the affected band". Both are true today and nothing fails if a profile in the band is added. Core documentation should not depend on a consumer-side inventory it cannot see.

## What this ticket owes

- The participant preference declared on the target profile, through the `declare_*` / `declare_measured_*` pair whose measured constructor carries a `TargetCompileProfileMeasurementSource`, so its validity stays `MeasuredEnvironment` and cannot widen into a portable claim.
- `BoundMetalCompileDeclaration::first_macos_apple9` declaring it from the retained 2026-08-07 calibration, citing that spike.
- **Silence meaning "no preference", never "no plan".** A profile that declares no row must select exactly as the balanced rule does, and must remain plannable. This is the same failure direction [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md) reasons about at length for its cost term: a preference row read as a hard bound would make silence unexecutable for a quantity no feasibility predicate consults.
- The identity consequence carried completely: moving a row moves the canonical descriptor and every pinned artifact identity and cache subject derived from it. Enumerate before editing, recompute on the merged tree, ledger each.

## Reserved

**This adds a `pub` `TargetProfileBuilder` declaration and moves an identity domain, so Tom accepts the exact surface** — the same two reservations that park `activate-measured-reduction-selection-from-a-target-cost-row`. Coordinate with that ticket rather than racing it: if both land, they should share one row-shaped answer rather than mint two kinds of preference row. If it lands first, this one likely becomes an instance of its mechanism rather than its own design.

## Explicit non-goals

No change to the cap's value or to the rule's shape — [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md) owns the rule. No selection change. No new measurement.

## Graph maintenance

Filed 2026-08-07 by the coordinator from the post-landing multi-lens audit of [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md), which recorded the constant's bound honestly in prose and could not build the seam within its scope.
