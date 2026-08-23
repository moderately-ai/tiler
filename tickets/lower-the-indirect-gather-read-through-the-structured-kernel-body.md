---
id: lower-the-indirect-gather-read-through-the-structured-kernel-body
title: Lower the indirect gather read through the structured kernel body
status: done
priority: p2
dependencies: [thread-resolved-lowering-into-the-governed-spelling-path]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, kernel-ir]
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
## Coordinator note — 2026-08-23: one contract paragraph expires when this lane lands

`restate-the-gather-standing-in-the-optimizer-contract-after-the-wall-retired` landed as `0b51531f` and repaired `docs/compiler/optimizer.md` to the standing true at that moment: the retired `GatherProofUnavailable`, the narrower `GatherIndexBoundsUnproved` for the undischarged population, a proved gather reaching `RegionSpellingKind::Gather`, and — as the reason a gather still does not compile end to end — the kernel-body wall **this ticket owns**.

That lane wrote the sentence **conditionally on purpose**, naming this ticket as in progress and saying the boundary "may itself move next", rather than asserting a permanent state. So this is a known expiry, not drift it left behind.

**When this ticket lands, that paragraph needs a further dated correction.** The scope is `contracts/optimizer`, which this ticket does **not** hold, so it is a follow-up to file at merge rather than something to reach for from this lane. This is the third link in that chain — `27fa3043`, then `0b51531f` — and each one flagged its own successor rather than leaving a stale sentence, which is the pattern to continue.

**Also verified by the coordinator at `db8ae185`, so you need not re-derive it:** `GATHER_KERNEL_BODY_RULE` is `"gather-kernel-body"` at `crates/tiler-compiler/src/pipeline/planning.rs`, and the classifier reports rather than refuses — it stops being reached once your body lands, which is the intended retirement path for it.

## FORCED HOLD — 2026-08-23: the work is complete and gated RED by design; do not merge yet

**Branch `tkt/lower-the-indirect-gather-read-through-the-structured-kernel-body`, commit `cd3d689a`, base `db8ae185`. Preserved and deliberately unmerged.** `worker-gatherbody` completed the lowering and told the coordinator not to merge. That instruction is correct and is being honoured.

**Why main must not take it yet.** One workspace test fails: `tiler-compiler request::tests::a_statically_proved_gather_is_declined_for_its_missing_kernel_body`, `left: None  right: Some(("kernel-lowering", "gather-kernel-body"))`. That is not a defect in the lowering — it is the **predicted** consequence of it. The classifier `pipeline::planning::kernel_lowering_failure` exists only to report the missing body; once the body lands its arm is unreachable, so `planning_capability_rule` answers `None` and the test asserting the classification fails. The gate is red **because the lane succeeded**.

**Why the lane did not fix it, and was right not to.** The fix lives in `implementation/compiler`, which this ticket does not declare, which the brief made a non-goal, and which was held at the time by a **live exclusive claim**. Reaching across a live claim to green a gate is exactly the shortcut that produces an unreviewed merge.

**RELEASE TRIGGER — merge this branch only after [`retire-the-gather-kernel-lowering-classification-after-the-body-landed`](retire-the-gather-kernel-lowering-classification-after-the-body-landed.md) lands, and gate the two together.** That ticket is `p1`, scoped `implementation/compiler`, depends on this one, and is filed **on this branch** rather than on `main` — so it becomes visible to the board only when this merges. The coordinator must therefore dispatch it from this branch's content or re-file it on `main`; it will not appear on the ready board by itself. `implementation/compiler` is currently held by `split-the-compiler-pipeline-test-monolith-by-orchestration-phase`.

**Known next wall, probed but not landed.** The lane patched the test to print, ran it, reverted, and confirmed a clean tree. The outcome after the classifier retires is `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`: `BoundedCarrier::of` in `crates/tiler-compiler/src/program.rs` materializes every boundary value at the program's arithmetic carrier, so a `tiler::u32@1` index input is declared `f32`. `tiler_ir::program::StorageScalar::U32` already exists — the missing half is the compiler's per-input carrier selection, not an IR carrier.

**Also outside that lane's scopes, for a coordinator pass:** `docs/roadmap.md`'s gather row still reads that `LogicalAccess` has no indirect relation and no lowering capability, which three landed commits have falsified.

### Sequencing, made explicit — the dependent lane must branch from `cd3d689a`, not from `main`

The release trigger above is correct but under-specified, and the under-specification is a trap. `retire-the-gather-kernel-lowering-classification-after-the-body-landed` declares `depends_on: [lower-the-indirect-gather-read-through-the-structured-kernel-body]`, and that ticket cannot reach `done` until its branch merges — which is precisely what the hold forbids. **So the dependent ticket will never surface on the ready board on its own.** Waiting for it to appear is waiting forever.

It is also not work that can be done on `main`: retiring a classifier whose arm is still reachable would make `main` refuse a gather it can no longer classify, turning a correct report into an absent one.

**The correct sequence, for whoever picks this up:**

1. Wait for `implementation/compiler` to free — currently held by `split-the-compiler-pipeline-test-monolith-by-orchestration-phase`.
2. Create the dependent lane's worktree from **`cd3d689a`** (this branch's tip), not from `main`, so the body is present and the classifier's arm is genuinely unreachable in its tree.
3. Land the classifier retirement there, over the body.
4. Merge the combined result into `main` **once**, and gate the merged tree. The expected baseline is 4060 workspace / 1350 release **plus** this lane's +5 new tests, minus whatever the retirement removes — derive it, do not assume it.

**Do not merge this branch alone at any point in that sequence.** The single red test is the whole reason the hold exists, and greening it by any route other than the dependent lane means editing a test to match code rather than fixing the code the test is about.
