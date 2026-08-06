---
id: move-the-elementary-activation-row-to-r6
title: Move the elementary activation row to R6 and restate its golden-count claims
status: done
priority: p2
dependencies: []
related: [compile-an-elementary-function-golden-through-the-metal-toolchain, move-the-structural-row-to-r6-and-retire-its-backend-residual, re-read-the-bf16-and-elementary-support-rows-against-source]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, support-matrix, metal]
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

## Outcome

**The row this moves and how far.** `docs/roadmap.md`'s [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix), row `Elementwise activation: tiler::silu-f32@1`, moves **R5 → R6**, bounded to offline translation and linking on one measured toolchain row that is not the compile-profile authority ledger's, with R7 unmet. It moves no other row and no dtype-ledger cell. Both R6 conjuncts were verified at source on this branch's base rather than taken from the landing's report:

- **Compiled and linked.** `crates/tiler-metal/src/golden_compilation.rs`'s module ledger records `elementary_silu_activation.metal` linking 3,779 bytes and naming `tiler_kernel_b1e08c4feb69be47` on Apple M4 Max / macOS 27.0 `26A5388g` / Xcode 27.0 `27A5228h` / Metal 32023.921 / SDK 27.0 `26A5388f` under the strict flag row. The call-level perturbation `precise::exp()` is pinned failing at `CompileStage::Metal` by `the_elementary_golden_without_its_operand_is_rejected_when_a_toolchain_resolves`.
- **Accepted by a declared realization.** `the_silu_kernel_records_the_f32_subnormal_gap` (`crates/tiler-metal/src/tests.rs:1904`) is two-sided: the strict declaration refuses, and the same `silu_expression()` at the same shape under a flush-honouring declaration yields an empty gap set, an empty `unstated_subnormal_arithmetic()`, a successful `require_declared_realization()`, one `precise::exp(`, and a retained `PreciseFp32Functions` requirement.

**Stated boundary, so the rung is not read past.** The compiled unit is `silu_kernel()`, the *strict*-declaration emission; the accepted unit is the flush-honouring one. They are the same expression at the same shape and differ only in the declared realization, so the two conjuncts are observations over one construct rather than over one artifact — stated here because the criterion does not require one unit and a reader would otherwise assume it.

**The two false reproduction commands, restated with their verified outputs.** `grep -rn 'precise::' crates/tiler-metal/goldens/` returned nothing; it now returns exactly one hit, `crates/tiler-metal/goldens/elementary_silu_activation.metal:54`. `golden_compilation.rs` named nine goldens; it now names ten (`const GOLDENS: [(&str, &str); 10]` at `:210`, and `ls -1 crates/tiler-metal/goldens/*.metal | wc -l` returns 10, kept equal by `every_checked_in_golden_is_compiled_by_this_module`). Both are corrected in tense inside the superseded span rather than deleted.

**Three further stale citations found and corrected in the same cell, beyond the ticket's three edits.**

1. The two `crates/tiler-metal/src/tests.rs` line citations moved with the landing's additions — the emission probe `:1810` → `:1832`, the realization probe `:1872` → `:1904` — and are now named by test rather than by line, the form that does not go stale.
2. The explain request-qualifier reading was `689c3aefc30f48d3` at `explain.rs:4183`; the pin moved twice more (→ `8966151e455093ea` at the output-arity budget step, → `ce6f9106c1c5933b` when `tiler.scalar::rsqrt-f32@1` entered the standard scalar profile) and the line moved with its ledger comment. Restated as `ce6f9106c1c5933b` with `grep -n 'tiler-explain-v7 request=' crates/tiler-compiler/src/explain.rs` as the line-free citation, naming the ledger comment as the authority because a live law- or key-registering landing moves it again.
3. The row said `emit.rs` emits `precise::exp` "never `air.fast_exp.f32`" — a source-text claim written in terms of a linked AIR intrinsic the emitter cannot choose. Corrected to `fast::exp` at the emission site, with the intrinsic question moved to the new measurement, which also states the bound the ticket asked for: under the governed flags the two spellings link byte-identically, so no Tiler-governed compilation distinguishes them and the namespace defends the selection only on a non-governed row.

**Cross-reference sweep — all twelve `contracts/navigation` files read for this row.** Two stale sites found, both in `docs/roadmap.md`, and both corrected:

- `docs/roadmap.md:435` listed SiLU under "semantic and reference admission with **no backend at all**", against a clause placing `Reindex` and `Broadcast` at R6. SiLU moved to the R6 clause with a dated in-tense note; it is the same third maturity claim and adds no execution row.
- The structural row's measurement span said "All **nine** goldens compile and link", now ten. The count is kept as the population of the run it records, with a parenthetical stating that the reproduction command below it now compiles ten and naming `const GOLDENS` as where the current population lives.

Sites read and found needing no edit: `docs/roadmap.md:555` (its SiLU mentions are the discharged `operation-set` wall; "the two elementary families" there is the normalization and the softmax, and `ElementwiseFamily` already reads `Add, Multiply, Silu`), `:392`, `:468`, `:471`, `:477`, and the absence-check block `:486–552` — none asserts this row's rung. `docs/status.md` makes **no** claim about this row at all: it has zero occurrences of `silu`, `elementwise activation`, `transcendental`, or `precise::`, and its only rung sentences are the structural row's. `docs/open-questions.md:103` names `tiler::silu-f32@1` as reference-evaluated against an exact rational enclosure and asserts no rung. The remaining nine navigation files have zero SiLU occurrences.

**Out of scope, filed here rather than edited.** `crates/tiler-metal/src/golden_compilation.rs:125` still reads "all nine fixtures compile and link" in its *structural* measurement paragraph while its elementary paragraph at `:149` correctly reads ten. Both are historical run records so neither is wrong as written, but the nine-paragraph is the one a reader is likeliest to read as current. `crates/**` is not `contracts/navigation`; noted for whoever next holds `implementation/metal`.

## Scope note

`project/tickets` is in `shared_scopes` because every claimed ticket declares it; the guard does not treat a ticket's own file as implicitly shared. Declaration and scheduling metadata for already-authorized work; no product scope moved.
