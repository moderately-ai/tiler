---
id: lower-the-indirect-gather-read-through-the-structured-kernel-body
title: Lower the indirect gather read through the structured kernel body
status: in-progress
priority: p2
dependencies: [thread-resolved-lowering-into-the-governed-spelling-path]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, kernel-ir]
claimed_from: todo
assignee: worker-gatherbody
lease_expires_at: 1787474269
---
## User-visible outcome

A verified scheduled region carrying `LogicalAccess::GatherSource` lowers to a `VerifiedKernel` whose body loads the address its index operand holds and reads the source at the coordinate that address names, so a statically proved gather reaches a kernel program instead of stopping at `body-refinement`.

## Why this exists

Filed 2026-08-23 from [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md), which made this refusal reachable for the first time. Before that lane a gather stopped at `RegionVocabularyWall::GatherProofUnavailable` and no gather region was ever built; that wall retired, `crate::physical::gather_region` now spells one, and the next authority with nothing to say about it is the structured kernel.

**Fact — the refusal is deliberate, named, and its own source says so.** `crates/tiler-ir/src/kernel/lower.rs` refuses the relation at the anchor `LogicalAccess::GatherSource { .. } => Err(KernelDiagnostic::BodyRefinement)`. Its comment states the gap positively and anticipates exactly this situation: `has no indirect kernel or backend route`, because `emitting one needs an address` load inside the body and no `ReadAddressing` form has one. It also states why a neighbouring relation must not be substituted — `LinearIdentity` would read the source at the invocation index and `BroadcastReplication` at a derived static coordinate, and both `return a wrong element silently`.

**Fact — the population is reachable and cheap, not exotic.** Only a *statically proved* gather is spelled, and both closed arguments are reachable. The inhabited one needs a `2^32` gathered extent, but the vacuous one needs only an empty result domain: `gather_program_over([4, 0], [2], 0)` in `crates/tiler-compiler/src/request/tests.rs` is a tiny legal program whose obligation is discharged vacuously and which reaches this boundary. `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` pins it.

**Fact — the compiler's classification of the refusal is a stopgap, not the fix.** `PhysicalError::Refinement`'s ordinary class is `InvalidCompilerOutput`, which for this population would be a false claim: the governed builder emitted exactly the region the schedule layer admits. `kernel_lowering_failure` in `crates/tiler-compiler/src/pipeline/planning.rs` therefore reports `("kernel-lowering", "gather-kernel-body")` as a missing capability instead. That function classifies a refusal already taken and never takes one, so it stops being reached the moment this ticket lands — but it should be **removed** by the lane that lands the body, not left standing over an unreachable case.

**Inference — this is upstream of the Metal emission.** [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) declares `implementation/metal` and `implementation/compiler` and depends on the threading lane; it does not declare `implementation/ir`, and nothing in the ticket graph owns `ReadAddressing`. A backend cannot emit a construct for a kernel body that does not exist, so this ticket is that ticket's prerequisite.

## Required work

- Re-audit every Fact above at your own base before editing; the anchors are quoted from source rather than from a rendered view.
- Admit an address-loading `ReadAddressing` form, or state with evidence why the existing vocabulary cannot carry one and what should replace it.
- Decide whether the refusal for a gather this build still cannot emit keeps `BodyRefinement` or moves to the `Unlowered*` class its siblings use — `UnloweredRegionProgram`'s own doc calls that separation "representable, not lowered", and `LogicalAccess::PartitionedCopySource` one line below the gather arm already takes it.
- Remove `kernel_lowering_failure`'s gather arm and `GATHER_KERNEL_BODY_RULE` once the body lands, and delete or invert `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` rather than leaving it asserting a refusal that no longer happens.
- Perturb each behaviour on its own subject and quote the failure text. A body that reads the source at the wrong coordinate is the failure mode this must discriminate, so a control that only shows *a* kernel was produced is not evidence.
- State every identity domain that steps. A kernel body change moves kernel-program identity and every artifact identity folding it.

## Non-goals

Scatter. The Metal emission. Invocation-scoped index validation — an undischarged gather never reaches this layer, because the region vocabulary declines it at `RegionVocabularyWall::GatherIndexBoundsUnproved`.

## Fact audit at `db8ae185b43c8b4c23bd5a29512e0b774de93432` (worker-gatherbody, 2026-08-23)

Every Fact above was re-read at the dispatched base before any edit. Each verdict names the command that produced it.

**Fact 1 — the refusal is deliberate, named, and its own source says so: verified, two anchors imprecise.** `git show db8ae185:crates/tiler-ir/src/kernel/lower.rs | grep -c -F '<anchor>'` returns **1** for `LogicalAccess::GatherSource { .. } => Err(KernelDiagnostic::BodyRefinement)` (line 930) and **1** for `has no indirect kernel or backend route`. It returns **0** for `emitting one needs an address` and **0** for `return a wrong element silently`. Both zeroes are the wrapped-comment cause `AGENTS.md` records, not an absent claim: the source breaks `emitting one needs an` / `address *load* inside the body` and `both return a` / `wrong element silently` across two `//` lines. The claims are true; the anchors were quoted across a line break. Shorter anchors that resolve: `emitting one needs an` and `wrong element silently`.

**Fact 1's supporting clause — "`PartitionedCopySource` one line below the gather arm": imprecise.** It is the next *arm*, at line 934, with three comment lines between. Nothing rests on the distance.

**Fact 2 — the population is reachable and cheap: verified, and reproduced.** `cargo nextest run -p tiler-compiler a_statically_proved_gather_is_declined_for_its_missing_kernel_body` at the base reports `1 test run: 1 passed`, so `gather_program_over([4, 0], [2], 0)` really does discharge its obligation vacuously, is spelled as a gather region, and reaches the `body-refinement` wall. `git show db8ae185:crates/tiler-compiler/src/request/tests.rs | grep -c -F 'gather_program_over([4, 0], [2], 0)'` returns **1**.

**Fact 3 — the compiler's classification is a stopgap: verified in substance, false in its scheduling clause.** `GATHER_KERNEL_BODY_RULE` occurs 3 times and `gather-kernel-body` once in `crates/tiler-compiler/src/pipeline/planning.rs` at the base, and the classifier takes no refusal of its own. The clause "it should be **removed** by the lane that lands the body" is **not executable by this lane** and is repaired rather than worked around: `implementation/compiler` is not among this ticket's scopes, and at dispatch it was held by a live exclusive claim (`re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement`, `worker-fusionrole`). The removal, the fixture's disposition, and the wall now behind it are [`retire-the-gather-kernel-lowering-classification-after-the-body-landed`](retire-the-gather-kernel-lowering-classification-after-the-body-landed.md).

## Outcome (worker-gatherbody, 2026-08-23)

**What landed.** `crates/tiler-ir/src/kernel/lower.rs` gains two `ReadAddressing` forms — `Gather(Box<GatherAddressing>)` for the data-dependent read and the fieldless `GatherAddress` for the operand that supplies its coordinates — plus `gather_direct_terms`, `gather_address_addressing`, `emit_gather_offset`, and a shared `emit_pointwise_loads`. The emitted address is `direct + coordinate * gathered_stride`, where `coordinate` is the U32 the invocation loads from the index operand and widens through the new `ConvertOp::U32ToIndex`. `crates/tiler-ir/src/kernel/verify.rs` gains `gather_address_reads`, the single derivation of which reads carry addresses, read by both the lowering's buffer declaration and `verify_signature`'s expected types.

**The refusal class stays `BodyRefinement`, and the `Unlowered*` alternative the ticket named is now a false claim.** `Unlowered*` says "representable, not lowered", which each of its two members backs with a *separate accepted boundary* owning the missing body. After this landing there is no such boundary for a gather: every `GatherSource` relation a `VerifiedScheduledRegion` can carry lowers, because `gather_address_addressing` admits all three relations `gather_index_read_map` derives. What remains under `BodyRefinement` are backstops for a self-inconsistent relation — a result shape that is not the composition, an address operand carrying some other map, an ordinal out of range — which is exactly "this region is not the canonical body I derived", and which the schedule verifier already refuses before lowering, so none is reachable through the public API.

**Identity.** `tiler.kernel.v9` does **not** step: `ConvertOp::U32ToIndex` takes tag `0x05` by appending, every earlier tag and field position is byte-identical, and no kernel the earlier vocabulary could express contains `0x05` in that position. Nothing else steps either — no new domain literal, so `crates/tiler-ir/src/domains.rs` is untouched; `KernelType::U32` and `StorageScalar::U32` already carried tags `0x07` and `0x04`; `tiler-artifact` encodes no `ConvertOp`; `tiler.schedule.v7`, `tiler.index-region.v11`, and `tiler.kernel-program.v13` see no vocabulary change. Verified by the whole `tiler-ir` and workspace suites, which carry the identity goldens.

**What is still open.** The vacuously proved fixture now reaches `StageElementType { position: 1, expected: U32, actual: F32 }` in kernel-program assembly, because `tiler-compiler` materializes every boundary value at the program's arithmetic carrier. That, the classifier removal, and the fixture's disposition are the follow-up ticket above; `emit-the-indirect-gather-on-metal` remains behind both.
