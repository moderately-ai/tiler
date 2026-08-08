---
id: refresh-the-navigation-execution-row-census-for-the-conformance-crate-device-runs
title: Refresh the navigation execution-row census for the conformance-crate device runs
status: todo
priority: p2
dependencies: []
related: [separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ, correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, navigation, reductions]
---
## Why this is filed rather than absorbed

Found by the worker on [`correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run`](correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run.md) while sweeping that paragraph's siblings. Everything below is in `contracts/navigation`, the same scope that ticket holds, but none of it is the wide-support paragraph that ticket owns, and its brief was explicit that it was small and must not expand. Every Fact was read in full at base `c0b2f06bfa38dced03b9d63f7ef2af96e0d5d47b`; re-read them at your own base before editing, because they are as perishable as any other ticket's.

## The common cause

`crates/tiler-conformance/src/serial_sum.rs` was **added on 2026-08-07** at `0f948637` ("Carry the device-executed value proof into the conformance crate"), and [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) then dispatched a `1x12` reduction from it the same day. Two navigation documents still census device execution as though the prototype were the only place a reduction has run, and as though one offline-compiler row covered everything. The producing ticket held `implementation/runtime`, `implementation/conformance`, and `contracts/numerics` — verified from its frontmatter — so it could not reach either document.

## Fact — the roadmap's two "one prototype execution row" rung cells undercount

Two rows in `docs/roadmap.md`'s operation-family support matrix carry identical rung-cell text. The anchor is `R7 bounded to checked target-neutral layers and one prototype execution row`, and `grep -cF` on it returns **2**, so it does not identify a row on its own. The two are distinguished by their family cell: the strict serial sum's row greps uniquely as `` `Sum` reduction | R6, with R7 bounded `` (count 1), and the other is the pointwise `f32` constants and separate-rounding arithmetic row.

Each cell's **Measurement** names only the retained runtime proof's thirty bit-compared cases under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` on one Apple M4 Max host. Since 2026-08-07 there is a second R7-bearing execution row for both families: `crates/tiler-conformance`'s `serial_sum` vertical builds a mapped sum through `StrictSerialF32Sum::apply` — anchor `over the reduced axis of a given shape` in `crates/tiler-conformance/src/serial_sum.rs` — so it exercises the strict serial sum *and* the constant, multiply, and add keys, and `docs/correctness-and-testing.md` records it dispatched on offline compiler `Apple metal version 32023.921` with SDK `macosx 27.0` build `26A5388f`, a different offline-compiler row from the `32023.883` both rung cells' measurements are bounded to.

**Do not assume the fix is "two rows".** Establish first whether the conformance vertical *re-homed* the prototype's corpus onto a newer toolchain row or is an independent run; `0f948637`'s title says "carry … into the conformance crate", and `prototypes/serial-sum-run/src/proof.rs` still exists and still carries `FLUSH_SUBNORMALS_TO_ZERO_F32` and two `FLUSH_AND_REASSOCIATE_F32` sites. A row count restated without that determination replaces one false claim with another.

## Fact — `docs/status.md`'s device-execution census says four and is short by at least two

Anchor: `device execution, and it is four runs rather than one` — `grep -cF` returns 1. The bullet asserts a census ("All four ran on one Apple M4 Max") and has already been corrected once for undercounting: it carries `Corrected 2026-08-06 — this read "three runs"`. Its four sub-bullets are the serial-sum runtime proof, the contraction vertical (2026-08-02), the L3 correctness cell (2026-08-05), and `The reassociating-contract run, 2026-08-02` (anchor greps, count 1).

Two device executions recorded elsewhere in the corpus are absent from it, both dated 2026-08-07:

1. The `1x12` separating run, which `docs/correctness-and-testing.md` carries in full under the anchor `the separating case has now been driven on hardware`.
2. The BF16 vertical dispatch, which **this same document's own authoritative-profile bullet** records as a Measurement — `crates/tiler-conformance` carrying a pure-BF16 `(x * 1.5) + 0.0` through emission, the Apple offline toolchain, and a dispatch on `Apple metal version 32023.921`. `docs/roadmap.md`'s reduced-precision-float row already reads `R7 as of 2026-08-07 for BF16 constant, multiply, and add`, so the roadmap has the run and the status census does not.

So the sentence's own "none of them widens another" discipline is intact; the count is what has gone stale, and a reader taking it as the corpus's device-execution census is short two runs.

## Fact — `The reassociating-contract run, 2026-08-02` bullet is shape-bounded and does not say so

That sub-bullet states `Both parallel reduction strategies executed against the reference at a `1x4` shape` and closes on the exact-under-every-grouping boundary. It was true when written and remains true about its own run; what it no longer conveys is that a second reduction shape has since executed on a different compiler row and that the three strategies do **not** agree there. The synchronization bullet it defers to (anchor `Measurement — both parallel strategies then executed against the reference, 2026-08-02`) is likewise `1x4`-only and `32023.883`-only.

## Closes when

- Both `docs/roadmap.md` rung cells state the execution rows their R7 is actually bounded to, after the re-homed-versus-independent question above is settled and stated.
- `docs/status.md`'s device-execution census states a count that matches its own enumeration, with each added run carrying its own compiler row.
- The reassociating-contract sub-bullet names both executed reduction shapes, or defers explicitly to [correctness and testing](../docs/correctness-and-testing.md) for the second.
- Nothing here restates the tree's participant-selection rule or its populations. `docs/compiler/fusion-and-scheduling.md` and `crates/tiler-compiler/src/physical.rs` own that, and the evidence rungs it separates — arithmetic bound, exhaustive finite population, `Unknown` on cost — must not be flattened into a navigation sentence.
- Corrections follow the corpus's practice: a claim true when written is dated beside; one never true is substituted with the retired wording quoted. Note that a retired sentence quoted verbatim stays greppable, so a later hit proves the string is present rather than that the claim stands.
