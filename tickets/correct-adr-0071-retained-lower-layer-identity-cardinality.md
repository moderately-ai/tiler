---
id: correct-adr-0071-retained-lower-layer-identity-cardinality
title: Correct ADR 0071's schedule-to-index-region retained-identity clause
status: done
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

## Outcome

ADR 0071's retained-identity Decision clause is amended, its "Partially realized clause — retained lower-layer identity" boundary entry is replaced, its status line records the second amendment beside the artifact-decoding one, the boundary preamble's "one marked exception" becomes two, and the `implementation_status` paragraph now names the relocated obligation instead of a missing edge. `decision_status` stays `accepted`. The superseded sentence is quoted verbatim at the clause rather than silently rewritten, as `AGENTS.md` requires of a durable decision.

### Verified before amending, by reading rather than inferring

Every claim this ticket inherited was checked against source at `63b02ec`, because it is a claim about an accepted decision.

**Confirmed — the cardinality is wrong in the clause.** `crates/tiler-compiler/src/physical.rs` builds each `ScheduledRegion` by struct literal from a `VerifiedTargetRequest`; `pointwise_region` returns `(region, request.serial_sum().members.pointwise().to_vec())` and `reduction_region` and `fused_region` have the same shape, so a region's member set is a slice and a fused region covers several of them. `crates/tiler-compiler/src/lowering.rs::resolve_lowering` loops over `request.serial_sum().members.all()` and pushes one `OccurrenceLowering` per member, each driving `legality.rs::emit_region`, which returns one `VerifiedIndexRegion`. One scheduled region therefore stands over N verified index regions and a single retained identity could name one of them.

**Confirmed — the two types are not two views of one layer.** `schedule::ScalarProgram` has exactly three variants, `MultiplyThenAdd`, `StrictSerialSum`, and `FusedMultiplyAddSerialSum`, all `f32` bit-pattern records. The index layer stores `operations: Vec<ScalarOperationData>` and `values: Vec<ScalarValueData>` on `VerifiedIndexRegionData`, verified against a `FrozenScalarRegistry` — an open registry-governed SSA graph. `schedule::TensorRole` is `Input | Intermediate | Output`; `index::TensorRole` is `Input | Output`. The schedule's `IndexRegion` literal sets `numerical: request.numerical_contract().realization()`; `grep -n numerical crates/tiler-ir/src/index/model.rs` returns nothing.

**Confirmed — the schedule module reaches no verified index region.** `grep -rn 'VerifiedIndexRegion' crates/tiler-compiler/src/` matches only `legality.rs` and `capability.rs`, never `physical.rs`.

**Confirmed — the obligation belongs at the program stage.** `crates/tiler-ir/src/program/model.rs`'s `StageData` carries `coverage: Vec<SemanticOccurrence>`, already one-to-many, and `SemanticOccurrence` there is a bare graph-local ordinal newtype over `u32`. `CanonicalKernelProgramIdentity`'s documentation states it folds the semantic graph identity, each stage's `CanonicalKernelIdentity` (which folds the canonical scheduled region), and the coverage those stages claim — so the schedule → kernel → program-stage chain is retained end to end and the index layer sits outside it.

### Retracted and sharpened: one inherited claim was overstated

This ticket, and `bind-the-scheduled-region-to-the-verified-index-region-identity`'s outcome before it, state that `CanonicalIndexRegionIdentity` is "consumed only by `pipeline.rs::refinement_label` to format an `EXPLAIN` string". **That is not accurate and the difference matters to whoever builds the fix.** Inside `crates/tiler-compiler/src/legality.rs` the identity is already folded into two complete compiler-owned identities: `encode_content_identity` writes `region_identity.as_bytes()` as the first field of `RefinementContentIdentity`, and `encode_occurrence_identity` folds that into `IndexRefinementIdentity`; `tiler_ir::index::ScalarAuthorityEvidence` separately binds its region-bound receipt to the same bytes.

The load-bearing claim survives in a stronger and more precise form, which is what the ADR now states: **the chain terminates inside `legality.rs`.** `pipeline.rs` retains the `IndexRefinement` in `CompletePlans` and consumes it in exactly two places, both explain — `record_refinement`, which renders eight trailing identity bytes through `refinement_label` as a presentation handle, and `record_numerical_equivalence`, which reads only `OccurrenceLowering::provider`. What the artifact plan records is `ResolvedLowering::providers()`, deduplicated lowering *provenance*, and no region or refinement identity. The exact absence check is `grep -rn 'CanonicalIndexRegionIdentity' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel crates/tiler-ir/src/program crates/tiler-artifact/src crates/tiler-metal/src`, which returns nothing.

So the work ahead is to carry an identity that already exists and is already complete into a verified product, not to derive one. `bind-stage-coverage-to-index-refinement-identity` carried the same overstatement and is corrected in place, so its worker does not inherit it.

### One thing this deliberately does not do

It does not contradict [the IR contract](../docs/ir.md) or [the fusion and scheduling contract](../docs/compiler/fusion-and-scheduling.md), both of which say a `ScheduledRegion` pairs one canonical `IndexRegion`. That is true of the `IndexRegion` the schedule module declares itself. The amended ADR says so explicitly, because a reader comparing the two documents would otherwise reasonably conclude one of them is wrong. The duplication between the two `IndexRegion` representations is a separate defect owned by `unify-schedule-index-region-with-verified-index-region`, and this record neither resolves it nor pre-empts it.

### Owner change

`bind-the-scheduled-region-to-the-verified-index-region-identity` is no longer named as the owner of any ADR 0071 clause; `bind-stage-coverage-to-index-refinement-identity` is the sole owner of the surviving obligation and is released by this ticket. Its coverage-type change is a public boundary in `tiler_ir::program`, so its exact signature stays Tom's under ADR 0075 — the ADR says so rather than leaving it to be discovered at review.

### Gate

`uv run --locked python scripts/docs.py render` and the full `uv run --locked python scripts/check_repository.py` both pass; `git diff --check` is clean and `tkt lint` reports no problems.
