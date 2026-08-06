---
id: move-the-elementary-activation-row-to-r6
title: Move the elementary activation row to R6 and restate its golden-count claims
status: in-progress
priority: p2
dependencies: []
related: [compile-an-elementary-function-golden-through-the-metal-toolchain, move-the-structural-row-to-r6-and-retire-its-backend-residual, re-read-the-bf16-and-elementary-support-rows-against-source]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, support-matrix, metal]
claimed_from: todo
assignee: agent-navigation-4
lease_expires_at: 1786036406
---
## The work

`docs/roadmap.md:469` (`Elementwise activation: tiler::silu-f32@1`) holds the row at **R5**, and states its own R6 criteria in the trigger cell: "a translation unit carrying this family's exponential and division observed accepted by a declared numerical realization, and compiled and linked through the Apple toolchain the way every other R6 row in this table was", owned by [`compile-an-elementary-function-golden-through-the-metal-toolchain`](compile-an-elementary-function-golden-through-the-metal-toolchain.md). Both conjuncts landed on that ticket's branch; `docs/roadmap.md` is `contracts/navigation`, which that ticket does not hold, so the cell was left untouched and the move is filed here rather than claimed there.

Three edits, and the second is the one a sweep would otherwise miss:

1. **The rung cell** moves `R5` → `R6`, bounded exactly as the structural row's precedent bounds it ([`move-the-structural-row-to-r6-and-retire-its-backend-residual`](move-the-structural-row-to-r6-and-retire-its-backend-residual.md)): a *translate-and-link* fact on one measured toolchain row, not a dispatch, and **not** the [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s row — the goldens compile under `metalfe-32023.921`, which that ledger excludes by name. R7 stays unmet: no dispatched device comparison, and no compiler-derived region through `emit`.
2. **Two embedded reproduction commands in the same cell are now false and must be restated, not deleted.** The cell reads "`grep -rn 'precise::' crates/tiler-metal/goldens/` returns nothing, and `golden_compilation.rs` names nine goldens, none of them carrying an exponential or a division". After the landing the grep returns hits in `crates/tiler-metal/goldens/elementary_silu_activation.metal` and the list names **ten**. The derivation those sentences carry is what the R5 hold was accepted on, so correct them in tense the way the structural row's span was corrected, rather than removing the reasoning.
3. **Sweep the navigation scope for every other claim about this row's rung**, the way the structural move found two stale cross-references it had created. `docs/roadmap.md:555` and `docs/status.md` are the two known candidates; neither has been read for this row.

## Evidence to state in the cell

Read from the landing rather than restated from memory:

- The golden is `crates/tiler-metal/goldens/elementary_silu_activation.metal`, emitted from the `silu_kernel` fixture, links **3,779 bytes**, and the linked library names `tiler_kernel_b1e08c4feb69be47`.
- Toolchain row, recorded from the host at run time: Apple M4 Max, macOS 27.0 (build 26A5388g), Xcode 27.0 (build 27A5228h), Metal 32023.921 (`metalfe-32023.921`, AIR-LLD 32023.921), macOS SDK 27.0 (build 26A5388f), flags `-target air64-apple-macos14.0 -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`.
- The acceptance half is `crate::tests::the_silu_kernel_records_the_f32_subnormal_gap`, now two-sided: the strict declaration is refused on the flushing row and a flush-honouring declaration over the same SiLU kernel yields an empty gap set and a successful `require_declared_realization`, with the accepted unit still carrying `precise::exp` and the precise-selection requirement.
- The intrinsic-selection measurement is new and worth the cell's space, because it is what the row's "never `air.fast_exp.f32`" claim rested on as a *source* assertion: `the_precise_namespace_survives_a_fast_row_when_a_toolchain_resolves` reads the intrinsic out of the linked library. Under the governed flags both spellings link byte-identically to `air.exp.f32`; under a fast row `precise::exp` still links `air.exp.f32` while unqualified `exp` links `air.fast_exp.f32`.

## Closes when

The rung cell states R6 with its measurement boundary, both embedded reproduction commands return what the text claims, no other navigation site still asserts the row is held at R5 by a compiled-golden residual, and R7's two unmet conjuncts are stated.

## Scope note

`project/tickets` is in `shared_scopes` because every claimed ticket declares it; the guard does not treat a ticket's own file as implicitly shared. Declaration and scheduling metadata for already-authorized work; no product scope moved.
