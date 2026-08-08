---
id: correct-the-roadmap-s-1x4-reduction-execution-row-for-the-twelve-contributor-run
title: Correct the roadmap's 1x4 reduction execution row for the twelve-contributor run
status: todo
priority: p3
dependencies: []
related: [separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ]
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [numerics, reductions, documentation]
---
## The stale sentence, and what falsifies it

`docs/roadmap.md`'s "Operation and dtype support" wide-support paragraph — the one beginning "Wide operation support is a durable project goal" — states that under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` "all three retained reduction strategies were emitted, linked, dispatched, and bit-compared **at a `1x4` shape**", and attributes every dispatch in the section to the host row with "offline Metal compiler `32023.883`".

**Fact, 2026-08-07.** [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) landed a second executed shape. All three retained alternatives were emitted, linked, dispatched, and bit-compared at **`1x12`** as well, from `crates/tiler-conformance`'s `serial_sum` vertical, on **Apple M4 Max / Apple9 / macOS 27.0 build `26A5388g` / offline compiler `Apple metal version 32023.921` / SDK `macosx 27.0` build `26A5388f`** — a different offline-compiler row from the `32023.883` the paragraph names. At that shape the single-workgroup tree published 6 partitions of 2 and returned `0x3f800001` while the multi-pass split published 4 of 3 and returned `0x3f800000`, each matched against its own declared grouping; the four-contributor sentence's "both parallel strategies `0x3f800001`" is a property of the narrower shape and not of the strategies.

`docs/correctness-and-testing.md` already carries the new measurement and its boundary in full. This ticket is the navigation ledger catching up, not a second authority for the numbers.

## Why it was not folded into the landing branch

`docs/roadmap.md` is `contracts/navigation`; the producing ticket declared `implementation/runtime`, `implementation/conformance`, and `contracts/numerics`. Widening that branch to take a fourth scope for one paragraph would have put a navigation-catalog edit inside an evidence branch, so the producing ticket's own "Scopes were under-declared" repair block deliberately left this out and asked for it to be filed.

## Closes when

The wide-support paragraph names both executed reduction shapes, `1x4` and `1x12`, states the offline-compiler row each is bounded to rather than one row for the whole section, and no longer implies that all three strategies agree at every executed shape. Read the paragraph in full before editing: it carries several other corrected-in-tense claims whose shape is load-bearing.
