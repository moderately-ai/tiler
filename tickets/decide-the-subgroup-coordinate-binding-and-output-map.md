---
id: decide-the-subgroup-coordinate-binding-and-output-map
title: Decide the subgroup coordinate binding and output map
status: done
priority: p1
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [admit-subgroup-bindings-into-the-schedule-vocabulary, compose-the-two-level-subgroup-and-workgroup-reduction, admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir]
scopes: [implementation/ir, implementation/compiler, research/scheduling, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, subgroup, coordinates, ownership, public-boundary, decision]
---
## User-visible outcome

A subgroup schedule says which subgroup and lane own each output without assuming an unstated vendor relation to the workgroup's local linear index. The verifier can prove the one-writer rule and KIR can consume the same binding.

## Source-first gap — 2026-08-11

The accepted subgroup model requires `threads_per_workgroup` to be an exact multiple of subgroup width, not necessarily equal to it. When `T > W`, one workgroup contains several subgroups and each subgroup may own a different output. `SubgroupLane` alone identifies a lane only within its subgroup; it does not identify the subgroup.

Vendor contracts and ADR 0096 decline to define a portable relation from subgroup coordinates to `LocalLinearInvocation`. Current `LocalCoordinateSource` has `LocalLinearInvocation` and `LocalWorkgroupPosition`, is carried only by `LocalCoordinates` inside `CooperativeTile`, and is not consumed as a subgroup source by lowering. Adding a bare `SubgroupLane` variant would therefore be dead and could let cooperative lowering misread it as the local-linear source.

The ownership path also needs an explicit mapping: current schedule admission equates work items, grid threads, and iteration elements, while a subgroup reduction may have one owning write per subgroup. `result_lane` is a physical lane ordinal, not an `OwnershipWitnessId`, and cannot alone derive which subgroup owns which output.

## Decision options

1. **Required direct subgroup coordinate/output binding on `SubgroupTree` (recommended).** Carry an abstract subgroup index plus lane relation, encode it under the new topology, and make KIR use governed subgroup-index/lane builtins. Do not wrap it in workgroup `LocalCoordinates` or infer it from local-linear position.
2. **Narrow first slice to exactly one subgroup per workgroup (`T == W`).** Derive output ownership from workgroup/global geometry, admit only exact contributor coverage initially if desired, and file the multi-subgroup carrier as a mandatory successor. Strict and smaller, but does not implement ADR 0094's full multiple-of-width surface.
3. **Required `lane_source: LocalCoordinateSource` only.** Gives the lane a carrier but still cannot identify a subgroup when `T > W`; incomplete.
4. **Infer subgroup index/lane from local-linear invocation.** Rejected unless a target declares and preflight confirms that exact relation; it is not portable and cannot be a default.

## Required decision evidence

- Walk `T == W` and `T == 2W` examples through iteration/output ownership, launch geometry, schedule identity, KIR builtins, and one-writer verification.
- Show how a wrong subgroup index, wrong result lane, or cross-subgroup shuffle is typed-refused.
- Separate schedule meaning from target/prepared-pipeline availability. Unknown or mismatched coordinate authority refuses before routing commit; it never selects a different map or backend.
- Account for how the later two-level subgroup/workgroup topology reuses the coordinate without importing workgroup row-major semantics.

## Closes when

Tom accepts one exact required carrier or explicitly narrows the first subgroup slice to one subgroup per workgroup, with no uncarried enum variant, inferred relation, or silent fallback.

## Accepted decision — 2026-08-11

Tom accepted option 1 in this conversation: `SubgroupTree` carries one required direct coordinate/output binding with abstract sources for the workgroup ordinal, subgroup index within that workgroup, and lane index within that subgroup. The carrier is part of the topology's canonical identity and is consumed by governed KIR builtins; it is not wrapped in `LocalCoordinates`, projected from `LocalLinearInvocation`, or supplied by a backend convention.

The initial mapping is exact and derived:

```text
subgroups_per_workgroup = threads_per_workgroup / subgroup_width
output_ordinal = workgroup_ordinal * subgroups_per_workgroup + subgroup_index
writer = lane_index == result_lane
```

Intrinsic verification requires nonzero supported width, exact workgroup divisibility, complete participating subgroups, `result_lane < width`, one writer per output, and every shuffle source remaining inside the same subgroup. The implementation must additionally refuse any launch tail whose activity cannot prove complete subgroups; it may not infer a partial-workgroup rule from vendor behavior.

Target availability and schedule meaning stay separate. The target profile and prepared pipeline must provide the exact coordinate/width authority the verified schedule requires. Silence, an unknown source, or any mismatch refuses before routing commit and never retries another coordinate map, subgroup width, provider, or backend.

This decision adds constant-size schedule/KIR state and fixed checked arithmetic only. It makes no kernel-performance claim.

## Outcome

The remaining public decision is closed. Implementation stays in `admit-subgroup-bindings-into-the-schedule-vocabulary` and `admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir`, both still gated by their complete prerequisites.
