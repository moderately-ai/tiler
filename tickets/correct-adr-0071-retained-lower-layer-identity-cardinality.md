---
id: correct-adr-0071-retained-lower-layer-identity-cardinality
title: Correct ADR 0071's schedule-to-index-region retained-identity clause
status: todo
priority: p1
dependencies: []
related: [bind-the-scheduled-region-to-the-verified-index-region-identity, unify-schedule-index-region-with-verified-index-region]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, ir, identity]
---
`bind-the-scheduled-region-to-the-verified-index-region-identity` set out to build the missing half of ADR 0071's retained-identity clause and found the clause itself states a layering the compile path does not have.

**Fact — ADR 0071's Decision reads** that "each verified structural layer retains the exact identity of the lower structural layer it refines: schedule to index region and kernel to schedule". Its Implementation-boundary entry, "Partially realized clause — retained lower-layer identity", records the schedule half as unrealized and names that ticket as owning it.

**Fact — the relation is not one-to-one.** `crates/tiler-compiler/src/physical.rs` builds every `ScheduledRegion` by struct literal from a `VerifiedTargetRequest`; `pointwise_region`, `reduction_region`, and `fused_region` each return `(ScheduledRegion, Vec<SemanticMemberId>)`, and a fused region covers several members. `crates/tiler-compiler/src/legality.rs::emit_region` separately returns one `VerifiedIndexRegion` **per semantic occurrence**. One scheduled region therefore stands over several verified index regions, and a single retained `CanonicalIndexRegionIdentity` could only name one of them or none.

**Fact — the two types are also not two views of one layer.** `schedule::ScalarProgram` is a closed three-variant enum of `f32` bit-pattern records; `index`'s scalar program is an open registry-governed SSA graph. Neither is a subset of the other: `schedule::TensorRole` has an `Intermediate` variant `index::TensorRole` lacks, and the schedule carries a `NumericalRealization` the index layer has no field for.

## Scope

Correct the Decision clause so it states the relation the architecture actually has, and rewrite the boundary entry so it stops recording a debt that cannot be paid in the shape it describes. The substantive claim to preserve is that a verified product must name the verified evidence it rests on; what changes is where that binding lives and its cardinality. `bind-stage-coverage-to-index-refinement-identity` proposes the kernel program's stage coverage as that place, and this record should either endorse that or say what else would satisfy the clause.

Do not simply delete the clause. Today `CanonicalIndexRegionIdentity` reaches no verified product's identity at all — it is derived in `emit_region`, carried on `IndexRefinement`, and consumed only by `pipeline.rs::refinement_label` to format an `EXPLAIN` string — so the index layer's verifier and identity derivation contribute to explain output and nothing else. That is a real gap; the clause named it in the wrong place.

## Closes when

ADR 0071's Decision and its boundary entry agree with the implemented layering, the surviving obligation is stated with its correct cardinality and owner, superseded text is marked rather than silently rewritten, and `uv run --locked python scripts/docs.py render` plus `uv run --locked python scripts/check_repository.py` pass.
