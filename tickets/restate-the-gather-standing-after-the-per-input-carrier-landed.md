---
id: restate-the-gather-standing-after-the-per-input-carrier-landed
title: Restate the gather standing after the per-input carrier landed
status: in-progress
priority: p2
dependencies: []
related: [route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type, restate-the-gather-standing-after-the-kernel-body-and-classifier-landed, emit-the-indirect-gather-on-metal]
scopes: [contracts/optimizer, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gather, contract, roadmap]
claimed_from: todo
assignee: worker-optrestate3
lease_expires_at: 1787484981
---
## User-visible outcome

`docs/compiler/optimizer.md` and `docs/roadmap.md` state the gather standing that is true after a program input's storage carrier is routed from its own resolved value type, instead of the program-assembly wall that landing removed.

## Why this exists

Filed 2026-08-23 by `worker-carrier` from [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md), which holds `implementation/compiler` and `project/tickets` and neither of the two documentation scopes these files sit in. It is the fourth link in the chain [`restate-the-gather-standing-after-the-kernel-body-and-classifier-landed`](restate-the-gather-standing-after-the-kernel-body-and-classifier-landed.md) is the third of: each gather landing has moved the wall the two documents describe, and each time the previous restatement became false rather than merely incomplete.

**Fact — `docs/compiler/optimizer.md` states the removed wall as live, measured at `46184f8c`.** The paragraph opening at the anchor `A statically proved gather now clears kernel lowering and stops one layer further down` states that the compile `stops inside `build_plan_program``, that `BoundedCarrier::of` materializes every declared value at the program's arithmetic carrier, and that `gather_program_over([4, 0], [2], 0)` reports `InvalidCompilerOutput(Program(CoreConstruction(StageElementType`. All three are false after the carrier landing: `crate::pipeline::compile` returns `Ok`, the input's carrier comes from `BoundedCarrier::of_input`, and the fixture assembles a verified kernel program whose `index` boundary is `StorageScalar::U32` / `KernelType::U32`.

**Fact — the same paragraph names a test that no longer exists.** It cites `a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier`; the carrier landing renamed it to `a_statically_proved_gather_compiles_with_its_index_at_its_own_carrier`. This is a **name in prose, not a markdown link**, so `make citations` cannot catch it — the citation gate resolves links, and a stale identifier inside backticks resolves to nothing and is checked by nobody.

**Fact — `docs/roadmap.md`'s indirect-gather row names this as an open R6 prerequisite.** Its rung cell says at the anchor `R6 needs the compiler's own per-input storage-carrier selection` that this ticket's parent is in progress on it. That half is now done; what remains for R6 is backend realization, which [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) owns, and the row's R5 cell about `UNPLANNED_OPERATIONS` and the missing fusion role is unaffected and should be preserved.

## Required work

- Re-audit every Fact above at your own base before editing; the carrier landing's commit is the base these were measured against and the documents may have moved again.
- Restate both documents to the standing that is true after the landing, correcting rather than rewriting: the repository convention is a dated correction that quotes the retired sentence, so the withdrawn text stays greppable.
- Repair the retired test name in `docs/compiler/optimizer.md`. Check for other prose references to it — `grep -rn a_statically_proved_gather_clears_kernel_lowering docs/ tickets/` — and decide per site whether the name is a live citation or a historical record.
- Say what the gather's remaining walls actually are, rather than that it "compiles": the Metal emission still refuses `KernelType::U32` at `msl_type`, and `crates/tiler-compiler/src/policy.rs` still lists `tiler::gather-f32@1` in `UNPLANNED_OPERATIONS` with no fusion role. Read both before writing either.

## Non-goals

`crates/`. The Metal emission itself. Re-opening the accepted data-dependent index surface. The R5 fusion-role work, which is its own ticket.
