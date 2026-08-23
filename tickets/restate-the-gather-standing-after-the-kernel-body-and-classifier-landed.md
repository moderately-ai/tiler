---
id: restate-the-gather-standing-after-the-kernel-body-and-classifier-landed
title: Restate the gather standing after the kernel body and classifier landed
status: done
priority: p2
dependencies: []
related: [retire-the-gather-kernel-lowering-classification-after-the-body-landed, lower-the-indirect-gather-read-through-the-structured-kernel-body, restate-the-gather-standing-in-the-optimizer-contract-after-the-wall-retired, route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type, emit-the-indirect-gather-on-metal]
scopes: [contracts/optimizer, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gather, contract, roadmap]
---
## User-visible outcome

The optimizer contract and the roadmap's gather row state the gather standing that is true after the kernel body landed, instead of two sentences that three landed commits have falsified.

## Why this exists

Filed 2026-08-23 by `worker-retireclass` from [`retire-the-gather-kernel-lowering-classification-after-the-body-landed`](retire-the-gather-kernel-lowering-classification-after-the-body-landed.md), which retired the classifier, and behind [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md), which emitted the body. Neither lane declares `contracts/optimizer` or `contracts/navigation`, so neither could repair the prose its own landing falsified. This is the third link in the chain [`restate-the-gather-standing-in-the-optimizer-contract-after-the-wall-retired`](restate-the-gather-standing-in-the-optimizer-contract-after-the-wall-retired.md) opened at `0b51531f` after `27fa3043`, and that lane wrote its sentence **conditionally on purpose**, naming the kernel-body ticket as in progress and saying the boundary "may itself move next". So this is a known expiry, not drift left behind.

Both documents are held together in one ticket because one landing falsified both, and splitting them would put two lanes on the same three commits' consequences.

**Fact — the optimizer contract's gather paragraph names a wall and a classifier that no longer exist.** `docs/compiler/optimizer.md` carries the anchor `gather is still not compilable end to end, for a different and later reason`, and the sentence continues that `crates/tiler-ir/src/kernel/lower.rs` answers `Err(KernelDiagnostic::BodyRefinement)` for `LogicalAccess::GatherSource` *unconditionally* and that `pipeline::planning::kernel_lowering_failure` classifies that refusal as `("kernel-lowering", "gather-kernel-body")` — the anchor `rather than invalid compiler output` ends it. All three clauses are now false: the body lowers, the unconditional refusal is gone, and the classifier and its rule constant were deleted. The paragraph also carries the anchor `owns that wall and was in progress when this correction was written`, which is the conditional flag its author left for this repair.

**Fact — the wall is still real, one layer further down, so the repair is not "a gather compiles".** A statically proved gather now clears kernel lowering and stops at program assembly: `gather_program_over([4, 0], [2], 0)` reports `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`, because `crates/tiler-compiler/src/program.rs`'s `BoundedCarrier::of` materializes every boundary value at the program's arithmetic carrier and so declares a `tiler::u32@1` index input as `f32`. [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md) owns that wall. `a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier` in `crates/tiler-compiler/src/request/tests.rs` pins it. **Repair to this, not to "a gather is supported"** — the standing moved one wall down, it did not disappear.

**Fact — the roadmap's gather row is falsified in the same direction and is a separate document.** `docs/roadmap.md`'s indirect-gather row carries the anchor `has no indirect relation, so the family has no realization law`, claiming `LogicalAccess` has no indirect relation and the family therefore has no realization law, lowering capability, fusion role, or executable plan. Three landings falsify it: `lower-a-recognized-gather-through-a-governed-capability` added the `GovernedGatherF32` lowering row, `thread-resolved-lowering-into-the-governed-spelling-path` made a proved gather resolve `RegionSpellingKind::Gather`, and the kernel-body lane added the `ReadAddressing::Gather` relation and its body. The same row's rung condition, anchor `R6 additionally needs a U32 storage carrier and backend realization`, also needs re-reading: the U32 storage carrier landed under [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md), so what remains for that rung is the compiler's per-input carrier selection and the backend, not the carrier vocabulary.

## Required work

- Re-audit every Fact above at your own base before editing; the anchors are quoted from source rather than from a rendered view, and each is scoped to the file its citation names.
- Repair the optimizer paragraph to the standing true at your base, and write it **conditionally** if a named ticket still owns the next wall — that is the convention this chain has followed twice and it is what kept each expiry visible rather than silent.
- Repair the roadmap row's claim and re-derive its rung, rather than editing the prose and leaving the rung letter alone.
- Follow the correction convention already in both documents: withdraw the retired wording in a dated note rather than restating it silently. Note that a `make citations` or grep census over these files **cannot shrink** across this repair, because the convention quotes the retired sentence verbatim inside the correction — expecting the count to fall is a false progress signal.
- Read both documents in full before declaring the sweep complete. Applying a landing means aligning catalogs, contracts, and terminology, not one paragraph.

## Non-goals

The per-input carrier work itself. The Metal emission. Any claim that a gather compiles end to end, which is false at the time of filing and must be re-derived rather than assumed either way.

## Coordinator correction — 2026-08-23: my brief's statement about `lower.rs` was imprecise, and the lane sharpened it

My brief told this lane that `crates/tiler-ir/src/kernel/lower.rs` "no longer refuses `LogicalAccess::GatherSource`". Retired wording preserved. **That is imprecise, and the lane was right to narrow it.** Verified by the coordinator at `a61995c7`: the file carries **two** `GatherSource` sites. Line 1007 is the new lowering arm, which returns `Ok(ReadAddressing::Gather(..))`. Line 1081 still returns `Err(KernelDiagnostic::BodyRefinement)` — but it sits inside `fn gather_address_addressing`, which refuses a `GatherSource` used as a **nested index-operand address**, i.e. a gather whose own index is itself gathered. That is a different construct and a deliberate refusal, not a leftover of the retired wall.

So the accurate statement is: a gather is no longer refused **as the read being lowered**, while a gather-of-a-gather **address** is still refused by name. A reader who took my wording literally and grepped for `GatherSource.*Err` would have found a hit and concluded the wall survived.

**Two further false claims the lane found by sweeping beyond the brief's named Facts, both verified by the coordinator.** `docs/roadmap.md` said ADR 0108 "remains proposed" — its frontmatter reads `decision_status: "accepted"`. And it said `AccessData` "carries one tensor ordinal", when `AccessData::GatherRead(GatherReadAccessData)` carries `source` and `index` plus axis, domain, and coordinate fields. The lane repaired both rather than leaving them for a fourth ticket, which was the right call: they were in scope, demonstrably false, and found by the full-document sweep the brief mandated.
