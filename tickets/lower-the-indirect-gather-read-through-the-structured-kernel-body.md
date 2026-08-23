---
id: lower-the-indirect-gather-read-through-the-structured-kernel-body
title: Lower the indirect gather read through the structured kernel body
status: review
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
